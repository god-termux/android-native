//! org.freedesktop.UPower: батарея телефона для апплета батареи COSMIC.

use crate::state::Battery;
use zbus::{interface, object_server::SignalEmitter};

pub const PATH: &str = "/org/freedesktop/UPower";
pub const DEVICE: &str = "/org/freedesktop/UPower/devices/battery_BAT0";
pub const DISPLAY: &str = "/org/freedesktop/UPower/devices/DisplayDevice";

pub struct UPower {
    pub on_battery: bool,
}

#[interface(name = "org.freedesktop.UPower")]
impl UPower {
    fn enumerate_devices(&self) -> Vec<zbus::zvariant::OwnedObjectPath> {
        vec![zbus::zvariant::ObjectPath::try_from(DEVICE).unwrap().into()]
    }
    fn get_display_device(&self) -> zbus::zvariant::OwnedObjectPath {
        zbus::zvariant::ObjectPath::try_from(DISPLAY).unwrap().into()
    }
    fn get_critical_action(&self) -> String {
        "HybridSleep".into()
    }
    #[zbus(property)]
    fn daemon_version(&self) -> String {
        "1.90.0".into()
    }
    #[zbus(property)]
    fn on_battery(&self) -> bool {
        self.on_battery
    }
    #[zbus(property)]
    fn lid_is_closed(&self) -> bool {
        false
    }
    #[zbus(property)]
    fn lid_is_present(&self) -> bool {
        false
    }
}

pub struct Device {
    pub b: Battery,
}

impl Device {
    fn state(&self) -> u32 {
        match self.b.status.as_str() {
            "CHARGING" => 1,
            "DISCHARGING" => 2,
            "FULL" => 4,
            "NOT_CHARGING" => 6,
            _ => 0,
        }
    }
}

#[interface(name = "org.freedesktop.UPower.Device")]
impl Device {
    #[zbus(property, name = "Type")]
    fn type_(&self) -> u32 {
        2 // Battery
    }
    #[zbus(property)]
    fn power_supply(&self) -> bool {
        true
    }
    #[zbus(property)]
    fn is_present(&self) -> bool {
        self.b.present
    }
    #[zbus(property)]
    fn is_rechargeable(&self) -> bool {
        true
    }
    #[zbus(property)]
    fn online(&self) -> bool {
        self.b.status == "CHARGING" || self.b.status == "FULL"
    }
    #[zbus(property, name = "State")]
    fn state_p(&self) -> u32 {
        self.state()
    }
    #[zbus(property)]
    fn percentage(&self) -> f64 {
        self.b.percent
    }
    #[zbus(property)]
    fn temperature(&self) -> f64 {
        self.b.temp
    }
    #[zbus(property)]
    fn time_to_empty(&self) -> i64 {
        0 // оценки нет: апплет покажет только процент
    }
    #[zbus(property)]
    fn time_to_full(&self) -> i64 {
        0
    }
    #[zbus(property)]
    fn energy(&self) -> f64 {
        self.b.percent
    }
    #[zbus(property)]
    fn energy_full(&self) -> f64 {
        100.0
    }
    #[zbus(property)]
    fn energy_full_design(&self) -> f64 {
        100.0
    }
    #[zbus(property)]
    fn energy_rate(&self) -> f64 {
        0.0
    }
    #[zbus(property)]
    fn voltage(&self) -> f64 {
        0.0
    }
    #[zbus(property)]
    fn capacity(&self) -> f64 {
        100.0
    }
    #[zbus(property)]
    fn technology(&self) -> u32 {
        2 // Li-poly
    }
    #[zbus(property)]
    fn battery_level(&self) -> u32 {
        1 // None: заряд сообщается числом
    }
    #[zbus(property)]
    fn warning_level(&self) -> u32 {
        if self.b.percent <= 5.0 { 4 } else if self.b.percent <= 10.0 { 3 } else { 1 }
    }
    #[zbus(property)]
    fn native_path(&self) -> String {
        "android-battery".into()
    }
    #[zbus(property)]
    fn vendor(&self) -> String {
        "Android".into()
    }
    #[zbus(property)]
    fn model(&self) -> String {
        "Battery".into()
    }
    #[zbus(property)]
    fn serial(&self) -> String {
        String::new()
    }
    #[zbus(property)]
    fn update_time(&self) -> u64 {
        std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_secs()).unwrap_or(0)
    }
    #[zbus(property)]
    fn icon_name(&self) -> String {
        let lvl = ((self.b.percent / 10.0).floor() as u32 * 10).min(100);
        let charging = self.b.status == "CHARGING";
        format!("battery-level-{lvl}{}-symbolic", if charging { "-charging" } else { "" })
    }
    #[zbus(property)]
    fn has_history(&self) -> bool {
        false
    }
    #[zbus(property)]
    fn has_statistics(&self) -> bool {
        false
    }
}

/// Рассылает PropertiesChanged для изменившихся свойств
pub async fn notify_device(emitter: &SignalEmitter<'_>, dev: &Device, old: &Battery) -> zbus::Result<()> {
    if dev.b.percent != old.percent {
        dev.percentage_changed(emitter).await?;
        dev.energy_changed(emitter).await?;
        dev.icon_name_changed(emitter).await?;
        dev.warning_level_changed(emitter).await?;
    }
    if dev.b.status != old.status {
        dev.state_changed(emitter).await?;
        dev.online_changed(emitter).await?;
        dev.icon_name_changed(emitter).await?;
    }
    if dev.b.present != old.present {
        dev.is_present_changed(emitter).await?;
    }
    Ok(())
}
