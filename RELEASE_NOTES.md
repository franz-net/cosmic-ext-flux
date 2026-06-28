<!-- Update this file before each tagged release. -->
<!-- The workflow appends auto-generated commit notes below this body. -->

## Highlights

This is a bug-fix release. **If the Flux applet crashed when you opened its popup on v3.1.0 or earlier, update to v3.1.1.**

- **Fixed: applet popup crash on some systems (#15).** On certain hardware the applet would die the moment its popup opened, with a Wayland `xdg_surface` "unconfigured_buffer" protocol error — and it happened *regardless* of popup size, so the v3.1.0 height fix didn't cover these users. The real cause was an upstream bug in the pinned `libcosmic` version. This release updates `libcosmic` to a revision that fixes it, with no change to how the applet works.

There are no other behaviour changes from v3.1.0.

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
