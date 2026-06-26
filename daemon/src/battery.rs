// SPDX-License-Identifier: GPL-3.0-only

//! UPower system-bus monitor: forwards the laptop's on-battery state to the
//! Wayland loop so playback can auto-pause on battery power (issue #1).
//!
//! Degrades gracefully: if the system bus or UPower is unavailable (desktops,
//! minimal systems), this logs once and returns, leaving battery auto-pause
//! inert. The raw on-battery state is always reported; whether it actually
//! pauses is gated by the `pause_on_battery` config in the renderer.

use crate::wayland::Command;
use futures_util::StreamExt;
use std::sync::mpsc::SyncSender;
use zbus::Connection;

#[zbus::proxy(
    interface = "org.freedesktop.UPower",
    default_service = "org.freedesktop.UPower",
    default_path = "/org/freedesktop/UPower"
)]
trait UPower {
    /// True when the system is running on battery (no external power).
    #[zbus(property)]
    fn on_battery(&self) -> zbus::Result<bool>;
}

/// Connect to UPower and forward `OnBattery` changes as `Command::SetOnBattery`.
pub async fn monitor(tx: SyncSender<Command>) {
    let conn = match Connection::system().await {
        Ok(c) => c,
        Err(e) => {
            tracing::warn!("UPower: no system bus ({e}); battery auto-pause disabled");
            return;
        }
    };
    let proxy = match UPowerProxy::new(&conn).await {
        Ok(p) => p,
        Err(e) => {
            tracing::warn!("UPower: unavailable ({e}); battery auto-pause disabled");
            return;
        }
    };

    // Push the current state immediately, then react to changes.
    match proxy.on_battery().await {
        Ok(on) => {
            tracing::info!("UPower: initial on_battery={on}");
            let _ = tx.send(Command::SetOnBattery(on));
        }
        Err(e) => tracing::warn!("UPower: failed to read OnBattery ({e})"),
    }

    let mut changes = proxy.receive_on_battery_changed().await;
    while let Some(change) = changes.next().await {
        if let Ok(on) = change.get().await {
            tracing::info!("UPower: on_battery={on}");
            let _ = tx.send(Command::SetOnBattery(on));
        }
    }

    tracing::warn!("UPower: change stream ended; battery auto-pause inactive");
}
