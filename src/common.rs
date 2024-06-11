use std::{
    fs,
    io::{Error, Write},
    path::PathBuf,
};

use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use strum_macros::EnumIter;

pub const UPDATE_MS: u64 = 100;
pub const MAGIC_NUM: isize = 100;

#[derive(Serialize, Deserialize)]
pub struct ConfigFile {
    pub port: String,
    pub netif: String,
    pub netspd: String,
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
            .join("save.json")
    }
}

pub enum ConfigType {
    PORT,
    NETIF,
    NETSPD,
}

pub struct Config {
    pub kind: ConfigType,
    pub data: String,
}

pub enum PacketType {
    CpuUsage = MAGIC_NUM,
    MemoryUsage,
    NetRx,
    NetTx,
    /* TODO: future support, maybe
     * CpuFreq,
     * DiskUsage,
     * DiskRead,
     * DiskWrite,
     *
     * GpuUsage,
     * CpuTemp,
     * GpuTemp,
     * DiskTemp,
     */
}

pub struct Enabled {
    cpu_usage: bool,
    mem_usage: bool,
    net_rx: bool,
    net_tx: bool,
}

#[derive(EnumIter)]
pub enum NetworkSpeed {
    Mbps100 = 100,
    Mbps500 = 500,
    Mbps1000 = 1000,
}

pub fn min_f32(x: f32, y: f32) -> f32 {
    if x < y {
        x
    } else {
        y
    }
}
