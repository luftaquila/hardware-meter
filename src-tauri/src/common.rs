use std::{
    fs,
    io::{Error, Write},
    path::PathBuf,
};

use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use strum_macros::{AsRefStr, EnumIter};

pub const MAX_CHANNEL:i32 = 6;

#[derive(Clone, Copy, Default, EnumIter, AsRefStr, Serialize, Deserialize)]
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

#[derive(EnumIter, AsRefStr, Serialize, Deserialize)]
pub enum Gauge {
    #[strum(serialize = "CPU Utilization")]
    CpuUsage { core: i32 },
    #[strum(serialize = "CPU Frequency")]
    CpuFreq { core: i32 },
    #[strum(serialize = "CPU Temperature")]
    CpuTemp,

    #[strum(serialize = "Memory Usage")]
    MemoryUsage,
    #[strum(serialize = "Swap Usage")]
    SwapUsage,

    #[strum(serialize = "Network Transmit Speed")]
    NetTx { netif: String, unit: NetworkSpeed },
    #[strum(serialize = "Network Receive Speed")]
    NetRx { netif: String, unit: NetworkSpeed },
    #[strum(serialize = "Network Receive & Transmit Speed")]
    NetTxRx { netif: String, unit: NetworkSpeed },

    #[strum(serialize = "Disk Usage")]
    DiskUsage { name: String },
    #[strum(serialize = "Disk Write Speed")]
    DiskTx { name: String },
    #[strum(serialize = "Disk Read Speed")]
    DiskRx { name: String },
    #[strum(serialize = "Disk Read & Write Speed")]
    DiskTxRx { name: String },

    #[strum(serialize = "GPU Utilization")]
    GpuUsage { id: i32 },
    #[strum(serialize = "GPU Core Frequency")]
    GpuFreq { id: i32 },
    #[strum(serialize = "GPU Temperature")]
    GpuTemp { id: i32 },
}

#[derive(Serialize, Deserialize)]
pub struct ConfigFile {
    pub port: String,
    pub active: Vec<Gauge>,
    pub update: u64,
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
