use strum::{EnumMessage, IntoEnumIterator};
use sysinfo::{Networks, System};

use crate::common::{ConfigFile, Gauge, NetworkSpeed, MAX_CHANNEL};

#[tauri::command]
pub fn config(conf: ConfigFile) {
    if let Err(_e) = conf.to_json() {
        // !TODO: error handling
    }
}

#[tauri::command]
pub fn get_channel_count() -> i32 {
    MAX_CHANNEL
}

#[tauri::command]
pub fn get_core_count() -> i32 {
    let mut sys = System::new();

    sys.refresh_cpu();
    sys.cpus().len() as i32
}

#[tauri::command]
pub fn get_gauge_types() -> (Vec<String>, Vec<String>) {
    let mut name = vec![];
    let mut desc = vec![];

    for gauge in Gauge::iter() {
        name.push(gauge.as_ref().to_string());
        desc.push(gauge.get_detailed_message().unwrap().to_string());
    }

    (name, desc)
}

#[tauri::command]
pub fn get_netifs() -> Vec<String> {
    Networks::new_with_refreshed_list()
        .list()
        .keys()
        .cloned()
        .collect()
}

#[tauri::command]
pub fn get_ports() -> Vec<(String, String)> {
    let mut ret = vec![];

    match serialport::available_ports() {
        Ok(ports) => {
            for p in ports {
                if let serialport::SerialPortType::UsbPort(ref usb) = p.port_type {
                    #[cfg(target_os = "windows")]
                    if let Some(ref product) = usb.product {
                        ret.push((p.port_name, product.clone()));
                    } else {
                        ret.push((p.port_name.clone(), p.port_name));
                    }

                    #[cfg(any(target_os = "macos", target_os = "linux"))]
                    if p.port_name.contains("cu") {
                        if let Some(ref product) = usb.product {
                            ret.push((p.port_name, &format!("{} ({})", &product, p.port_name)));
                        } else {
                            ret.push((p.port_name, p.port_name));
                        }
                    }
                }
            }

            ret
        }
        Err(_) => ret,
    }
}

#[tauri::command]
pub fn get_speed_units() -> (Vec<String>, Vec<String>) {
    let mut name = vec![];
    let mut desc = vec![];

    for spd in NetworkSpeed::iter() {
        name.push(spd.as_ref().to_string());
        desc.push(spd.get_detailed_message().unwrap().to_string());
    }

    (name, desc)
}

#[tauri::command]
pub fn open_config_dir() {
    open::that(ConfigFile::path()).ok();
}
