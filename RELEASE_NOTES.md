<!-- Update this file before each tagged release. -->
<!-- The workflow appends auto-generated commit notes below this body. -->

## Highlights

<!-- Replace with the key changes for this release -->
- **Fixed: daemon not switched over when upgrading from `cosmic-flux` via sudo.** The post-install script could neither stop the old `cosmic-flux-daemon` nor enable the new unit from a sudo context (no access to the user session bus), leaving the old daemon running from a deleted binary until the next login. The service is now enabled system-wide via `systemctl --global enable`, with a best-effort immediate stop/start for the installing user.
- If you already upgraded to v2.0.0 and hit this, either log out and back in, or run:
  ```sh
  systemctl --user stop cosmic-flux-daemon
  systemctl --user enable --now cosmic-ext-flux-daemon
  ```
- Reminder from v2.0.0: re-add the **Flux** applet to your panel once (the App ID changed) via Settings > Desktop > Panel > Applets. See the [v2.0.0 notes](https://github.com/franz-net/cosmic-ext-flux/releases/tag/v2.0.0) for everything in the rename release.

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
