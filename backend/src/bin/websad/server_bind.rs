use serde_json::Value;
use std::net::IpAddr;

pub(super) fn parse_bind(
    data: &Value,
    current: IpAddr,
    current_port: u16,
) -> Result<(IpAddr, u16), String> {
    let address = match data.get("bind_address") {
        None => current,
        Some(value) => value
            .as_str()
            .ok_or("bind_address must be an IP literal")?
            .parse::<IpAddr>()
            .map_err(|_| "bind_address must be an IP literal".to_string())?,
    };
    let port = match data.get("port") {
        None => current_port,
        Some(value) => value
            .as_u64()
            .filter(|p| (1..=65535).contains(p))
            .ok_or("port must be an integer from 1 to 65535")? as u16,
    };
    Ok((address, port))
}
