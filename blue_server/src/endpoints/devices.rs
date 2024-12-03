use actix_web::{
    HttpRequest, HttpResponse,
    error::InternalError,
    get,
    http::{StatusCode, header::HeaderValue},
    web,
};
use chrono::{DateTime, Utc};
use sailfish::TemplateSimple;
use serde::Deserialize;

use crate::devices_lock;

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

#[get("/api/v1/time")]
pub async fn time_limits(req: HttpRequest) -> actix_web::Result<HttpResponse> {
    let Some(limits) = devices_lock!(req).take_limits().await else {
        return Ok(HttpResponse::new(StatusCode::CONFLICT));
    };

    Ok(HttpResponse::Ok().json(limits))
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
