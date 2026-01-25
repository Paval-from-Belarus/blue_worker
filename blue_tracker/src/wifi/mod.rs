use embassy_net::{
    DhcpConfig, Runner, Stack, StackResources,
    tcp::client::{TcpClient, TcpClientState},
};
use embassy_time::{Duration, Timer};
use esp_hal::peripherals::WIFI;
use esp_radio::wifi::{
    ClientConfig, ModeConfig, PowerSaveMode, WifiController, WifiDevice,
    WifiEvent, WifiStaState,
};
use reqwless::client::{HttpClient, TlsConfig};
use smoltcp::{iface::SocketStorage, socket::dhcpv4, wire::DhcpOption};

use crate::mk_static;

// const SSID: &str = "private_wireless_network";
// const PASSWORD: &str = "WriteOnceRunAnywhere";
const SSID: &str = "Cudy-10B0";
const PASSWORD: &str = "43149155";

pub async fn start_task(
    radio_init: &'static esp_radio::Controller<'_>,
    device: WIFI<'static>,
    spawner: &embassy_executor::Spawner,
    state: &'static crate::SharedState,
) {
    let (mut controller, interfaces) =
        esp_radio::wifi::new(&radio_init, device, Default::default())
            .expect("Failed to initialize Wi-Fi controller");
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
    let net_seed = rng.random() as u64 | ((rng.random() as u64) << 32);
    let tls_seed = rng.random() as u64 | ((rng.random() as u64) << 32);

    let dhcp_config = DhcpConfig::default();
    // dhcp_config.hostname = Some(String::from_str("implRust").unwrap());

    let config = embassy_net::Config::dhcpv4(dhcp_config);

    // Init network stack
    let (stack, runner) = embassy_net::new(
        interfaces.sta,
        config,
        mk_static!(StackResources<3>, StackResources::<3>::new()),
        net_seed,
    );

    controller.set_power_saving(PowerSaveMode::None).unwrap();

    spawner.spawn(connection(controller, state)).unwrap();
    spawner.spawn(net_task(runner)).unwrap();

    wait_for_connection(stack).await;

    access_website(stack, tls_seed).await;

    spawner.spawn(scan_task(state)).unwrap();
}

async fn wait_for_connection(stack: Stack<'_>) {
    log::info!("Waiting for link to be up");
    loop {
        if stack.is_link_up() {
            break;
        }
        Timer::after(Duration::from_millis(500)).await;
    }

    log::info!("Waiting to get IP address...");
    loop {
        if let Some(config) = stack.config_v4() {
            log::info!("Got IP: {}", config.address);
            break;
        }
        Timer::after(Duration::from_millis(500)).await;
    }
}

#[embassy_executor::task]
async fn connection(
    mut controller: WifiController<'static>,
    state: &'static crate::SharedState,
) {
    log::info!("start connection task");
    log::info!("Device capabilities: {:?}", controller.capabilities());

    loop {
        state.set_connection_state(esp_radio::wifi::sta_state());

        match esp_radio::wifi::sta_state() {
            WifiStaState::Connected => {
                // wait until we're no longer connected
                controller.wait_for_event(WifiEvent::StaDisconnected).await;
                Timer::after(Duration::from_millis(5000)).await
            }
            _ => {}
        }

        if !matches!(controller.is_started(), Ok(true)) {
            let config = ClientConfig::default()
                .with_ssid(SSID.try_into().unwrap())
                .with_password(PASSWORD.try_into().unwrap());

            let config = ModeConfig::Client(config);
            controller.set_config(&config).unwrap();
            log::info!("Starting wifi");

            controller.start_async().await.unwrap();
            log::info!("Wifi started!");
        }
        log::info!("About to connect...");

        match controller.connect_async().await {
            Ok(_) => log::info!("Wifi connected!"),
            Err(e) => {
                log::info!("Failed to connect to wifi: {:?}", e);
                Timer::after(Duration::from_millis(5000)).await
            }
        }
    }
}

#[embassy_executor::task]
async fn net_task(mut runner: Runner<'static, WifiDevice<'static>>) {
    runner.run().await
}

#[embassy_executor::task]
async fn scan_task(state: &'static crate::SharedState) {
    let mut prev_scan = embassy_time::Instant::now();

    loop {
        let mut scan = state.scan.wait().await;
        let now = embassy_time::Instant::from_millis(scan.duration);
        let elapsed = now.duration_since(prev_scan).as_millis();
        prev_scan = now;

        scan.duration = elapsed;

        log::info!("Scan complete:\n {:?}", scan);
    }
}

async fn access_website(stack: Stack<'static>, tls_seed: u64) {
    use embassy_net::dns::DnsSocket;

    let mut rx_buffer = [0; 4096];
    let mut tx_buffer = [0; 4096];
    let dns = DnsSocket::new(stack);
    let tcp_state = TcpClientState::<1, 4096, 4096>::new();
    let tcp = TcpClient::new(stack, &tcp_state);

    let tls = TlsConfig::new(
        tls_seed,
        &mut rx_buffer,
        &mut tx_buffer,
        reqwless::client::TlsVerify::None,
    );

    let mut client = HttpClient::new_with_tls(&tcp, &dns, tls);
    let mut buffer = [0u8; 4096];
    let mut http_req = client
        .request(
            reqwless::request::Method::GET,
            "https://jsonplaceholder.typicode.com/posts/1",
        )
        .await
        .unwrap();
    let response = http_req.send(&mut buffer).await.unwrap();

    log::info!("Got response");
    let res = response.body().read_to_end().await.unwrap();

    let content = core::str::from_utf8(res).unwrap();
    log::info!("{}", content);
}
