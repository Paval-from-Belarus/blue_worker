use core::net::Ipv4Addr;

use embassy_net::IpAddress;
use esp_hal::{
    peripherals::WIFI,
    time::{self, Duration},
};
use esp_radio::wifi::{ClientConfig, ModeConfig, PowerSaveMode, ScanConfig};
use smoltcp::{iface::SocketStorage, socket::dhcpv4, wire::DhcpOption};

use embedded_io::*;

const SSID: &str = "private_wireless_network";
const PASSWORD: &str = "WriteOnceRunAnywhere";

pub async fn spawn(
    radio_init: &esp_radio::Controller<'_>,
    device: WIFI<'static>,
) {
    let (mut controller, mut interfaces) =
        esp_radio::wifi::new(&radio_init, device, Default::default())
            .expect("Failed to initialize Wi-Fi controller");
    let wifi_iface = create_interface(&mut interfaces.sta);
    let mut socket_set_entries: [SocketStorage; 3] = Default::default();
    let mut socket_set =
        smoltcp::iface::SocketSet::new(&mut socket_set_entries[..]);
    let mut dhcp_socket = dhcpv4::Socket::new();

    dhcp_socket.set_outgoing_options(&[DhcpOption {
        kind: 12,
        data: b"esp-radio",
    }]);
    socket_set.add(dhcp_socket);
    let rng = esp_hal::rng::Rng::new();
    let now = || {
        esp_hal::time::Instant::now()
            .duration_since_epoch()
            .as_millis()
    };

    let stack = blocking_network_stack::Stack::new(
        wifi_iface,
        interfaces.sta,
        socket_set,
        now,
        rng.random(),
    );

    controller.set_power_saving(PowerSaveMode::None).unwrap();

    let client_config = ModeConfig::Client(
        ClientConfig::default()
            .with_ssid(SSID.into())
            .with_password(PASSWORD.into()),
    );

    let res = controller.set_config(&client_config);
    log::info!("wifi_set_configuration returned {:?}", res);

    controller.start().unwrap();
    log::info!("is wifi started: {:?}", controller.is_started());

    log::info!("Start Wifi Scan");
    let scan_config = ScanConfig::default().with_max(10);
    let res = controller.scan_with_config(scan_config).unwrap();
    for ap in res {
        log::info!("{:?}", ap);
    }

    log::info!("{:?}", controller.capabilities());
    log::info!("wifi_connect {:?}", controller.connect());

    // wait to get connected
    log::info!("Wait to get connected");
    loop {
        match controller.is_connected() {
            Ok(true) => break,
            Ok(false) => {}
            Err(err) => {
                log::info!("{:?}", err);
                loop {}
            }
        }
    }
    log::info!("{:?}", controller.is_connected());

    // wait for getting an ip address
    log::info!("Wait to get an ip address");
    loop {
        stack.work();

        if stack.is_iface_up() {
            log::info!("got ip {:?}", stack.get_ip_info());
            break;
        }
    }

    log::info!("Start busy loop on main");

    let mut rx_buffer = [0u8; 1536];
    let mut tx_buffer = [0u8; 1536];
    let mut socket = stack.get_socket(&mut rx_buffer, &mut tx_buffer);

    loop {
        log::info!("Making HTTP request");
        socket.work();

        socket
            .open(IpAddress::Ipv4(Ipv4Addr::new(142, 250, 185, 115)), 80)
            .unwrap();

        socket
            .write(b"GET / HTTP/1.0\r\nHost: www.mobile-j.de\r\n\r\n")
            .unwrap();

        socket.flush().unwrap();

        let deadline = time::Instant::now() + Duration::from_secs(20);
        let mut buffer = [0u8; 512];
        while let Ok(len) = socket.read(&mut buffer) {
            let to_print =
                unsafe { core::str::from_utf8_unchecked(&buffer[..len]) };
            log::info!("{}", to_print);

            if time::Instant::now() > deadline {
                log::warn!("Timeout");
                break;
            }
        }

        socket.disconnect();

        let deadline = time::Instant::now() + Duration::from_secs(5);
        while time::Instant::now() < deadline {
            socket.work();
        }
    }
}

// some smoltcp boilerplate
fn timestamp() -> smoltcp::time::Instant {
    smoltcp::time::Instant::from_micros(
        esp_hal::time::Instant::now()
            .duration_since_epoch()
            .as_micros() as i64,
    )
}

pub fn create_interface(
    device: &mut esp_radio::wifi::WifiDevice,
) -> smoltcp::iface::Interface {
    // users could create multiple instances but since they only have one WifiDevice
    // they probably can't do anything bad with that
    smoltcp::iface::Interface::new(
        smoltcp::iface::Config::new(smoltcp::wire::HardwareAddress::Ethernet(
            smoltcp::wire::EthernetAddress::from_bytes(&device.mac_address()),
        )),
        device,
        timestamp(),
    )
}
