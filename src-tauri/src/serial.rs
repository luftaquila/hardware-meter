use std::{process, sync::mpsc, sync::mpsc::Receiver, thread, time};

use sysinfo::{MemoryRefreshKind, Networks, System};

use crate::common::{ConfigFile, Gauge};

pub fn serial_thread(rx: Receiver<ConfigFile>) {
    let mut serial = None;

    let mut current = ConfigFile {
        port: String::new(),
        active: vec![],
        update: 200, // default delay value
    };

    // let mut sys = System::new();
    // let mut networks = Networks::new_with_refreshed_list();
    // let ram = MemoryRefreshKind::new().with_ram();
    //
    // sys.refresh_cpu_usage();

    loop {
        thread::sleep(time::Duration::from_millis(current.update));

        match rx.try_recv() {
            Ok(config) => {
                current = config;

                // close previous port
                if serial.is_some() {
                    drop(serial);
                }

                // open new port
                serial = match serialport::new(&current.port, 115200)
                    .timeout(time::Duration::from_millis(10))
                    .open()
                {
                    Ok(p) => Some(p),
                    Err(e) => None, // !TODO: error dialog
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
            let mut packet: Vec<u8> = vec![0; 4];
            let mut result = true;

            for (i, gauge) in current.active.iter().enumerate() {
                match gauge {
                    Gauge::CpuUsage { core, value } => {}
                    Gauge::CpuFreq { core, value } => {}
                    Gauge::CpuTemp(value) => {}
                    Gauge::MemoryUsage(value) => {}
                    Gauge::SwapUsage(value) => {}
                    Gauge::NetTx { netif, value } => {}
                    Gauge::NetRx { netif, value } => {}
                    Gauge::NetTxRx { netif, value } => {}
                    Gauge::DiskUsage { name, value } => {}
                    Gauge::DiskTx { name, value } => {}
                    Gauge::DiskRx { name, value } => {}
                    Gauge::DiskTxRx { name, value } => {}
                    Gauge::GpuUsage { id, value } => {}
                    Gauge::GpuFreq { id, value } => {}
                    Gauge::GpuTemp(value) => {}
                }

                // set port designator
                packet[0] = i as u8;

                // transmit packet
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
