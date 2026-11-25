use std::net::SocketAddr;

use axum::http::HeaderMap;

#[derive(Debug)]
pub struct ClientInfo {
    pub ip: String,
    pub user_agent: Option<String>,
    pub device_type: DeviceType,
}

pub fn client_info(headers: &HeaderMap, remote: SocketAddr) -> ClientInfo {
    let ip = headers
        .get("cf-connecting-ip")
        .or_else(|| headers.get("x-forwarded-for"))
        .and_then(|v| v.to_str().ok())
        .map(|v| v.split(',').next().unwrap_or(v).trim().to_string())
        .unwrap_or_else(|| remote.ip().to_string());

    let user_agent = headers.get("user-agent").and_then(|v| v.to_str().ok());

    let device_type = user_agent_to_device_type(user_agent);

    ClientInfo {
        ip,
        user_agent: user_agent.map(str::to_string),
        device_type,
    }
}

#[derive(Debug)]
enum DeviceType {
    Browser,
    Desktop,
    Mobile,
}

pub fn user_agent_to_device_type(ua: Option<&str>) -> DeviceType {
    let Some(ua) = ua else {
        return DeviceType::Browser;
    };

    let ua = ua.to_ascii_lowercase();

    if ua.contains("mobile") {
        DeviceType::Mobile
    } else if ua.contains("electron") {
        DeviceType::Desktop
    } else {
        DeviceType::Browser
    }
}
