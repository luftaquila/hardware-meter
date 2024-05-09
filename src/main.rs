use std::{env, thread, time};
use sysinfo::System;

fn main() {
    let mut sys = System::new();

    /* parse port from command line arguments */
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

    sys.refresh_cpu_usage();

    loop {
        thread::sleep(time::Duration::from_millis(100));
        sys.refresh_cpu_usage();

        let usage = sys.global_cpu_info().cpu_usage();
        serial
            .write(&usage.to_le_bytes())
            .expect("[ERR] write failed");
    }
}
