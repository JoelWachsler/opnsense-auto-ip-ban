use rdkafka::Timestamp;
use serde::Deserialize;

use crate::config::Config;

pub async fn get_ips_to_ban_at_timestamp(
    timestamp: Timestamp,
    config: &Config,
) -> Result<Vec<String>, Box<dyn std::error::Error>> {
    let client = reqwest::Client::new();

    let time = LokiTimes::from(timestamp, config.loki_url.to_owned());
    let url = time.to_loki_url();
    let res = client.get(url).send().await?.text().await?;

    log::debug!("{}", res);

    let parse = serde_json::from_str::<LokiResult>(&res)?;
    log::debug!("{:#?}", parse);

    let rows = parse
        .data
        .result
        .iter()
        .flat_map(|result| result.values.to_owned())
        .flat_map(|value| {
            let row = value.get(1);
            row.map(|row| row.to_owned())
        })
        .collect::<Vec<String>>();

    let res = rows
        .iter()
        .flat_map(|row| extract_ip(row))
        .map(|extracted_ip| extracted_ip.to_string())
        .collect::<Vec<String>>();

    Ok(res)
}

fn extract_ip(msg: &str) -> Option<&str> {
    let re = regex::Regex::new(r"client: (\d+\.\d+\.\d+\.\d+)").unwrap();
    re.captures(msg)
        .and_then(|res| res.get(1))
        .map(|res| res.as_str())
}

#[derive(Debug, Deserialize)]
struct LokiResult {
    data: LokiResultData,
}

#[derive(Debug, Deserialize)]
struct LokiResultData {
    result: Vec<LokiResultDataResult>,
}

#[derive(Debug, Deserialize)]
struct LokiResultDataResult {
    values: Vec<Vec<String>>,
}

struct LokiTimes {
    loki_base_url: String,
    start_time: i64,
    end_time: i64,
}

impl LokiTimes {
    fn to_loki_url(&self) -> String {
        format!(
            "{loki_url}/loki/api/v1/query_range?direction=BACKWARD&limit=1000&query=%7Bnamespace%3D%22mailu-2%22%7D%20%7C%3D%20%22login%20failed%22&start={start_time}&end={end_time}",
            loki_url=self.loki_base_url,
            start_time=self.start_time,
            end_time=self.end_time,
        )
    }

    fn from(timestamp: Timestamp, loki_url: String) -> LokiTimes {
        let timestamp_as_millis = timestamp.to_millis().unwrap();
        let shift = 1000000;

        LokiTimes {
            loki_base_url: loki_url,
            // this should be 5 minutes before end_time
            start_time: (timestamp_as_millis - (5 * 60 * 1000)) * shift,
            end_time: timestamp_as_millis * shift,
        }
    }
}

#[cfg(test)]
mod tests {
    use rdkafka::Timestamp;

    use super::{extract_ip, LokiTimes};

    #[test]
    fn test_url_creation() {
        let timestamp = Timestamp::CreateTime(2000000000000);
        let loki_time = LokiTimes::from(timestamp, "http://loki".to_string());

        assert_eq!(
            loki_time.to_loki_url(),
            "http://loki/loki/api/v1/query_range?direction=BACKWARD&limit=1000&query=%7Bnamespace%3D%22mailu-2%22%7D%20%7C%3D%20%22login%20failed%22&start=1999999700000000000&end=2000000000000000000",
        );
    }

    #[test]
    fn test_timestamp_creation() {
        let timestamp = Timestamp::CreateTime(2000000000000);
        let loki_time = LokiTimes::from(timestamp, "http://loki".to_string());

        assert_eq!(loki_time.start_time, 1999999700000000000);
        assert_eq!(loki_time.end_time, 2000000000000000000);
    }

    #[test]
    fn test_ip_extraction() {
        let msg = "2022-05-21T23:01:44.470722263Z stderr F 2022/05/21 23:01:44 [info] 12#12: *6465 client login failed: \"Authentication credentials invalid\" while in http auth state, client: 85.202.169.35, server: 0.0.0.0:25, login: \"info@test.se\"";
        let extracted = extract_ip(msg);

        assert_eq!(extracted, Option::Some("85.202.169.35"));
    }

    #[test]
    fn test_ip_extraction_2() {
        let msg = "2022-09-18T08:27:47.941845889Z stderr F 2022/09/18 08:27:47 [info] 12#12: *66595 client login failed: \"AUTH not supported\" while in http auth state, client: 156.96.56.80, server: 0.0.0.0:25, login: \"www\"";
        let extracted = extract_ip(msg);

        assert_eq!(extracted, Option::Some("156.96.56.80"));
    }
}
