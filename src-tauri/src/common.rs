use std::{
    fs,
    io::{Error, Write},
    path::PathBuf,
};

use directories::ProjectDirs;
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Serialize, Deserialize)]
pub enum NetworkSpeed {
    Mbps1 = 1,
    Mbps5 = 5,
    Mbps10 = 10,
    Mbps50 = 50,
    Mbps100 = 100,
    Mbps500 = 500,
    Gbps1 = 1000,
    Gbps5 = 5000,
}

#[derive(Serialize, Deserialize)]
pub enum Gauge {
    CpuUsage { core: i32 },
    CpuFreq { core: i32 },
    CpuTemp,

    MemoryUsage,
    SwapUsage,

    NetTx { netif: String, unit: NetworkSpeed },
    NetRx { netif: String, unit: NetworkSpeed },
    NetTxRx { netif: String, unit: NetworkSpeed },

    DiskUsage { name: String },
    DiskTx { name: String },
    DiskRx { name: String },
    DiskTxRx { name: String },

    GpuUsage { id: i32 },
    GpuFreq { id: i32 },
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

    fn path() -> PathBuf {
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
