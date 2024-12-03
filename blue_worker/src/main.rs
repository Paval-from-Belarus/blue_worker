#![no_std]
#![no_main]

mod shared_data;
mod worker;

// use esp32_nimble::{BLEDevice, BLEScan};
use esp_backtrace as _;
use esp_hal::prelude::*;
use esp_idf_hal::task;
use log::info;

use embassy_executor::Spawner;
use embassy_time::{Duration, Timer};
// use esp_idf_sys as _;

extern crate alloc;

#[esp_hal_embassy::main]
async fn main(spawner: Spawner) {
    // esp_idf_sys::link_patches();

    let peripherals = esp_hal::init({
        let mut config = esp_hal::Config::default();
        config.cpu_clock = CpuClock::max();
        config
    });

    esp_alloc::heap_allocator!(72 * 1024);

    esp_println::logger::init_logger_from_env();

    let timer0 = esp_hal::timer::timg::TimerGroup::new(peripherals.TIMG1);
    esp_hal_embassy::init(timer0.timer0);

    let timer1 = esp_hal::timer::timg::TimerGroup::new(peripherals.TIMG0);
    let _init = esp_wifi::init(
        timer1.timer0,
        esp_hal::rng::Rng::new(peripherals.RNG),
        peripherals.RADIO_CLK,
    )
    .unwrap();

    // let ble_device = BLEDevice::take();
    //
    // let mut ble_scan = BLEScan::new();

    task::block_on(async {
        // ble_scan.active_scan(true).interval(100).window(99);
        //
        // ble_scan
        //     .start(ble_device, 5000, |device, data| {
        //         log::info!("Advertised Device: ({:?}, {:?})", device, data);
        //         None::<()>
        //     })
        //     .await?;
    });

    log::info!("Scan finished");

    let _ = spawner;

    loop {
        info!("Hello world!");
        Timer::after(Duration::from_secs(1)).await;
    }
}
