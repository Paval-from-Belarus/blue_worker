#![feature(let_chains)]

use std::sync::Arc;

use actix_files::Files;
use actix_web::{App, HttpServer};
use model::DeviceSharedState;
use tokio::sync::RwLock;

mod endpoints;
mod model;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    env_logger::builder().init();

    HttpServer::new(move || {
        App::new()
            .app_data(Arc::new(RwLock::new(DeviceSharedState::default())))
            .service(endpoints::devices::index)
            .service(endpoints::devices::devices_list)
            .service(endpoints::devices::add_devices)
            .service(endpoints::devices::time_limis)
            .service(Files::new("/images", "./templates/images"))
            .service(Files::new("/scripts", "./templates/scripts"))
            .service(Files::new("/css", "./templates/css"))
    })
    .bind(("0.0.0.0", 8080))?
    .run()
    .await?;

    Ok(())
}
