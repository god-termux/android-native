//! org.freedesktop.NetworkManager: Wi-Fi телефона для апплета сети COSMIC (библиотека nmrs).
//! Объекты статичны, значения вычисляются из общего состояния; при его смене рассылаются PropertiesChanged.

use crate::state::Wifi;
use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};
use zbus::{
    interface,
    object_server::SignalEmitter,
    zvariant::{ObjectPath, OwnedObjectPath, OwnedValue, Value},
};

pub type Shared = Arc<Mutex<Wifi>>;

pub const NM: &str = "/org/freedesktop/NetworkManager";
pub const SETTINGS: &str = "/org/freedesktop/NetworkManager/Settings";
pub const CONN: &str = "/org/freedesktop/NetworkManager/Settings/1";
pub const DEV: &str = "/org/freedesktop/NetworkManager/Devices/1";
pub const AP: &str = "/org/freedesktop/NetworkManager/AccessPoint/1";
pub const ACTIVE: &str = "/org/freedesktop/NetworkManager/ActiveConnection/1";
pub const IP4: &str = "/org/freedesktop/NetworkManager/IP4Config/1";
const NONE: &str = "/";

fn p(s: &str) -> OwnedObjectPath {
    ObjectPath::try_from(s.to_string()).unwrap().into()
}

fn ov(v: Value<'_>) -> OwnedValue {
    OwnedValue::try_from(v).unwrap()
}

fn uuid_of(ssid: &str) -> String {
    // стабильный псевдо-UUID из имени сети
    let mut h: u64 = 1469598103934665603;
    for b in ssid.bytes() {
        h = (h ^ b as u64).wrapping_mul(1099511628211);
    }
    format!("{:08x}-0000-4000-8000-{:012x}", (h >> 32) as u32, h & 0xffff_ffff_ffff)
}

fn snapshot(s: &Shared) -> Wifi {
    s.lock().unwrap().clone()
}

// ---------- NetworkManager ----------
pub struct Manager(pub Shared);

#[interface(name = "org.freedesktop.NetworkManager")]
impl Manager {
    fn get_devices(&self) -> Vec<OwnedObjectPath> {
        vec![p(DEV)]
    }
    fn get_all_devices(&self) -> Vec<OwnedObjectPath> {
        vec![p(DEV)]
    }
    fn get_device_by_ip_iface(&self, _iface: &str) -> OwnedObjectPath {
        p(DEV)
    }
    fn check_connectivity(&self) -> u32 {
        if snapshot(&self.0).connected { 4 } else { 1 }
    }
    fn deactivate_connection(&self, _active: OwnedObjectPath) {}
    fn activate_connection(&self, _c: OwnedObjectPath, _d: OwnedObjectPath, _o: OwnedObjectPath) -> OwnedObjectPath {
        p(ACTIVE)
    }
    fn add_and_activate_connection(
        &self,
        _c: HashMap<String, HashMap<String, OwnedValue>>,
        _d: OwnedObjectPath,
        _o: OwnedObjectPath,
    ) -> (OwnedObjectPath, OwnedObjectPath) {
        (p(CONN), p(ACTIVE))
    }
    #[zbus(property)]
    fn networking_enabled(&self) -> bool {
        true
    }
    #[zbus(property)]
    fn wireless_enabled(&self) -> bool {
        true
    }
    #[zbus(property)]
    fn wireless_hardware_enabled(&self) -> bool {
        true
    }
    #[zbus(property)]
    fn wwan_enabled(&self) -> bool {
        false
    }
    #[zbus(property)]
    fn wwan_hardware_enabled(&self) -> bool {
        false
    }
    #[zbus(property)]
    fn active_connections(&self) -> Vec<OwnedObjectPath> {
        if snapshot(&self.0).connected { vec![p(ACTIVE)] } else { vec![] }
    }
    #[zbus(property)]
    fn primary_connection(&self) -> OwnedObjectPath {
        if snapshot(&self.0).connected { p(ACTIVE) } else { p(NONE) }
    }
    #[zbus(property)]
    fn state(&self) -> u32 {
        if snapshot(&self.0).connected { 70 } else { 20 }
    }
    #[zbus(property)]
    fn connectivity(&self) -> u32 {
        if snapshot(&self.0).connected { 4 } else { 1 }
    }
    #[zbus(property)]
    fn connectivity_check_enabled(&self) -> bool {
        false
    }
    #[zbus(property)]
    fn connectivity_check_uri(&self) -> String {
        String::new()
    }
    #[zbus(property)]
    fn version(&self) -> String {
        "1.46.0".into()
    }
}

// ---------- Settings ----------
pub struct Settings(pub Shared);

#[interface(name = "org.freedesktop.NetworkManager.Settings")]
impl Settings {
    fn list_connections(&self) -> Vec<OwnedObjectPath> {
        if snapshot(&self.0).connected { vec![p(CONN)] } else { vec![] }
    }
    fn get_connection_by_uuid(&self, _uuid: &str) -> OwnedObjectPath {
        p(CONN)
    }
    fn reload_connections(&self) -> bool {
        true
    }
    #[zbus(property)]
    fn connections(&self) -> Vec<OwnedObjectPath> {
        if snapshot(&self.0).connected { vec![p(CONN)] } else { vec![] }
    }
    #[zbus(property)]
    fn can_modify(&self) -> bool {
        false
    }
}

pub struct SavedConnection(pub Shared);

#[interface(name = "org.freedesktop.NetworkManager.Settings.Connection")]
impl SavedConnection {
    fn get_settings(&self) -> HashMap<String, HashMap<String, OwnedValue>> {
        let w = snapshot(&self.0);
        let mut conn = HashMap::new();
        conn.insert("id".to_string(), ov(Value::from(w.ssid.clone())));
        conn.insert("uuid".to_string(), ov(Value::from(uuid_of(&w.ssid))));
        conn.insert("type".to_string(), ov(Value::from("802-11-wireless".to_string())));
        conn.insert("autoconnect".to_string(), ov(Value::from(true)));
        conn.insert("interface-name".to_string(), ov(Value::from("wlan0".to_string())));
        let mut wl = HashMap::new();
        wl.insert("ssid".to_string(), ov(Value::from(w.ssid.as_bytes().to_vec())));
        wl.insert("mode".to_string(), ov(Value::from("infrastructure".to_string())));
        let mut sec = HashMap::new();
        sec.insert("key-mgmt".to_string(), ov(Value::from("wpa-psk".to_string())));
        let mut out = HashMap::new();
        out.insert("connection".to_string(), conn);
        out.insert("802-11-wireless".to_string(), wl);
        out.insert("802-11-wireless-security".to_string(), sec);
        out
    }
    fn delete(&self) {}
    fn update(&self, _s: HashMap<String, HashMap<String, OwnedValue>>) {}
    #[zbus(property)]
    fn unsaved(&self) -> bool {
        false
    }
    #[zbus(property)]
    fn filename(&self) -> String {
        String::new()
    }
    #[zbus(property)]
    fn flags(&self) -> u32 {
        0
    }
}

// ---------- Устройство wlan0 ----------
pub struct Device(pub Shared);

#[interface(name = "org.freedesktop.NetworkManager.Device")]
impl Device {
    fn disconnect(&self) {}
    #[zbus(property)]
    fn udi(&self) -> String {
        "/sys/devices/android/wlan0".into()
    }
    #[zbus(property)]
    fn interface(&self) -> String {
        "wlan0".into()
    }
    #[zbus(property)]
    fn ip_interface(&self) -> String {
        "wlan0".into()
    }
    #[zbus(property)]
    fn driver(&self) -> String {
        "android".into()
    }
    #[zbus(property)]
    fn device_type(&self) -> u32 {
        2 // WIFI
    }
    #[zbus(property)]
    fn state(&self) -> u32 {
        if snapshot(&self.0).connected { 100 } else { 30 }
    }
    #[zbus(property)]
    fn state_reason(&self) -> (u32, u32) {
        (0, 0)
    }
    #[zbus(property)]
    fn managed(&self) -> bool {
        true
    }
    #[zbus(property)]
    fn autoconnect(&self) -> bool {
        true
    }
    #[zbus(property)]
    fn hw_address(&self) -> String {
        "02:00:00:00:00:00".into()
    }
    #[zbus(property, name = "PermHwAddress")]
    fn perm_hw_address(&self) -> String {
        "02:00:00:00:00:00".into()
    }
    #[zbus(property)]
    fn active_connection(&self) -> OwnedObjectPath {
        if snapshot(&self.0).connected { p(ACTIVE) } else { p(NONE) }
    }
    #[zbus(property)]
    fn ip4_config(&self) -> OwnedObjectPath {
        if snapshot(&self.0).connected { p(IP4) } else { p(NONE) }
    }
    #[zbus(property)]
    fn firmware_missing(&self) -> bool {
        false
    }
}

pub struct Wireless(pub Shared);

#[interface(name = "org.freedesktop.NetworkManager.Device.Wireless")]
impl Wireless {
    fn request_scan(&self, _options: HashMap<String, OwnedValue>) {}
    fn get_access_points(&self) -> Vec<OwnedObjectPath> {
        if snapshot(&self.0).connected { vec![p(AP)] } else { vec![] }
    }
    #[zbus(property)]
    fn mode(&self) -> u32 {
        2 // infra
    }
    #[zbus(property)]
    fn bitrate(&self) -> u32 {
        snapshot(&self.0).mbps * 1000
    }
    #[zbus(property)]
    fn access_points(&self) -> Vec<OwnedObjectPath> {
        if snapshot(&self.0).connected { vec![p(AP)] } else { vec![] }
    }
    #[zbus(property)]
    fn active_access_point(&self) -> OwnedObjectPath {
        if snapshot(&self.0).connected { p(AP) } else { p(NONE) }
    }
    #[zbus(property)]
    fn wireless_capabilities(&self) -> u32 {
        0x1ff
    }
    #[zbus(property)]
    fn last_scan(&self) -> i64 {
        0
    }
}

pub struct AccessPoint(pub Shared);

#[interface(name = "org.freedesktop.NetworkManager.AccessPoint")]
impl AccessPoint {
    #[zbus(property)]
    fn ssid(&self) -> Vec<u8> {
        snapshot(&self.0).ssid.into_bytes()
    }
    #[zbus(property)]
    fn strength(&self) -> u8 {
        snapshot(&self.0).quality
    }
    #[zbus(property)]
    fn hw_address(&self) -> String {
        "02:00:00:00:00:01".into()
    }
    #[zbus(property)]
    fn flags(&self) -> u32 {
        1 // privacy
    }
    #[zbus(property)]
    fn wpa_flags(&self) -> u32 {
        0
    }
    #[zbus(property)]
    fn rsn_flags(&self) -> u32 {
        0x108 // PSK + TKIP/CCMP
    }
    #[zbus(property)]
    fn frequency(&self) -> u32 {
        snapshot(&self.0).mhz
    }
    #[zbus(property)]
    fn max_bitrate(&self) -> u32 {
        snapshot(&self.0).mbps * 1000
    }
    #[zbus(property)]
    fn mode(&self) -> u32 {
        2
    }
    #[zbus(property)]
    fn last_seen(&self) -> i32 {
        0
    }
}

pub struct ActiveConnection(pub Shared);

#[interface(name = "org.freedesktop.NetworkManager.Connection.Active")]
impl ActiveConnection {
    #[zbus(property)]
    fn connection(&self) -> OwnedObjectPath {
        p(CONN)
    }
    #[zbus(property)]
    fn specific_object(&self) -> OwnedObjectPath {
        p(AP)
    }
    #[zbus(property)]
    fn id(&self) -> String {
        snapshot(&self.0).ssid
    }
    #[zbus(property)]
    fn uuid(&self) -> String {
        uuid_of(&snapshot(&self.0).ssid)
    }
    #[zbus(property, name = "Type")]
    fn type_(&self) -> String {
        "802-11-wireless".into()
    }
    #[zbus(property)]
    fn devices(&self) -> Vec<OwnedObjectPath> {
        vec![p(DEV)]
    }
    #[zbus(property)]
    fn state(&self) -> u32 {
        2 // activated
    }
    #[zbus(property)]
    fn default(&self) -> bool {
        true
    }
    #[zbus(property)]
    fn default6(&self) -> bool {
        false
    }
    #[zbus(property)]
    fn vpn(&self) -> bool {
        false
    }
    #[zbus(property)]
    fn ip4_config(&self) -> OwnedObjectPath {
        p(IP4)
    }
    #[zbus(property)]
    fn ip6_config(&self) -> OwnedObjectPath {
        p(NONE)
    }
    #[zbus(property)]
    fn master(&self) -> OwnedObjectPath {
        p(NONE)
    }
}

pub struct Ip4Config(pub Shared);

#[interface(name = "org.freedesktop.NetworkManager.IP4Config")]
impl Ip4Config {
    #[zbus(property)]
    fn address_data(&self) -> Vec<HashMap<String, OwnedValue>> {
        let w = snapshot(&self.0);
        let mut m = HashMap::new();
        m.insert("address".to_string(), ov(Value::from(w.ip)));
        m.insert("prefix".to_string(), ov(Value::from(24u32)));
        vec![m]
    }
    #[zbus(property)]
    fn gateway(&self) -> String {
        String::new()
    }
    #[zbus(property)]
    fn nameserver_data(&self) -> Vec<HashMap<String, OwnedValue>> {
        vec![]
    }
}

/// Рассылает PropertiesChanged для свойств, зависящих от состояния Wi-Fi.
pub async fn notify(server: &zbus::ObjectServer, old: &Wifi, new: &Wifi) -> zbus::Result<()> {
    let conn_changed = old.connected != new.connected;
    let ssid_changed = old.ssid != new.ssid;
    if conn_changed || ssid_changed {
        let i = server.interface::<_, Manager>(NM).await?;
        let e: &SignalEmitter<'_> = i.signal_emitter();
        let m = i.get().await;
        m.active_connections_changed(e).await?;
        m.primary_connection_changed(e).await?;
        m.state_changed(e).await?;
        m.connectivity_changed(e).await?;
        let i = server.interface::<_, Settings>(SETTINGS).await?;
        let s = i.get().await;
        s.connections_changed(i.signal_emitter()).await?;
        let i = server.interface::<_, Device>(DEV).await?;
        let d = i.get().await;
        d.state_changed(i.signal_emitter()).await?;
        d.active_connection_changed(i.signal_emitter()).await?;
        d.ip4_config_changed(i.signal_emitter()).await?;
        let i = server.interface::<_, Wireless>(DEV).await?;
        let w = i.get().await;
        w.access_points_changed(i.signal_emitter()).await?;
        w.active_access_point_changed(i.signal_emitter()).await?;
        w.bitrate_changed(i.signal_emitter()).await?;
        let i = server.interface::<_, ActiveConnection>(ACTIVE).await?;
        let a = i.get().await;
        a.id_changed(i.signal_emitter()).await?;
        a.uuid_changed(i.signal_emitter()).await?;
    }
    if conn_changed || ssid_changed || old.quality != new.quality || old.mhz != new.mhz || old.mbps != new.mbps {
        let i = server.interface::<_, AccessPoint>(AP).await?;
        let a = i.get().await;
        a.ssid_changed(i.signal_emitter()).await?;
        a.strength_changed(i.signal_emitter()).await?;
        a.frequency_changed(i.signal_emitter()).await?;
        a.max_bitrate_changed(i.signal_emitter()).await?;
    }
    if old.ip != new.ip {
        let i = server.interface::<_, Ip4Config>(IP4).await?;
        i.get().await.address_data_changed(i.signal_emitter()).await?;
    }
    Ok(())
}
