use actix_web::{HttpRequest, HttpResponse, get, http::StatusCode, web};
use sailfish::TemplateSimple;

use crate::{
    devices_lock,
    model::{DeviceId, DeviceStateLock},
};

#[derive(TemplateSimple)]
#[template(path = "score_table/index.stpl")]
pub struct ScoreTablePage {}

#[get("/api/v1/devices")]
pub async fn devices(req: HttpRequest) -> actix_web::Result<HttpResponse> {
    todo!()
}

#[get("/api/v1/devices/{id}")]
pub async fn device_by_id(
    req: HttpRequest,
    query: web::Path<DeviceId>,
) -> actix_web::Result<HttpResponse> {
    let Some(device_data) = devices_lock!(req).find_by_id(&query) else {
        return Ok(HttpResponse::new(StatusCode::NOT_FOUND));
    };

    Ok(HttpResponse::Ok().json(device_data))
}
