use strum::IntoEnumIterator;
use sysinfo::{Networks, System};

use crate::common::{ConfigFile, Gauge, NetworkSpeed, MAX_CHANNEL};

#[tauri::command]
pub fn get_channel_count() -> i32 {
    MAX_CHANNEL
}

#[tauri::command]
pub fn get_ports() -> String {
    match serialport::available_ports() {
        Ok(ports) => {
            let mut html = String::from("<option disabled selected>Select serial port</option>");
            for p in ports {
                if let serialport::SerialPortType::UsbPort(ref usb) = p.port_type {
                    #[cfg(target_os = "windows")]
                    if let Some(ref product) = usb.product {
                        html.push_str(&format!(
                            "<option value='{}'>{}</option>",
                            p.port_name, &product
                        ));
                    } else {
                        html.push_str(&format!(
                            "<option value='{}'>{}</option>",
                            p.port_name, p.port_name
                        ));
                    }

                    #[cfg(any(target_os = "macos", target_os = "linux"))]
                    if p.port_name.contains("cu") {
                        if let Some(ref product) = usb.product {
                            html.push_str(&format!(
                                "<option value='{}'>{} ({})</option>",
                                p.port_name, &product, p.port_name
                            ));
                        } else {
                            html.push_str(&format!(
                                "<option value='{}'>{}</option>",
                                p.port_name, p.port_name
                            ));
                        }
                    }
                }
            }

            html
        }
        Err(_) => String::from("<option disabled selected>No ports found!</option>"),
    }
}

#[tauri::command]
pub fn open_config_dir() {
    open::that(ConfigFile::path()).ok();
}

#[tauri::command]
pub fn get_gauge_types() -> Vec<String> {
    let mut ret = vec![];
    for gauge in Gauge::iter() {
        ret.push(gauge.as_ref().to_string());
    }
    ret
}

#[tauri::command]
pub fn get_core_count() -> i32 {
    let mut sys = System::new();

    sys.refresh_cpu();
    sys.cpus().len() as i32
}

#[tauri::command]
pub fn get_netifs() -> Vec<String> {
    Networks::new_with_refreshed_list().list().keys().cloned().collect()
}

#[tauri::command]
pub fn get_speed_units() -> Vec<String> {
    let mut ret = vec![];
    for spd in NetworkSpeed::iter() {
        ret.push(spd.as_ref().to_string());
    }
    ret
}
