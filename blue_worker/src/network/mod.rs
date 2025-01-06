use esp_idf_svc::{
    eventloop::EspSystemEventLoop,
    hal::modem::WifiModem,
    nvs::EspDefaultNvsPartition,
    wifi::{BlockingWifi, EspWifi},
};

use esp_idf_svc::hal::sys::esp_crt_bundle_attach;

use crate::{Device, NetworkConfig};

pub struct HttpDevice<'a> {
    device: BlockingWifi<EspWifi<'a>>,
    config: NetworkConfig,
}

impl HttpDevice<'_> {
    pub fn new(
        modem: WifiModem,
        nvs: EspDefaultNvsPartition,
        config: NetworkConfig,
    ) -> anyhow::Result<Self> {
        let device = {
            use embedded_svc::wifi;

            let sys_loop = EspSystemEventLoop::take()?;

            let mut wifi = BlockingWifi::wrap(
                EspWifi::new(modem, sys_loop.clone(), Some(nvs.clone()))
                    .unwrap(),
                sys_loop.clone(),
            )?;

            log::info!("Wi-Fi config: {:?}", config);
            wifi.set_configuration(&wifi::Configuration::Client(
                wifi::ClientConfiguration {
                    ssid: config.ssid.try_into().expect("Invalid ssid"),
                    password: config
                        .password
                        .try_into()
                        .expect("Invalid password"),
                    auth_method: esp_idf_svc::wifi::AuthMethod::WPA2Personal,
                    ..Default::default()
                },
            ))
            .expect("Failed configure wi-fi");

            wifi.start().expect("Failed start wi-fi");

            wifi.connect().expect("Failed connect to wi-fi");

            wifi.wait_netif_up()?;

            log::info!("Wi-Fi is connected");

            while !wifi.is_connected().unwrap() {
                let config = wifi.get_configuration().unwrap();
                log::info!("Waiting for station: {:?}", config);
            }

            wifi
        };

        Ok(Self { device, config })
    }
}

impl<'a> Device<'a> for HttpDevice<'a> {
    fn send_scan(&self, scan: blue_types::Scan) -> anyhow::Result<()> {
        use embedded_svc::http::client::Client;
        use esp_idf_svc::http::client::{
            Configuration as HttpConfig, EspHttpConnection,
        };

        use esp_idf_svc::hal::io::Write;

        use std::time::Duration;

        let devices_url = self.config.base_url;

        let http_connection = EspHttpConnection::new(&HttpConfig {
            use_global_ca_store: true,
            crt_bundle_attach: Some(esp_crt_bundle_attach),
            ..Default::default()
        })?;

        let mut http_client = Client::wrap(http_connection);

        let body = scan.to_bytes();

        let headers = [
            ("Content-Type", "application/octet-stream"),
            ("Content-Length", &format!("{}", body.len())),
            ("Connection", "Keep-Alive"),
        ];

        let Ok(mut request) = http_client.put(devices_url, &headers) else {
            log::warn!("Failed to initiate request");
            std::thread::sleep(Duration::from_millis(1000));
            return Ok(());
        };

        let _ = request.write_all(&body);

        let _ = request.flush();

        let _ = request.submit().inspect(|response| {
            log::info!("Server sends status {}", response.status());
        });

        Ok(())
    }
}
