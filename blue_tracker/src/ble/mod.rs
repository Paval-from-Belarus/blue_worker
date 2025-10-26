use alloc::{string::ToString, vec::Vec};
use bt_hci::controller::ExternalController;
use embassy_time::{Duration, Timer};
use esp_hal::peripherals::BT;
use esp_radio::ble::controller::BleConnector;

use trouble_host::prelude::*;

use crate::SharedState;

const CONNECTIONS_MAX: usize = 1;
const L2CAP_CHANNELS_MAX: usize = 1;

pub async fn start_scan(
    radio_init: &'static esp_radio::Controller<'_>,
    device: BT<'static>,
    state: &'static SharedState,
) {
    let transport =
        BleConnector::new(radio_init, device, Default::default()).unwrap();

    let ble_controller = ExternalController::<_, 20>::new(transport);
    let mut resources: HostResources<
        DefaultPacketPool,
        CONNECTIONS_MAX,
        L2CAP_CHANNELS_MAX,
    > = HostResources::new();

    let stack = trouble_host::new(ble_controller, &mut resources);

    let Host {
        central,
        mut runner,
        ..
    } = stack.build();

    let mut scanner = Scanner::new(central);

    let worker = ScanWorker { state };

    let _ =
        embassy_futures::join::join(runner.run_with_handler(&worker), async {
            let config = ScanConfig {
                active: true,
                phys: PhySet::M1,
                interval: Duration::from_millis(500),
                window: Duration::from_millis(500),
                ..Default::default()
            };

            let _session = scanner.scan(&config).await.unwrap();
            // Scan forever
            loop {
                Timer::after(Duration::from_secs(1)).await;
            }
        })
        .await;
}

struct ScanWorker {
    state: &'static SharedState,
}

impl EventHandler for ScanWorker {
    fn on_adv_reports(&self, it: LeAdvReportsIter<'_>) {
        let devices = it
            .filter_map(|report| report.ok())
            .map(|report| {
                let name =
                    str::from_utf8(report.data).ok().map(|s| s.to_string());

                blue_types::DeviceData {
                    rssi: report.rssi,
                    address: report.addr.into_inner().into(),
                    name,
                }
            })
            .collect::<Vec<_>>();

        let scan = blue_types::Scan {
            duration: embassy_time::Instant::now().as_millis() as u64,
            devices,
        };

        self.state.scan.signal(scan);
    }
}
