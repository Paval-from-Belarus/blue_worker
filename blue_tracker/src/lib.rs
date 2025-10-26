#![no_std]

use embassy_sync::{blocking_mutex::raw::NoopRawMutex, signal::Signal};

pub mod ble;
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
    pub scan: Signal<NoopRawMutex, blue_types::Scan>,
}

impl SharedState {
    pub fn new() -> Self {
        Self {
            scan: Signal::new(),
        }
    }
}
