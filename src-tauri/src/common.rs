use std::{
    fs,
    io::{Error, Write},
    path::PathBuf,
};

use directories::ProjectDirs;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub enum GaugeType {
    CpuUsage { core: i32, value: f32 },
    CpuFreq { core: i32, value: f32 },
    CpuTemp(f32),

    MemoryUsage(f32),
    SwapUsage(f32),

    NetTx { netif: String, value: u64 },
    NetRx { netif: String, value: u64 },
    NetTxRx { netif: String, value: u64 },

    DiskUsage { name: String, value: f32 },
    DiskTx { name: String, value: u64 },
    DiskRx { name: String, value: u64 },
    DiskTxRx { name: String, value: u64 },

    GpuUsage { id: i32, value: f32 },
    GpuFreq { id: i32, value: f32 },
    GpuTemp(f32),
}

#[derive(Serialize, Deserialize)]
pub struct Gauge {
    id: u32,
    kind: GaugeType,
}

#[derive(Serialize, Deserialize)]
pub struct ConfigFile {
    pub port: String,
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

    fn path() -> PathBuf {
        ProjectDirs::from("", "luftaquila", "cpu-meter")
            .unwrap()
            .data_local_dir()
            .to_path_buf()
            .join("config.json")
    }
}
