#![windows_subsystem = "windows"]

use std::{
    fs,
    io::{Error, Write},
    path::PathBuf,
    process,
    sync::mpsc,
    thread, time,
};

use directories::ProjectDirs;
use sysinfo::{MemoryRefreshKind, Networks, System};
use tray_item::{IconSource, TrayItem};

use strum::IntoEnumIterator;
use strum_macros::EnumIter;

use serde::{Deserialize, Serialize};

const UPDATE_MS: u64 = 100;
const MAGIC_NUM: isize = 100;

#[derive(Serialize, Deserialize)]
struct ConfigFile {
    port: String,
    netif: String,
    netspd: String,
}

impl ConfigFile {
    fn to_json(&self) -> Result<(), Error> {
        let mut file = fs::File::create(ConfigFile::path())?;

        let serialized = serde_json::to_string(self)?;
        file.write_all(serialized.as_bytes())?;

        Ok(())
    }

    fn from_json() -> Result<ConfigFile, Error> {
        let json = fs::read_to_string(ConfigFile::path())?;

        Ok(serde_json::from_str(&json)?)
    }

    fn path() -> PathBuf {
        ProjectDirs::from("", "luftaquila", "cpu-meter")
            .unwrap()
            .data_local_dir()
            .to_path_buf()
            .join("save.json")
    }
}

enum ConfigType {
    PORT,
    NETIF,
    NETSPD,
}

struct Config {
    kind: ConfigType,
    data: String,
}

enum PacketType {
    CPU = MAGIC_NUM,
    MEMORY,
    RX,
    TX,
}

#[derive(EnumIter)]
enum NetworkSpeed {
    Mbps100 = 100,
    Mbps500 = 500,
    Mbps1000 = 1000,
}

fn min_f32(x: f32, y: f32) -> f32 {
    if x < y {
        x
    } else {
        y
    }
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
    thread::spawn(move || {
        let mut sys = System::new();
        let mut networks = Networks::new_with_refreshed_list();
        let ram = MemoryRefreshKind::new().with_ram();

        sys.refresh_cpu_usage();

        let mut serial = None;
        let mut name = String::new();

        let mut interface = None;
        let mut speed = 100; // default 100 Mbps

        loop {
            thread::sleep(time::Duration::from_millis(UPDATE_MS));

            match rx.try_recv() {
                Ok(config) => {
                    match config.kind {
                        ConfigType::PORT => {
                            if serial.is_some() {
                                // close previous port
                                drop(serial);
                            }

                            // open new selected port
                            name = config.data.clone();
                            serial = match serialport::new(&config.data, 115200)
                                .timeout(time::Duration::from_millis(10))
                                .open()
                            {
                                Ok(p) => Some(p),
                                Err(e) => {
                                    eprintln!("[ERR] port {} open failed: {}", config.data, e);
                                    None
                                }
                            };

                            if serial.is_none() {
                                continue;
                            }
                        }
                        ConfigType::NETIF => {
                            interface = Some(config.data.clone());
                        }
                        ConfigType::NETSPD => {
                            speed = config.data.parse().unwrap();
                        }
                    }

                    /* save config to file */
                    let mut save = match ConfigFile::from_json() {
                        Ok(file) => file,
                        Err(_) => ConfigFile {
                            port: String::new(),
                            netif: String::new(),
                            netspd: String::new(),
                        },
                    };

                    match config.kind {
                        ConfigType::PORT => save.port = config.data,
                        ConfigType::NETIF => save.netif = config.data,
                        ConfigType::NETSPD => save.netspd = config.data,
                    }

                    let _ = save.to_json();
                }
                Err(mpsc::TryRecvError::Empty) => {} // do nothing
                Err(mpsc::TryRecvError::Disconnected) => {
                    eprintln!("[ERR] thread comm disconnected!");
                    process::exit(1);
                }
            }

            // write usage to serial
            if let Some(ref mut port) = serial {
                sys.refresh_cpu_usage();
                let mut cpu = sys.global_cpu_info().cpu_usage().to_le_bytes();
                cpu[0] = PacketType::CPU as u8;

                sys.refresh_memory_specifics(ram);
                let mut mem =
                    (sys.used_memory() as f32 / sys.total_memory() as f32 * 100.0).to_le_bytes();
                mem[0] = PacketType::MEMORY as u8;

                networks.refresh();

                let mut result = true;

                if interface.is_some() {
                    if let Some(network) = networks.get(&interface.clone().unwrap()) {
                        let rx_bps = network.received() as f32 / UPDATE_MS as f32 / 1000.0 * 8.0;
                        let mut net_rx =
                            min_f32(rx_bps / speed as f32 * 100.0, 100.0).to_le_bytes();
                        net_rx[0] = PacketType::RX as u8;

                        if let Err(_) = port.write(&net_rx) {
                            result = false;
                        }

                        let tx_bps = network.transmitted() as f32 / UPDATE_MS as f32 / 1000.0 * 8.0;
                        let mut net_tx =
                            min_f32(tx_bps / speed as f32 * 100.0, 100.0).to_le_bytes();
                        net_tx[0] = PacketType::TX as u8;

                        if let Err(_) = port.write(&net_tx) {
                            result = false;
                        }
                    }
                }

                if let Err(_) = port.write(&cpu) {
                    result = false;
                }

                if let Err(_) = port.write(&mem) {
                    result = false;
                }

                if result == false {
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
    });

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
