use anyhow::{bail, Result};
use chrono::{Local, TimeZone, Utc};
use psutil::{cpu, disk, host, memory, sensors};
use serde::Deserialize;
use serde_yaml;
use std::{fmt::Debug, time::UNIX_EPOCH};

#[derive(Debug, Deserialize)]
pub struct Config {
    pub username: String,
    pub password: String,
    pub broker: String,
    pub port: Option<u16>,
    pub disk_use_percent: Option<bool>,
    pub disk_paths: Option<Vec<String>>,
    pub processor_use: Option<bool>,
    pub processor_temperature: Option<bool>,
    pub memory_use: Option<bool>,
    pub last_boot: Option<bool>,
    pub hostname: Option<bool>,
}

fn round(number: f32, ndigits: usize) -> String {
    format!("{:.n$}", number, n = ndigits)
}

pub fn get_hostname() -> String {
    host::info().hostname().to_string()
}

pub fn get_disk_use_percent(path: &str) -> String {
    let percent = disk::disk_usage(path).unwrap().percent();
    round(percent, 1)
}

pub fn get_processor_use(interval: u64) -> String {
    let mut collector = cpu::CpuPercentCollector::new().unwrap();
    std::thread::sleep(std::time::Duration::from_secs(interval));
    let proc_use = collector.cpu_percent().unwrap();
    round(proc_use, 1)
}

pub fn get_processor_temperature() -> String {
    let proc_temp = sensors::temperatures()
        .last()
        .unwrap()
        .as_ref()
        .unwrap()
        .current()
        .celsius();
    round(proc_temp as f32, 1)
}

pub fn get_memory_use() -> String {
    let virt_mem = memory::virtual_memory().unwrap().used();
    round(virt_mem as f32 / 1024.0 / 1024.0, 1)
}

pub fn get_last_boot() -> String {
    let unix_boot_time = host::boot_time().unwrap();
    let boot_time =
        chrono::Utc.timestamp(unix_boot_time.duration_since(UNIX_EPOCH).unwrap().into(), 0);
    // let boot_time = Utc
    //     .timestamp(unix_boot_time.duration_since(UNIX_EPOCH).unwrap())
    //     .unwrap();
    // boot_time.format()
    format!("{}", boot_time.to_rfc3339())
}

pub fn load_config() -> Result<Config> {
    use std::fs::File;
    use std::path::Path;

    let config_paths = vec!["/etc/server_status/config.yaml", "config.yaml"];
    let f: File;

    for path in config_paths {
        if Path::new(path).exists() {
            f = File::open(path)?;
            return Ok(serde_yaml::from_reader(f)?);
        }
    }
    bail!("No configuration file was found")
}
