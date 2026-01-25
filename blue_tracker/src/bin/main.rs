#![no_std]
#![no_main]
#![deny(
    clippy::mem_forget,
    reason = "mem::forget is generally not safe to do with esp_hal types, especially those \
    holding buffers for the duration of a data transfer."
)]

use embassy_executor::Spawner;
use esp_backtrace as _;
use esp_hal::clock::CpuClock;
use esp_hal::rmt::Rmt;
use esp_hal::timer::timg::TimerGroup;
use log::info;
extern crate alloc;

// This creates a default app-descriptor required by the esp-idf bootloader.
// For more information see: <https://docs.espressif.com/projects/esp-idf/en/stable/esp32/api-reference/system/app_image_format.html#application-description>
esp_bootloader_esp_idf::esp_app_desc!();

#[esp_rtos::main]
async fn main(spawner: Spawner) {
    esp_println::logger::init_logger_from_env();

    let config = esp_hal::Config::default().with_cpu_clock(CpuClock::max());
    let p = esp_hal::init(config);

    esp_alloc::heap_allocator!(#[unsafe(link_section = ".dram2_uninit")] size: 65536);
    // COEX needs more RAM - so we've added some more
    esp_alloc::heap_allocator!(size: 64 * 1024);

    let timg0 = TimerGroup::new(p.TIMG0);
    let sw_interrupt =
        esp_hal::interrupt::software::SoftwareInterruptControl::new(
            p.SW_INTERRUPT,
        );
    esp_rtos::start(timg0.timer0, sw_interrupt.software_interrupt0);

    info!("Embassy initialized!");

    let radio_init =
        esp_radio::init().expect("Failed to initialize Wi-Fi/BLE controller");

    let radio_init =
        &*blue_tracker::mk_static!(esp_radio::Controller, radio_init);

    let state = blue_tracker::SharedState::new();
    let state = &*blue_tracker::mk_static!(blue_tracker::SharedState, state);

    let rmt: Rmt<'_, esp_hal::Async> = {
        let frequency = esp_hal::time::Rate::from_mhz(80);
        Rmt::new(p.RMT, frequency)
    }
    .expect("Failed to initialize RMT")
    .into_async();

    blue_tracker::control::start_task(rmt, p.GPIO8, state, &spawner)
        .await
        .expect("Failed to start LED task");

    blue_tracker::wifi::start_task(&radio_init, p.WIFI, &spawner, state).await;
    blue_tracker::ble::start_scan(&radio_init, p.BT, state).await;
}
