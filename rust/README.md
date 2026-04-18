# exhale (Rust port)

Cross-platform Rust port of the macOS-only Swift app.  Same overlay, same
breathing animation, same hotkeys, same settings panel — on **macOS, Windows,
and Linux**.

## Build

All platforms:

```sh
cargo build --release
```

Binary lands at `target/release/exhale` (or `.exe` on Windows).

### macOS

No extra prerequisites beyond Rust.

### Windows

No extra prerequisites.  Works with both the MSVC and GNU toolchains.

### Linux

Requires the usual desktop-app dev headers for the system-tray crate's GTK
bindings.  On Debian/Ubuntu:

```sh
sudo apt install \
    libgtk-3-dev libayatana-appindicator3-dev \
    libxcb1-dev libxkbcommon-dev
```

On Fedora/RHEL:

```sh
sudo dnf install \
    gtk3-devel libayatana-appindicator-gtk3-devel \
    libxcb-devel libxkbcommon-devel
```

X11 and Xfixes are loaded dynamically via `x11-dl`, so you don't need their
`-dev` packages at build time — only the runtime libraries, which ship on
every X11 desktop.

## Run

```sh
cargo run --release
```

Settings are saved as TOML under the platform config dir:

| Platform | Path |
|----------|------|
| macOS    | `~/Library/Application Support/exhale/settings.toml` |
| Windows  | `%APPDATA%\exhale\settings.toml` |
| Linux    | `~/.config/exhale/settings.toml` |

## Crates

- `exhale-core`   — settings, breathing controller, types.  Zero GUI deps.
- `exhale-render` — wgpu renderer + WGSL shader.
- `exhale-app`    — winit event loop, egui settings panel, tray, hotkeys,
                    platform glue.

## Platform notes

- **macOS**: the overlay floats above fullscreen apps (screen-saver window
  level), joins every Space, and stays out of Cmd+Tab.  `AppVisibility`
  toggles `NSApp.setActivationPolicy` between `.regular` and `.accessory`.
- **Windows**: the overlay uses `WS_EX_LAYERED | WS_EX_TRANSPARENT |
  WS_EX_TOOLWINDOW | WS_EX_NOACTIVATE | WS_EX_TOPMOST`.  `AppVisibility`
  toggles `WS_EX_APPWINDOW` / `WS_EX_TOOLWINDOW` on the settings window so
  "DockOnly" shows a taskbar entry and "TopBarOnly" hides it.
- **Linux (X11)**: click-through via `XFixesSetWindowShapeRegion` with an
  empty input region; always-on-top via `_NET_WM_STATE_ABOVE`; workspace-
  spanning via `_NET_WM_STATE_STICKY`; `AppVisibility` toggles
  `_NET_WM_STATE_SKIP_TASKBAR` / `SKIP_PAGER` on the settings window.
- **Linux (Wayland)**: transparent overlay works; click-through is not yet
  implemented — winit doesn't expose `wl_surface::set_input_region` and
  most compositors require `wlr_layer_shell` for screen-spanning overlays.
