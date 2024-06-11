use std::{io::Write, process, sync::mpsc, sync::mpsc::Receiver, thread, time};
use sysinfo::{MemoryRefreshKind, Networks, System};

use crate::common::{min_f32, Config, ConfigFile, ConfigType, PacketType, UPDATE_MS};

pub fn serial_thread(rx: Receiver<Config>) {
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
            cpu[0] = PacketType::CpuUsage as u8;

            sys.refresh_memory_specifics(ram);
            let mut mem =
                (sys.used_memory() as f32 / sys.total_memory() as f32 * 100.0).to_le_bytes();
            mem[0] = PacketType::MemoryUsage as u8;

            networks.refresh();

            let mut result = true;

            if interface.is_some() {
                if let Some(network) = networks.get(&interface.clone().unwrap()) {
                    let rx_bps = network.received() as f32 / UPDATE_MS as f32 / 1000.0 * 8.0;
                    let mut net_rx = min_f32(rx_bps / speed as f32 * 100.0, 100.0).to_le_bytes();
                    net_rx[0] = PacketType::NetRx as u8;

                    if let Err(_) = port.write(&net_rx) {
                        result = false;
                    }

                    let tx_bps = network.transmitted() as f32 / UPDATE_MS as f32 / 1000.0 * 8.0;
                    let mut net_tx = min_f32(tx_bps / speed as f32 * 100.0, 100.0).to_le_bytes();
                    net_tx[0] = PacketType::NetTx as u8;

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
}
