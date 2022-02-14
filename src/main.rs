use anyhow::Result;
// use heim::{cpu, disk};
use paho_mqtt as mqtt;
use server_status_rs::*;

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
    client
        .disconnect(None)
        .expect("Failed to disconnect from the MQTT broker");
}

/// Connect to the MQTT broker and return the client
fn connect_mqtt(config: &Config) -> Result<mqtt::Client> {
    let port = &config.port.unwrap_or(1883);
    let host = format!("tcp://{}:{}", &config.broker, port);
    let client = mqtt::Client::new(host)?;
    let options = mqtt::ConnectOptionsBuilder::new()
        .user_name(&config.username)
        .password(&config.password)
        .finalize();
    client.connect(options)?;
    Ok(client)
}

/// Send each configured message to the MQTT client
fn send_status(config: &Config, client: &mqtt::Client) {
    let prefix = &format!("server_status/{}/", get_hostname());

    loop {
        if config.processor_use.unwrap_or_default() {
            let key = "processor_use";
            let msg = mqtt::Message::new(prefix.to_owned() + key, get_processor_use(15), 2);
            match client.publish(msg.clone()) {
                Ok(_) => (),
                Err(_) => eprintln!("Failed to send message, {}: {}", key, msg.to_string()),
            }
        } else {
            std::thread::sleep(std::time::Duration::from_secs(15))
        }

        if config.disk_use_percent.unwrap_or_default() {
            let default = vec!["/".to_string()];
            let paths = config.disk_paths.as_ref().unwrap_or(&default);

            for (i, path) in paths.iter().enumerate() {
                let key = format!("disk_use_percent_disk{}", i);
                let msg =
                    mqtt::Message::new(prefix.to_owned() + &key, get_disk_use_percent(path), 2);
                match client.publish(msg.clone()) {
                    Ok(_) => (),
                    Err(_) => eprintln!("Failed to send message, {}: {}", key, msg.to_string()),
                }
            }
        }

        if config.processor_temperature.unwrap_or_default() {
            send_msg(&client, "processor_temperature", get_processor_temperature);
        }

        if config.memory_use.unwrap_or_default() {
            send_msg(&client, "memory_use", get_memory_use);
        }

        if config.last_boot.unwrap_or_default() {
            send_msg(&client, "last_boot", get_last_boot);
        }
    }
}

/// Formats and sends an mqtt message with the specified key and the result of the val_fn.
fn send_msg(client: &mqtt::Client, key: &str, val_fn: fn() -> String) {
    let prefix = format!("server_status/{}/", get_hostname());
    let msg = mqtt::Message::new(prefix + key, val_fn(), 2);
    match client.publish(msg.clone()) {
        Ok(_) => (),
        Err(_) => eprintln!("Failed to send message, {}: {}", key, msg.to_string()),
    }
}
