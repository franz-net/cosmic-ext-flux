// SPDX-License-Identifier: GPL-3.0-only

//! D-Bus server interface for the daemon.
//! Serves `io.github.franz_net.CosmicExtFlux1` on the session bus.

use crate::wayland::{Command, DaemonState};
use anyhow::Result;
use std::sync::{Arc, Mutex};
use zbus::{interface, Connection};

/// Directories blocked from being used as wallpaper sources.
const BLOCKED_PREFIXES: &[&str] = &["/dev/", "/proc/", "/sys/", "/run/"];

/// Validate and canonicalize a source path. Returns the canonical path string or an error.
pub fn validate_source_path(path: &str) -> Result<String, String> {
    let canonical = std::fs::canonicalize(path)
        .map_err(|e| format!("Invalid path: {e}"))?;
    let canonical_str = canonical
        .to_str()
        .ok_or_else(|| "Path is not valid UTF-8".to_string())?;
    for prefix in BLOCKED_PREFIXES {
        if canonical_str.starts_with(prefix) {
            return Err(format!("Path under {prefix} is not allowed"));
        }
    }
    let meta = std::fs::metadata(&canonical)
        .map_err(|e| format!("Cannot access file: {e}"))?;
    if !meta.is_file() {
        return Err("Path is not a regular file".to_string());
    }
    Ok(canonical_str.to_string())
}

struct WallpaperInterface {
    command_tx: std::sync::mpsc::SyncSender<Command>,
    state: Arc<Mutex<DaemonState>>,
}

#[interface(name = "io.github.franz_net.CosmicExtFlux1")]
impl WallpaperInterface {
    async fn set_source(&self, path: String) -> zbus::fdo::Result<()> {
        let validated = validate_source_path(&path)
            .map_err(|e| zbus::fdo::Error::InvalidArgs(e))?;
        self.command_tx
            .try_send(Command::SetSource(validated))
            .map_err(|_| zbus::fdo::Error::Failed("Command queue full".to_string()))
    }

    async fn play(&self) -> zbus::fdo::Result<()> {
        self.command_tx
            .try_send(Command::Play)
            .map_err(|_| zbus::fdo::Error::Failed("Command queue full".to_string()))
    }

    async fn pause(&self) -> zbus::fdo::Result<()> {
        self.command_tx
            .try_send(Command::Pause)
            .map_err(|_| zbus::fdo::Error::Failed("Command queue full".to_string()))
    }

    async fn stop(&self) -> zbus::fdo::Result<()> {
        self.command_tx
            .try_send(Command::Stop)
            .map_err(|_| zbus::fdo::Error::Failed("Command queue full".to_string()))
    }

    async fn set_fit_mode(&self, mode: String) -> zbus::fdo::Result<()> {
        let fit = match mode.as_str() {
            "zoom" => crate::wayland::FitMode::Zoom,
            "fit" => crate::wayland::FitMode::Fit,
            "stretch" => crate::wayland::FitMode::Stretch,
            _ => {
                return Err(zbus::fdo::Error::InvalidArgs(format!(
                    "Unknown fit mode: {mode}"
                )));
            }
        };
        self.command_tx
            .try_send(Command::SetFitMode(fit))
            .map_err(|_| zbus::fdo::Error::Failed("Command queue full".to_string()))
    }

    async fn set_span_mode(&self, enabled: bool) -> zbus::fdo::Result<()> {
        self.command_tx
            .try_send(Command::SetSpanMode(enabled))
            .map_err(|_| zbus::fdo::Error::Failed("Command queue full".to_string()))
    }

    async fn set_fps_cap(&self, fps: u32) -> zbus::fdo::Result<()> {
        // 0 = follow source framerate
        if fps != 0 && !(5..=60).contains(&fps) {
            return Err(zbus::fdo::Error::InvalidArgs(
                "FPS cap must be 0 (auto) or between 5 and 60".to_string(),
            ));
        }
        self.command_tx
            .try_send(Command::SetFpsCap(fps))
            .map_err(|_| zbus::fdo::Error::Failed("Command queue full".to_string()))
    }

    async fn set_pause_on_fullscreen(&self, enabled: bool) -> zbus::fdo::Result<()> {
        self.command_tx
            .try_send(Command::SetPauseOnFullscreen(enabled))
            .map_err(|_| zbus::fdo::Error::Failed("Command queue full".to_string()))
    }

    async fn set_pause_on_maximized(&self, enabled: bool) -> zbus::fdo::Result<()> {
        self.command_tx
            .try_send(Command::SetPauseOnMaximized(enabled))
            .map_err(|_| zbus::fdo::Error::Failed("Command queue full".to_string()))
    }

    async fn set_pause_on_battery(&self, enabled: bool) -> zbus::fdo::Result<()> {
        self.command_tx
            .try_send(Command::SetPauseOnBattery(enabled))
            .map_err(|_| zbus::fdo::Error::Failed("Command queue full".to_string()))
    }


    /// Returns all daemon state in a single D-Bus call: (playing, error, cpu, memory, fps, source_fps).
    async fn get_state(&self) -> (bool, String, f64, f64, f64, f64) {
        let s = self.state.lock().ok();
        let playing = s.as_ref().map(|s| s.playing).unwrap_or(false);
        let error = s.as_ref().and_then(|s| s.error.clone()).unwrap_or_default();
        // Truncate error for D-Bus
        let error = if error.len() > 256 {
            error.char_indices().take_while(|&(i, _)| i < 256).map(|(_, c)| c).collect()
        } else {
            error
        };
        let cpu = s.as_ref().map(|s| s.cpu_percent as f64).unwrap_or(0.0);
        let memory = s.as_ref().map(|s| s.memory_mb as f64).unwrap_or(0.0);
        let fps = s.as_ref().map(|s| s.fps as f64).unwrap_or(0.0);
        let source_fps = s.as_ref().map(|s| s.source_fps as f64).unwrap_or(0.0);
        (playing, error, cpu, memory, fps, source_fps)
    }

    #[zbus(property)]
    async fn span_mode(&self) -> bool {
        self.state.lock().map(|s| s.span_mode).unwrap_or(false)
    }

    #[zbus(property)]
    async fn playing(&self) -> bool {
        self.state.lock().map(|s| s.playing).unwrap_or(false)
    }

    #[zbus(property)]
    async fn source(&self) -> String {
        self.state.lock().map(|s| s.source_path.clone()).unwrap_or_default()
    }

    #[zbus(property)]
    async fn fit_mode(&self) -> String {
        self.state.lock().map(|s| s.fit_mode.as_str().to_string()).unwrap_or_default()
    }

    #[zbus(property)]
    async fn error(&self) -> String {
        let msg = self
            .state
            .lock()
            .ok()
            .and_then(|s| s.error.clone())
            .unwrap_or_default();
        // Truncate to avoid exposing long internal paths/details (UTF-8 safe)
        if msg.len() > 256 {
            msg.char_indices()
                .take_while(|&(i, _)| i < 256)
                .map(|(_, c)| c)
                .collect()
        } else {
            msg
        }
    }

    #[zbus(property)]
    async fn cpu_percent(&self) -> f64 {
        self.state.lock().map(|s| s.cpu_percent as f64).unwrap_or(0.0)
    }

    #[zbus(property)]
    async fn memory_mb(&self) -> f64 {
        self.state.lock().map(|s| s.memory_mb as f64).unwrap_or(0.0)
    }

    #[zbus(property)]
    async fn fps(&self) -> f64 {
        self.state.lock().map(|s| s.fps as f64).unwrap_or(0.0)
    }

    #[zbus(property)]
    async fn fps_cap(&self) -> u32 {
        self.state.lock().map(|s| s.fps_cap).unwrap_or(15)
    }

    #[zbus(property)]
    async fn source_fps(&self) -> f64 {
        self.state.lock().map(|s| s.source_fps as f64).unwrap_or(0.0)
    }
}

pub async fn serve(
    state: Arc<Mutex<DaemonState>>,
    command_tx: std::sync::mpsc::SyncSender<Command>,
) -> Result<()> {
    let iface = WallpaperInterface { command_tx, state };

    let conn = Connection::session().await?;

    conn.object_server()
        .at("/io/github/franz_net/CosmicExtFlux", iface)
        .await?;

    conn.request_name("io.github.franz_net.CosmicExtFlux1")
        .await?;

    tracing::info!("D-Bus interface active at io.github.franz_net.CosmicExtFlux1");

    std::future::pending::<()>().await;
    unreachable!()
}

#[cfg(test)]
mod tests {
    use super::validate_source_path;

    #[test]
    fn rejects_nonexistent_path() {
        assert!(validate_source_path("/no/such/flux-test-path-xyz.mp4").is_err());
    }

    #[test]
    fn rejects_blocked_prefix() {
        // /dev/null exists and canonicalizes, so it exercises the prefix guard
        // (not the existence check).
        let err = validate_source_path("/dev/null").unwrap_err();
        assert!(err.contains("/dev/"), "unexpected error: {err}");
    }

    #[test]
    fn rejects_directory() {
        let dir = std::env::temp_dir().join("flux_test_dir_validate");
        std::fs::create_dir_all(&dir).unwrap();
        let res = validate_source_path(dir.to_str().unwrap());
        std::fs::remove_dir_all(&dir).ok();
        let err = res.unwrap_err();
        assert!(err.contains("not a regular file"), "unexpected error: {err}");
    }

    #[test]
    fn accepts_regular_file_and_canonicalizes() {
        let path = std::env::temp_dir().join("flux_test_file_validate.bin");
        std::fs::write(&path, b"x").unwrap();
        let res = validate_source_path(path.to_str().unwrap());
        std::fs::remove_file(&path).ok();
        let canonical = res.expect("a regular file should validate");
        assert!(canonical.ends_with("flux_test_file_validate.bin"));
    }
}
