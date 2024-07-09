#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod command;
mod common;
mod serial;

use std::{sync::mpsc, thread};

use notify::RecursiveMode;
use notify_debouncer_mini::new_debouncer;
use tauri::Manager;
use tauri::{CustomMenuItem, SystemTray, SystemTrayEvent, SystemTrayMenu, SystemTrayMenuItem};

use crate::{
    command::{
        config, get_channel_count, get_core_count, get_gauge_types, get_netifs, get_ports,
        get_speed_units, open_config_dir,
    },
    common::ConfigFile,
    serial::serial_thread,
};

fn main() {
    let mut config_exists = false;
    let (tx, rx) = mpsc::channel();

    /* create config file monitor */
    let ctx = tx.clone();
    let (etx, erx) = mpsc::channel();
    let mut debouncer = new_debouncer(std::time::Duration::from_millis(100), etx).unwrap();

    debouncer
        .watcher()
        .watch(&ConfigFile::dir(), RecursiveMode::NonRecursive)
        .unwrap();

    thread::spawn(move || {
        for result in erx {
            match result {
                Ok(events) => events.iter().for_each(|event| {
                    if event.path == ConfigFile::path() {
                        if let Ok(file) = ConfigFile::from_json() {
                            ctx.send(file).unwrap();
                        }
                    }
                }),
                Err(e) => {
                    // !TODO: error handling
                }
            }
        }
    });

    /* create serial port thread */
    thread::spawn(move || serial_thread(rx));

    /* use previous config file if exists */
    if let Ok(file) = ConfigFile::from_json() {
        config_exists = true;
        tx.send(file).unwrap();
    }

    /* set system tray menu */
    let tray_menu = SystemTrayMenu::new()
        .add_item(CustomMenuItem::new("about".to_string(), "About"))
        .add_native_item(SystemTrayMenuItem::Separator)
        .add_item(CustomMenuItem::new("settings".to_string(), "Settings"))
        .add_native_item(SystemTrayMenuItem::Separator)
        .add_item(CustomMenuItem::new("quit".to_string(), "Quit"));

    tauri::Builder::default()
        .system_tray(SystemTray::new().with_menu(tray_menu))
        .on_system_tray_event(|app, event| match event {
            SystemTrayEvent::LeftClick {
                position: _,
                size: _,
                ..
            } => {
                app.get_window("main").unwrap().show().unwrap();
            }
            SystemTrayEvent::MenuItemClick { id, .. } => match id.as_str() {
                "about" => {
                    open::that("https://github.com/luftaquila/cpu-meter").unwrap();
                }
                "settings" => {
                    app.get_window("main").unwrap().show().unwrap();
                }
                "quit" => {
                    std::process::exit(0);
                }
                _ => {}
            },
            _ => {}
        })
        .on_window_event(|event| match event.event() {
            tauri::WindowEvent::CloseRequested { api, .. } => {
                event.window().hide().unwrap();
                api.prevent_close();
            }
            _ => {}
        })
        .invoke_handler(tauri::generate_handler![
            config,
            get_core_count,
            get_channel_count,
            get_gauge_types,
            get_netifs,
            get_speed_units,
            get_ports,
            open_config_dir
        ])
        .build(tauri::generate_context!())
        .expect("error while building tauri application")
        .run(move |app, event| match event {
            tauri::RunEvent::Ready => {
                // show settings on launch only if there is no valid config file
                if !config_exists {
                    app.get_window("main").unwrap().show().unwrap();
                }
            }
            tauri::RunEvent::ExitRequested { api, .. } => {
                api.prevent_exit();
            }
            _ => {}
        });
}
