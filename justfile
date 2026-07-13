default: build

build:
    cargo build

build-release:
    cargo build --release

run-daemon:
    RUST_LOG=debug cargo run -p cosmic-ext-flux-daemon

run-applet:
    RUST_LOG=debug cargo run -p cosmic-ext-applet-flux

# Rewrites the unit ExecStart and desktop Exec to the ~/.local/bin binaries so
# this install cleanly overrides a system .deb (user systemd + XDG dirs take
# precedence) without colliding or needing sudo.
# Local user install (no sudo); reverse with `just uninstall`.
install: build-release
    install -Dm755 target/release/cosmic-ext-flux-daemon \
        ~/.local/bin/cosmic-ext-flux-daemon
    install -Dm755 target/release/cosmic-ext-applet-flux \
        ~/.local/bin/cosmic-ext-applet-flux
    install -Dm644 applet/resources/app.desktop \
        ~/.local/share/applications/io.github.franz_net.CosmicExtAppletFlux.desktop
    sed -i "s|^Exec=cosmic-ext-applet-flux|Exec=$HOME/.local/bin/cosmic-ext-applet-flux|" \
        ~/.local/share/applications/io.github.franz_net.CosmicExtAppletFlux.desktop
    install -Dm644 applet/resources/icon.svg \
        ~/.local/share/icons/hicolor/scalable/apps/io.github.franz_net.CosmicExtAppletFlux.svg
    install -Dm644 applet/resources/icon-stopped.svg \
        ~/.local/share/icons/hicolor/scalable/apps/io.github.franz_net.CosmicExtAppletFlux-stopped.svg
    install -Dm644 data/cosmic-ext-flux-daemon.service \
        ~/.config/systemd/user/cosmic-ext-flux-daemon.service
    sed -i "s|^ExecStart=/usr/bin/cosmic-ext-flux-daemon|ExecStart=$HOME/.local/bin/cosmic-ext-flux-daemon|" \
        ~/.config/systemd/user/cosmic-ext-flux-daemon.service
    systemctl --user daemon-reload
    # Enable + start the daemon so it runs now and after every reboot. The .deb's
    # postinst does this via `systemctl --global enable`; a source install has to
    # do it too, or the daemon never comes up (and the applet's Play has nothing
    # to talk to). Best-effort: the user systemd manager is present in a desktop
    # session but may be absent in a bare SSH/CI shell, so don't fail the install.
    systemctl --user enable --now cosmic-ext-flux-daemon || \
        echo "NOTE: could not enable the service (no user session?). Run this from your desktop session: systemctl --user enable --now cosmic-ext-flux-daemon"

# For diagnosing the applet popup crash. After running: remove and re-add the
# Flux applet on the panel (so it relaunches), reproduce the crash, then collect:
#   journalctl --user -b --no-pager | grep -iE 'flux|xdg_surface|xdg_popup' > flux-debug.log
# Run `just install` again afterward to return to a normal (quiet) applet.
# Like `install` but runs the applet under WAYLAND_DEBUG=1 + RUST_LOG=debug.
install-debug: install
    sed -i "s|^Exec=|Exec=env WAYLAND_DEBUG=1 RUST_LOG=cosmic_ext_applet_flux=debug |" \
        ~/.local/share/applications/io.github.franz_net.CosmicExtAppletFlux.desktop
    @echo "Debug applet installed. Remove + re-add the Flux applet on the panel, reproduce,"
    @echo "then: journalctl --user -b --no-pager | grep -iE 'flux|xdg_surface|xdg_popup' > flux-debug.log"

# Remove a local user install (does not touch a system .deb).
uninstall:
    # Stop + disable before removing the unit so no enabled symlink dangles.
    systemctl --user disable --now cosmic-ext-flux-daemon 2>/dev/null || true
    rm -f ~/.local/bin/cosmic-ext-flux-daemon \
        ~/.local/bin/cosmic-ext-applet-flux \
        ~/.local/share/applications/io.github.franz_net.CosmicExtAppletFlux.desktop \
        ~/.local/share/icons/hicolor/scalable/apps/io.github.franz_net.CosmicExtAppletFlux.svg \
        ~/.local/share/icons/hicolor/scalable/apps/io.github.franz_net.CosmicExtAppletFlux-stopped.svg \
        ~/.config/systemd/user/cosmic-ext-flux-daemon.service
    systemctl --user daemon-reload

check:
    cargo clippy --all-targets --all-features

clean:
    cargo clean
