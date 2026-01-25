use embassy_time::{Duration, Timer};
use esp_hal::rmt::Rmt;

#[derive(Debug, thiserror_no_std::Error)]
pub enum ControlError {
    #[error("Failed to init LEDC timer: {0:?}")]
    LedcTimerError(#[from] esp_hal::ledc::timer::Error),
    #[error("Failed to init LEDC channel: {0:?}")]
    LedcChannelError(#[from] esp_hal::ledc::channel::Error),

    #[error("Failed to spawn Emabassy task: {0}")]
    EmbassySpawnError(#[from] embassy_executor::SpawnError),
}

/// Module for hardware control panel (buttons and led)
pub async fn start_task(
    rmt: Rmt<'static, esp_hal::Async>,
    led_pin: esp_hal::peripherals::GPIO8<'static>,
    state: &'static crate::SharedState,
    spawner: &embassy_executor::Spawner,
) -> Result<(), ControlError> {
    spawner.spawn(task(rmt, led_pin, state))?;

    Ok(())
}

#[embassy_executor::task]
async fn task(
    rmt: Rmt<'static, esp_hal::Async>,
    led_pin: esp_hal::peripherals::GPIO8<'static>,
    state: &'static crate::SharedState,
) {
    let rmt_channel = rmt.channel0;
    let mut rmt_buffer = [esp_hal::rmt::PulseCode::default();
        esp_hal_smartled::buffer_size_async(1)];

    let mut led = esp_hal_smartled::SmartLedsAdapterAsync::new(
        rmt_channel,
        led_pin,
        &mut rmt_buffer,
    );

    const RED: smart_leds::RGB8 = smart_leds::RGB::new(255, 0, 0);
    const BLUE: smart_leds::RGB8 = smart_leds::RGB::new(0, 0, 255);
    const GREEN: smart_leds::RGB8 = smart_leds::RGB8::new(0, 255, 0);
    const BLACK: smart_leds::RGB8 = smart_leds::RGB8::new(0, 0, 0);

    const DISCONNECTED_TIMEOUT: Duration = Duration::from_millis(300);
    const CONNECTED_TIMEOUT: Duration = Duration::from_millis(700);
    const CONNECTING_TIMEOUT: Duration = Duration::from_millis(100);

    let mut blink_active_color;
    let mut timeout;
    let mut should_blink = false;

    let level = 100;

    use smart_leds::SmartLedsWriteAsync;

    loop {
        match state.connection_state() {
            crate::ConnectionState::Disconnected => {
                blink_active_color = RED;
                timeout = DISCONNECTED_TIMEOUT;
            }
            crate::ConnectionState::Connected => {
                blink_active_color = GREEN;
                timeout = CONNECTED_TIMEOUT;
            }
            crate::ConnectionState::Connecting => {
                blink_active_color = BLUE;
                timeout = CONNECTING_TIMEOUT;
            }
        }

        should_blink = !should_blink;

        let blink_color = if should_blink {
            blink_active_color
        } else {
            BLACK
        };

        led.write(smart_leds::brightness(
            smart_leds::gamma([blink_color].into_iter()),
            level,
        ))
        .await
        .unwrap();

        Timer::after(timeout).await;
    }
}
