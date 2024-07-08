use std::{
    fs,
    io::{Error, Write},
    path::PathBuf,
};

use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use strum_macros::{AsRefStr, EnumIter, EnumMessage};

pub const MAX_CHANNEL: i32 = 6;

#[derive(Clone, Copy, Default, AsRefStr, EnumIter, Serialize, Deserialize)]
pub enum NetworkSpeed {
    #[strum(serialize = "1 Mbps")]
    Mbps1 = 1,
    #[strum(serialize = "5 Mbps")]
    Mbps5 = 5,
    #[strum(serialize = "10 Mbps")]
    Mbps10 = 10,
    #[strum(serialize = "50 Mbps")]
    Mbps50 = 50,
    #[default]
    #[strum(serialize = "100 Mbps")]
    Mbps100,
    #[strum(serialize = "500 Mbps")]
    Mbps500 = 500,
    #[strum(serialize = "1 Gbps")]
    Gbps1 = 1000,
    #[strum(serialize = "5 Gbps")]
    Gbps5 = 5000,
}

#[derive(AsRefStr, EnumIter, EnumMessage, Serialize, Deserialize)]
pub enum Gauge {
    #[strum(message = "CpuUsage", detailed_message = "CPU Utilization")]
    CpuUsage { core: i32 },
    #[strum(message = "CpuFreq", detailed_message = "CPU Frequency")]
    CpuFreq { core: i32 },
    #[strum(message = "CpuTemp", detailed_message = "CPU Temperature")]
    CpuTemp,

    #[strum(message = "MemoryUsage", detailed_message = "Memory Usage")]
    MemoryUsage,
    #[strum(message = "SwapUsage", detailed_message = "Swap Usage")]
    SwapUsage,

    #[strum(message = "NetTx", detailed_message = "Network Transmit Speed")]
    NetTx { netif: String, unit: NetworkSpeed },
    #[strum(message = "NetRx", detailed_message = "Network Receive Speed")]
    NetRx { netif: String, unit: NetworkSpeed },
    #[strum(message = "NetTxRx", detailed_message = "Network Receive & Transmit Speed")]
    NetTxRx { netif: String, unit: NetworkSpeed },

    #[strum(message = "DiskUsage", detailed_message = "Disk Usage")]
    DiskUsage { name: String },
    #[strum(message = "DiskTx", detailed_message = "Disk Write Speed")]
    DiskTx { name: String },
    #[strum(message = "DiskRx", detailed_message = "Disk Read Speed")]
    DiskRx { name: String },
    #[strum(message = "DiskTxRx", detailed_message = "Disk Read & Write Speed")]
    DiskTxRx { name: String },

    #[strum(message = "GpuUsage", detailed_message = "GPU Utilization")]
    GpuUsage { id: i32 },
    #[strum(message = "GpuFreq", detailed_message = "GPU Core Frequency")]
    GpuFreq { id: i32 },
    #[strum(message = "GpuTemp", detailed_message = "GPU Temperature")]
    GpuTemp { id: i32 },
}

#[derive(Serialize, Deserialize)]
pub struct ConfigFile {
    pub power: bool,
    pub port: String,
    pub update: u64,
    pub active: Vec<Gauge>,
}

impl ConfigFile {
    pub fn to_json(&self) -> Result<(), Error> {
        let mut file = fs::File::create(ConfigFile::path())?;

        let serialized = serde_json::to_string(self)?;
        file.write_all(serialized.as_bytes())?;

        Ok(())
    }

    pub fn from_json() -> Result<ConfigFile, Error> {
        let json = fs::read_to_string(ConfigFile::path())?;

        Ok(serde_json::from_str(&json)?)
    }

    pub fn path() -> PathBuf {
        ProjectDirs::from("", "luftaquila", "cpu-meter")
            .unwrap()
            .data_local_dir()
            .to_path_buf()
            .join("config.json")
    }
}

pub fn min_f32(x: f32, y: f32) -> f32 {
    if x < y {
        x
    } else {
        y
    }
}
