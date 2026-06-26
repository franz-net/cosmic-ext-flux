<!-- Update this file before each tagged release. -->
<!-- The workflow appends auto-generated commit notes below this body. -->

## Highlights

- **New: automatically pause when an app goes fullscreen.** Flux draws behind your windows, so while an app is fullscreen the wallpaper is fully hidden — Flux now stops decoding it, dropping daemon CPU to ~0% (measured 11% → 0% on a 60 fps source) and saving power, then resumes the instant you leave fullscreen. On by default; toggle **"Pause when an app is fullscreen"** in the applet popup. (#13)
- **Optional: also pause when a window is maximized.** Off by default — enable **"Also pause when an app is maximized"** in the applet if you'd rather the wallpaper stop whenever a window covers the screen. A manual pause always takes precedence: leaving fullscreen will never override a pause you set yourself.
- **The daemon now logs to the systemd journal by default** (`journalctl --user -u cosmic-ext-flux-daemon`), making problems easier to diagnose. Set `RUST_LOG` to change the level.

No need to re-add the applet this time — the App ID is unchanged since v2.0.0, and your settings migrate automatically.

## Install

```sh
sudo apt-get install -y gstreamer1.0-plugins-base gstreamer1.0-plugins-good gstreamer1.0-plugins-bad gstreamer1.0-vaapi
sudo dpkg -i cosmic-ext-flux_<version>_amd64.deb
```

Add the **Flux** applet to your panel via Settings > Desktop > Panel > Applets.

## Uninstall

```sh
systemctl --user disable --now cosmic-ext-flux-daemon
sudo dpkg -r cosmic-ext-flux
```

## Requirements

- COSMIC desktop environment
- GStreamer 1.x with video decode plugins
- VA-API drivers recommended for hardware decode
