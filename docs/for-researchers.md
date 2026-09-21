# exhale for researchers

Reference for a lab deciding whether exhale can be the stimulus in a small study: what the app
is, how study mode locks a participant's pacing, what the session log records, and how to cite
the software.

## What exhale is

exhale is an always-on-top, translucent overlay that expands while you inhale and contracts
while you exhale. Its pace is set by four phase durations: inhale, a hold after inhale, exhale,
and a hold after exhale. Shape, color and opacity are separate appearance settings and don't
affect timing.

exhale needs no sensor and no account, and it has no telemetry: see [`PRIVACY.md`](../PRIVACY.md)
for exactly what it does and doesn't collect. It runs as a native binary on macOS, Windows and
Linux. The [README](../README.md) covers installation, keyboard shortcuts and the
platform-specific overlay behavior this document doesn't repeat.

## Parameters and defaults

| Parameter | Default | What it does |
|---|---|---|
| Inhale (`inhale_duration`) | 5 s | Seconds the overlay takes to expand |
| Post-inhale hold (`post_inhale_hold_duration`) | 0 s | Seconds held expanded before the exhale starts |
| Exhale (`exhale_duration`) | 5 s | Seconds the overlay takes to contract |
| Post-exhale hold (`post_exhale_hold_duration`) | 0 s | Seconds held contracted before the next inhale |
| Drift (`drift`) | 1.0 (off) | Each cycle's length is the previous cycle's length times this number, so anything above 1.0 makes every cycle a little longer than the last |
| Randomization (`randomized_timing_*`, one field per phase) | 0% (off) | Random jitter on that phase, as a fraction of its length in either direction. The Settings slider shows it as a percent from 0 to 100 and stores it as 0.0 to 1.0, so 0.25 on a 4 s phase gives 3 to 5 s, and a phase set to 0 stays 0 |
| Reminder interval (`reminder_interval_minutes`) | 0 min (off) | Minutes between reminder notifications |
| Auto-stop (`auto_stop_minutes`) | 0 min (off) | Minutes of running before the overlay stops itself |

Rate is 60 divided by the cycle length in seconds. The shipped default, `5 / 0 / 5 / 0`, is a
10-second cycle: 6 a minute. `2.5 / 0 / 2.5 / 0` is a 5-second cycle: 12 a minute. A phase set
to 0 takes no time at all, and a phase set to anything above 0 never runs shorter than 0.1 s.

## Evidence and its limits

Every claim exhale's defaults and interface rest on is traced to a source in
[`docs/CITATIONS.md`](CITATIONS.md), and the [reading guide](CITATIONS.md#how-to-read-this) at
its top explains the verification status, access level and evidence tier behind each entry. The
[gaps ledger](CITATIONS.md#gaps-and-unsupported-choices) at the end of that document lists, in
its own words, where exhale ships something the literature doesn't settle. That corpus is what
states which claims the app is and isn't allowed to make.

## Study mode

A `study.toml` file locks a participant's pacing so they can neither see nor change the rate
they've been assigned, which is what makes the app usable for a blinded arm.

### The file

`study.toml` sits beside `settings.toml`, in the same per-platform configuration folder (see the
README's [Settings](../README.md#settings) table):

| Platform | Path |
|---|---|
| macOS | `~/Library/Application Support/com.peterklingelhofer.exhale/study.toml` |
| Windows | `%APPDATA%\peterklingelhofer\exhale\config\study.toml` |
| Linux | `~/.config/exhale/study.toml` |

The Mac App Store's sandboxed container path doesn't apply here: study builds are the
direct-download builds, the DMG and the standalone `.exe`, distributed outside the App Store and
the Microsoft Store.

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

Only six fields are required. Everything else defaults to the values above:

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
- The four durations: each finite and `>= 0`, and their sum must be greater than `0` (the app's
  message for a file that fails this reads "the four durations add up to 0, which is not a pace")
- `drift`: finite and `>= 1.0`
- Each `randomized_timing_*` field: finite and between `0.0` and `1.0`
- `auto_stop_minutes` and `reminder_interval_minutes`: finite and `>= 0`
- Any field name the app doesn't recognize fails the whole file, so a typo is reported rather
  than silently ignored
- Every error is prefixed `study.toml:` and names the field and the value it rejected

### What locking changes

With a valid file loaded, the app writes its eleven numeric fields (the four durations, `drift`,
the four `randomized_timing_*` fields, `auto_stop_minutes` and `reminder_interval_minutes`) into
the running settings and holds them there. Settings shows a Study card with the participant id
and condition code in place of the Timing, Randomization and Timers cards, the preset chips and
the pacing readout, and the Research menu item disappears from the tray and, on macOS, the app
menu. About, Appearance and the Start, Stop, Reset and Quit buttons stay exactly as they are.
Reset to Defaults restores appearance and keyboard shortcuts and leaves the lock in force.

An invalid file is reported in the Study card and in the app's own log, and the app runs
unlocked, using whatever `settings.toml` already holds, rather than refusing to start. Check
Settings after installing a file to confirm it actually took.

The locked values are also written to `settings.toml` by the app's normal save path, so a machine
that later loses its `study.toml` keeps running at the study pace until someone changes it by
hand. Deleting the file turns the lock off at the next launch, without reverting the pace itself.

### Control arm

The only control arm this file can express right now is the same overlay at a different, normal
pace, set through the same four durations, for example `2.5 / 0 / 2.5 / 0` (a five-second cycle,
12 a minute). The overlay's easing and shape don't depend on the durations, so pace is the only
thing that differs between a study arm and a control arm built this way. A test in
`exhale-core`, `phase_progress_is_independent_of_phase_duration`, is what pins that down.

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
`inhale` and `exhale`, and a `4 / 4 / 4 / 4` cycle logs four. One example line per event, in
the same order as the table above:

```
{"t":1758400000123,"mono_ms":3,"event":"app_start","version":"2.0.22","os":"macos","participant_id":"P07","condition":"B","inhale_duration":5,"post_inhale_hold_duration":0,"exhale_duration":5,"post_exhale_hold_duration":0,"drift":1,"randomized_timing_inhale":0,"randomized_timing_post_inhale_hold":0,"randomized_timing_exhale":0,"randomized_timing_post_exhale_hold":0,"auto_stop_minutes":5,"reminder_interval_minutes":0}
{"t":1758400185340,"mono_ms":185340,"event":"log_cleared","version":"2.0.22","os":"macos","participant_id":"P07","condition":"B","inhale_duration":5,"post_inhale_hold_duration":0,"exhale_duration":5,"post_exhale_hold_duration":0,"drift":1,"randomized_timing_inhale":0,"randomized_timing_post_inhale_hold":0,"randomized_timing_exhale":0,"randomized_timing_post_exhale_hold":0,"auto_stop_minutes":5,"reminder_interval_minutes":0}
{"t":1758400430000,"mono_ms":429880,"event":"app_stop"}
{"t":1758400000130,"mono_ms":10,"event":"animation_start","inhale_duration":5,"post_inhale_hold_duration":0,"exhale_duration":5,"post_exhale_hold_duration":0,"drift":1}
{"t":1758400300141,"mono_ms":300021,"event":"animation_stop"}
{"t":1758400090000,"mono_ms":89880,"event":"pause"}
{"t":1758400095000,"mono_ms":94880,"event":"resume"}
{"t":1758400150000,"mono_ms":149880,"event":"reminder"}
{"t":1758400300140,"mono_ms":300020,"event":"auto_stop"}
{"t":1758400005134,"mono_ms":5014,"event":"phase_end","phase":"inhale","cycle":0,"scheduled_ms":5000,"actual_ms":5003.9}
{"t":1758400420000,"mono_ms":419880,"event":"export","path":"/Users/p07/Desktop/exhale-log-P07-1758400420.jsonl"}
```

Clear empties the log in place but doesn't reset the clock `mono_ms` is measured from, so a
`log_cleared` line and everything after it keeps counting from the original app launch rather
than starting over at zero.

`actual_ms` always includes the delay between when a phase was due to end and when the
background controller thread's tick loop notices, so it's never less than `scheduled_ms`. That
delay isn't corrected for afterward: each phase is scheduled from a deadline set when the
previous one ended, so a late tick pushes every later phase's start back by the same amount
instead of being absorbed into it, which is what the cumulative drift row further down adds up.

### Export and clear

Pressing Export in Settings copies the live log to a new file. It looks for a Desktop folder
first, then Documents, then Downloads, then the home directory, and uses whichever one exists.
The copy is named `exhale-log-<participant_id>-<unix_secs>.jsonl`, with anything in the
participant id outside `A`-`Z`, `a`-`z`, `0`-`9`, `_` and `-` replaced with `_`. Exporting also
appends one `export` event, carrying the copy's path, to the live log after the copy is made, so
the exported file never contains a record of its own export.

Clear, which appears only after a successful export, empties the live log and records a
`log_cleared` line with the same fields `app_start` carries, so a cleared log still opens with a
valid header.

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

Short check runs on one macOS machine on 2026-09-20 and 2026-09-21 at `2 / 0 / 2 / 0` put the
per-phase overshoot between 0.1 and 6.1 ms and the mean cycle within 0.1% of the configured
4.00 s. Those are two dozen cycles on a development machine. Run the validation on the exact
build the participants get, and name that build's measured pace in the pre-registration.

## Obtaining the build

Study builds are published on the [Releases
page](https://github.com/peterklingelhofer/exhale/releases) under a tag named `vX.Y.Z-studyN`,
for example `v2.0.23-study1`. They're the direct-download builds, the macOS DMG and the Windows
`.exe`, rather than the Mac App Store or Microsoft Store listings, and every participant in a
study runs the identical binary from that one tag.

To confirm which build is installed, read the version string back: it's in the footer of the
Settings window on every platform, and in the About panel on macOS.

## Citing

Citation metadata lives in [`CITATION.cff`](../CITATION.cff) at the repo root, in the format
GitHub and most reference managers read automatically. It carries no DOI yet: one comes from
Zenodo when a tagged release is archived there, and this file will carry it once it exists. The
release script updates `CITATION.cff`'s `version` field on every tag, so cite the exact version
you ran and the file will already name it.

## Contact

Questions, problems or bug reports go to [GitHub
Issues](https://github.com/peterklingelhofer/exhale/issues).
