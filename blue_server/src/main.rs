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
    env_logger::builder()
        .filter_level(log::LevelFilter::Info)
        .init();

    let shared_state = DeviceSharedState::load_from_config("shared_state.json").await;

    let shared_lock = Arc::new(RwLock::new(shared_state));

    HttpServer::new(move || {
        App::new()
            .app_data(Arc::clone(&shared_lock))
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
