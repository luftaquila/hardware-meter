use std::{env, thread, time};

use sysinfo::System;
use tray_item::{IconSource, TrayItem};

fn main() {
    /* create tray icon */
    let mut tray = TrayItem::new(
        "cpu-meter @luftaquila\ngithub.com/luftaquila/cpu-meter",
        IconSource::Resource("tray-default"),
    )
    .unwrap();

    tray.add_menu_item("Quit", || {
        std::process::exit(0);
    })
    .unwrap();

    /* get port from argument */
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        eprintln!("[ERR] no port specified");
        return;
    }

    let port = &args[1];
    let mut serial = serialport::new(port, 115200)
        .timeout(time::Duration::from_millis(10))
        .open()
        .expect("[ERR] cannot open port");

    /* get cpu usage and write it to serial */
    let mut sys = System::new();
    sys.refresh_cpu_usage();

    loop {
        thread::sleep(time::Duration::from_millis(200));
        sys.refresh_cpu_usage();

        let usage = sys.global_cpu_info().cpu_usage();
        serial
            .write(&usage.to_le_bytes())
            .expect("[ERR] write failed");
    }
}
