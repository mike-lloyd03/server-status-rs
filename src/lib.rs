use anyhow::Result;
use psutil::{cpu, disk, memory, sensors};
use serde::Deserialize;
use serde_yaml;

#[derive(Debug, Deserialize)]
pub struct Config {
    pub username: String,
    pub password: String,
    pub broker: String,
    pub port: String,
    pub disk_use_percent: bool,
    pub disk_paths: Vec<String>,
    pub processor_use: bool,
    pub processor_temperature: bool,
    pub memory_use: bool,
    pub last_boot: bool,
}

fn round(number: f32, ndigits: usize) -> String {
    format!("{:.n$}", number, n = ndigits)
}

// Requires host feature of psutil which is currently broken
// pub fn get_hostname() -> &str {}

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
    let proc_temp = sensors::temperatures()[0]
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

// Requires host feature of psutil which is currently broken
// pub fn get_last_boot() -> String {}

pub fn load_config() -> Result<Config> {
    use std::env;
    use std::fs::File;
    use std::path::Path;

    let environ = env::var("ENV").unwrap_or("production".to_string());
    let sys_config_path: &Path;

    if environ == "development" {
        sys_config_path = Path::new("config.yaml");
    } else {
        sys_config_path = Path::new("/etc/server_status/config.yaml");
    }

    let f = File::open(sys_config_path)?;

    Ok(serde_yaml::from_reader(f)?)
}
