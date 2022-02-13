use anyhow::Result;
// use heim::{cpu, disk};
use paho_mqtt as mqtt;
use server_status_rs::{
    get_disk_use_percent, get_memory_use, get_processor_temperature, get_processor_use,
    load_config, Config,
};

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
            let pt_msg = mqtt::Message::new(
                TOPIC_PREFIX.to_owned() + "processor_temperature",
                get_processor_temperature(),
                2,
            );
            client.publish(pt_msg).unwrap();
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
