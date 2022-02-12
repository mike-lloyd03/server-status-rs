use anyhow::Result;
// use heim::{cpu, disk};
use paho_mqtt as mqtt;
use psutil::{cpu, disk, memory, sensors};
use serde::Deserialize;
use serde_yaml;

#[derive(Debug, Deserialize)]
struct Config {
    username: String,
    password: String,
    broker: String,
    port: String,
    disk_use_percent: bool,
    disk_paths: Vec<String>,
    processor_use: bool,
    processor_temperature: bool,
    memory_use: bool,
    last_boot: bool,
}

const TOPIC_PREFIX: &str = "server_status/TEST/";

fn main() {
    let config = match load_config() {
        Ok(c) => c,
        Err(err) => {
            eprintln!("Failed to load the configuration file: {:?}", err);
            std::process::exit(1);
        }
    };

    let client = match connect_mqtt(&config) {
        Ok(c) => c,
        Err(err) => {
            eprintln!("Failed to connect to mqtt broker: {:?}", err);
            std::process::exit(1)
        }
    };
    send_status(&config, &client);
    client.disconnect(None).unwrap();
}

fn connect_mqtt(config: &Config) -> Result<mqtt::Client> {
    let host = format!("tcp://{}:{}", &config.broker, &config.port);
    let client = mqtt::Client::new(host)?;
    let options = mqtt::ConnectOptionsBuilder::new()
        .user_name(&config.username)
        .password(&config.password)
        .finalize();
    client.connect(options)?;
    Ok(client)
}

fn send_status(config: &Config, client: &mqtt::Client) {
    loop {
        if config.processor_use {
            let pu_msg = mqtt::Message::new(
                TOPIC_PREFIX.to_owned() + "processor_use",
                get_processor_use(15),
                2,
            );
            client.publish(pu_msg).unwrap()
        } else {
            std::thread::sleep(std::time::Duration::from_secs(15))
        }

        if config.processor_temperature {
            //     let pt_msg = mqtt::Message::new(
            //         TOPIC_PREFIX.to_owned() + "processor_temperature",
            //         get_processor_temperature(),
            //         2,
            //     );
            //     client.publish(pt_msg).unwrap();
        }

        if config.disk_use_percent {
            for (i, path) in config.disk_paths.iter().enumerate() {
                let du_msg = mqtt::Message::new(
                    TOPIC_PREFIX.to_owned() + "disk_use_percent_disk" + &i.to_string(),
                    get_disk_use_percent(path),
                    2,
                );
                client.publish(du_msg).unwrap();
            }
        }

        if config.memory_use {
            let mu_msg =
                mqtt::Message::new(TOPIC_PREFIX.to_owned() + "memory_use", get_memory_use(), 2);
            client.publish(mu_msg).unwrap();
        }
    }
}

fn round(number: f32, ndigits: usize) -> String {
    format!("{:.n$}", number, n = ndigits)
}

fn round64(number: f64, ndigits: usize) -> String {
    format!("{:.n$}", number, n = ndigits)
}

// Requires host feature of psutil which is currently broken
// fn get_hostname() -> &str {}

fn get_disk_use_percent(path: &str) -> String {
    let percent = disk::disk_usage(path).unwrap().percent();
    round(percent, 1)
}

fn get_processor_use(interval: u64) -> String {
    let mut collector = cpu::CpuPercentCollector::new().unwrap();
    std::thread::sleep(std::time::Duration::from_secs(interval));
    let proc_use = collector.cpu_percent().unwrap();
    round(proc_use, 1)
}

// fn get_processor_temperature() -> String {
//     let proc_temp = sensors::temperatures()[0].unwrap();
//     round64(proc_temp.current().celsius(), 1)
// }

fn get_memory_use() -> String {
    let virt_mem = memory::virtual_memory().unwrap().used();
    round(virt_mem as f32 / 1024.0 / 1024.0, 1)
}

// Requires host feature of psutil which is currently broken
// fn get_last_boot() -> String {}

fn load_config() -> Result<Config> {
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
