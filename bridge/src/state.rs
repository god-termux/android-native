//! Снимок состояния Android из файлов мода xfcestatus ($PREFIX/tmp/god-status/{battery,wifi}).
//! Формат строк — вывод xfce-status: «<txt>…</txt><tool>Battery 12%, DISCHARGING, 33.0°C</tool>» и т.п.

use std::{fs, path::PathBuf};

#[derive(Clone, Debug, PartialEq)]
pub struct Battery {
    pub present: bool,
    pub percent: f64,
    /// CHARGING | DISCHARGING | FULL | NOT_CHARGING
    pub status: String,
    pub temp: f64,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Wifi {
    pub connected: bool,
    pub ssid: String,
    pub quality: u8,
    pub dbm: i32,
    pub ip: String,
    pub mbps: u32,
    pub mhz: u32,
}

pub fn dir() -> PathBuf {
    std::env::var_os("GOD_STATUS_DIR").map(PathBuf::from).unwrap_or_else(|| {
        let prefix = std::env::var("PREFIX").unwrap_or_else(|_| "/data/data/com.termux/files/usr".into());
        PathBuf::from(prefix).join("tmp/god-status")
    })
}

fn tool_text(raw: &str) -> Option<String> {
    let a = raw.find("<tool>")? + 6;
    let b = raw[a..].find("</tool>")? + a;
    Some(raw[a..b].to_string())
}

pub fn read_battery() -> Battery {
    let none = Battery { present: false, percent: 0.0, status: "UNKNOWN".into(), temp: 0.0 };
    let Ok(raw) = fs::read_to_string(dir().join("battery")) else { return none };
    let Some(tip) = tool_text(&raw) else { return none };
    // «Battery 12%, DISCHARGING, 33.0°C»
    let rest = tip.trim().strip_prefix("Battery").unwrap_or(&tip).trim();
    let mut parts = rest.split(',').map(str::trim);
    let percent = parts.next().and_then(|p| p.trim_end_matches('%').parse::<f64>().ok());
    let status = parts.next().unwrap_or("UNKNOWN").to_string();
    let temp = parts.next().and_then(|t| t.trim_end_matches("°C").parse::<f64>().ok()).unwrap_or(0.0);
    match percent {
        Some(percent) => Battery { present: true, percent, status, temp },
        None => none,
    }
}

pub fn read_wifi() -> Wifi {
    let none = Wifi { connected: false, ssid: String::new(), quality: 0, dbm: -100, ip: String::new(), mbps: 0, mhz: 0 };
    let Ok(raw) = fs::read_to_string(dir().join("wifi")) else { return none };
    let Some(tip) = tool_text(&raw) else { return none };
    let mut lines = tip.lines();
    let first = lines.next().unwrap_or("");
    let Some(ssid) = first.strip_prefix("Wi-Fi ") else { return none };
    let mut w = Wifi { connected: true, ssid: ssid.trim().to_string(), ..none };
    for l in lines {
        if let Some(q) = l.strip_prefix("Quality ") {
            // «100% (-47 dBm)»
            w.quality = q.split('%').next().and_then(|v| v.trim().parse().ok()).unwrap_or(0);
            w.dbm = q.split('(').nth(1).and_then(|v| v.trim_end_matches(" dBm)").trim().parse().ok()).unwrap_or(-100);
        } else if let Some(i) = l.strip_prefix("IP ") {
            // «192.168.0.3, 72 Mbps, 2412 MHz»
            let mut p = i.split(',').map(str::trim);
            w.ip = p.next().unwrap_or("").to_string();
            w.mbps = p.next().and_then(|v| v.trim_end_matches(" Mbps").parse().ok()).unwrap_or(0);
            w.mhz = p.next().and_then(|v| v.trim_end_matches(" MHz").parse().ok()).unwrap_or(0);
        }
    }
    w
}
