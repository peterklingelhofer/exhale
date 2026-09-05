# exhale

A minimal cross-platform breathing overlay: a friendly indicator and reminder to take full, deep breaths while looking at screens.

Demanding work at a keyboard measurably changes how you breathe. Data-entry operators monitored across full working days ran significantly *lower* end-tidal CO2 and *faster* respiration during data entry than during relaxation ([Schleifer & Ley 1994](docs/CITATIONS.md#schleifer1994-vdt-petco2), [Schleifer et al. 2008](docs/CITATIONS.md#schleifer2008-emg-gaps-computer-work)): two small samples from one research group, 34 people in all. The same pattern appears under cognitive load generally, where breathing gets faster and end-tidal CO2 falls while depth stays roughly stable ([Grassmann et al. 2016](docs/CITATIONS.md#grassmann2016-cognitive-load-respiration), 54 experiments), so the keyboard is where the effect was measured rather than its proven cause. A hyperventilation theory of job stress proposes that the pattern involves a shift from diaphragmatic to thoracic breathing ([Schleifer, Ley & Spalding 2002](docs/CITATIONS.md#schleifer2002-hyperventilation-job-stress)); that shift is a theory; no study has measured it in screen users. Screen posture compounds it: heavy smartphone use tracks with worse head posture and lower peak expiratory flow ([Jung et al. 2016](docs/CITATIONS.md#jung2016-smartphone-posture-respiration)), and forward head posture is associated with FVC reductions of 0.25 to 0.81 L ([Deniz et al. 2024](docs/CITATIONS.md#deniz2024-forward-head-lung-volumes)).

Slow paced breathing is the breathing practice with the most published evidence behind it ([Laborde et al. 2022](docs/CITATIONS.md#laborde2022-vsb-meta), 223 studies; [Fincham et al. 2023](docs/CITATIONS.md#fincham2023-breathwork-meta), g = -0.35 for self-reported stress; [Zaccaro et al. 2018](docs/CITATIONS.md#zaccaro2018-slow-breathing-review)). Whether it counters the over-breathing above is untested, and one study found it can add to it (gap 5 in the ledger). exhale is a way to run one with no sensor, no account and no telemetry. In two single-session comparisons, a plain expanding shape at 6 breaths per minute raised heart rate variability as much as sensor-driven biofeedback ([Tabor et al. 2022](docs/CITATIONS.md#tabor2022-guided-breathing-design), [Laborde et al. 2021](docs/CITATIONS.md#laborde2021-spb-6cpm-biofeedback)); the biofeedback group in the second reported slightly more positive mood, and finding a personal resonance frequency still takes a sensor, so the claim is narrow: for one session's physiological effect, the hardware added nothing.

Blink rate also falls sharply at a display ([Tsubota & Nakamori 1993](docs/CITATIONS.md#tsubota1993-vdt-blink), [Rosenfield 2011](docs/CITATIONS.md#rosenfield2011-computer-vision-syndrome), [Sheppard & Wolffsohn 2018](docs/CITATIONS.md#sheppard2018-digital-eye-strain)). exhale does nothing about that; it paces breathing only.

Every claim above and below is sourced in **[docs/CITATIONS.md](docs/CITATIONS.md)**, 48 sources whose bibliographic records were checked against Crossref, Open Library or PubMed. Each link lands on that source's corpus entry, which carries its DOI, its access level, its evidence tier and its caveats, rather than on the paper directly, because several of these findings are weaker than a bare citation would suggest. The [gaps ledger](docs/CITATIONS.md#gaps-and-unsupported-choices) collects fourteen places where exhale ships something the literature doesn't back.

The overlay is a translucent always-on-top window that gently expands on inhale and contracts on exhale. Inhale, post-inhale hold, exhale, and post-exhale hold durations are all configurable.

**The default is `5` / `0` / `5` / `0`.** Five seconds in, five out, no holds. That's 6 breaths per minute, the rate at which most of the direct evidence was gathered. The band tested head-on runs 5 to 7 breaths per minute, and every rate in it beat spontaneous breathing ([You et al. 2023](docs/CITATIONS.md#you2023-respiratory-frequency)); average individual resonance frequency sits near 5.5 ([Lehrer & Gevirtz 2014](docs/CITATIONS.md#lehrer2014-hrv-biofeedback)), and one study found 5.5 beat 6 ([Lin et al. 2014](docs/CITATIONS.md#lin2014-equal-ratio-hrv)). In a four-way head-to-head (n = 84), 6 breaths per minute raised heart rate variability more than either box breathing or 4-7-8 ([Marchant et al. 2025](docs/CITATIONS.md#marchant2025-square-478-six)). Holding the breath is the hard part for a beginner; the slow part isn't, so this is also the gentlest place to start.

Box breathing is `4` / `4` / `4` / `4` and exhale supports it, but note it's a 16-second cycle, or 3.75 breaths per minute: the holds hide the fact that it's *slower* than it looks. It lost that head-to-head ([Marchant et al. 2025](docs/CITATIONS.md#marchant2025-square-478-six), whose authors note square and 4-7-8 breathing "have little empirical support"), and in a month-long randomised trial it didn't separate from the control on mood while exhale-emphasising cyclic sighing did ([Balban et al. 2023](docs/CITATIONS.md#balban2023-cyclic-sighing)); the box arm had 21 people and the two patterns were never tested against each other. The same applies to 4-7-8.

**On making the exhale longer than the inhale:** a preference. Whether a longer exhale raises heart rate variability more than an equal one is split: three studies found an effect ([Bae et al. 2021](docs/CITATIONS.md#bae2021-exhalation-inhalation-ratio), [Van Diest et al. 2014](docs/CITATIONS.md#vandiest2014-ie-ratio-relaxation), [Laborde et al. 2021](docs/CITATIONS.md#laborde2021-ie-ratio-pauses)), one found the *equal* ratio better ([Lin et al. 2014](docs/CITATIONS.md#lin2014-equal-ratio-hrv)), and one found no difference across an original experiment and its own replication ([Meehan & Shaffer 2024](docs/CITATIONS.md#meehan2024-longer-exhalations)), whose review of the older literature adds three further nulls and one result the other way. No mechanism claim survives that split, so exhale doesn't make one.

The subjective evidence is thinner than it's usually presented. One study of 30 people found the longer exhale produced more reported relaxation, stress reduction, mindfulness and positive energy, while slowing the rate alone moved only one of those four ([Van Diest et al. 2014](docs/CITATIONS.md#vandiest2014-ie-ratio-relaxation)). A larger one, 84 people, measured mood across both ratios at 6 breaths per minute and found no meaningful change in any condition ([Marchant et al. 2025](docs/CITATIONS.md#marchant2025-square-478-six)), and a third found *every* slow pattern beat baseline on relaxation with no ratio-specific edge ([Lin et al. 2014](docs/CITATIONS.md#lin2014-equal-ratio-hrv)). A month-long trial points toward exhale emphasis on mood ([Balban et al. 2023](docs/CITATIONS.md#balban2023-cyclic-sighing)), though its cyclic-sighing arm also adds a double inhale. Prefer a longer exhale if it feels better to you; rate does most of the work, and ratio is a preference with a small and contested edge.

Being inside the tested band is a claim about coverage; it says nothing about optimality. That band is five values wide, and resonance frequency varies from person to person ([Lehrer & Gevirtz 2014](docs/CITATIONS.md#lehrer2014-hrv-biofeedback)), which no single shipped number can accommodate. Treat 6 a minute as a good starting point rather than as your number. exhale's earlier default, `5` in / `10` out at 4 breaths per minute, is still one click away as a preset; nobody has measured 4 a minute. All of this, including the arithmetic, is laid out in [the gaps ledger](docs/CITATIONS.md#gaps-and-unsupported-choices).

Take breaks if intense feelings arise; don't overdo it. Few adverse effects are expected from *slow* breathing specifically ([Laborde et al. 2022](docs/CITATIONS.md#laborde2022-vsb-meta)), but exhale's sliders can also be set to fast, hold-heavy patterns that leave that evidence base for the high-ventilation literature, where transient tetany and light-headedness are documented ([Fincham et al. 2024](docs/CITATIONS.md#fincham2024-high-ventilation-rct)).

## Disclaimer

The information and guidance provided by this app are intended for general informational purposes only and aren't medical advice. The creator isn't a medical professional. Always seek the advice of a qualified healthcare provider with any questions about your health, and don't disregard or delay professional medical advice because of this app. Use is at your own risk.

## What to expect

**How long before it does anything.** In a lab comparison, HRV changes appeared roughly two minutes
into a paced session ([Tabor et al. 2022](docs/CITATIONS.md#tabor2022-guided-breathing-design)). Expect the effect
to last about as long as the pacer runs: the one study of an ambient on-screen pacer during real
information work found breathing rate dropped while pacing was active and didn't persist as a
lasting change ([Moraveji et al. 2011](docs/CITATIONS.md#moraveji2011-peripheral-paced-respiration)). The trial that
showed a month-long mood benefit used five minutes a day
([Balban et al. 2023](docs/CITATIONS.md#balban2023-cyclic-sighing)). Running exhale all day is fine; just don't
expect all-day carryover from it.

**If you use the reminder timer instead of the always-on overlay.** Breaks of ten minutes or less
reduce fatigue and increase vigor ([Albulescu et al. 2022](docs/CITATIONS.md#albulescu2022-micro-breaks), 22
studies), and self-reported relief is higher after a single instructed deep breath than before it
([Vlemincx et al. 2016](docs/CITATIONS.md#vlemincx2016-sigh-relief)). That's the closest published support for
exhale's smallest gesture, which is being told to take one breath.

**Breathe through your nose.** exhale can't show you this and doesn't try, but it's free. Nasal
respiration entrains oscillations in human piriform cortex, amygdala and hippocampus, and the effect
is specific to the nasal route rather than to breathing as such
([Zelano et al. 2016](docs/CITATIONS.md#zelano2016-nasal-respiration-limbic)).

**Tune the numbers to yourself.** Resonance frequency is individual, and taller people and men tend
to have lower ones ([Lehrer & Gevirtz 2014](docs/CITATIONS.md#lehrer2014-hrv-biofeedback)). That's the honest
reason exhale's timings are sliders rather than a hardcoded rate: there's no single correct number
to ship.

**Why breathing reaches how you feel at all.** Heart rate rises on inhalation and falls on
exhalation ([Yasuma & Hayano 2004](docs/CITATIONS.md#yasuma2004-rsa)), which is the coupling every HRV claim here
rests on. Separately, a small population of neurons in the mouse breathing rhythm generator projects
onto the locus coeruleus, and ablating them left breathing intact while increasing calm behaviour
([Yackle et al. 2017](docs/CITATIONS.md#yackle2017-breathing-arousal-neurons)). That's a plausible route from
breathing pattern to arousal state. It's mice, and it's a reason rather than a result.

## Research and evidence

exhale's premise, its defaults and its interface choices are traced to the literature in
**[docs/CITATIONS.md](docs/CITATIONS.md)**: 48 sources, 45 verified against the Crossref REST API,
2 against Open Library and 1 against PubMed, each carrying an access level, an evidence tier and the
specific claims it's allowed to back.

The corpus is deliberately not a sales pitch. Alongside the meta-analyses that support slow paced
breathing it carries the published dissent: a meta-analysis finds sustained slow breathing lowers
systolic pressure by about 5.6 mmHg ([Chaddha et al. 2019](docs/CITATIONS.md#chaddha2019-slow-breathing-bp)), and a
letter in a hypertension journal argues that case should be considered closed
([van Dijk et al. 2018](docs/CITATIONS.md#vandijk2018-close-the-book)). It also carries the failed replications, a null result
against exhale's own genre ([Johnson & Rosenfield 2023](docs/CITATIONS.md#johnson2023-20-20-20), where scheduled
on-screen breaks did nothing), and a
[ledger of fourteen places](docs/CITATIONS.md#gaps-and-unsupported-choices) where exhale ships
something the literature doesn't settle. Four highlights:

- The **tested range is only 5 to 7 breaths per minute wide** ([You et al. 2023](docs/CITATIONS.md#you2023-respiratory-frequency)), and individual resonance frequency varies within and beyond it ([Lehrer & Gevirtz 2014](docs/CITATIONS.md#lehrer2014-hrv-biofeedback)). exhale's default of 6 sits inside it; box breathing, at 3.75, doesn't.
- Whether a **longer exhale** beats an equal one on heart rate variability is split five ways ([Bae et al. 2021](docs/CITATIONS.md#bae2021-exhalation-inhalation-ratio), [Van Diest et al. 2014](docs/CITATIONS.md#vandiest2014-ie-ratio-relaxation), [Laborde et al. 2021](docs/CITATIONS.md#laborde2021-ie-ratio-pauses), [Lin et al. 2014](docs/CITATIONS.md#lin2014-equal-ratio-hrv), [Meehan & Shaffer 2024](docs/CITATIONS.md#meehan2024-longer-exhalations)). On how people report *feeling*, one study favours it ([Van Diest et al. 2014](docs/CITATIONS.md#vandiest2014-ie-ratio-relaxation)) and a larger one found no mood difference ([Marchant et al. 2025](docs/CITATIONS.md#marchant2025-square-478-six)), which is why exhale offers the ratio as a preference rather than a recommendation.
- The closest published analogue to exhale, an ambient on-screen pacer running during real information work, lowered breathing rate **only while it was running** ([Moraveji et al. 2011](docs/CITATIONS.md#moraveji2011-peripheral-paced-respiration)). Treat an always-on overlay as an effect that lasts as long as it's on. Visual pacing also changes breathing more than audio while feeling *less* calming ([Wongsuphasawat et al. 2012](docs/CITATIONS.md#wongsuphasawat2012-cant-force-calm)), which is a trade exhale makes deliberately.
- Slow pacing itself mildly increases over-breathing ([Marchant et al. 2025](docs/CITATIONS.md#marchant2025-square-478-six)), and screen workers are already mildly hypocapnic ([Schleifer & Ley 1994](docs/CITATIONS.md#schleifer1994-vdt-petco2), [Schleifer et al. 2008](docs/CITATIONS.md#schleifer2008-emg-gaps-computer-work)). Nobody has tested a slow pacer on that population, which is exactly exhale's user.

The tradition exhale actually descends from is named rather than airbrushed. The four-phase
inhale / hold / exhale / hold structure is pranayama ([Satyananda Saraswati 1999](docs/CITATIONS.md#satyananda1999-apmb),
[Muktibodhananda 1998](docs/CITATIONS.md#muktibodhananda1998-hatha-yoga-pradipika)), and both references are carried
at evidence tier **E**: citable for lineage, never for whether anything works. Both are catalogue
records only, checked against Open Library and not read. Retrofitting a 2020s HRV
citation onto an instruction that's centuries older would be revisionist about the app's own design
history, and knowing the longer-exhale idea reached breathing apps through this tradition rather than
through a laboratory is worth weighing when reading the studies that later tested it.

The corpus is generated. [`docs/CITATIONS.csl.json`](docs/CITATIONS.csl.json) holds
the records, [`docs/citations-notes.md`](docs/citations-notes.md) holds the prose, and
[`scripts/generate-citations.py`](scripts/generate-citations.py) renders the two into `CITATIONS.md`:

```sh
uv run --no-project scripts/generate-citations.py           # regenerate
uv run --no-project scripts/generate-citations.py --check   # fail if stale
```

`--check` also validates citekey format, enum values, every anchor link the gaps ledger makes into
the corpus, the source counts quoted in this file, and the deep link the app itself compiles in, so a
renamed entry or a stale number breaks the build rather than rotting silently. It also refuses a
short list of claims the corpus doesn't support, anywhere they could be asserted rather than
discussed: the Rust sources, the Snap description and the Microsoft Store listing copy. A store
listing is edited somewhere a README review never reaches, which is exactly how the two drift.

**In the app.** A *Research* item in the macOS app menu, directly under About, and the same item in
the system-tray menu on every platform. Both open the gaps ledger rather than the top of the
reference list, which is the whole point of the item: the label is plain, so the anchor carries the
intent. The Timing panel offers five patterns as one-click presets, computes the current rate from
the four duration fields, and prints it against the tested range, unprompted, so a configuration
outside that range says so where it's being chosen. Selecting box breathing makes the panel state
that it's 3.8 breaths a minute and slower than anything tested directly, which is the honest thing
to say about a pattern people arrive looking for by name. No preset carries a badge, a rank or an evidentiary caption: the arithmetic
below them is computed live for whichever is selected, so it can't go stale and doesn't have to
be maintained against the corpus.

The binary states arithmetic and one range and nothing else: no effect, no benefit, no condition.
That restraint is deliberate. Everything evidentiary lives in this repository, which can be
corrected in an afternoon, rather than in a signed binary that takes a store-review cycle to
withdraw. Four sources are flagged `inAppCitable: false` in the corpus for exactly that reason, and
the build fails if a shipped preset points at one of them.

Nothing in this section is medical advice; see the [disclaimer](#disclaimer) above.

## Download

Pre-built binaries for each OS are on the [Releases](https://github.com/peterklingelhofer/exhale/releases) page. Using the latest release is recommended; if you hit a problem, please [open an issue](https://github.com/peterklingelhofer/exhale/issues/new).

**Mac**

[<img src="https://user-images.githubusercontent.com/60944077/232312847-df673556-fb5e-49b4-8037-4d38267e6e18.png"  width="157" height="63"></img>](https://apps.apple.com/us/app/exhale-breath/id6447758995?mt=12)

**Windows**: install the MSIX from the Microsoft Store, or grab the standalone `.exe` from Releases.

**Linux**: install the Snap from the Snap Store, or build from source.

## Usage

![circle](https://user-images.githubusercontent.com/60944077/226204981-f390facc-4f6c-4bec-8784-23203aa64efc.gif)
![rectangle](https://user-images.githubusercontent.com/60944077/226204986-7522cb4d-7df1-4d65-96de-e629197e9854.gif)
<img width="447" height="981" alt="Settings panel" src="https://github.com/user-attachments/assets/32e1d10e-72e3-4acb-ae35-be186cd7cb19" />

### Global keyboard shortcuts

Only one shortcut ships bound by default:

| Shortcut                                    | Action                 |
|---------------------------------------------|------------------------|
| <kbd>Ctrl</kbd>+<kbd>Shift</kbd>+<kbd>,</kbd> | Open / close Preferences |

Every other action (Start / Stop / Reset / Quit) is **unbound** on first launch so exhale never collides with another app's global shortcut without the user opting in. Customise via either path:

- **Right-click** the Start / Stop / Reset / Quit buttons in the Preferences panel -> "Change Shortcut…" -> press your combo
- **Tray menu** -> "Keyboard Shortcuts ▶" -> pick any of the five actions to start a capture

Press **Esc** in the capture overlay to cancel. "Reset Shortcut to Default" in the right-click menu restores the per-action factory value (which, for everything except Preferences, is "unbound"). Reset to Defaults in the panel clears all custom bindings too.

**Notice:** A high opacity value can obscure the Preferences pane in the current workspace. Bind a Reset shortcut as described above and use it to recover, or:

1. Swipe to a different workspace.
2. Close Preferences from the menu bar.
3. Re-open Preferences in the current workspace and adjust Opacity.
4. Switch back.

## Architecture

Single Rust workspace (`rust/`) producing one cross-platform binary.

- **Renderer**: `wgpu` + a single WGSL fragment shader (`crates/exhale-render`)
- **Window system**: `winit`
- **Settings UI**: `egui` (hand-rolled stepper, segmented picker, control buttons painted directly via `egui::Painter` to match `NSSegmentedControl` / `NSStepper` look)
- **AppKit interop**: typed FFI via `objc2` for the menu-bar, status-bar level, and `NSApplicationActivationPolicy` paths. `platform/mac.rs` and `timers.rs` still use raw `msg_send!` where typed bindings don't exist yet (window-level juggling, NSUserNotification plumbing)
- **Threading model**: per-overlay-window render thread + per-window `wgpu::Device` so overlay frame delivery isn't gated by the main thread's message queue or the settings window's GPU submissions
- **Animation cadence**: 24 fps while the breath animation is running (matches the legacy Swift `MetalBreathingController`); drops to 1 fps when the controller has nothing dynamic to draw (paused, fullscreen-with-matching-colors tint, or all-zero durations). Hardcoded; per-frame CPU runs ≤ 2 % on every scene tested, so the earlier user-tunable preset was removed

### Crates

- `exhale-core`: settings + `SettingsDiff`, breathing controller (deadline-scheduled background thread), poison-tolerant lock helpers, easing tables. Zero GUI deps.
- `exhale-render`: `wgpu` renderer + WGSL fragment shader, headless benchmarking harness (`cargo run --example cpu_bench`).
- `exhale-app`: winit event loop, split egui settings panel (`settings_window.rs` + `widgets.rs` + `theme.rs`), per-overlay render thread, tray, hotkeys, platform glue (`objc2` / `windows-sys` / `x11-dl`).

## Build & run

The `cargo run` family builds and then launches the binary in one step. The `cargo build` family only compiles; you have to invoke the binary yourself afterwards.

| Command                  | Builds | Runs | Build profile               |
|--------------------------|:------:|:----:|-----------------------------|
| `cargo run`              |  Yes   | Yes  | Dev (debug, fast compile)   |
| `cargo run --release`    |  Yes   | Yes  | Release (optimised)         |
| `cargo build`            |  Yes   | No   | Dev                         |
| `cargo build --release`  |  Yes   | No   | Release                     |

All commands run from `rust/`. Use dev builds while iterating (compile is ~10× faster), release for the real binary you'd ship or benchmark. Binaries land at:

- Dev:     `rust/target/debug/exhale` (or `.exe` on Windows)
- Release: `rust/target/release/exhale` (or `.exe` on Windows)

### Browsing the type-level docs

```sh
cargo doc --no-deps --workspace --open
```

Generates HTML docs for the three local crates and opens them in your browser. `--no-deps` skips the ~200 dependency crates so you only see exhale's own types. See [LEARNING.md](LEARNING.md) for a beginner's tour of the codebase.

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

**macOS**: no extra prerequisites beyond Rust.

**Windows**: no extra prerequisites. Works with both the MSVC and GNU toolchains.

**Linux**: exhale dynamically loads several system libraries at run time. To build and run on Debian/Ubuntu:

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

If you're **running** a pre-built binary (not compiling from source), the bare runtime packages are enough; drop the `-dev` suffixes:

```sh
sudo apt install libgtk-3-0 libayatana-appindicator3-1 libwayland-client0 libxkbcommon0 libxdo3 libssl3
```

X11 and Xfixes are loaded via `x11-dl` at run time using whatever's already installed by the X11 desktop, so they're not in the list.

What each one is for:
- `libgtk-3` + `libayatana-appindicator3`: system-tray icon backend
- `libwayland-client` + `libxkbcommon`: winit's Wayland + keyboard input
- `libxdo`: `global-hotkey` crate's X11 keyboard binding (`libxdo.so.3` at runtime)
- `libssl`: TLS for crates that fetch over HTTPS
- `pkg-config`: build-time library discovery (compile-only)

## Settings

Settings are saved as TOML under the platform config dir (via the `directories` crate's `ProjectDirs::from("com", "peterklingelhofer", "exhale")`):

| Platform | Path |
|----------|------|
| macOS (dev / standalone) | `~/Library/Application Support/com.peterklingelhofer.exhale/settings.toml` |
| macOS (Mac App Store)    | `~/Library/Containers/peterklingelhofer.exhale/Data/Library/Application Support/com.peterklingelhofer.exhale/settings.toml` |
| Windows  | `%APPDATA%\peterklingelhofer\exhale\config\settings.toml` |
| Linux    | `~/.config/exhale/settings.toml` |

The MAS path differs because the App Store build runs sandboxed; the sandbox redirects `~/Library/Application Support` writes into the per-app container. Settings are reloaded on launch and persisted on every change via a debounced background writer thread; corrupt TOML is logged and the file is rewritten with defaults.

## Platform notes

- **macOS**: the overlay floats above fullscreen apps (screen-saver window level), joins every Space, and stays out of Cmd+Tab. `AppVisibility` toggles `NSApp.setActivationPolicy` between `.regular` and `.accessory`.
- **Windows**: the overlay uses `WS_EX_LAYERED | WS_EX_TRANSPARENT | WS_EX_TOOLWINDOW | WS_EX_NOACTIVATE | WS_EX_TOPMOST`. `AppVisibility` toggles `WS_EX_APPWINDOW` / `WS_EX_TOOLWINDOW` on the settings window so "DockOnly" shows a taskbar entry and "TopBarOnly" hides it. Tested + recommended on **Windows 11**; some Windows 10 GPU + driver combinations report only `Opaque` alpha modes to Vulkan, in which case exhale falls back to the windowed mode described below.
- **Linux (X11)**: click-through via `XFixesSetWindowShapeRegion` with an empty input region; always-on-top via `_NET_WM_STATE_ABOVE`; workspace-spanning via `_NET_WM_STATE_STICKY`; `AppVisibility` toggles `_NET_WM_STATE_SKIP_TASKBAR` / `SKIP_PAGER` on the settings window.
- **Linux (Wayland)**: exhale picks one of two paths at startup based on whether the compositor exposes alpha-capable swap chains to wgpu:
   - **Compositor supports alpha** (rare on current Mutter/GNOME, supported by some KWin setups): the overlay is placed at `AlwaysOnBottom` because Wayland's security model doesn't surface a portable click-through / always-on-top protocol to winit (`wp_input_region` isn't exposed). Your app windows cover the overlay by default; to see the breath animation, **narrow your foreground windows so they don't fill the whole screen** and the animation shows through the gap.
   - **Compositor only exposes Opaque alpha** (typical real-hardware Wayland session on Ubuntu / Fedora GNOME): exhale falls back to the **windowed mode** described below.
   For full topmost + click-through overlay behavior on Linux, log out and pick an X11 session at the login screen.
- **Windowed-mode fallback** (Wayland sessions without alpha, some Windows 10 + Vulkan combinations, WARP / Microsoft Basic Render Driver, remote-desktop sessions): the breath animation runs in a **480×360 movable, resizable "exhale" window** with normal decorations and full window-manager participation (Alt-Tab, taskbar, native close button). You can use it two ways: (1) **as a foreground window**, watching the breath animation directly the same way you'd watch any other app, or (2) **as an edge-strip overlay**, by sending the window behind your other apps (Alt-Tab past it / click on the window manager to lower it), switching exhale to **Rectangle mode**, and narrowing the windows in front so the animation shows through the side / bottom strips you've left open. The Stop button (and the global Stop hotkey, if bound) hides this window; clicking the window's native close X does the same thing; both halt the animation but leave the tray icon and settings panel running, so Start brings the animation window back. The settings panel is still the way to fully quit (Quit button, or close the settings window on Linux).

## Performance vs the legacy Swift build

Live A/B on macOS (M3 Max, default settings, single monitor, settings window closed). 30 s window, 15 samples via `ps -o %cpu`; both numbers normalised to one CPU core:

| Build           | avg CPU |    range |
|-----------------|--------:|---------:|
| Swift (Release) |  4.95 % | 3.2 – 6.6 |
| Rust  (Release) |  3.19 % | 1.5 – 4.3 |

Rust runs about **36 % lower CPU in steady state**. The delta is statistically clear (means ~5σ apart) but small in absolute terms (~1.8 percentage points). Opening the settings window adds roughly 1–2 pp on both builds; each additional monitor adds another ~0.2–0.4 pp on Rust (one render thread per overlay).

Reproduce via `cargo run --release --example cpu_bench -p exhale-render` for the headless per-frame number, or by running both binaries side-by-side under `ps -o %cpu` for the live-process number above.

## Ship & distribute

Per-release build, sign, package, and store-upload instructions for every supported target live in [DEPLOYMENT.md](DEPLOYMENT.md). Quick summary:

| Target                       | Script                          |
|------------------------------|---------------------------------|
| Mac App Store                | `rust/scripts/bundle-mas.sh`    |
| Microsoft Store              | `rust\scripts\bundle-msix.ps1`  |
| Snap Store                   | CI builds, manual upload via Multipass `snap-creds` VM |
| Linux `.deb` / AppImage      | `cargo deb` + `rust/scripts/bundle-appimage.sh` |
| Windows standalone `.exe`    | `cargo build --release`         |

CI in [.github/workflows/release.yml](.github/workflows/release.yml) builds every artifact on a `v*` tag.

## Minimal Python script fallback

For tinkerers, distros where the Snap doesn't fit (Alpine, NixOS, immutable distros), or anyone who'd rather just read 200 lines of Python and tweak constants at the top of a file:

![exhalePython](https://user-images.githubusercontent.com/60944077/222979803-c88ebc65-b799-4ca7-b265-54beb27fcb00.gif)

```sh
git clone https://github.com/peterklingelhofer/exhale.git
cd exhale/python
python main.py
```

Modify the constants at the top of [`python/main.py`](python/main.py) for inhale/exhale duration in seconds, shape mode, and full-screen toggle.

**The Rust binary is the recommended path on every supported OS, including Wayland.** On a typical Wayland desktop the compositor doesn't expose alpha-capable swap chains, so the Rust binary opens as a regular movable window; you can either watch the animation directly in that window, or send it behind your other apps and narrow them so the animation peeks through the edges, exactly the same "make room for the overlay" trick this Python script uses in its bars mode (see the [Linux (Wayland) platform note](#platform-notes) above for details). The Python script is a hackable single-file alternative; it isn't meant as a performance recommendation.

## Companion repository

A Perl version of this exists at <https://github.com/franco3445/Breathing>.

---

## Deprecated implementations

The implementations below are superseded by the Rust port above and are kept in the repo for historical reference only. They won't receive new features or fixes. Use the Rust binary on every supported OS.

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
