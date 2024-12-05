use std::time::Duration;

use blue_types::{DeviceData, Scan};
use embedded_svc::{http::client::Client, wifi};
use esp32_nimble::{BLEDevice, BLEScan};
use esp_idf_hal::{io::Write, prelude::Peripherals, sys::esp_crt_bundle_attach, task};
use esp_idf_svc::{
    eventloop::EspSystemEventLoop,
    nvs::EspDefaultNvsPartition,
    wifi::{BlockingWifi, ClientConfiguration, EspWifi},
};

use esp_idf_svc::http::client::{Configuration as HttpConfig, EspHttpConnection};

#[derive(Debug)]
#[toml_cfg::toml_config]
pub struct NetworkConfig {
    #[default("private_wireless_network")]
    pub ssid: &'static str,
    #[default("")]
    pub password: &'static str,

    #[default("http://localhost:8080")]
    pub base_url: &'static str,
}

fn main() -> anyhow::Result<()> {
    esp_idf_svc::sys::link_patches();
    esp_idf_svc::log::EspLogger::initialize_default();

    log::info!("Wi-Fi config: {:?}", NETWORK_CONFIG);

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
    }))
    .expect("Failed configure wi-fi");

    wifi.start().expect("Failed start wi-fi");

    wifi.connect().expect("Failed connect to wi-fi");

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

    loop {
        let scan = scan_devices(&mut ble_scan, ble_device);

        let request_body = scan.to_vec();

        let Ok(mut request) = http_client.put(
            &devices_url,
            &[("Content-Type", "application/octet-stream")],
        ) else {
            log::warn!("Failed to initiate request");
            std::thread::sleep(Duration::from_millis(100));
            continue;
        };

        let _ = request.write_all(&request_body);

        let _ = request.submit().inspect(|response| {
            log::info!("Server sends status {}", response.status());
        });
    }
}

fn scan_devices(ble_scan: &mut BLEScan, ble_device: &BLEDevice) -> Scan {
    task::block_on(async {
        ble_scan.active_scan(true).interval(100).window(99);

        let scan_duration = 5_000;

        let mut devices = Vec::<DeviceData>::new();

        let _ = ble_scan
            .start(ble_device, scan_duration as i32, |device, data| {
                let name = data
                    .name()
                    .and_then(|raw_name| {
                        core::str::from_utf8(raw_name)
                            .inspect_err(|_cause| {
                                log::debug!("{raw_name} is not valid utf-8");
                            })
                            .ok()
                    })
                    .map(|name| name.to_string());

                let address = device.addr().as_le_bytes().into();
                let rssi = device.rssi();

                let device_data = DeviceData {
                    name,
                    address,
                    rssi,
                };

                log::info!("Device data: {:?}", device_data);

                devices.push(device_data);

                None::<()>
            })
            .await
            .expect("Failed to scan devices");

        Scan {
            duration: scan_duration,
            devices,
        }
    })
}
