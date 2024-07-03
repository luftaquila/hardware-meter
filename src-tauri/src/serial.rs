use std::sync::mpsc::Receiver;

use sysinfo::{MemoryRefreshKind, Networks, System};

use crate::common::ConfigFile;

pub fn serial_thread(rx: Receiver<ConfigFile>) {
    let mut sys = System::new();
    let mut networks = Networks::new_with_refreshed_list();
    let ram = MemoryRefreshKind::new().with_ram();

    sys.refresh_cpu_usage();
}
