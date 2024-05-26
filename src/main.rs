#![windows_subsystem = "windows"]

use std::{fs, io::Write, process, sync::mpsc, thread, time};

use directories::ProjectDirs;
use sysinfo::System;
use tray_item::{IconSource, TrayItem};

macro_rules! config {
    () => {{
        ProjectDirs::from("", "luftaquila", "cpu-meter")
            .unwrap()
            .data_local_dir()
            .to_path_buf()
    }};
}

fn main() {
    /* create tray icon */
    #[cfg(target_os = "windows")]
    let mut tray = TrayItem::new(
        "cpu-meter @luftaquila",
        IconSource::Resource("tray-default"),
    )
    .unwrap();

    #[cfg(target_os = "macos")]
    let mut tray = TrayItem::new("cpu-meter @luftaquila", IconSource::Resource("")).unwrap();

    #[cfg(target_os = "linux")]
    let mut tray = TrayItem::new(
        "cpu-meter @luftaquila",
        IconSource::Resource("tray-default"),
    )
    .unwrap();

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

            #[cfg(any(target_os = "macos", target_os = "linux"))]
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
        let mut name = String::new();

        loop {
            thread::sleep(time::Duration::from_millis(200));

            match rx.try_recv() {
                Ok(port_name) => {
                    if serial.is_some() {
                        // close previous port
                        drop(serial);
                    }

                    // open new selected port
                    name = port_name.clone();
                    serial = match serialport::new(&port_name, 115200)
                        .timeout(time::Duration::from_millis(10))
                        .open()
                    {
                        Ok(p) => Some(p),
                        Err(_) => process::exit(1),
                    };

                    // write latest port to file
                    let config = config!();
                    fs::create_dir_all(config.clone()).unwrap();

                    if !config.join("prev.txt").exists() {
                        let mut file = fs::File::create(&config.join("prev.txt")).unwrap();
                        file.write_all(port_name.as_bytes()).unwrap();
                    }
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

                match port.write(&usage.to_le_bytes()) {
                    Ok(_) => {}
                    Err(_) => {
                        /* try recover serial */
                        loop {
                            thread::sleep(time::Duration::from_millis(500));

                            serial = match serialport::new(name.as_str(), 115200)
                                .timeout(time::Duration::from_millis(10))
                                .open()
                            {
                                Ok(s) => Some(s),
                                Err(_) => None,
                            };

                            if serial.is_some() {
                                break;
                            }
                        }
                    }
                }
            }
        }
    });

    /* read previous port from file */
    let prev = config!().join("prev.txt");

    if prev.exists() {
        let prev = fs::read_to_string(&prev).unwrap();
        tx.send(prev).unwrap();
    }

    /* this blocks */
    #[cfg(target_os = "macos")]
    tray.inner_mut().display();

    // prevent main thread from exiting
    loop {
        thread::park();
    }
}
