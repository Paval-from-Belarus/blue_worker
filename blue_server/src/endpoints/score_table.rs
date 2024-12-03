use actix_web::{HttpRequest, HttpResponse, Responder, get, http::StatusCode, web};
use chrono::{DateTime, Utc};
use sailfish::TemplateSimple;
use serde::Deserialize;

use crate::devices_lock;

#[derive(Debug, Clone, Deserialize)]
pub struct DeviceQuery {
    pub start: DateTime<Utc>,
    pub end: DateTime<Utc>,
}

#[derive(TemplateSimple)]
#[template(path = "score_table/index.stpl")]
pub struct ScoreTablePage {}

#[get("/score_table")]
pub async fn index(req: HttpRequest) -> impl Responder {
    HttpResponse::Ok()
}

#[get("/api/v1/time")]
pub async fn time_limits(req: HttpRequest) -> actix_web::Result<HttpResponse> {
    let Some(limits) = devices_lock!(req).take_limits().await else {
        return Ok(HttpResponse::new(StatusCode::CONFLICT));
    };

    Ok(HttpResponse::Ok().json(limits))
}

#[get("/api/v1/devices")]
pub async fn devices(
    req: HttpRequest,
    query: web::Query<DeviceQuery>,
) -> actix_web::Result<HttpResponse> {
    let Some(snapshot) = devices_lock!(req)
        .take_snapshot(query.start, query.end)
        .await
    else {
        return Ok(HttpResponse::new(StatusCode::NOT_FOUND));
    };

    Ok(HttpResponse::Ok().json(snapshot))
}
