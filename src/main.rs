#![windows_subsystem = "windows"]

mod common;
mod serial;

use std::{sync::mpsc, thread};
use strum::IntoEnumIterator;
use sysinfo::Networks;
use tray_item::{IconSource, TrayItem};

use crate::{
    common::{Config, ConfigFile, ConfigType, NetworkSpeed},
    serial::serial_thread,
};

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

    /* serial port selector */

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
                        tx.send(Config {
                            kind: ConfigType::PORT,
                            data: name.clone(),
                        })
                        .unwrap();
                    })
                    .unwrap();
            } else {
                tray.inner_mut()
                    .add_menu_item(&p.port_name, move || {
                        tx.send(Config {
                            kind: ConfigType::PORT,
                            data: name.clone(),
                        })
                        .unwrap();
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
                                tx.send(Config {
                                    kind: ConfigType::PORT,
                                    data: name.clone(),
                                })
                                .unwrap();
                            },
                        )
                        .unwrap();
                } else {
                    tray.inner_mut()
                        .add_menu_item(&p.port_name, move || {
                            tx.send(Config {
                                kind: ConfigType::PORT,
                                data: name.clone(),
                            })
                            .unwrap();
                        })
                        .unwrap();
                }
            }
        }
    }

    #[cfg(not(target_os = "macos"))]
    tray.inner_mut().add_separator().unwrap();

    /* network interface selector */
    let networks = Networks::new_with_refreshed_list();
    let networks: Vec<String> = networks.list().keys().cloned().collect();

    for network in networks {
        let tx = tx.clone();
        tray.inner_mut()
            .add_menu_item(&network.clone(), move || {
                tx.send(Config {
                    kind: ConfigType::NETIF,
                    data: network.to_string().clone(),
                })
                .unwrap();
            })
            .unwrap();
    }

    #[cfg(not(target_os = "macos"))]
    tray.inner_mut().add_separator().unwrap();

    /* network speed selector */
    for spd in NetworkSpeed::iter() {
        let tx = tx.clone();
        let spd = (spd as i32).to_string();

        tray.inner_mut()
            .add_menu_item(&(spd.clone() + " Mbps"), move || {
                tx.send(Config {
                    kind: ConfigType::NETSPD,
                    data: spd.clone(),
                })
                .unwrap();
            })
            .unwrap();
    }

    #[cfg(not(target_os = "macos"))]
    tray.inner_mut().add_separator().unwrap();

    tray.add_menu_item("Quit", || {
        std::process::exit(0);
    })
    .unwrap();

    /* create serial port thread */
    thread::spawn(move || serial_thread(rx));

    /* read previous config from file */
    if let Ok(json) = ConfigFile::from_json() {
        if !json.port.is_empty() {
            tx.send(Config {
                kind: ConfigType::PORT,
                data: json.port,
            })
            .unwrap();
        }

        if !json.netif.is_empty() {
            tx.send(Config {
                kind: ConfigType::NETIF,
                data: json.netif,
            })
            .unwrap();
        }

        if !json.netspd.is_empty() {
            tx.send(Config {
                kind: ConfigType::NETSPD,
                data: json.netspd,
            })
            .unwrap();
        }
    }

    /* this blocks */
    #[cfg(target_os = "macos")]
    tray.inner_mut().display();

    // prevent main thread from exiting
    loop {
        thread::park();
    }
}
