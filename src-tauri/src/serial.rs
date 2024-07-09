use std::{process, sync::mpsc, sync::mpsc::Receiver, thread, time};

use sysinfo::{Networks, System};

use crate::common::{min_f32, ConfigFile, Gauge};

pub fn serial_thread(rx: Receiver<ConfigFile>) {
    let mut serial = None;

    let mut current = ConfigFile {
        power: false,
        port: String::new(),
        gauges: vec![],
        update: 200, // default delay value
    };

    let mut sys = System::new();
    let mut networks = Networks::new_with_refreshed_list();
    sys.refresh_cpu_usage();

    loop {
        thread::sleep(time::Duration::from_millis(current.update));

        match rx.try_recv() {
            Ok(config) => {
                println!("config: {config:?}");
                current = config;

                // close previous port
                // !TODO: only when required
                if serial.is_some() {
                    drop(serial);
                }

                // open new port
                serial = match serialport::new(&current.port, 115200)
                    .timeout(time::Duration::from_millis(10))
                    .open()
                {
                    Ok(p) => Some(p),
                    Err(_e) => None, // !TODO: error dialog
                };
            }
            Err(mpsc::TryRecvError::Empty) => {} // do nothing
            Err(mpsc::TryRecvError::Disconnected) => {
                // !TODO: error dialog
                process::exit(1);
            }
        }

        /* send active gauges to serial port */
        if let Some(ref mut port) = serial {
            #[allow(unused_assignments)]
            let mut packet = [0; 4];

            let mut result = true;

            sys.refresh_cpu();
            sys.refresh_memory();
            networks.refresh();

            for (i, gauge) in current.gauges.iter().enumerate() {
                match gauge {
                    Gauge::Disabled => {}
                    Gauge::CpuUsage { core } => {
                        if *core == -1 {
                            packet = sys.global_cpu_info().cpu_usage().to_le_bytes();
                        } else {
                            packet = sys
                                .cpus()
                                .get(*core as usize)
                                .unwrap()
                                .cpu_usage()
                                .to_le_bytes();
                        }
                    }
                    Gauge::CpuFreq { core } => {
                        if *core == -1 {
                            packet = (sys.global_cpu_info().frequency() as u32).to_le_bytes();
                        } else {
                            packet = (sys.cpus().get(*core as usize).unwrap().frequency() as u32)
                                .to_le_bytes();
                        }
                    }
                    Gauge::CpuTemp => {
                        packet = [0; 4]; // !TODO
                    }
                    Gauge::MemoryUsage => {
                        packet = (sys.used_memory() as f32 / sys.total_memory() as f32 * 100.0)
                            .to_le_bytes();
                    }
                    Gauge::SwapUsage => {
                        packet = (sys.used_swap() as f32 / sys.total_swap() as f32 * 100.0)
                            .to_le_bytes();
                    }
                    Gauge::NetTx { netif, unit } => {
                        if let Some(netif) = networks.get(netif) {
                            let bps =
                                netif.transmitted() as f32 / current.update as f32 / 1000.0 * 8.0;
                            packet =
                                min_f32(bps / *unit as i32 as f32 * 100.0, 100.0).to_le_bytes();
                        } else {
                            packet = [0; 4];
                        }
                    }
                    Gauge::NetRx { netif, unit } => {
                        if let Some(netif) = networks.get(netif) {
                            let bps =
                                netif.received() as f32 / current.update as f32 / 1000.0 * 8.0;
                            packet =
                                min_f32(bps / *unit as i32 as f32 * 100.0, 100.0).to_le_bytes();
                        } else {
                            packet = [0; 4];
                        }
                    }
                    Gauge::NetTxRx { netif, unit } => {
                        if let Some(netif) = networks.get(netif) {
                            let bps = (netif.transmitted() + netif.received()) as f32
                                / current.update as f32
                                / 1000.0
                                * 8.0;
                            packet =
                                min_f32(bps / *unit as i32 as f32 * 100.0, 100.0).to_le_bytes();
                        } else {
                            packet = [0; 4];
                        }
                    }
                    Gauge::DiskUsage { name: _ } => {
                        packet = [0; 4]; // !TODO
                    }
                    Gauge::DiskTx { name: _ } => {
                        packet = [0; 4]; // !TODO
                    }
                    Gauge::DiskRx { name: _ } => {
                        packet = [0; 4]; // !TODO
                    }
                    Gauge::DiskTxRx { name: _ } => {
                        packet = [0; 4]; // !TODO
                    }
                    Gauge::GpuUsage { id: _ } => {
                        packet = [0; 4]; // !TODO
                    }
                    Gauge::GpuFreq { id: _ } => {
                        packet = [0; 4]; // !TODO
                    }
                    Gauge::GpuTemp { id: _ } => {
                        packet = [0; 4]; // !TODO
                    }
                }

                // set port designator
                packet[0] = i as u8;

                // transmit packet
                println!("packet sent: {:?}", packet);
                if let Err(_) = port.write(&packet) {
                    result = false;
                }
            }

            /* serial transmit failed; try to recover */
            if result == false {
                loop {
                    thread::sleep(time::Duration::from_millis(500));

                    serial = match serialport::new(&current.port, 115200)
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
