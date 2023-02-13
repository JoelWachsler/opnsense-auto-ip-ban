use std::net;

use crate::{config::Config, update_alias};

pub async fn ban_ip(input: &str, config: &Config) -> Result<(), Box<dyn std::error::Error>> {
    let ip = parse_ip(input)?;

    if should_ban(&ip) {
        log::info!("Will ban {}", ip);
        update_alias::update_alias(format!("{ip}/24"), config).await?
    } else {
        log::info!("Will not ban {}", ip);
    }

    Ok(())
}

fn should_ban(ip: &net::Ipv4Addr) -> bool {
    !ip.is_private()
}

fn parse_ip(ip: &str) -> Result<net::Ipv4Addr, net::AddrParseError> {
    let ip: net::Ipv4Addr = ip.to_string().parse()?;

    let [f1, f2, f3, _] = ip.octets();
    Ok(net::Ipv4Addr::new(f1, f2, f3, 0))
}

#[cfg(test)]
mod tests {
    use crate::ip_ban::{parse_ip, should_ban};

    #[test]
    fn test_local_addr() {
        let ip = parse_ip("192.168.1.1").unwrap();
        assert!(!should_ban(&ip));
    }

    #[test]
    fn test_local_cluster_addr() {
        let ip = parse_ip("10.233.66.81").unwrap();
        assert!(!should_ban(&ip));
    }

    #[test]
    fn test_local_remote_addr() {
        let ip = parse_ip("85.202.169.35").unwrap();
        assert!(should_ban(&ip));
    }
}
