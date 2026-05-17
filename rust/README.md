# exhale (Rust port)

Cross-platform Rust port of the macOS-only Swift app.  Same overlay, same
breathing animation, same hotkeys, same settings panel — on **macOS, Windows,
and Linux**.

## Build & run

The `cargo run` family **builds and then launches** the binary in one step.
The `cargo build` family only compiles — you have to invoke the binary
yourself afterwards.

| Command                  | Builds | Runs | Build profile               |
|--------------------------|:------:|:----:|-----------------------------|
| `cargo run`              |  ✓     |  ✓   | Dev (debug, fast compile)   |
| `cargo run --release`    |  ✓     |  ✓   | Release (optimised)         |
| `cargo build`            |  ✓     |  —   | Dev                         |
| `cargo build --release`  |  ✓     |  —   | Release                     |

Use dev builds while iterating (compile is ~10× faster), release for the
real binary you'd ship or benchmark.  Binaries land at:

- Dev:     `target/debug/exhale` (or `.exe` on Windows)
- Release: `target/release/exhale` (or `.exe` on Windows)

### Running an already-built binary

After `cargo build`, run the binary directly without going through cargo:

**macOS**
```sh
./target/release/exhale          # release
./target/debug/exhale            # dev
```

**Linux**
```sh
./target/release/exhale          # release
./target/debug/exhale            # dev
```

**Windows (PowerShell or cmd)**
```sh
.\target\release\exhale.exe      # release
.\target\debug\exhale.exe        # dev
```

### Platform prerequisites

**macOS** — no extra prerequisites beyond Rust.

**Windows** — no extra prerequisites.  Works with both the MSVC and GNU
toolchains.

**Linux** — requires GTK dev headers for the system-tray crate.  On
Debian/Ubuntu:

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

## Settings

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

## Known macOS-fidelity gaps vs the Swift original

The Rust port reaches feature parity with the Swift original on every
documented setting and behaviour.  Three native-macOS UX touches are
intentionally **not** ported because doing them right would require
rewriting the settings window as an AppKit hierarchy instead of an
egui one:

- **`NSColorPanel` colour picker** — the inhale / exhale / background
  colour swatches use egui's built-in colour picker.  Native
  `NSColorPanel` integration (eyedropper, system palettes) is
  feasible via target/action bridging but costs ~300-500 LOC of
  Objective-C glue with proper colour-space conversion and
  multi-target tracking.  Cross-platform Discord / Slack / VS Code
  all use custom pickers too; this isn't a glaring gap.
- **`NSStepper` widget** — the stepper buttons next to each numeric
  field are hand-painted to match macOS's `NSStepper` visually.  A
  real `NSStepper` lives in an `NSView` hierarchy that can't be
  hosted inside an egui frame without rebuilding the whole settings
  window as AppKit.  The hand-painted version is pixel-close.
- **Accessibility tree / VoiceOver** — egui doesn't expose an
  accessibility tree (no AX backend).  Native AppKit controls would,
  but the same constraint as `NSStepper` applies.  Tracking
  upstream: <https://github.com/emilk/egui/issues/3604>.

The animation cadence and CPU-vs-Swift comparison live in
`docs/PERFORMANCE.md` (TODO).  Currently the Rust port matches Swift
at 24 / 12 fps with ~3-5× lower per-frame CPU on the complex scenes
(ripple, circle gradient) and ~2× higher on trivial scenes
(fullscreen-solid); see the `cpu_bench` example in
`crates/exhale-render/examples` for the comparison harness.
