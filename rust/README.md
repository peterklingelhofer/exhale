# exhale (Rust port)

Cross-platform Rust port of the macOS-only Swift app.  Same overlay, same
breathing animation, same hotkeys, same settings panel — on **macOS, Windows,
and Linux** from one codebase.

- **Renderer**: `wgpu` + a single WGSL fragment shader (`crates/exhale-render`)
- **Window system**: `winit`
- **Settings UI**: `egui` (hand-rolled stepper, segmented picker, control
  buttons painted directly via `egui::Painter` to match `NSSegmentedControl`
  / `NSStepper` look)
- **AppKit interop**: typed FFI via `objc2`, no raw `msg_send!` after the
  migration
- **Threading model**: per-overlay-window render thread + per-window
  `wgpu::Device` so overlay frame delivery isn't gated by the main thread's
  message queue or the settings window's GPU submissions
- **Settings cadence**: 24 fps fast / 12 fps slow (matches Swift's
  `MetalBreathingController`).  Hardcoded — earlier user-tunable preset was
  removed since per-frame CPU runs ≤ 2 % on every scene tested

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

Settings are saved as TOML under the platform config dir
(via the `directories` crate's `ProjectDirs::from("com",
"peterklingelhofer", "exhale")`):

| Platform | Path |
|----------|------|
| macOS    | `~/Library/Application Support/com.peterklingelhofer.exhale/settings.toml` |
| Windows  | `%APPDATA%\peterklingelhofer\exhale\config\settings.toml` |
| Linux    | `~/.config/exhale/settings.toml` |

Settings are reloaded on launch and persisted on every change via a
debounced background writer thread; corrupt TOML is logged and the
file is rewritten with defaults.

## Crates

- `exhale-core`   — settings + `SettingsDiff`, breathing controller
                    (deadline-scheduled background thread), poison-tolerant
                    lock helpers, easing tables.  Zero GUI deps.
- `exhale-render` — `wgpu` renderer + WGSL fragment shader, headless
                    benchmarking harness (`cargo run --example cpu_bench`).
- `exhale-app`    — winit event loop, split egui settings panel
                    (`settings_window.rs` + `widgets.rs` + `theme.rs`),
                    per-overlay render thread, tray, hotkeys, platform glue
                    (objc2 / windows-sys / x11-dl).

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

## Performance

Hardcoded cadence: 24 fps fast / 12 fps slow (matches Swift's
`MetalBreathingController.swift`).  Headless render bench at this
cadence on M3 Max (`cargo run --release --example cpu_bench -p
exhale-render`):

| Scene                            | Δ avg CPU | Δ peak CPU | effective fps |
|----------------------------------|----------:|-----------:|--------------:|
| rect + gradient                  |     1.5 % |      1.7 % |          19.3 |
| circle + gradient                |     1.3 % |      1.6 % |          19.5 |
| fullscreen + solid               |     1.5 % |      1.9 % |          19.3 |
| rect + hold ripple gradient      |     1.3 % |      1.8 % |          17.3 |
| rect + hold ripple stark         |     1.2 % |      1.5 % |          17.9 |
| circle + hold ripple gradient    |     1.4 % |      2.1 % |          17.5 |

Compared on the same hardware against Swift's `PerformanceTests`
(`swift/exhaleTests/exhaleTests.swift::PerformanceTests`) which uses
the same `getrusage` / 5×1s-sample methodology:

| Scene                            | Swift Δ avg | Rust Δ avg |
|----------------------------------|------------:|-----------:|
| Circle + gradient                |       4.6 % |      1.3 % |
| Rect + ripple gradient           |       6.1 % |      1.3 % |
| Rect + ripple stark              |       4.3 % |      1.2 % |
| Circle + ripple gradient         |       6.0 % |      1.4 % |
| Fullscreen + solid               |       0.0 % |      1.5 % |
| Rect + gradient                  |       0.6 % |      1.5 % |

**Per-frame animation work is ~3-5× cheaper in Rust on the complex
scenes** (anything with hold-ripple or circle SDF math).  SwiftUI
optimises trivial scenes to ~zero — Rust's shader runs the same
fragment work regardless of scene complexity, so simple scenes cost
slightly more (still under 2 %).  Notable: Rust's per-scene variance
is much lower (1.2 – 1.5 % across everything) which matters for
predictable battery-life modelling.

Caveat: the `cpu_bench` harness is **headless** — it renders to an
offscreen `wgpu::Texture`, skipping the compositor cost the live app
pays on `WindowServer` / DWM.  Real-world CPU is somewhere between
the bench number and the Swift number, likely closer to 2-3 % on
complex scenes.

## Ship & distribute

| Target                       | Status            | Notes |
|------------------------------|-------------------|-------|
| macOS standalone (signed)    | ✅ ready          | `scripts/bundle-mas.sh` (universal binary, Developer-ID signed .pkg, sandbox entitlements) |
| Mac App Store                | ⚠️ blockers       | TCP single-instance guard won't work under sandbox without `network.server` entitlement; replace with file lock.  See `crates/exhale-app/src/main.rs:857-947` |
| Windows standalone           | ✅ ready          | `cargo build --release` + `bundle-msix.ps1` for MSIX wrapper |
| Microsoft Store              | ⚠️ blockers       | Manifest missing Wide310x150 and Square71x71 tile assets; `MaxVersionTested` is 22H2 (bump to 24H2) |
| Linux `.deb` / AppImage      | ✅ ready          | `cargo deb` + `scripts/bundle-appimage.sh` |

The full app-store readiness gap analysis lives in `docs/SHIPPING.md`
(TODO) — short version: both stores are ~1 day of focused fixes away
for a v1 listing.
