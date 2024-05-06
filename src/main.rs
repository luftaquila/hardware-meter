use std::{thread, time};
use sysinfo::System;

fn main() {
    let mut sys = System::new();
    sys.refresh_cpu_usage();

    loop {
        thread::sleep(time::Duration::from_secs(1));
        sys.refresh_cpu_usage();
        println!("{:?}", sys.global_cpu_info().cpu_usage());
    }
}
