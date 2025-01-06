#[cfg(feature = "http")]
mod network;
#[cfg(feature = "serial")]
mod serial;

use blue_types::{DeviceData, Scan};
use esp_idf_svc::hal::prelude::Peripherals;
use esp_idf_svc::{
    bt::{BtClassic, BtDriver},
    nvs::EspDefaultNvsPartition,
};

#[derive(Debug, Clone)]
#[toml_cfg::toml_config]
#[cfg(feature = "wifi")]
pub struct NetworkConfig {
    #[default("private_wireless_network")]
    pub ssid: &'static str,
    #[default("")]
    pub password: &'static str,

    #[default("http://localhost:8080")]
    pub base_url: &'static str,
}

pub trait Device<'a> {
    fn send_scan(&mut self, scan: Scan) -> anyhow::Result<()>;
}

#[cfg(feature = "http")]
use network::HttpDevice;

fn main() -> anyhow::Result<()> {
    esp_idf_svc::sys::link_patches();
    esp_idf_svc::log::EspLogger::initialize_default();

    let peripherals = Peripherals::take()?;
    let nvs = EspDefaultNvsPartition::take()?;

    #[cfg(not(feature = "wifi"))]
    let bl_modem = peripherals.modem;

    #[cfg(feature = "serial")]
    let mut device = {
        use esp_idf_svc::hal;
        use serial::SerialDevice;

        let tx = peripherals.pins.gpio5;
        let rx = peripherals.pins.gpio6;

        let config = hal::uart::config::Config::new()
            .baudrate(hal::units::Hertz(115_200));

        let uart = hal::uart::UartDriver::new(
            peripherals.uart1,
            tx,
            rx,
            Option::<hal::gpio::Gpio0>::None,
            Option::<hal::gpio::Gpio1>::None,
            &config,
        )
        .expect("Failed to access uart driver");

        SerialDevice::new(uart)
    };

    #[cfg(feature = "wifi")]
    let (wifi_modem, bl_modem) = peripherals.modem.split();

    #[cfg(feature = "http")]
    let device =
        HttpDevice::new(wifi_modem, nvs.clone(), NETWORK_CONFIG.clone())
            .expect("Failed to create http device");

    let mut bt_driver =
        BtDriver::<'static, BtClassic>::new(bl_modem, Some(nvs)).unwrap();

    bt_driver.set_device_name("Rust Worker 1").unwrap();

    loop {
        let scan = scan_devices(&mut bt_driver);

        device.send_scan(scan)?;
    }
}

static mut DEVICES: Vec<DeviceData> = vec![];
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
