// SPDX-License-Identifier: GPL-3.0-only

mod app;
mod config;
mod dbus_client;
mod i18n;

fn main() -> cosmic::iced::Result {
    // The applet had no log subscriber, so its tracing went nowhere. Install a
    // quiet one (warn) that RUST_LOG can turn up — e.g. `just install-debug`
    // runs the applet with RUST_LOG=...debug + WAYLAND_DEBUG=1 to capture the
    // popup lifecycle and the raw Wayland protocol trace for crash diagnosis.
    // try_init so we don't conflict if libcosmic installs its own subscriber.
    let _ = tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("warn")),
        )
        .try_init();

    migrate_legacy_config();
    let requested_languages = i18n_embed::DesktopLanguageRequester::requested_languages();
    i18n::init(&requested_languages);
    cosmic::applet::run::<app::AppModel>(())
}

/// One-time migration of applet config from the pre-rename App ID
/// (`com.system76.CosmicAppletFlux`). Copies (never deletes) and only when
/// the new location doesn't exist yet. The daemon performs the same
/// migration; whichever starts first wins.
fn migrate_legacy_config() {
    let config_home = std::env::var("XDG_CONFIG_HOME")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|_| {
            let home = std::env::var("HOME").unwrap_or_default();
            std::path::PathBuf::from(home).join(".config")
        });
    let old = config_home.join("cosmic/com.system76.CosmicAppletFlux");
    let new = config_home.join("cosmic/io.github.franz_net.CosmicExtAppletFlux");
    if !old.is_dir() || new.exists() {
        return;
    }
    if let Err(e) = copy_dir_recursive(&old, &new) {
        eprintln!("failed to migrate legacy config: {e}");
    }
}

/// Recursive copy of regular files and directories; symlinks are skipped.
fn copy_dir_recursive(src: &std::path::Path, dst: &std::path::Path) -> std::io::Result<()> {
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
