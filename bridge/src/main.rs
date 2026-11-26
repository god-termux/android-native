//! god-android-bridge: D-Bus-мост Android -> COSMIC.
//! Отдаёт на «системной» шине (DBUS_SYSTEM_BUS_ADDRESS) службы org.freedesktop.UPower и org.freedesktop.NetworkManager,
//! данные берёт из файлов мода xfcestatus ($PREFIX/tmp/god-status), собственных обращений к Android не делает.

mod nm;
mod state;
mod upower;

use std::{sync::{Arc, Mutex}, time::Duration};
use zbus::connection::Builder;

#[tokio::main(flavor = "current_thread")]
async fn main() -> zbus::Result<()> {
    let addr = std::env::var("DBUS_SYSTEM_BUS_ADDRESS").expect("нужен DBUS_SYSTEM_BUS_ADDRESS");
    let mut battery = state::read_battery();
    let wifi: nm::Shared = Arc::new(Mutex::new(state::read_wifi()));
    let conn = Builder::address(addr.as_str())
        .name("org.freedesktop.UPower")
        .name("org.freedesktop.NetworkManager")
        .serve_at(nm::NM, nm::Manager(wifi.clone()))
        .serve_at(nm::SETTINGS, nm::Settings(wifi.clone()))
        .serve_at(nm::CONN, nm::SavedConnection(wifi.clone()))
        .serve_at(nm::DEV, nm::Device(wifi.clone()))
        .serve_at(nm::DEV, nm::Wireless(wifi.clone()))
        .serve_at(nm::AP, nm::AccessPoint(wifi.clone()))
        .serve_at(nm::ACTIVE, nm::ActiveConnection(wifi.clone()))
        .serve_at(nm::IP4, nm::Ip4Config(wifi.clone()))
        .serve_at(upower::PATH, upower::UPower { on_battery: battery.status == "DISCHARGING" })
        .serve_at(upower::DEVICE, upower::Device { b: battery.clone() })
        .serve_at(upower::DISPLAY, upower::Device { b: battery.clone() })
        .build()
        .await?;
    eprintln!("god-android-bridge: UPower на {addr}, батарея {battery:?}");

    let server = conn.object_server();
    loop {
        tokio::time::sleep(Duration::from_secs(4)).await;
        let new_wifi = state::read_wifi();
        let old_wifi = wifi.lock().unwrap().clone();
        if new_wifi != old_wifi {
            *wifi.lock().unwrap() = new_wifi.clone();
            nm::notify(&server, &old_wifi, &new_wifi).await?;
        }
        let new = state::read_battery();
        if new == battery {
            continue;
        }
        for path in [upower::DEVICE, upower::DISPLAY] {
            let iface = server.interface::<_, upower::Device>(path).await?;
            let old = battery.clone();
            {
                let mut d = iface.get_mut().await;
                d.b = new.clone();
            }
            upower::notify_device(iface.signal_emitter(), &*iface.get().await, &old).await?;
        }
        let up = server.interface::<_, upower::UPower>(upower::PATH).await?;
        let on = new.status == "DISCHARGING";
        if up.get().await.on_battery != on {
            up.get_mut().await.on_battery = on;
            up.get().await.on_battery_changed(up.signal_emitter()).await?;
        }
        battery = new;
    }
}
