mod network;
mod serial;

use blue_types::{DeviceData, Scan};
use esp_idf_svc::hal::prelude::Peripheral;
use esp_idf_svc::{
    bt::{BtClassic, BtDriver},
    nvs::EspDefaultNvsPartition,
};

pub trait Device<'a> {
    fn send_scan(&'a self, scan: Scan) -> anyhow::Result<()>;
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

    #[cfg(feature = "wifi")]
    let (wifi_modem, bl_modem) = peripherals.modem.split();

    #[cfg(feature = "http")]
    let device = HttpDevice::new(wifi_modem, nvs.clone())
        .expect("Failed to create http device");

    #[cfg(feature = "serial")]
    let device = ();

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

#[cfg(feature = "serial")]
fn send_scan(_scan: Scan, _device: ()) -> anyhow::Result<()> {
    Ok(())
}
