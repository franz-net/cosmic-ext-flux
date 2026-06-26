<!-- Update this file before each tagged release. -->
<!-- The workflow appends auto-generated commit notes below this body. -->

## Highlights

- **New: optionally pause on battery power.** Save laptop battery by automatically pausing the wallpaper while you're on battery, resuming when you plug back in. **Opt-in** — enable **"Pause on battery power"** in the applet popup (off by default). The daemon reads the on-battery state from UPower; on systems without UPower it simply does nothing. (#1)
- **Fixed: applet could crash when opening its popup.** The added setting pushed the popup past its height limit and tripped a Wayland surface error; the popup now fits all its rows.

Both auto-pause conditions from v3.0 (a fullscreen/maximized window, and now on-battery) share one rule: a manual pause always wins, and the wallpaper only plays when you want it to and nothing is asking it to pause.

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
