mod shared_data;
mod worker;

use std::time::Duration;

use embedded_svc::{
    http::{self, client::Client},
    wifi,
};
use esp32_nimble::{BLEDevice, BLEScan};
use esp_idf_hal::{prelude::Peripherals, sys::esp_crt_bundle_attach, task};
use esp_idf_svc::{
    eventloop::EspSystemEventLoop,
    nvs::EspDefaultNvsPartition,
    wifi::{BlockingWifi, ClientConfiguration, EspWifi},
};

use esp_idf_svc::http::client::{Configuration as HttpConfig, EspHttpConnection};
use shared_data::DeviceData;

#[toml_cfg::toml_config]
pub struct NetworkConfig {
    #[default("private_wireless_network")]
    pub ssid: &'static str,
    #[default("")]
    pub password: &'static str,

    #[default("http://192.168.50.1:8080")]
    pub base_url: &'static str,
}

fn main() -> anyhow::Result<()> {
    esp_idf_svc::sys::link_patches();
    esp_idf_svc::log::EspLogger::initialize_default();

    let peripherals = Peripherals::take()?;
    let sys_loop = EspSystemEventLoop::take()?;
    let nvs = EspDefaultNvsPartition::take()?;

    let mut wifi = BlockingWifi::wrap(
        EspWifi::new(peripherals.modem, sys_loop.clone(), Some(nvs)).unwrap(),
        sys_loop.clone(),
    )?;

    wifi.set_configuration(&wifi::Configuration::Client(ClientConfiguration {
        ssid: NETWORK_CONFIG.ssid.try_into().expect("Invalid ssid"),
        password: NETWORK_CONFIG
            .password
            .try_into()
            .expect("Invalid password"),
        auth_method: esp_idf_svc::wifi::AuthMethod::WPA2Personal,
        ..Default::default()
    }))?;

    wifi.start()?;

    wifi.connect()?;

    wifi.wait_netif_up()?;

    while !wifi.is_connected().unwrap() {
        let config = wifi.get_configuration().unwrap();
        log::info!("Waiting for station: {:?}", config);
    }

    log::info!("Wi-Fi is connected");

    let http_connection = EspHttpConnection::new(&HttpConfig {
        use_global_ca_store: true,
        crt_bundle_attach: Some(esp_crt_bundle_attach),
        ..Default::default()
    })?;

    let mut http_client = Client::wrap(http_connection);

    let devices_url = format!("{}/api/v1/devices", NETWORK_CONFIG.base_url);

    let ble_device = BLEDevice::take();

    let mut ble_scan = BLEScan::new();

    let devices = task::block_on(async {
        ble_scan.active_scan(true).interval(100).window(99);

        let devices = Vec::<DeviceData>::new();

        let _ = ble_scan
            .start(ble_device, 5000, |device, data| {
                log::info!("Advertised Device: ({:?}, {:?})", device, data);

                // devices.push(d); //todo: save device data
                None::<()>
            })
            .await
            .expect("Failed to scan devices");

        devices
    });

    let request = http_client
        .request(http::Method::Get, &time_url, &[("accept", "text/plain")])
        .unwrap();

    let _response = request.submit().unwrap();
    //todo: extract timestamp here

    loop {
        std::thread::sleep(Duration::from_millis(200));
        log::info!("Hello world");
    }
}
