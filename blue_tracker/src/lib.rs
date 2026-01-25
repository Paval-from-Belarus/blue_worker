#![no_std]

use core::sync::atomic::AtomicUsize;

use embassy_sync::{blocking_mutex::raw::NoopRawMutex, signal::Signal};

pub mod ble;
pub mod control;
pub mod wifi;

#[macro_export]
macro_rules! mk_static {
    ($t:ty,$val:expr) => {{
        static STATIC_CELL: static_cell::StaticCell<$t> =
            static_cell::StaticCell::new();
        #[deny(unused_attributes)]
        let x = STATIC_CELL.uninit().write(($val));
        x
    }};
}

extern crate alloc;

pub struct SharedState {
    wifi_state: AtomicUsize,
    pub scan: Signal<NoopRawMutex, blue_types::Scan>,
}

#[derive(Debug, Clone, Copy)]
#[repr(usize)]
pub enum ConnectionState {
    Disconnected = 1,
    Connecting = 2,
    Connected = 3,
}

impl From<esp_radio::wifi::WifiStaState> for ConnectionState {
    fn from(value: esp_radio::wifi::WifiStaState) -> Self {
        match value {
            esp_radio::wifi::WifiStaState::Connected => Self::Connected,
            esp_radio::wifi::WifiStaState::Started => Self::Connecting,
            _ => Self::Disconnected,
        }
    }
}

impl SharedState {
    pub fn new() -> Self {
        Self {
            wifi_state: AtomicUsize::new(ConnectionState::Connecting as _),
            scan: Signal::new(),
        }
    }

    pub fn set_connection_state(&self, s: impl Into<ConnectionState>) {
        let state: ConnectionState = s.into();

        self.wifi_state
            .store(state as usize, core::sync::atomic::Ordering::SeqCst);
    }

    pub fn connection_state(&self) -> ConnectionState {
        let val = self.wifi_state.load(core::sync::atomic::Ordering::SeqCst);
        match val {
            _ if val == ConnectionState::Connecting as usize => {
                ConnectionState::Connecting
            }
            _ if val == ConnectionState::Connected as usize => {
                ConnectionState::Connected
            }
            _ if val == ConnectionState::Disconnected as usize => {
                ConnectionState::Disconnected
            }
            _ => {
                unreachable!("Invalid wifi_state")
            }
        }
    }
}
