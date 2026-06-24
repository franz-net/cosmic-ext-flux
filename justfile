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

# Remove a local user install (does not touch a system .deb).
uninstall:
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
