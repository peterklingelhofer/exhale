# exhale for researchers

Reference for a lab deciding whether exhale can be the stimulus in a small study: what the app
is, how study mode locks a participant's pacing, what the session log records, and how to cite
the software.

## What exhale is

exhale is a translucent overlay that expands while you inhale and contracts while you exhale, and
on most setups it floats above your other windows. Its pace is set by four phase durations:
inhale, a hold after inhale, exhale, and a hold after exhale. Shape, color and opacity are
separate appearance settings and don't change the durations.

exhale needs no sensor and no account, and it has no telemetry: see [`PRIVACY.md`](../PRIVACY.md)
for what it does and doesn't collect. It runs as a native binary on macOS, Windows and Linux. The
[README](../README.md) covers installation, keyboard shortcuts and the platform-specific overlay
behavior this document doesn't repeat.

## Parameters and defaults

| Parameter | Default | What it does |
|---|---|---|
| Inhale (`inhale_duration`) | 5 s | Seconds the overlay takes to expand |
| Post-inhale hold (`post_inhale_hold_duration`) | 0 s | Seconds held expanded before the exhale starts |
| Exhale (`exhale_duration`) | 5 s | Seconds the overlay takes to contract |
| Post-exhale hold (`post_exhale_hold_duration`) | 0 s | Seconds held contracted before the next inhale |
| Drift (`drift`) | 1.0 (off) | Each cycle's length is the previous cycle's length times this number, so anything above 1.0 makes every cycle a little longer than the last |
| Randomization (`randomized_timing_*`, one field per phase) | 0% (off) | Random jitter on that phase, as a fraction of its length in either direction. The Settings window shows it as a percent and stores it as 0.0 to 1.0, so 0.25 on a 4 s phase gives 3 to 5 s, and a phase set to 0 stays 0 |
| Reminder interval (`reminder_interval_minutes`) | 0 min (off) | Minutes between reminder notifications |
| Auto-stop (`auto_stop_minutes`) | 0 min (off) | Minutes of running before the overlay stops itself |

Rate is 60 divided by the cycle length in seconds. The shipped default, `5 / 0 / 5 / 0`, is a
10-second cycle: 6 a minute. `2.5 / 0 / 2.5 / 0` is a 5-second cycle: 12 a minute. A phase set
to 0 takes no time at all, and a phase set to anything above 0 never runs shorter than 0.1 s.

## Evidence and its limits

[`docs/CITATIONS.md`](CITATIONS.md) exists so that a reader can check which of exhale's claims and
defaults rest on published evidence and which don't. Its
[reading guide](CITATIONS.md#how-to-read-this) explains the verification status, access level and
evidence tier behind each entry, and its tier table states how exhale may use a source of each
tier. The [gaps ledger](CITATIONS.md#gaps-and-unsupported-choices) at the end of that document
lists, in its own words, where exhale ships something the literature doesn't settle. It includes
drift, which it describes as an invention of this app, and randomized timing, which it says has no
literature behind it.

## Study mode

A `study.toml` file locks a participant's pacing in the app: Settings shows no rate and no pacing
controls, which is what makes the app usable for a blinded arm. The assigned pace stays readable
on the overlay itself, in `study.toml` and in the log.

### The file

`study.toml` sits beside `settings.toml`, in the same per-platform configuration folder (see the
README's [Settings](../README.md#settings) table):

| Platform | Path |
|---|---|
| macOS | `~/Library/Application Support/com.peterklingelhofer.exhale/study.toml` |
| Windows | `%APPDATA%\peterklingelhofer\exhale\config\study.toml` |
| Linux | `~/.config/exhale/study.toml` |

These paths belong to builds that run outside a package sandbox: a binary built from source, the
`.deb` and the `.AppImage`. The Mac App Store build is sandboxed and keeps its files in the
container path the README's Settings table lists, and this guide gives no path for the Snap Store
or MSIX packages.

A complete file, with every field:

```toml
participant_id = "P07"                     # opaque id the lab assigns, the app never interprets it
condition = "B"                            # opaque condition code, the app never interprets it
inhale_duration = 5.0                      # seconds
post_inhale_hold_duration = 0.0            # seconds
exhale_duration = 5.0                      # seconds
post_exhale_hold_duration = 0.0            # seconds
drift = 1.0                                # per-cycle multiplier, 1.0 is off
randomized_timing_inhale = 0.0             # 0.0 to 1.0, jitter on the inhale phase
randomized_timing_post_inhale_hold = 0.0   # 0.0 to 1.0, jitter on the post-inhale hold
randomized_timing_exhale = 0.0             # 0.0 to 1.0, jitter on the exhale phase
randomized_timing_post_exhale_hold = 0.0   # 0.0 to 1.0, jitter on the post-exhale hold
auto_stop_minutes = 30.0                   # minutes before the overlay stops itself, 0 is off
reminder_interval_minutes = 0.0            # minutes between reminder notifications, 0 is off
```

Only six fields are required. Of the others, `drift` defaults to `1.0` and the rest to `0.0`:

```toml
participant_id = "P07"
condition = "B"
inhale_duration = 5.0
post_inhale_hold_duration = 0.0
exhale_duration = 5.0
post_exhale_hold_duration = 0.0
```

### Validation rules

- `participant_id` and `condition`: not empty after trimming whitespace
- The four durations: each finite, `>= 0` and `<= 86400` (one day in seconds, the longest phase
  the controller schedules), and their sum must be greater than `0` (the app's message for a file
  that fails the sum rule reads "the four durations add up to 0, which isn't a pace")
- `drift`: finite and `>= 1.0`
- Each `randomized_timing_*` field: finite and between `0.0` and `1.0`
- `auto_stop_minutes` and `reminder_interval_minutes`: finite and `>= 0`
- Any field name the app doesn't recognize fails the whole file, so a typo always produces an
  error
- Every error starts with `study.toml:`, and a range error names the field and the value it
  rejected

### What locking changes

With a valid file loaded, the app writes its eleven numeric fields (the four durations, `drift`,
the four `randomized_timing_*` fields, `auto_stop_minutes` and `reminder_interval_minutes`) into
the running settings and holds them there. Settings shows a Study card with the participant id
and condition code in place of the Timing, Randomization and Timers cards, the preset chips and
the pacing readout, and the Research menu item disappears from the tray and, on macOS, the app
menu. About and the Start, Stop, Reset and Quit buttons stay as they are. Reset to Defaults
restores appearance and keyboard shortcuts and leaves the lock in force.

The Appearance card stays editable. In either arm a participant can change anything in it,
including the Animation setting (Linear or Sinusoidal), and the log doesn't record those choices.
Changing a color, the opacity, the shape, the gradient, the hold ripple or the Animation setting
while the animation runs restarts it at the start of an inhale and resets `cycle` to 0, and the
log has no line for the restart. With the Fullscreen shape and identical inhale and exhale colors,
the app draws a still tint and doesn't advance phases, so the log gets no `phase_end` lines while
that combination is set.

An invalid file is reported in a Study card at the top of Settings and in the app's own log. The
app starts without the lock and uses whatever `settings.toml` already holds. Check Settings after
installing a file to confirm the lock took effect.

The locked values are also written to `settings.toml` by the app's normal save path, so a machine
that later loses its `study.toml` keeps running at the study pace until someone changes it by
hand. Deleting the file turns the lock off at the next launch, without reverting the pace itself.

### Control arm

A control arm in this file is the same overlay at a different, normal pace, set through the same
four durations, for example `2.5 / 0 / 2.5 / 0` (a five-second cycle, 12 a minute). For a given
Animation setting, the overlay's progress through an inhale or exhale depends only on the fraction
of the phase that has elapsed, so a 2.5 s phase plays the same motion as a 5 s phase at twice the
speed. A test in `exhale-core`, `phase_progress_is_independent_of_phase_duration`, checks this at
2.5 s and 5 s for both settings of Animation, Linear and Sinusoidal.

With both arms on the same appearance settings, pace is the only thing that differs between a study
arm and a control arm built this way. Appearance isn't locked (see
[What locking changes](#what-locking-changes)), so that holds only while participants leave the
Appearance card alone.

## Session log

A study build keeps `exhale-log.jsonl` beside `settings.toml`, appending to it for as long as
`study.toml` is in force. It's JSON Lines: one JSON object per line. Every
line carries `t`, the wall-clock time as milliseconds since the Unix epoch, and `mono_ms`,
milliseconds since the log was opened at app start.

### Events

Every line also carries `event`. Extra fields per event:

| event | fields |
|---|---|
| `app_start` | `version`, `os`, `participant_id`, `condition`, `inhale_duration`, `post_inhale_hold_duration`, `exhale_duration`, `post_exhale_hold_duration`, `drift`, `randomized_timing_inhale`, `randomized_timing_post_inhale_hold`, `randomized_timing_exhale`, `randomized_timing_post_exhale_hold`, `auto_stop_minutes`, `reminder_interval_minutes` |
| `log_cleared` | the same fields as `app_start` |
| `app_stop` | none |
| `animation_start` | `inhale_duration`, `post_inhale_hold_duration`, `exhale_duration`, `post_exhale_hold_duration`, `drift` |
| `animation_stop` | none |
| `pause` | none |
| `resume` | none |
| `reminder` | none |
| `auto_stop` | none |
| `phase_end` | `phase` (one of `inhale`, `hold_after_inhale`, `exhale`, `hold_after_exhale`), `cycle` (integer, 0-based cycle count at the time), `scheduled_ms` (number), `actual_ms` (number) |
| `export` | `path` |

`os` is one of `macos`, `windows`, `linux`. `version` is exhale's own version string, the same
one the Settings footer and macOS About panel show. A phase set to 0 in the study file is
passed over and writes no `phase_end` line, so a `5 / 0 / 5 / 0` cycle logs two of them,
`inhale` and `exhale`, and a `4 / 4 / 4 / 4` cycle logs four. The current interface has no Pause
control, so a session writes no `pause` or `resume` lines. One example line per event, in the same
order as the table above:

```
{"t":1758400000123,"mono_ms":3,"event":"app_start","version":"2.0.28","os":"macos","participant_id":"P07","condition":"B","inhale_duration":5,"post_inhale_hold_duration":0,"exhale_duration":5,"post_exhale_hold_duration":0,"drift":1,"randomized_timing_inhale":0,"randomized_timing_post_inhale_hold":0,"randomized_timing_exhale":0,"randomized_timing_post_exhale_hold":0,"auto_stop_minutes":5,"reminder_interval_minutes":0}
{"t":1758400185340,"mono_ms":185340,"event":"log_cleared","version":"2.0.28","os":"macos","participant_id":"P07","condition":"B","inhale_duration":5,"post_inhale_hold_duration":0,"exhale_duration":5,"post_exhale_hold_duration":0,"drift":1,"randomized_timing_inhale":0,"randomized_timing_post_inhale_hold":0,"randomized_timing_exhale":0,"randomized_timing_post_exhale_hold":0,"auto_stop_minutes":5,"reminder_interval_minutes":0}
{"t":1758400430000,"mono_ms":429880,"event":"app_stop"}
{"t":1758400000130,"mono_ms":10,"event":"animation_start","inhale_duration":5,"post_inhale_hold_duration":0,"exhale_duration":5,"post_exhale_hold_duration":0,"drift":1}
{"t":1758400300141,"mono_ms":300021,"event":"animation_stop"}
{"t":1758400090000,"mono_ms":89880,"event":"pause"}
{"t":1758400095000,"mono_ms":94880,"event":"resume"}
{"t":1758400150000,"mono_ms":149880,"event":"reminder"}
{"t":1758400300140,"mono_ms":300020,"event":"auto_stop"}
{"t":1758400005134,"mono_ms":5014,"event":"phase_end","phase":"inhale","cycle":0,"scheduled_ms":5000,"actual_ms":5003.9}
{"t":1758400420000,"mono_ms":419880,"event":"export","path":"~/Desktop/exhale-log-P07-1758400420.jsonl"}
```

Clear empties the log in place and leaves the clock `mono_ms` is measured from alone, so a
`log_cleared` line and everything after it keeps counting from the original app launch.

`actual_ms` always includes the delay between when a phase was due to end and when the
background controller thread's tick loop notices, so it's never less than `scheduled_ms`. That
delay isn't corrected for afterward: each phase is scheduled from a deadline set when the
previous one ended, so a late tick pushes every later phase's start back by the same amount, and
the cumulative drift row further down adds those delays up.

### Export and clear

Pressing Export in Settings copies the live log to a new file on the same machine. It uses the
first of these folders that the operating system defines for the user: Desktop, Documents,
Downloads, then the home directory. The app doesn't send the copy anywhere, so the participant
hands it to the lab. The copy is named `exhale-log-<participant_id>-<unix_secs>.jsonl`, with
anything in the participant id outside `A`-`Z`, `a`-`z`, `0`-`9`, `_` and `-` replaced with `_`.
Exporting also appends one `export` event to the live log after the copy is made, so the exported
file never contains a record of its own export.

The event's `path` is the copy's location relative to the home folder, written with a `~` prefix
and the platform's path separator, as in the example line above. A copy outside the home folder
logs only its file name, so the account's name stays out of the log in both cases.

Clear, which appears only after a successful export in the current run of the app, empties the
live log and records a `log_cleared` line with the same fields `app_start` carries, so a cleared
log still opens with a valid header.

## Pacing validation

To check exhale's timing against its own schedule: write a `study.toml` with
`auto_stop_minutes = 30.0` and the default `5.0 / 0.0 / 5.0 / 0.0` durations (any
`participant_id` and `condition` work, since this run isn't measuring a participant), launch the
build, leave it running until auto-stop fires, open Settings and press Export, then run:

```sh
uv run --no-project scripts/pacing-validation.py <exported file>
```

The script prints a Markdown table: platform and version, the configured cycle length, how many
complete cycles it measured, the mean cycle length and its error against the configured one, the
single worst cycle and its error, cumulative drift across the whole run, and the mean overshoot
of each phase in milliseconds.

No 30-minute run has been recorded yet. Its table will belong in `docs/pacing-validation.md`, a
filename that's a placeholder until that file exists.

A one-minute check run of a development build on one macOS machine on 2026-10-18, at
`2 / 0 / 2 / 0`, measured 14 complete cycles: a mean cycle of 4.00 s (+0.1%), a worst cycle of
4.01 s (+0.2%), cumulative drift of +0.03 s and per-phase overshoot between 0.1 and 4.9 ms. One
short run on a development machine says little about a participant's computer, so run the
validation on the exact build the participants get, and name that build's measured pace in the
pre-registration.

## Obtaining the build

A study build is tagged `vX.Y.Z-studyN`, for example `v2.0.29-study1`, and every participant in a
study runs the identical binary from that one tag. No study build has been tagged yet.

Pushing a `v*` tag runs the release workflow, which attaches a `.deb`, an `.AppImage`, a `.snap`
and an `.msix` to a draft release on the [Releases
page](https://github.com/peterklingelhofer/exhale/releases). It attaches no macOS build and no
standalone Windows build, so a study that needs either one builds it from the tag's source:

- Windows: `cargo build --release -p exhale-app`, run in `rust/` on a Windows machine, writes a
  self-contained `target/release/exhale.exe`. The file isn't code-signed, so Windows shows a
  "publisher unknown" SmartScreen warning.
- macOS: the repo has no DMG script, and [DEPLOYMENT.md](../DEPLOYMENT.md) lists a standalone
  macOS download as not shipped. The same `cargo build --release -p exhale-app`, run on a Mac,
  writes `target/release/exhale`. That binary isn't an app bundle, so it drops reminder
  notifications, and the log still records each `reminder` line.

Build each file once and give every participant that one file.

To confirm which build is installed, read the version string back: it's in the footer of the
Settings window on every platform, and in the About panel on macOS. The string is the crate
version at the tagged commit, so it carries no `-studyN` suffix.

## Citing

Citation metadata lives in [`CITATION.cff`](../CITATION.cff) at the repo root, in the Citation
File Format, which GitHub reads to offer a "Cite this repository" button. It carries no DOI yet:
one comes from Zenodo when a tagged release is archived there, and this file will carry it once it
exists. The release script updates the `version` and `date-released` fields in `CITATION.cff` along
with the app's own version, so cite the exact version you ran and the file will already name it.

## Contact

Questions, problems or bug reports go to [GitHub
Issues](https://github.com/peterklingelhofer/exhale/issues).
