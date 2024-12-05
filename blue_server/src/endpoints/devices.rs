use actix_web::{
    HttpRequest, HttpResponse,
    error::{ErrorBadRequest, InternalError},
    get,
    http::{StatusCode, header::HeaderValue},
    put, web,
};
use chrono::{DateTime, Utc};
use futures::StreamExt;
use sailfish::TemplateSimple;
use serde::Deserialize;

use crate::{devices_lock, devices_mut_lock};

#[derive(Debug, Clone, Deserialize)]
pub struct DeviceQuery {
    pub start: Option<DateTime<Utc>>,
    pub end: Option<DateTime<Utc>>,
}

#[derive(TemplateSimple)]
#[template(path = "devices/index.stpl")]
pub struct ScoreTablePage {}

#[get("/devices")]
pub async fn index() -> actix_web::Result<HttpResponse> {
    let body = ScoreTablePage {}
        .render_once()
        .map_err(|cause| InternalError::new(cause, StatusCode::INTERNAL_SERVER_ERROR))?;

    Ok(HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(body))
}

const MAX_SIZE: usize = 262_144;

#[put("/api/v1/devices")]
pub async fn add_devices(
    req: HttpRequest,
    mut payload: web::Payload,
) -> actix_web::Result<HttpResponse> {
    let mut raw_data = web::BytesMut::new();

    while let Some(chunk) = payload.next().await {
        let chunk = chunk?;
        // limit max size of in-memory payload
        if (raw_data.len() + chunk.len()) > MAX_SIZE {
            return Err(ErrorBadRequest("overflow"));
        }
        raw_data.extend_from_slice(&chunk);
    }

    let Some(scan) = blue_types::Scan::from_bytes(&raw_data) else {
        return Err(ErrorBadRequest("failed to parse"));
    };

    match devices_mut_lock!(req).put_devices(scan).await {
        Ok(_) => Ok(HttpResponse::Ok().into()),
        Err(_) => Err(ErrorBadRequest("Failed to save data")),
    }
}

#[get("/api/v1/devices")]
pub async fn devices_list(
    req: HttpRequest,
    // query: web::Query<DeviceQuery>,
) -> actix_web::Result<HttpResponse> {
    // log::info!("Start time = {:?}. End time = {:?}", query.start, query.end);

    let Some(snapshot) = devices_lock!(req)
        .take_snapshot(Utc::now(), Utc::now())
        .await
    else {
        return Ok(HttpResponse::new(StatusCode::NOT_FOUND));
    };

    let body = serde_json::to_string(&snapshot).expect("Server json is always valid");

    Ok(HttpResponse::Ok()
        .content_type("application/json; charset=utf-8")
        .body(body))
}
