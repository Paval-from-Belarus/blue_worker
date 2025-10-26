use core::cell::RefCell;

use bt_hci::controller::ExternalController;
use embassy_time::{Duration, Timer};
use esp_hal::peripherals::BT;
use esp_radio::ble::controller::BleConnector;

use trouble_host::prelude::*;

const CONNECTIONS_MAX: usize = 1;
const L2CAP_CHANNELS_MAX: usize = 1;

pub async fn spawn(
    radio_init: &esp_radio::Controller<'_>,
    device: BT<'static>,
) {
    let transport =
        BleConnector::new(&radio_init, device, Default::default()).unwrap();
    let ble_controller = ExternalController::<_, 20>::new(transport);
    let mut resources: HostResources<
        DefaultPacketPool,
        CONNECTIONS_MAX,
        L2CAP_CHANNELS_MAX,
    > = HostResources::new();

    let address: Address =
        Address::random([0xff, 0x8f, 0x1b, 0x05, 0xe4, 0xff]);

    let stack = trouble_host::new(ble_controller, &mut resources)
        .set_random_address(address);

    let Host {
        central,
        mut runner,
        ..
    } = stack.build();

    let printer = Printer {
        seen: RefCell::new(heapless::Deque::new()),
    };
    let mut scanner = Scanner::new(central);
    let _ =
        embassy_futures::join::join(runner.run_with_handler(&printer), async {
            let config = ScanConfig {
                active: true,
                phys: PhySet::M1,
                interval: Duration::from_millis(500),
                window: Duration::from_millis(500),
                ..Default::default()
            };

            let mut _session = scanner.scan(&config).await.unwrap();
            // Scan forever
            loop {
                Timer::after(Duration::from_secs(1)).await;
            }
        })
        .await;
}

struct Printer {
    seen: RefCell<heapless::Deque<BdAddr, 128>>,
}

impl EventHandler for Printer {
    fn on_adv_reports(&self, mut it: LeAdvReportsIter<'_>) {
        let mut seen = self.seen.borrow_mut();
        while let Some(Ok(report)) = it.next() {
            if seen.iter().find(|b| b.raw() == report.addr.raw()).is_none() {
                log::info!("discovered: {:?}", report.addr);
                if seen.is_full() {
                    seen.pop_front();
                }
                seen.push_back(report.addr).unwrap();
            }
        }
    }
}
