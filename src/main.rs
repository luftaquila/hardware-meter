#![windows_subsystem = "windows"]

use std::{sync::mpsc, thread, time};

use sysinfo::System;
use tray_item::{IconSource, TrayItem};

fn main() {
    /* create tray icon */
    let mut tray = TrayItem::new("cpu-meter @luftaquila", IconSource::Resource("")).unwrap();

    tray.add_menu_item("About", || {
        open::that("https://github.com/luftaquila/cpu-meter").expect("[ERR] Cannot open browser");
    })
    .unwrap();

    #[cfg(not(target_os = "macos"))]
    tray.inner_mut().add_separator().unwrap();

    /* refresh port not supported as tray-item-rs does not support delete item */
    // tray.inner_mut().add_menu_item("Refresh", || {}).unwrap();

    let (tx, rx) = mpsc::channel();

    let ports = serialport::available_ports().expect("[ERR] No ports found!");
    for p in ports {
        if let serialport::SerialPortType::UsbPort(ref usb) = p.port_type {
            let name = p.port_name.clone();
            let tx = tx.clone();

            #[cfg(target_os = "windows")]
            if let Some(ref product) = usb.product {
                tray.inner_mut()
                    .add_menu_item(&product, move || {
                        tx.send(name.clone()).unwrap();
                    })
                    .unwrap();
            } else {
                tray.inner_mut()
                    .add_menu_item(&p.port_name, move || {
                        tx.send(name.clone()).unwrap();
                    })
                    .unwrap();
            }

            #[cfg(target_os = "macos")]
            if name.contains("cu") {
                // list calling units only
                if let Some(ref product) = usb.product {
                    tray.inner_mut()
                        .add_menu_item(
                            &[
                                product.to_string(),
                                " (".to_string(),
                                p.port_name,
                                ")".to_string(),
                            ]
                            .join(""),
                            move || {
                                tx.send(name.clone()).unwrap();
                            },
                        )
                        .unwrap();
                } else {
                    tray.inner_mut()
                        .add_menu_item(&p.port_name, move || {
                            tx.send(name.clone()).unwrap();
                        })
                        .unwrap();
        }
    }
        }
    }

    #[cfg(not(target_os = "macos"))]
    tray.inner_mut().add_separator().unwrap();

    tray.add_menu_item("Quit", || {
        std::process::exit(0);
    })
    .unwrap();

    /* create serial port thread */
    thread::spawn(move || {
        let mut sys = System::new();
        sys.refresh_cpu_usage();

        let mut serial = None;

        loop {
            thread::sleep(time::Duration::from_millis(200));

            match rx.try_recv() {
                Ok(port_name) => {
                    if serial.is_some() {
                        // close previous port
                        drop(serial);
                    }

                    // open new selected port
                    serial = Some(
                        serialport::new(&port_name, 115200)
                            .timeout(time::Duration::from_millis(10))
                            .open()
                            .expect(&format!("[ERR] cannot open port {}", &port_name)),
                    );
                }
                Err(mpsc::TryRecvError::Empty) => {
                    // do nothing
                }
                Err(mpsc::TryRecvError::Disconnected) => {
                    break;
                }
            }

            // write usage to serial
            if let Some(ref mut port) = serial {
                sys.refresh_cpu_usage();
                let usage = sys.global_cpu_info().cpu_usage();

                port.write(&usage.to_le_bytes())
                    .expect("[ERR] write failed");
            }
        }
    });

    /* this blocks */
    #[cfg(target_os = "macos")]
    tray.inner_mut().display();

    // prevent main thread from exiting
    loop {
        thread::park();
    }
}
