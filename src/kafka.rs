use rdkafka::{
    config::{FromClientConfig, RDKafkaLogLevel},
    consumer::{CommitMode, Consumer, StreamConsumer},
    error::KafkaError,
    message::BorrowedMessage,
    ClientConfig, Message,
};

use crate::{config::Config, ip_ban, loki_lookup};

pub async fn listen_for_ban_messages(config: &Config) -> Result<(), Box<dyn std::error::Error>> {
    log::info!("Creating consumer");

    let consumer: StreamConsumer = kafka_config(config);

    let topics = [config.topic.as_str()];
    log::info!("Subscribing to topic: {:?}", topics);
    consumer.subscribe(&topics).expect("Failed to subscribe");

    log::info!("Waiting for messages...");

    loop {
        match handle_message(consumer.recv().await, &consumer, config).await {
            Ok(_) => log::info!("Successfully handled message!"),
            Err(e) => log::error!("Failed to handle message: {:?}, {}", e, e.to_string()),
        };
    }
}

async fn handle_message(
    msg_result: Result<BorrowedMessage<'_>, KafkaError>,
    consumer: &StreamConsumer,
    config: &Config,
) -> Result<(), Box<dyn std::error::Error>> {
    let msg = msg_result?;
    let timestamp = msg.timestamp();

    log::info!("Got a new timestamp: {:?}", timestamp);
    let ips_to_ban = loki_lookup::get_ips_to_ban_at_timestamp(timestamp, config).await?;

    for ip in ips_to_ban {
        ip_ban::ban_ip(&ip, config).await?;
    }

    consumer.commit_message(&msg, CommitMode::Sync)?;

    Ok(())
}

fn kafka_config<T: FromClientConfig>(config: &Config) -> T {
    log::info!("Group id is: {}", config.group_id);
    log::info!("Group id is: {}", config.group_id);

    ClientConfig::new()
        .set("group.id", config.group_id.to_owned())
        .set("bootstrap.servers", config.bootstrap_servers.to_owned())
        .set("enable.auto.commit", "false")
        // .set("enable.auto.offset.store", "false")
        // .set("statistics.interval.ms", "0")
        .set("auto.offset.reset", "earliest")
        .set_log_level(RDKafkaLogLevel::Debug)
        .create()
        .expect("Kafka config creation error")
}
