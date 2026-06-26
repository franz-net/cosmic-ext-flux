// SPDX-License-Identifier: GPL-3.0-only

use cosmic::cosmic_config;
use cosmic::cosmic_config::{cosmic_config_derive::CosmicConfigEntry, CosmicConfigEntry};

#[derive(Debug, Clone, PartialEq, CosmicConfigEntry)]
#[version = 5]
pub struct Config {
    pub source_path: String,
    pub fit_mode: String,
    pub autostart: bool,
    pub span_mode: bool,
    pub fps_cap: u32,
    pub pause_on_fullscreen: bool,
    pub pause_on_maximized: bool,
    pub pause_on_battery: bool,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            source_path: String::new(),
            fit_mode: String::new(),
            autostart: false,
            span_mode: false,
            fps_cap: 0, // 0 = follow source framerate
            pause_on_fullscreen: true, // pause when an app is fullscreen (issue #13)
            pause_on_maximized: false, // also pause when an app is maximized (opt-in)
            pause_on_battery: false, // pause on battery power (opt-in, issue #1)
        }
    }
}
