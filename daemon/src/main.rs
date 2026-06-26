// SPDX-License-Identifier: GPL-3.0-only

mod battery;
mod dbus;
mod decoder;
mod wayland;

use anyhow::Result;
use std::path::{Path, PathBuf};
use std::sync::{
    atomic::AtomicBool,
    Arc, Mutex,
};
use tracing_subscriber::EnvFilter;
use wayland::{Command, DaemonState};

fn main() -> Result<()> {
    // Default to info-level logging so the systemd journal captures daemon
    // lifecycle events (startup, output config, auto-pause) without needing
    // RUST_LOG set; RUST_LOG still overrides when present.
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")),
        )
        .init();

    tracing::info!("Starting cosmic-ext-flux-daemon");

    // Initialize GStreamer once for the entire process
    gstreamer::init()?;

    let frame_buffer: Arc<Mutex<Option<Vec<u8>>>> = Arc::new(Mutex::new(None));
    let frame_dirty = Arc::new(AtomicBool::new(false));

    let state = Arc::new(Mutex::new(DaemonState {
        source_path: String::new(),
        playing: false,
        fit_mode: wayland::FitMode::Zoom,
        span_mode: false,
        error: None,
        cpu_percent: 0.0,
        memory_mb: 0.0,
        fps: 0.0,
        fps_cap: 0,
        source_fps: 0.0,
    }));

    // Bounded channel prevents D-Bus flood from exhausting memory
    let (command_tx, command_rx) = std::sync::mpsc::sync_channel::<Command>(64);

    // D-Bus server runs in a separate thread with a lightweight single-threaded
    // tokio runtime. The same runtime also drives the UPower battery monitor.
    let dbus_state = Arc::clone(&state);
    let dbus_tx = command_tx.clone();
    let battery_tx = command_tx.clone();
    std::thread::spawn(move || {
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .expect("tokio runtime");
        rt.block_on(async move {
            tokio::spawn(battery::monitor(battery_tx));
            if let Err(e) = dbus::serve(dbus_state, dbus_tx).await {
                tracing::error!("D-Bus server error: {e}");
            }
        });
    });

    // One-time migration from the pre-rename App ID / cache dirs
    migrate_legacy_dirs();

    // Auto-restore: read applet config and queue commands if autostart is enabled
    restore_from_config(&command_tx);

    // Load cached frame so there's something to display immediately on restart
    if let Some((cached_frame, _w, _h)) = wayland::load_frame_cache() {
        if let Ok(mut fb) = frame_buffer.lock() {
            *fb = Some(cached_frame);
        }
        frame_dirty.store(true, std::sync::atomic::Ordering::Relaxed);
        tracing::info!("Loaded cached frame for immediate display");
    }

    // Wayland event loop runs on the main thread (blocking)
    wayland::run(frame_buffer, frame_dirty, state, command_rx)?;

    Ok(())
}

/// Read the applet's cosmic-config and queue startup commands if autostart is enabled.
fn restore_from_config(tx: &std::sync::mpsc::SyncSender<Command>) {
    let config_dir = match dirs_config_path() {
        Some(p) => p,
        None => return,
    };

    // Apply the auto-pause preferences regardless of autostart, so they take
    // effect even when the user starts playback manually later.
    let pause_on_fullscreen = read_config_bool(&config_dir, "pause_on_fullscreen").unwrap_or(true);
    let _ = tx.send(Command::SetPauseOnFullscreen(pause_on_fullscreen));
    let pause_on_maximized = read_config_bool(&config_dir, "pause_on_maximized").unwrap_or(false);
    let _ = tx.send(Command::SetPauseOnMaximized(pause_on_maximized));
    let pause_on_battery = read_config_bool(&config_dir, "pause_on_battery").unwrap_or(false);
    let _ = tx.send(Command::SetPauseOnBattery(pause_on_battery));

    let autostart = read_config_bool(&config_dir, "autostart").unwrap_or(false);
    if !autostart {
        tracing::info!("Autostart disabled, skipping restore");
        return;
    }

    let source = read_config_string(&config_dir, "source_path").unwrap_or_default();
    if source.is_empty() {
        tracing::info!("No source path configured, skipping restore");
        return;
    }

    // Validate path with the same rules as the D-Bus boundary
    let source = match dbus::validate_source_path(&source) {
        Ok(validated) => validated,
        Err(e) => {
            tracing::warn!("Config source_path failed validation: {e}");
            return;
        }
    };

    tracing::info!("Auto-restoring wallpaper: {source}");

    let fit_mode = read_config_string(&config_dir, "fit_mode").unwrap_or_default();
    let span_mode = read_config_bool(&config_dir, "span_mode").unwrap_or(false);
    let fps_cap = read_config_u32(&config_dir, "fps_cap").unwrap_or(0);

    if !fit_mode.is_empty() {
        let _ = tx.send(Command::SetFitMode(wayland::FitMode::from_str(&fit_mode)));
    }
    let _ = tx.send(Command::SetSpanMode(span_mode));
    let _ = tx.send(Command::SetFpsCap(fps_cap));
    let _ = tx.send(Command::SetSource(source));
}

/// One-time migration from the pre-rename identifiers: the applet config under
/// `com.system76.CosmicAppletFlux` and the frame cache under `cosmic-flux`.
/// Copies (never deletes) and only when the new location doesn't exist yet.
fn migrate_legacy_dirs() {
    let config_home = config_home();
    migrate_dir(
        &config_home.join("cosmic/com.system76.CosmicAppletFlux"),
        &config_home.join("cosmic/io.github.franz_net.CosmicExtAppletFlux"),
    );

    let cache_home = std::env::var("XDG_CACHE_HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|_| {
            let home = std::env::var("HOME").unwrap_or_default();
            PathBuf::from(home).join(".cache")
        });
    migrate_dir(
        &cache_home.join("cosmic-flux"),
        &cache_home.join("cosmic-ext-flux"),
    );
}

fn migrate_dir(old: &Path, new: &Path) {
    if !old.is_dir() || new.exists() {
        return;
    }
    match copy_dir_recursive(old, new) {
        Ok(()) => tracing::info!("Migrated {} -> {}", old.display(), new.display()),
        Err(e) => tracing::warn!("Failed to migrate {}: {e}", old.display()),
    }
}

/// Recursive copy of regular files and directories; symlinks are skipped.
fn copy_dir_recursive(src: &Path, dst: &Path) -> std::io::Result<()> {
    std::fs::create_dir_all(dst)?;
    for entry in std::fs::read_dir(src)? {
        let entry = entry?;
        let file_type = entry.file_type()?;
        let to = dst.join(entry.file_name());
        if file_type.is_dir() {
            copy_dir_recursive(&entry.path(), &to)?;
        } else if file_type.is_file() {
            std::fs::copy(entry.path(), &to)?;
        }
    }
    Ok(())
}

fn config_home() -> PathBuf {
    std::env::var("XDG_CONFIG_HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|_| {
            let home = std::env::var("HOME").unwrap_or_default();
            PathBuf::from(home).join(".config")
        })
}

/// cosmic-config stores each field as a separate RON file under
/// ~/.config/cosmic/<APP_ID>/v<VERSION>/<field_name>
fn dirs_config_path() -> Option<PathBuf> {
    let config_home = config_home();
    // Try newest version first, fall back to older
    for ver in ["v5", "v4", "v3", "v2", "v1"] {
        let dir = config_home.join(format!("cosmic/io.github.franz_net.CosmicExtAppletFlux/{ver}"));
        if dir.is_dir() {
            return Some(dir);
        }
    }
    None
}

fn read_config_string(dir: &PathBuf, key: &str) -> Option<String> {
    let content = std::fs::read_to_string(dir.join(key)).ok()?;
    let trimmed = content.trim();
    // RON strings are wrapped in quotes — use strip_prefix/suffix for UTF-8 safety
    if let Some(inner) = trimmed.strip_prefix('"').and_then(|s| s.strip_suffix('"')) {
        Some(inner.to_string())
    } else {
        Some(trimmed.to_string())
    }
}

fn read_config_bool(dir: &PathBuf, key: &str) -> Option<bool> {
    let content = std::fs::read_to_string(dir.join(key)).ok()?;
    match content.trim() {
        "true" => Some(true),
        "false" => Some(false),
        _ => None,
    }
}

fn read_config_u32(dir: &PathBuf, key: &str) -> Option<u32> {
    let content = std::fs::read_to_string(dir.join(key)).ok()?;
    content.trim().parse().ok()
}
