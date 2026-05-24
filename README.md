# exhale

A minimal cross-platform breathing overlay — a friendly indicator and reminder to take full, deep breaths while looking at screens. Research indicates we blink less and breathe more shallowly when staring at displays, and this is intended as a small tool to help counter that.

The overlay is a translucent always-on-top window that gently expands on inhale and contracts on exhale. Inhale, post-inhale hold, exhale, and post-exhale hold durations are all configurable. A good starting point is `4` seconds in and `4` seconds out; eventually `6` and `8` with the out twice as long as the in (to engage the parasympathetic nervous system). Box breathing is `4` / `4` / `4` / `4`. Take breaks if intense feelings arise — it's important not to overdo it.

## Disclaimer

The information and guidance provided by this app are intended for general informational purposes only and are not medical advice. The creator is not a medical professional. Always seek the advice of a qualified healthcare provider with any questions about your health, and do not disregard or delay professional medical advice because of this app. Use is at your own risk.

## Download

Pre-built binaries for each OS are on the [Releases](https://github.com/peterklingelhofer/exhale/releases) page. Using the latest release is recommended; if you hit a problem, please [open an issue](https://github.com/peterklingelhofer/exhale/issues/new).

**Mac**

[<img src="https://user-images.githubusercontent.com/60944077/232312847-df673556-fb5e-49b4-8037-4d38267e6e18.png"  width="157" height="63"></img>](https://apps.apple.com/us/app/exhale-breath/id6447758995?mt=12)

**Windows** — install the MSIX from the Microsoft Store, or grab the standalone `.exe` from Releases.

**Linux** — install the Snap from the Snap Store, or build from source.

## Usage

![circle](https://user-images.githubusercontent.com/60944077/226204981-f390facc-4f6c-4bec-8784-23203aa64efc.gif)
![rectangle](https://user-images.githubusercontent.com/60944077/226204986-7522cb4d-7df1-4d65-96de-e629197e9854.gif)
<img width="447" height="981" alt="Settings panel" src="https://github.com/user-attachments/assets/32e1d10e-72e3-4acb-ae35-be186cd7cb19" />

The **Tint** (Pause) feature tints the screen with the current background colour, useful for nighttime work and compounds well with [Night Shift](https://support.apple.com/en-us/102191) and [f.lux](https://justgetflux.com/).

### Global keyboard shortcuts

| Shortcut                                    | Action               |
|---------------------------------------------|----------------------|
| <kbd>Ctrl</kbd>+<kbd>Shift</kbd>+<kbd>A</kbd> | Start animation      |
| <kbd>Ctrl</kbd>+<kbd>Shift</kbd>+<kbd>S</kbd> | Stop animation       |
| <kbd>Ctrl</kbd>+<kbd>Shift</kbd>+<kbd>D</kbd> | Tint screen          |
| <kbd>Ctrl</kbd>+<kbd>Shift</kbd>+<kbd>F</kbd> | Reset to defaults    |
| <kbd>Ctrl</kbd>+<kbd>Shift</kbd>+<kbd>W</kbd> or <kbd>Ctrl</kbd>+<kbd>Shift</kbd>+<kbd>,</kbd> | Open/Close preferences |

**Notice:** A high opacity value can obscure the Preferences pane in the current workspace. Use <kbd>Ctrl</kbd>+<kbd>Shift</kbd>+<kbd>F</kbd> to reset, or:

1. Swipe to a different workspace.
2. Close Preferences from the menu bar.
3. Re-open Preferences in the current workspace and adjust Opacity.
4. Switch back.

## Architecture

Single Rust workspace (`rust/`) producing one cross-platform binary.

- **Renderer**: `wgpu` + a single WGSL fragment shader (`crates/exhale-render`)
- **Window system**: `winit`
- **Settings UI**: `egui` (hand-rolled stepper, segmented picker, control buttons painted directly via `egui::Painter` to match `NSSegmentedControl` / `NSStepper` look)
- **AppKit interop**: typed FFI via `objc2`, no raw `msg_send!` after the migration
- **Threading model**: per-overlay-window render thread + per-window `wgpu::Device` so overlay frame delivery isn't gated by the main thread's message queue or the settings window's GPU submissions
- **Animation cadence**: 24 fps fast / 12 fps slow (matches the legacy Swift `MetalBreathingController`). Hardcoded — per-frame CPU runs ≤ 2 % on every scene tested, so the earlier user-tunable preset was removed

### Crates

- `exhale-core` — settings + `SettingsDiff`, breathing controller (deadline-scheduled background thread), poison-tolerant lock helpers, easing tables. Zero GUI deps.
- `exhale-render` — `wgpu` renderer + WGSL fragment shader, headless benchmarking harness (`cargo run --example cpu_bench`).
- `exhale-app` — winit event loop, split egui settings panel (`settings_window.rs` + `widgets.rs` + `theme.rs`), per-overlay render thread, tray, hotkeys, platform glue (`objc2` / `windows-sys` / `x11-dl`).

## Build & run

The `cargo run` family **builds and then launches** the binary in one step. The `cargo build` family only compiles — you have to invoke the binary yourself afterwards.

| Command                  | Builds | Runs | Build profile               |
|--------------------------|:------:|:----:|-----------------------------|
| `cargo run`              |  Yes   | Yes  | Dev (debug, fast compile)   |
| `cargo run --release`    |  Yes   | Yes  | Release (optimised)         |
| `cargo build`            |  Yes   | No   | Dev                         |
| `cargo build --release`  |  Yes   | No   | Release                     |

All commands run from `rust/`. Use dev builds while iterating (compile is ~10× faster), release for the real binary you'd ship or benchmark. Binaries land at:

- Dev:     `rust/target/debug/exhale` (or `.exe` on Windows)
- Release: `rust/target/release/exhale` (or `.exe` on Windows)

### Running an already-built binary

After `cargo build`, run the binary directly without going through cargo:

**macOS / Linux**

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

**Windows** — no extra prerequisites. Works with both the MSVC and GNU toolchains.

**Linux** — exhale dynamically loads several system libraries at run time. To build AND run on Debian/Ubuntu:

```sh
sudo apt install \
    libgtk-3-dev libayatana-appindicator3-dev \
    libwayland-dev libxkbcommon-dev libxdo-dev \
    libssl-dev pkg-config
```

On Fedora/RHEL:

```sh
sudo dnf install \
    gtk3-devel libayatana-appindicator-gtk3-devel \
    wayland-devel libxkbcommon-devel libxdo-devel \
    openssl-devel pkgconf-pkg-config
```

If you're **running** a pre-built binary (not compiling from source), the bare runtime packages are enough — drop the `-dev` suffixes:

```sh
sudo apt install libgtk-3-0 libayatana-appindicator3-1 libwayland-client0 libxkbcommon0 libxdo3 libssl3
```

X11 and Xfixes are loaded via `x11-dl` at run time using whatever's already installed by the X11 desktop, so they're not in the list.

What each one is for:
- `libgtk-3` + `libayatana-appindicator3`: system-tray icon backend
- `libwayland-client` + `libxkbcommon`: winit's Wayland + keyboard input
- `libxdo`: `global-hotkey` crate's X11 keyboard binding (the `libxdo.so.3` you saw missing)
- `libssl`: TLS for crates that fetch over HTTPS
- `pkg-config`: build-time library discovery (compile-only)

## Settings

Settings are saved as TOML under the platform config dir (via the `directories` crate's `ProjectDirs::from("com", "peterklingelhofer", "exhale")`):

| Platform | Path |
|----------|------|
| macOS    | `~/Library/Application Support/com.peterklingelhofer.exhale/settings.toml` |
| Windows  | `%APPDATA%\peterklingelhofer\exhale\config\settings.toml` |
| Linux    | `~/.config/exhale/settings.toml` |

Settings are reloaded on launch and persisted on every change via a debounced background writer thread; corrupt TOML is logged and the file is rewritten with defaults.

## Platform notes

- **macOS**: the overlay floats above fullscreen apps (screen-saver window level), joins every Space, and stays out of Cmd+Tab. `AppVisibility` toggles `NSApp.setActivationPolicy` between `.regular` and `.accessory`.
- **Windows**: the overlay uses `WS_EX_LAYERED | WS_EX_TRANSPARENT | WS_EX_TOOLWINDOW | WS_EX_NOACTIVATE | WS_EX_TOPMOST`. `AppVisibility` toggles `WS_EX_APPWINDOW` / `WS_EX_TOOLWINDOW` on the settings window so "DockOnly" shows a taskbar entry and "TopBarOnly" hides it. Tested + recommended on **Windows 11**; some Windows 10 GPU + driver combinations report only `Opaque` alpha modes to Vulkan, in which case exhale falls back to the windowed mode described below.
- **Linux (X11)**: click-through via `XFixesSetWindowShapeRegion` with an empty input region; always-on-top via `_NET_WM_STATE_ABOVE`; workspace-spanning via `_NET_WM_STATE_STICKY`; `AppVisibility` toggles `_NET_WM_STATE_SKIP_TASKBAR` / `SKIP_PAGER` on the settings window.
- **Linux (Wayland)**: exhale picks one of two paths at startup based on whether the compositor exposes alpha-capable swap chains to wgpu:
   - **Compositor supports alpha** (rare on current Mutter/GNOME, supported by some KWin setups): the overlay is placed at `AlwaysOnBottom` because Wayland's security model doesn't surface a portable click-through / always-on-top protocol to winit (`wp_input_region` isn't exposed). Your app windows cover the overlay by default; to see the breath animation, **narrow your foreground windows so they don't fill the whole screen** and the animation shows through the gap.
   - **Compositor only exposes Opaque alpha** (typical real-hardware Wayland session on Ubuntu / Fedora GNOME): exhale falls back to the **windowed mode** described below.
   For full topmost + click-through overlay behavior on Linux, log out and pick an X11 session at the login screen.
- **Windowed-mode fallback** (Wayland sessions without alpha, some Windows 10 + Vulkan combinations, WARP / Microsoft Basic Render Driver, remote-desktop sessions): the breath animation runs in a **480×360 movable, resizable "exhale" window** with normal decorations and full window-manager participation (Alt-Tab, taskbar, native close button). You can use it two ways: (1) **as a foreground window**, watching the breath animation directly the same way you'd watch any other app, or (2) **as an edge-strip overlay**, by sending the window behind your other apps (Alt-Tab past it / click on the window manager to lower it), switching exhale to **Rectangle mode**, and narrowing the windows in front so the animation shows through the side / bottom strips you've left open. The Stop button (and the global Stop hotkey, if bound) hides this window; clicking the window's native close X does the same thing — both halt the animation but leave the tray icon and settings panel running, so Start brings the animation window back. The settings panel is still the way to fully quit (Quit button, or close the settings window on Linux).

## Performance vs the legacy Swift build

Live A/B on macOS (M3 Max, default settings, single monitor, settings window closed). 30 s window, 15 samples via `ps -o %cpu`; both numbers normalised to one CPU core:

| Build           | avg CPU |    range |
|-----------------|--------:|---------:|
| Swift (Release) |  4.95 % | 3.2 – 6.6 |
| Rust  (Release) |  3.19 % | 1.5 – 4.3 |

Rust runs about **36 % lower CPU in steady state**. The delta is statistically robust (means ~5σ apart) but small in absolute terms (~1.8 percentage points). Opening the settings window adds roughly 1–2 pp on both builds; each additional monitor adds another ~0.2–0.4 pp on Rust (one render thread per overlay).

Reproduce via `cargo run --release --example cpu_bench -p exhale-render` for the headless per-frame number, or by running both binaries side-by-side under `ps -o %cpu` for the live-process number above.

## Ship & distribute

| Target                       | Status | Notes |
|------------------------------|--------|-------|
| Mac App Store                | ready  | Sandbox-safe `flock(2)` single-instance guard; sandbox-friendly AppleEvent registration; `scripts/bundle-mas.sh` (universal binary, Developer-ID signed `.pkg`, sandbox entitlements) |
| macOS standalone (signed)    | ready  | `scripts/bundle-mas.sh` |
| Microsoft Store              | ready  | MSIX wrapper via `bundle-msix.ps1`, all required tile assets generated (Wide310x150, Square71x71, Square310x310, SplashScreen) |
| Windows standalone           | ready  | `cargo build --release` produces a self-contained `.exe` |
| Snap Store                   | ready  | Strict-confined snap with the `gnome` extension. Upload is currently manual from a Multipass `snap-creds` VM (`snapcraft upload`) |
| Linux `.deb` / AppImage      | ready  | `cargo deb` + `scripts/bundle-appimage.sh` |

## Minimal Python script fallback

For tinkerers, distros where the Snap doesn't fit (Alpine, NixOS, immutable distros), or anyone who'd rather just read 200 lines of Python and tweak constants at the top of a file:

![exhalePython](https://user-images.githubusercontent.com/60944077/222979803-c88ebc65-b799-4ca7-b265-54beb27fcb00.gif)

```sh
git clone https://github.com/peterklingelhofer/exhale.git
cd exhale/python
python main.py
```

Modify the constants at the top of [`python/main.py`](python/main.py) for inhale/exhale duration in seconds, shape mode, and full-screen toggle.

**The Rust binary is the recommended path on every supported OS, including Wayland.** On a typical Wayland desktop the compositor doesn't expose alpha-capable swap chains, so the Rust binary opens as a regular movable window — you can either watch the animation directly in that window OR send it behind your other apps and narrow them so the animation peeks through the edges, exactly the same "make room for the overlay" trick this Python script uses in its bars mode (see the [Linux (Wayland) platform note](#platform-notes) above for details). The Python script is a hackable single-file alternative, not a performance recommendation.

## Companion repository

A Perl version of this exists at <https://github.com/franco3445/Breathing>.

---

## Deprecated implementations

The implementations below are superseded by the Rust port above and are kept in the repo for historical reference only. They will not receive new features or fixes. Use the Rust binary on every supported OS.

### Swift macOS app (`swift/`)

The original macOS-only implementation, written in SwiftUI + Metal. The Rust port is a strict superset: same overlay, same hotkeys, same settings, plus Windows and Linux support, with measurably lower per-frame CPU on every complex scene (see Performance table above). The Mac App Store listing will be updated to the Rust build going forward; the Swift source remains for reference.

```sh
git clone https://github.com/peterklingelhofer/exhale.git
cd exhale/swift
xed .
```

### TypeScript / Electron app (`typescript/`)

Cross-platform Electron build that predates the Rust port. The Rust binary covers macOS + Windows + Linux from a single ~10 MB native executable, with far lower CPU than the Electron build (which bundles a full Chromium runtime). Settings live in `localStorage` and have to be edited via DevTools; the Rust port has a real settings UI.

```sh
git clone https://github.com/peterklingelhofer/exhale.git
cd exhale/typescript
pnpm install
pnpm start
```

To recompile automatically with [electron-reload](https://github.com/yan-foto/electron-reload):

```sh
pnpm watch
```
