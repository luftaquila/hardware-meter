#[tauri::command]
pub fn refresh_port() -> String {
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

