use esp_idf_svc::{hal::uart::UartDriver, io::Write};

use crate::Device;

pub struct SerialDevice<'a> {
    driver: UartDriver<'a>,
}

impl<'a> SerialDevice<'a> {
    pub fn new(driver: UartDriver<'a>) -> Self {
        Self { driver }
    }
}

impl<'a> Device<'a> for SerialDevice<'a> {
    fn send_scan(&mut self, scan: blue_types::Scan) -> anyhow::Result<()> {
        let json = scan.to_json();

        self.driver.write_all(json.as_bytes()).unwrap();

        Ok(())
    }
}
