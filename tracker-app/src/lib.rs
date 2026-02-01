use btleplug::api::{Central, Manager as _, Peripheral as _, ScanFilter};
use btleplug::platform::Manager;
use eframe::egui;
use jni::{JNIEnv, JavaVM};
use ndk_sys::{
    ALooper, ALooper_prepare, ASensor, ASensorEvent, ASensorEventQueue,
    ASensorEventQueue_enableSensor, ASensorEventQueue_getEvents,
    ASensorEventQueue_setEventRate, ASensorManager,
    ASensorManager_createEventQueue, ASensorManager_getDefaultSensor,
    ASensorManager_getInstance,
};
use std::ptr;
use std::sync::mpsc::{channel, Receiver, Sender};
use std::time::Duration; use tokio::runtime::Runtime;
use winit::event_loop::EventLoop;

// App state for egui
struct IndoorLocApp {
    accel_data: [f32; 3],            // Accelerometer readings
    gyro_data: [f32; 3],             // Gyroscope readings
    ble_devices: Vec<(String, i16)>, // (Name, RSSI)
    wifi_aps: Vec<(String, i32)>,    // (SSID, RSSI)
    position: (f32, f32),            // Estimated 2D position (x, y)
    sensor_rx: Receiver<SensorData>, // Channel for sensor data
    ble_rx: Receiver<Vec<(String, i16)>>, // Channel for BLE data
    wifi_rx: Receiver<Vec<(String, i32)>>, // Channel for Wi-Fi data
}

#[derive(Debug)]
struct SensorData {
    accel: [f32; 3],
    gyro: [f32; 3],
}

impl eframe::App for IndoorLocApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Update state from channels
        if let Ok(sensor_data) = self.sensor_rx.try_recv() {
            self.accel_data = sensor_data.accel;
            self.gyro_data = sensor_data.gyro;
            // Simple dead reckoning: integrate accel for position (simplified)
            // In production, use Kalman filter
            self.position.0 += self.accel_data[0] * 0.01; // dt=0.01s
            self.position.1 += self.accel_data[1] * 0.01;
        }
        if let Ok(ble_data) = self.ble_rx.try_recv() {
            self.ble_devices = ble_data;
        }
        if let Ok(wifi_data) = self.wifi_rx.try_recv() {
            self.wifi_aps = wifi_data;
        }

        // UI layout
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Indoor Localization Dashboard");

            // Display sensor data
            ui.label(format!(
                "Accelerometer: x={:.2}, y={:.2}, z={:.2}",
                self.accel_data[0], self.accel_data[1], self.accel_data[2]
            ));
            ui.label(format!(
                "Gyroscope: x={:.2}, y={:.2}, z={:.2}",
                self.gyro_data[0], self.gyro_data[1], self.gyro_data[2]
            ));

            // Display position
            ui.label(format!(
                "Position: x={:.2}m, y={:.2}m",
                self.position.0, self.position.1
            ));

            // Plot position (simple 2D map)
            // egui::plot::Plot::new("Position Map").show(ui, |plot_ui| {
            //     plot_ui.points(
            //         egui::plot::Points::new(vec![[
            //             self.position.0 as f64,
            //             self.position.1 as f64,
            //         ]])
            //         .radius(5.0)
            //         .color(egui::Color32::RED),
            //     );
            // });

            // Display BLE and Wi-Fi data
            ui.collapsing("BLE Devices", |ui| {
                for (name, rssi) in &self.ble_devices {
                    ui.label(format!("Device: {}, RSSI: {} dBm", name, rssi));
                }
            });
            ui.collapsing("Wi-Fi Access Points", |ui| {
                for (ssid, rssi) in &self.wifi_aps {
                    ui.label(format!("AP: {}, RSSI: {} dBm", ssid, rssi));
                }
            });
        });

        // Request repaint for real-time updates
        ctx.request_repaint();
    }
}

// Sensor polling thread
fn start_sensor_thread(tx: Sender<SensorData>) {
    let sensor_manager = unsafe { ASensorManager_getInstance() };
    if sensor_manager.is_null() {
        log::error!("Failed to get sensor manager");
        return;
    }

    let accel = unsafe {
        ASensorManager_getDefaultSensor(
            sensor_manager,
            ndk_sys::ASENSOR_TYPE_ACCELEROMETER as i32,
        )
    };
    let gyro = unsafe {
        ASensorManager_getDefaultSensor(
            sensor_manager,
            ndk_sys::ASENSOR_TYPE_GYROSCOPE as i32,
        )
    };
    let looper = unsafe {
        ALooper_prepare(ndk_sys::ALOOPER_PREPARE_ALLOW_NON_CALLBACKS as i32)
    };
    let event_queue = unsafe {
        ASensorManager_createEventQueue(
            sensor_manager,
            looper,
            3,
            None,
            ptr::null_mut(),
        )
    };

    unsafe {
        ASensorEventQueue_enableSensor(event_queue, accel);
        ASensorEventQueue_setEventRate(event_queue, accel, 100_000); // 10 Hz
        ASensorEventQueue_enableSensor(event_queue, gyro);
        ASensorEventQueue_setEventRate(event_queue, gyro, 100_000);
    }

    loop {
        let mut event: ASensorEvent = unsafe { std::mem::zeroed() };
        let num_events =
            unsafe { ASensorEventQueue_getEvents(event_queue, &mut event, 1) };
        if num_events > 0 {
            let mut sensor_data = SensorData {
                accel: [0.0; 3],
                gyro: [0.0; 3],
            };
            unsafe {
                if event.type_ == ndk_sys::ASENSOR_TYPE_ACCELEROMETER as i32 {
                    sensor_data.accel = [
                        event
                            .__bindgen_anon_1
                            .__bindgen_anon_1
                            .acceleration
                            .__bindgen_anon_1
                            .__bindgen_anon_1
                            .x,
                        event
                            .__bindgen_anon_1
                            .__bindgen_anon_1
                            .acceleration
                            .__bindgen_anon_1
                            .__bindgen_anon_1
                            .y,
                        event
                            .__bindgen_anon_1
                            .__bindgen_anon_1
                            .acceleration
                            .__bindgen_anon_1
                            .__bindgen_anon_1
                            .z,
                    ];
                } else if event.type_ == ndk_sys::ASENSOR_TYPE_GYROSCOPE as i32
                {
                    sensor_data.gyro = [
                        event
                            .__bindgen_anon_1
                            .__bindgen_anon_1
                            .gyro
                            .__bindgen_anon_1
                            .__bindgen_anon_1
                            .x,
                        event
                            .__bindgen_anon_1
                            .__bindgen_anon_1
                            .gyro
                            .__bindgen_anon_1
                            .__bindgen_anon_1
                            .y,
                        event
                            .__bindgen_anon_1
                            .__bindgen_anon_1
                            .gyro
                            .__bindgen_anon_1
                            .__bindgen_anon_1
                            .z,
                    ];
                }
            }
            let _ = tx.send(sensor_data);
        }
        std::thread::sleep(Duration::from_millis(10));
    }
}

// BLE scanning thread
fn start_ble_thread(vm: JavaVM, tx: Sender<Vec<(String, i16)>>) {
    use tokio_stream::StreamExt;

    let rt = tokio::runtime::Builder::new_current_thread()
        .build()
        .unwrap();
    
    let _env = vm
        .attach_current_thread()
        .expect("Failed to access java env");

    // btleplug::platform::init(&*env).unwrap();

    rt.block_on(async {
        let manager = Manager::new().await.unwrap();
        let adapters = manager.adapters().await.unwrap();
        let central = adapters.into_iter().next().unwrap();

        log::info!("Starting scan");

        central.start_scan(ScanFilter::default()).await.unwrap();
        let mut events = central.events().await.unwrap();

        let mut devices = Vec::new();
        while let Some(event) = events.next().await {
            if let btleplug::api::CentralEvent::DeviceDiscovered(id) = event {
                if let Ok(periph) = central.peripheral(&id).await {
                    if let Some(props) = periph.properties().await.unwrap() {
                        let name = props.local_name.unwrap_or_default();
                        if let Some(rssi) = props.rssi {
                            devices.push((name, rssi));
                            let _ = tx.send(devices.clone());
                        }
                    }
                }
            }
        }
    });
}

// Wi-Fi scanning thread
fn start_wifi_thread(tx: Sender<Vec<(String, i32)>>) {
    // let mut env = ndk_glue::native_activity().vm().attach_current_thread();
    // loop {
    //     let context = ndk_glue::native_activity().activity();
    //     let wifi_mgr_class =
    //         env.find_class("android/net/wifi/WifiManager").unwrap();
    //     let get_system_service = env
    //         .get_method_id(
    //             "android/content/Context",
    //             "getSystemService",
    //             "(Ljava/lang/String;)Ljava/lang/Object;",
    //         )
    //         .unwrap();
    //     let wifi_str = env.new_string("wifi").unwrap();
    //     let wifi_mgr = env
    //         .call_method_unchecked(
    //             context,
    //             get_system_service,
    //             jni::signature::JavaType::Object("".into()),
    //             &[jni::objects::JValueGen::Object(wifi_str.into()).borrow()],
    //         )
    //         .unwrap()
    //         .l()
    //         .unwrap();
    //
    //     let start_scan = env
    //         .get_method_id(wifi_mgr_class, "startScan", "()Z")
    //         .unwrap();
    //     env.call_method_unchecked(
    //         wifi_mgr,
    //         start_scan,
    //         jni::signature::JavaType::Primitive(
    //             jni::signature::Primitive::Boolean,
    //         ),
    //         &[],
    //     )
    //     .unwrap();
    //
    //     let get_scan_results = env
    //         .get_method_id(
    //             wifi_mgr_class,
    //             "getScanResults",
    //             "()Ljava/util/List;",
    //         )
    //         .unwrap();
    //     let results = env
    //         .call_method_unchecked(
    //             wifi_mgr,
    //             get_scan_results,
    //             jni::signature::JavaType::Object("".into()),
    //             &[],
    //         )
    //         .unwrap()
    //         .l()
    //         .unwrap();
    //
    //     let list_class = env.find_class("java/util/List").unwrap();
    //     let size_id = env.get_method_id(list_class, "size", "()I").unwrap();
    //     let size = env
    //         .call_method_unchecked(
    //             results,
    //             size_id,
    //             jni::signature::JavaType::Primitive(
    //                 jni::signature::Primitive::Int,
    //             ),
    //             &[],
    //         )
    //         .unwrap()
    //         .i()
    //         .unwrap();
    //
    //     let get_id = env
    //         .get_method_id(list_class, "get", "(I)Ljava/lang/Object;")
    //         .unwrap();
    //     let mut aps = Vec::new();
    //     for i in 0..size {
    //         let item = env
    //             .call_method_unchecked(
    //                 results,
    //                 get_id,
    //                 jni::signature::JavaType::Object("".into()),
    //                 &[jni::objects::JValueGen::Int(i).borrow()],
    //             )
    //             .unwrap()
    //             .l()
    //             .unwrap();
    //         let scan_result_class = env.get_object_class(&item).unwrap();
    //         let ssid_id = env
    //             .get_method_id(scan_result_class, "SSID", "Ljava/lang/String;")
    //             .unwrap();
    //         let rssi_id =
    //             env.get_method_id(scan_result_class, "level", "I").unwrap();
    //
    //         let ssid = env
    //             .call_method_unchecked(
    //                 item,
    //                 ssid_id,
    //                 jni::signature::JavaType::Object("".into()),
    //                 &[],
    //             )
    //             .unwrap()
    //             .l()
    //             .unwrap();
    //         let ssid_str: String = env
    //             .get_string(&jni::objects::JString::from(ssid))
    //             .unwrap()
    //             .into();
    //         let rssi = env
    //             .call_method_unchecked(
    //                 item,
    //                 rssi_id,
    //                 jni::signature::JavaType::Primitive(
    //                     jni::signature::Primitive::Int,
    //                 ),
    //                 &[],
    //             )
    //             .unwrap()
    //             .i()
    //             .unwrap();
    //
    //         aps.push((ssid_str, rssi));
    //     }
    //     let _ = tx.send(aps);
    //     std::thread::sleep(Duration::from_secs(5)); // Scan every 5s
    // }
}

#[cfg(target_os = "android")]
use egui_winit::winit;

#[cfg(target_os = "android")]
#[no_mangle]
fn android_main(app: winit::platform::android::activity::AndroidApp) {
    android_logger::init_once(
        android_logger::Config::default()
            .with_max_level(log::LevelFilter::Info),
    );
    log::info!("Indoor Loc App Started");

    // Channels for async data
    let (sensor_tx, sensor_rx) = channel();
    let (ble_tx, ble_rx) = channel();
    let (wifi_tx, wifi_rx) = channel();

    let vm = unsafe { JavaVM::from_raw(app.vm_as_ptr().cast()).unwrap() };

    // Start sensor, BLE, and Wi-Fi threads
    std::thread::spawn(move || start_sensor_thread(sensor_tx));
    std::thread::spawn(move || start_ble_thread(vm, ble_tx));
    // std::thread::spawn(move || start_wifi_thread(wifi_tx));

    // Initialize egui
    // let event_loop = EventLoop::new().unwrap();

    let options = eframe::NativeOptions {
        android_app: Some(app),
        renderer: eframe::Renderer::Wgpu,
        ..Default::default()
    };

    let app = IndoorLocApp {
        accel_data: [0.0; 3],
        gyro_data: [0.0; 3],
        ble_devices: Vec::new(),
        wifi_aps: Vec::new(),
        position: (0.0, 0.0),
        sensor_rx,
        ble_rx,
        wifi_rx,
    };

    eframe::run_native(
        "Indoor Localization App",
        options,
        Box::new(|_cc| Ok(Box::new(app))),
    )
    .unwrap();
}
