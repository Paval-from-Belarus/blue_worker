use std::time::Duration;

use blue_types::{DeviceData, Scan};
use embedded_svc::{http::client::Client, wifi};
use esp_idf_svc::hal::{
    io::Write, prelude::Peripherals, sys::esp_crt_bundle_attach,
};
use esp_idf_svc::{
    bt::{BtClassic, BtDriver},
    eventloop::EspSystemEventLoop,
    nvs::EspDefaultNvsPartition,
    wifi::{BlockingWifi, ClientConfiguration, EspWifi},
};

use esp_idf_svc::http::client::{
    Configuration as HttpConfig, EspHttpConnection,
};

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

    let (wifi_modem, bl_modem) = peripherals.modem.split();

    let mut wifi = BlockingWifi::wrap(
        EspWifi::new(wifi_modem, sys_loop.clone(), Some(nvs.clone())).unwrap(),
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

    log::info!("Wi-Fi is connected");

    while !wifi.is_connected().unwrap() {
        let config = wifi.get_configuration().unwrap();
        log::info!("Waiting for station: {:?}", config);
    }

    let devices_url = NETWORK_CONFIG.base_url;

    let mut bt_driver =
        BtDriver::<'static, BtClassic>::new(bl_modem, Some(nvs)).unwrap();

    bt_driver.set_device_name("Rust Worker 1").unwrap();

    loop {
        let scan = scan_devices(&mut bt_driver);

        let http_connection = EspHttpConnection::new(&HttpConfig {
            use_global_ca_store: true,
            crt_bundle_attach: Some(esp_crt_bundle_attach),
            ..Default::default()
        })?;

        let mut http_client = Client::wrap(http_connection);

        let body = scan.to_vec();

        let headers = [
            ("Content-Type", "application/octet-stream"),
            ("Content-Length", &format!("{}", body.len())),
            ("Connection", "Keep-Alive"),
        ];

        let Ok(mut request) = http_client.put(devices_url, &headers) else {
            log::warn!("Failed to initiate request");
            std::thread::sleep(Duration::from_millis(1000));
            continue;
        };

        let _ = request.write_all(&body);

        let _ = request.flush();

        let _ = request.submit().inspect(|response| {
            log::info!("Server sends status {}", response.status());
        });
    }
}

fn scan_devices(bt_driver: &mut BtDriver<BtClassic>) -> Scan {
    unsafe { DEVICES.clear() };

    let scan_duration = 15_000;

    let _ = bt_driver.start_scan(scan_duration as u32, |data| {
        let name = data.name;
        if let Some(ref name) = name {
            log::info!("Device with name {name}");
        }

        let address = data.addr.into();
        let rssi = data.rssi;

        let device_data = DeviceData {
            name,
            address,
            rssi,
        };

        unsafe {
            DEVICES.push(device_data);
        }
    });

    unsafe {
        Scan {
            devices: DEVICES.clone(),
            duration: scan_duration,
        }
    }
}

static mut DEVICES: Vec<DeviceData> = vec![];
