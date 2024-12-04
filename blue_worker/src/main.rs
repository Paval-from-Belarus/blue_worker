mod shared_data;
mod worker;

use std::time::Duration;

use esp32_nimble::{BLEDevice, BLEScan};
use esp_idf_hal::{prelude::Peripherals, task};

fn main() {
    esp_idf_svc::sys::link_patches();
    esp_idf_svc::log::EspLogger::initialize_default();


    let peripherals = Peripherals::take();

    let ble_device = BLEDevice::take();

    let mut ble_scan = BLEScan::new();

    task::block_on(async {
        ble_scan.active_scan(true).interval(100).window(99);

        let maybe_scan = ble_scan
            .start(ble_device, 5000, |device, data| {
                log::info!("Advertised Device: ({:?}, {:?})", device, data);
                None::<()>
            })
            .await;

        match maybe_scan {
            Ok(_) => {
                log::info!("Scan finished");

                loop {
                    std::thread::sleep(Duration::from_millis(200));
                    log::info!("Hello world");
                }
            }

            Err(cause) => {
                log::warn!("Failed to scan devices");

                loop {
                    std::thread::sleep(Duration::from_millis(200));
                    log::info!("Hello world");
                }
            }
        }
    });
}
