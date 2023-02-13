mod config;
mod ip_ban;
mod kafka;
mod loki_lookup;
mod update_alias;

#[tokio::main]
async fn main() -> std::result::Result<(), Box<dyn std::error::Error>> {
    log4rs::init_file("config/log4rs.yml", Default::default())
        .expect("Failed to read log4rs config");
    let config = config::Config::new();

    kafka::listen_for_ban_messages(&config).await?;

    Ok(())
}
