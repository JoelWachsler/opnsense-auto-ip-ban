use serde::{Deserialize, Serialize};

use crate::config::Config;

#[derive(Debug, Deserialize)]
struct InputAlias {
    alias: InputAliasAlias,
}

#[derive(Debug, Deserialize)]
struct InputAliasAlias {
    enabled: String,
    name: String,
    content: std::collections::BTreeMap<String, ValueSelect>,
}

#[derive(Debug, Serialize, Deserialize)]
struct ValueSelect {
    value: String,
    selected: i8,
}

#[derive(Debug, Serialize)]
struct UpdateAliasRequest {
    alias: UpdateAlias,
    network: String,
}

#[derive(Debug, Serialize)]
struct UpdateAlias {
    alias: UpdateAliasAlias,
    network_content: String,
}

#[derive(Debug, Serialize)]
struct UpdateAliasAlias {
    enabled: String,
    name: String,
    #[serde(rename = "type")]
    type_name: String,
    proto: String,
    updatefreq: String,
    content: String,
    interface: String,
    counters: String,
    description: String,
}

pub async fn update_alias(ip_to_ban: String, config: &Config) -> Result<(), reqwest::Error> {
    let client = reqwest::Client::builder()
        .danger_accept_invalid_certs(true)
        .build()?;

    let json = get_input(&client, config).await?;

    update(json, &ip_to_ban, &client, config).await?;
    reconfigure(client, config).await?;

    log::info!("{ip_to_ban} has been banned", ip_to_ban = ip_to_ban);

    Ok(())
}

async fn reconfigure(client: reqwest::Client, config: &Config) -> Result<(), reqwest::Error> {
    let reconfigure_url = format!(
        "{host}/api/firewall/alias/reconfigure",
        host = config.opnsense_host
    );

    let res = client
        .post(reconfigure_url)
        .basic_auth(
            config.opnsense_key.to_owned(),
            Option::Some(config.opnsense_secret.to_owned()),
        )
        .header("Content-Type", "application/json")
        .body("{}")
        .send()
        .await?;

    log::debug!("{:#?}", res);
    log::debug!("{:#?}", res.text().await?);

    Ok(())
}

async fn update(
    json: InputAlias,
    ip_to_ban: &String,
    client: &reqwest::Client,
    config: &Config,
) -> Result<(), reqwest::Error> {
    let mut content: Vec<String> = json
        .alias
        .content
        .iter()
        .filter(|(_, v)| v.selected == 1)
        .map(|(k, _)| k.clone())
        .collect::<Vec<String>>();

    content.push(ip_to_ban.to_owned());

    let content = content.join("\n");
    let output = UpdateAlias {
        alias: UpdateAliasAlias {
            enabled: json.alias.enabled,
            name: json.alias.name,
            type_name: "network".to_string(),
            proto: "".to_string(),
            updatefreq: "".to_string(),
            content,
            interface: "".to_string(),
            counters: "0".to_string(),
            description: "".to_string(),
        },
        network_content: "".to_string(),
    };

    let update_alias_url = format!(
        "{host}/api/firewall/alias/setItem/{uuid}",
        host = config.opnsense_host,
        uuid = config.opnsense_alias_uuid,
    );

    log::debug!("{:#?}", output);

    let output_json = serde_json::to_string(&output).expect("Failed to serialize output");

    log::debug!("{:#?}", output_json);

    let res = client
        .post(update_alias_url)
        .basic_auth(
            config.opnsense_key.to_owned(),
            Option::Some(config.opnsense_secret.to_owned()),
        )
        .header("Content-Type", "application/json")
        .body(output_json)
        .send()
        .await?;

    log::debug!("{:#?}", res);
    log::debug!("{:#?}", res.text().await?);

    Ok(())
}

async fn get_input(
    client: &reqwest::Client,
    config: &Config,
) -> Result<InputAlias, reqwest::Error> {
    let get_alias_url = format!(
        "{host}/api/firewall/alias/getItem/{uuid}",
        host = config.opnsense_host,
        uuid = config.opnsense_alias_uuid,
    );

    log::debug!("Alias: {}", get_alias_url);

    let res = client
        .get(get_alias_url)
        .basic_auth(
            config.opnsense_key.to_owned(),
            Option::Some(config.opnsense_secret.to_owned()),
        )
        .send()
        .await?;

    let json_text = res.text().await?;
    log::debug!("Text from opnsense: {}", json_text);
    let json: InputAlias = serde_json::from_str(&json_text).expect("JSON parse error");

    log::debug!("{:#?}", json);

    Ok(json)
}
