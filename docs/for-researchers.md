# exhale for researchers

Reference for a lab deciding whether exhale can be the stimulus in a small study: what the app
is, how study mode locks a participant's pacing and appearance, what the session log records, and
how to cite the software.

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

The appearance settings and their defaults are in [Appearance and display
fields](#appearance-and-display-fields).

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

A `study.toml` file locks a participant's pacing and appearance in the app. Settings shows no rate
and no pacing controls, which is what makes the app usable for a blinded arm. The lock covers the
app's interface only. The [threat model](#threat-model) lists the places where the assigned pace
stays readable.

### The file

`study.toml` sits beside `settings.toml`, in the same per-platform configuration folder (see the
README's [Settings](../README.md#settings) table):

| Platform | Path |
|---|---|
| macOS | `~/Library/Application Support/com.peterklingelhofer.exhale/study.toml` |
| Windows | `%APPDATA%\peterklingelhofer\exhale\config\study.toml` |
| Linux | `~/.config/exhale/study.toml`, or `$XDG_CONFIG_HOME/exhale/study.toml` when `XDG_CONFIG_HOME` is set |

These paths belong to builds that run outside a package sandbox: a binary built from source, the
`.deb` and the `.AppImage`. The Mac App Store build is sandboxed and keeps its files in the
container path the README's Settings table lists, and this guide gives no path for the Snap Store
or MSIX packages. The session log lives in the same folder as `study.toml`.

The app reads `study.toml` once, at launch. A file installed, edited or removed while the app runs
takes effect at the next launch, so quit the app first. A second launch while the app runs exits
without reading the file, and on macOS it brings the running app forward.

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
inhale_color = [1.0, 0.0, 0.0, 1.0]        # red, green, blue, alpha, each 0.0 to 1.0, alpha above 0.0
exhale_color = [0.0, 0.0, 1.0, 1.0]        # red, green, blue, alpha, each 0.0 to 1.0, alpha above 0.0
background_color = [0.0, 0.0, 0.0, 0.0]    # red, green, blue, alpha, each 0.0 to 1.0
overlay_opacity = 0.25                     # above 0.0, up to 1.0
shape = "rectangle"                        # fullscreen, rectangle or circle
color_fill_gradient = "on"                 # off, inner or on
animation_mode = "sinusoidal"              # linear or sinusoidal
hold_ripple_mode = "gradient"              # off, stark or gradient
display_mode = "overlay"                   # overlay or window
```

Only six fields are required. Of the others, `drift` defaults to `1.0`, each of the nine
appearance and display fields defaults to the shipped value in the next section's table, and the
rest default to `0.0`:

```toml
participant_id = "P07"
condition = "B"
inhale_duration = 5.0
post_inhale_hold_duration = 0.0
exhale_duration = 5.0
post_exhale_hold_duration = 0.0
```

### Appearance and display fields

The file can set nine appearance and display fields. They have the names and value types of the
same settings in `settings.toml`, so a line copies across as written. Study mode locks all nine.
A field the file leaves out is locked at the shipped default in the table below, whatever the
participant's own `settings.toml` holds, so two participants whose files omit a field see the
same thing.

| Field | Values | Default | What it sets |
|---|---|---|---|
| `inhale_color` | four numbers, `[red, green, blue, alpha]`, each 0.0 to 1.0, alpha above 0.0 | `[1.0, 0.0, 0.0, 1.0]` (red) | Color of the inhale phase |
| `exhale_color` | four numbers, `[red, green, blue, alpha]`, each 0.0 to 1.0, alpha above 0.0 | `[0.0, 0.0, 1.0, 1.0]` (blue) | Color of the exhale phase |
| `background_color` | four numbers, `[red, green, blue, alpha]`, each 0.0 to 1.0 | `[0.0, 0.0, 0.0, 0.0]` (clear) | Background tint. Its alpha sets how opaque the tint is, and a window draws it solid whatever the alpha. No effect when `shape` is `"fullscreen"` |
| `overlay_opacity` | above 0.0, up to 1.0 | `0.25` | Opacity of the overlay. A window draws at full strength, so the field has no effect when `display_mode` is `"window"`, and a value of 0 is rejected there as well |
| `shape` | `"fullscreen"`, `"rectangle"`, `"circle"` | `"rectangle"` | Shape of the animation |
| `color_fill_gradient` | `"off"`, `"inner"`, `"on"` | `"on"` | Gradient style of the fill, where `"off"` is a solid color. No effect when `shape` is `"fullscreen"` |
| `animation_mode` | `"linear"`, `"sinusoidal"` | `"sinusoidal"` | Easing of each inhale and exhale. `"linear"` moves at a constant speed, and `"sinusoidal"` eases in and out |
| `hold_ripple_mode` | `"off"`, `"stark"`, `"gradient"` | `"gradient"` | Ripple around the screen perimeter during holds: none, a hard-edged band (`"stark"`) or a soft glow (`"gradient"`) |
| `display_mode` | `"overlay"`, `"window"` | `"overlay"` | `"overlay"` is one translucent, click-through window per monitor. `"window"` is one regular window, full screen on the primary display |

Integers work in a color, as in `[1, 0, 0, 1]`. Show In (`app_visibility`) isn't one of the nine,
and it stays the participant's.

With `display_mode = "window"` the app opens the animation window full screen and borderless on
the primary display at launch. Where the system names no primary display, the window fills the
display it opens on. What Stop and Start do to the window depends on the platform:

- macOS: the full screen stays in the current Space and auto-hides the menu bar and the Dock. Stop
  takes the window out of full screen and hides it, so the menu bar comes back, and Start shows
  the window and returns it to full screen. The window refuses native full screen, so fn-F and
  Ctrl-Cmd-F can't put it in a Space of its own.
- Windows: Stop hides the window, and Start shows it again, still full screen.
- Linux with X11: Stop takes the window out of full screen and hides it, and Start shows it and
  returns it to full screen.
- Linux with Wayland: the app has no way to hide a window there. Stop takes the window out of full
  screen and leaves it in place as a normal window showing the stopped frame, and Start returns it
  to full screen.

A Wayland session, or a machine whose graphics driver can't draw a see-through overlay, shows a
window whatever `display_mode` says, and only `"window"` makes that window full screen.

To choose a look, set it in the app on a machine that has no `study.toml`, then copy the nine
lines from `settings.toml`. In the current format they're the lines above `app_visibility`. Each
color takes six lines, and the numbers carry the digits of the stored value:

```toml
inhale_color = [
    0.20000000298023224,
    0.4000000059604645,
    0.6000000238418579,
    1.0,
]
exhale_color = [
    0.8999999761581421,
    0.800000011920929,
    0.10000000149011612,
    1.0,
]
background_color = [
    0.10000000149011612,
    0.10000000149011612,
    0.10000000149011612,
    0.5,
]
overlay_opacity = 0.6000000238418579
display_mode = "window"
shape = "circle"
color_fill_gradient = "off"
animation_mode = "linear"
hold_ripple_mode = "stark"
```

Paste the lines into `study.toml` as they stand. The file accepts the multi-line arrays and the
long numbers. Copy no other line from `settings.toml`. It also holds `app_visibility`,
`is_animating`, the window positions and the keyboard shortcuts, and `study.toml` rejects any
field it doesn't recognize.

### Validation rules

The app checks these rules at launch and reports the first one that fails, so a file with two
mistakes needs two fixes:

- `participant_id`: not empty after trimming whitespace, and at most 64 characters after trimming
- `condition`: not empty after trimming whitespace
- The four durations: each 0 or from 0.1 to 86400 seconds, and the four can't all be 0. 0.1 s is
  the shortest phase the controller runs and 86400 s (one day) is the longest it schedules. A
  value between 0 and 0.1 fails, because the controller would run it at 0.1 s while the log
  stated the shorter length
- `drift`: finite and 1.0 or more
- Each `randomized_timing_*` field: finite and from 0.0 to 1.0
- `auto_stop_minutes`: 0 (off) or from 0.1 to 10080 (one week)
- `reminder_interval_minutes`: 0 (off) or from 1 to 10080
- Each color: exactly four numbers, each from 0.0 to 1.0
- `inhale_color` and `exhale_color`: an alpha above 0.0, the fourth number. A phase color with an
  alpha of 0 draws nothing, in an overlay and in a window alike. `background_color` can keep its
  default alpha of 0, which is a clear tint
- `overlay_opacity`: above 0.0 and up to 1.0. An opacity of 0 draws nothing, so the app rejects it
  even with `display_mode = "window"`, where a window draws at full strength and the field has no
  effect. A file that works in one display mode then works in the other
- `shape = "fullscreen"` needs two different colors. The app rejects the file, on `exhale_color`,
  when every component of the inhale and exhale colors differs by less than 0.001. It compares
  the colors the participant would see, so a color the file leaves out counts at its shipped
  default. Two matching colors make the fullscreen shape draw a still tint, and no phase advances
- Any field name the app doesn't recognize fails the whole file, so a typo always produces an
  error. That includes fields that belong to `settings.toml`, such as `app_visibility`
- The file has to be UTF-8 text. A byte order mark and Windows line endings are fine

Every error the app shows starts with `study.toml`, then names the line and the field when it can
find them, and never quotes the value it rejected. [Fail-closed
behavior](#fail-closed-behavior) lists the messages.

### What locking changes

With a valid file loaded, the app writes twenty fields into the running settings and holds them
there: the eleven pacing fields (the four durations, `drift`, the four `randomized_timing_*`
fields, `auto_stop_minutes` and `reminder_interval_minutes`) and the nine appearance and display
fields. The interface then changes in these ways:

- A Study card replaces the Timing, Randomization and Timers cards, the preset chips and the
  pacing readout. It shows the participant id and the Export and Clear buttons (see [Export and
  clear](#export-and-clear)). The condition code stays in the file and the log and doesn't appear
  on screen.
- The Appearance card shows Show In and nothing else. Show In (Top Bar, Dock or Both) and the
  keyboard shortcuts stay the participant's.
- The Research item disappears from the tray and, on macOS, the app menu.
- About and the Start, Stop, Reset and Quit buttons stay as they are.
- Reset restores Show In and the keyboard shortcuts to their defaults, and it forgets the saved
  position of the animation window. The twenty locked fields keep the file's values. The button's
  tooltip reads "Reset the keyboard shortcuts and Show In to their defaults." and, on a second
  line, "The study locks everything else." Its confirmation asks "Reset the keyboard shortcuts and
  Show In?"

After you install a file and launch the app, open Settings. A Study card with the participant id
means the lock took effect. A card that starts "Study can't start" means it didn't (see
[Fail-closed behavior](#fail-closed-behavior)).

The app saves the locked values to `settings.toml` through its normal save path, so a machine that
later loses its `study.toml` keeps running at the study pace and look until someone changes them by
hand. Deleting the file turns the lock off at the next launch, without reverting either.

### Restarts, auto-stop and reminders

Start, Stop and Reset keep working in study mode, so a participant can restart the breath.

- Start begins a new run. The breath restarts at the start of an inhale, `cycle` goes back to 0,
  the accumulated drift returns to 1.0, and the auto-stop countdown starts over. Every start
  writes an `animation_start` line. A Start hotkey press while the animation runs restarts it the
  same way and writes another `animation_start` line.
- Stop halts the animation and writes `animation_stop`. A Stop while the animation is already
  stopped writes nothing.
- A phase that a Stop or a restart cuts short writes no `phase_end` line.
- Auto-stop counts from the latest Start, and the launch counts as the first one. When the
  countdown ends the app writes `auto_stop`, then `animation_stop`. A participant can press Start
  again, which begins a new run with a new countdown.
- Reset writes a `reset` line. It leaves the pace and the auto-stop countdown alone, because
  neither timer field changes.
- Reminders fire at their interval, counted from launch. Start, Stop and Reset don't restart that
  count, and an auto-stop doesn't end it, so reminders keep firing after the animation stops. Each
  one writes a `reminder` line. A study that can't start (see [Fail-closed
  behavior](#fail-closed-behavior)) sends no reminders.

### Fail-closed behavior

A study file that exists and can't be used never falls back to the participant's own settings,
because that would run a session at a pace the lab didn't assign. The app fails closed when:

- the file doesn't parse, or it breaks a [validation rule](#validation-rules)
- the app can't read the file: a folder stands in its place, the account lacks permission, or the
  text isn't UTF-8
- the file is a link to a file that doesn't exist
- another entry in the folder has a name that starts with `study`, in any case, and isn't exactly
  `study.toml`. Examples are `study.toml.txt` (an editor that hides extensions writes it),
  `Study.toml` and `study (1).toml`. The check runs on every OS, and it fails the study even when
  a correct `study.toml` sits beside the stray entry
- the app can't open the participant's session log, for example because the folder isn't
  writable. The app finds this out after it has read the file

In that state the app:

- shows a Study card at the top of Settings. It reads "Study can't start: " followed by a
  summary, then "Tell whoever runs the study."
- disables the Start button in the panel (its tooltip reads "The study can't start. Tell whoever
  runs the study.") and in the tray menu, and the Start hotkey does nothing
- hides the Timing, Randomization and Timers cards, the presets, the rate readout, every
  Appearance row but Show In, and the Research item, as a locked study does
- holds the animation still from launch on
- sends no reminders
- writes no session log, so the lab learns of the failure from the participant
- records the full detail, configured values included, in the debug log only (see [The debug
  log](#the-debug-log))

The summary names the file, and the line and the field when the app can find them. It never
quotes a configured value or a piece of the file, so the card doesn't reveal the pace. A few
examples, where the line numbers depend on the file:

| Cause | The card reads |
|---|---|
| A value of the wrong type | `study.toml, line 3, field inhale_duration: has to be a number` |
| A required field is missing | `study.toml, field exhale_duration: is required but missing` |
| A misspelled or unknown field | `study.toml, line 7, field exhale_duraton: isn't a field exhale knows` |
| A value outside its range | `study.toml, line 8, field drift: has to be 1.0 or more` |
| An overlay opacity of 0 | `study.toml, line 8, field overlay_opacity: has to be above 0 and at most 1` |
| A phase color with an alpha of 0 | `study.toml, line 6, field inhale_color: has to have an alpha above 0` |
| A name that isn't one of the allowed values | `study.toml, line 7, field shape: has to be one of fullscreen, rectangle, circle` |
| All four durations are 0 | `study.toml: the four durations can't all be 0` |
| Fullscreen with matching colors | `study.toml, line 9, field exhale_color: has to differ from inhale_color when the shape is fullscreen` |
| A misnamed file | `study.toml: found "study.toml.txt" in the settings folder. Rename it to study.toml` |
| A folder or an unreadable file in its place | `study.toml: couldn't be read: ` and then the system's reason, such as `Is a directory (os error 21)` |
| Text that isn't UTF-8 | `study.toml: isn't UTF-8 text. Save it as UTF-8` |
| A link to a missing file | `study.toml: is a link to a file that doesn't exist` |
| The session log can't be opened | `study.toml: the session log can't be written` |

Fix the file, then quit the app and launch it again. The app doesn't re-read the file while it
runs.

### Threat model

The blinding covers the app's interface: Settings shows no rate, no durations and no presets, and
the Research item is gone. Outside the interface the pace stays readable in these places:

- The overlay shows it to anyone who watches and times a few cycles.
- `study.toml` holds the pace, the condition code and the look as plain text. Anyone with access
  to the folder can read, edit or delete the file, and deleting it lifts the lock at the next
  launch.
- `settings.toml` holds the pace and the look too, because the app saves the locked values like
  any other setting.
- The exported log lists every locked value in its `app_start` header, and each `phase_end` line
  carries the scheduled length of its phase.
- The debug log `exhale.log` holds the full detail of a `study.toml` error, configured values
  included.

exhale doesn't sign, encrypt or checksum the log or the study file. Study mode blinds a
cooperating participant, and a participant who looks for the assignment can find it.

### Control arm

A control arm in this file is the same overlay at a different, normal pace, set through the same
four durations, for example `2.5 / 0 / 2.5 / 0` (a five-second cycle, 12 a minute). For a given
Animation setting, the overlay's progress through an inhale or exhale depends only on the fraction
of the phase that has elapsed, so a 2.5 s phase plays the same motion as a 5 s phase at twice the
speed. A test in `exhale-core`, `phase_progress_is_independent_of_phase_duration`, checks this at
2.5 s and 5 s for both settings of Animation, Linear and Sinusoidal.

The file locks the appearance too, so both arms can share one look: give both files the same
appearance fields, or leave the fields out of both, which locks both at the shipped defaults. With
the same look in both arms, pace is the only thing that differs between a study arm and a control
arm built this way, whatever a participant does in Settings.

## Session log

A study build writes one live log per participant id, named `exhale-log-<id>.jsonl`, in the
folder that holds `settings.toml` and `study.toml`. It appends to that file for as long as
`study.toml` is in force. It's JSON Lines: one JSON object per line. Every line carries `t`, the
wall-clock time as milliseconds since the Unix epoch, and `mono_ms`, milliseconds since the log
was opened at app start.

`<id>` is the participant id with surrounding whitespace removed, each character outside `A`-`Z`,
`a`-`z`, `0`-`9`, `_` and `-` replaced with `_`, and the result cut to 64 characters. The id
`P07` gives `exhale-log-P07.jsonl`. A new participant id starts a new file, so a machine handed to
the next participant never mixes two people's lines. Ids that clean to the same text share one
file: `P/07` and `P_07`, and `P07` and `p07` on a disk that ignores case, as the default macOS and
Windows file systems do. The `participant_id` in each `app_start` line still tells the sessions
apart, but a lab that wants one file per participant should assign ids that differ by more than
case or punctuation.

After `study.toml` is removed the app stops writing to the log, and the file stays on disk until
someone deletes it.

### Events

Every line also carries `event`. Extra fields per event:

| event | fields |
|---|---|
| `app_start` | the 24 header fields listed below |
| `log_cleared` | the same fields as `app_start` |
| `app_stop` | none |
| `animation_start` | `inhale_duration`, `post_inhale_hold_duration`, `exhale_duration`, `post_exhale_hold_duration`, `drift` |
| `animation_stop` | none |
| `pause` | none |
| `resume` | none |
| `reset` | none |
| `reminder` | none |
| `auto_stop` | none |
| `phase_end` | `phase` (one of `inhale`, `hold_after_inhale`, `exhale`, `hold_after_exhale`), `cycle` (integer, 0-based cycle count at the time), `scheduled_ms` (number), `actual_ms` (number) |
| `export` | `path` (the copy's file name) |

The header fields, in the order they appear after `t`, `mono_ms` and `event`:

| Field | Value |
|---|---|
| `version` | exhale's own version string, the same one the Settings footer and macOS About panel show |
| `os` | `macos`, `windows` or `linux` |
| `participant_id` | the id from the study file, as written |
| `condition` | the condition code from the study file |
| `inhale_duration` | seconds |
| `post_inhale_hold_duration` | seconds |
| `exhale_duration` | seconds |
| `post_exhale_hold_duration` | seconds |
| `drift` | per-cycle multiplier |
| `randomized_timing_inhale` | fraction, 0 to 1 |
| `randomized_timing_post_inhale_hold` | fraction, 0 to 1 |
| `randomized_timing_exhale` | fraction, 0 to 1 |
| `randomized_timing_post_exhale_hold` | fraction, 0 to 1 |
| `auto_stop_minutes` | minutes, 0 is off |
| `reminder_interval_minutes` | minutes, 0 is off |
| `inhale_color` | array of four numbers: red, green, blue, alpha |
| `exhale_color` | array of four numbers |
| `background_color` | array of four numbers |
| `overlay_opacity` | number from 0 to 1 |
| `shape` | `fullscreen`, `rectangle` or `circle` |
| `color_fill_gradient` | `off`, `inner` or `on` |
| `animation_mode` | `linear` or `sinusoidal` |
| `hold_ripple_mode` | `off`, `stark` or `gradient` |
| `display_mode` | `overlay` or `window` |

The header holds the values the app applied: the file's value, or the shipped default for a field
the file leaves out. Numbers print in the shortest form that reads back as the same value, so an
`overlay_opacity` of 0.6 prints as `0.6` where `settings.toml` writes `0.6000000238418579`.

A launch writes `app_start`, then `animation_start`, because the app always starts with the
animation running. A phase set to 0 in the study file is passed over and writes no `phase_end`
line, so a `5 / 0 / 5 / 0` cycle logs two of them, `inhale` and `exhale`, and a `4 / 4 / 4 / 4`
cycle logs four. The current interface has no Pause control, so a session writes no `pause` or
`resume` lines. A `reset` line marks every Reset the app acts on: the panel's button and the Reset
hotkey after their confirmation, and the tray item, which acts at once. One example line per
event, in the same order as the table above:

```
{"t":1758400000123,"mono_ms":3,"event":"app_start","version":"2.0.28","os":"macos","participant_id":"P07","condition":"B","inhale_duration":5,"post_inhale_hold_duration":0,"exhale_duration":5,"post_exhale_hold_duration":0,"drift":1,"randomized_timing_inhale":0,"randomized_timing_post_inhale_hold":0,"randomized_timing_exhale":0,"randomized_timing_post_exhale_hold":0,"auto_stop_minutes":5,"reminder_interval_minutes":0,"inhale_color":[1,0,0,1],"exhale_color":[0,0,1,1],"background_color":[0,0,0,0],"overlay_opacity":0.25,"shape":"rectangle","color_fill_gradient":"on","animation_mode":"sinusoidal","hold_ripple_mode":"gradient","display_mode":"overlay"}
{"t":1758400185340,"mono_ms":185340,"event":"log_cleared","version":"2.0.28","os":"macos","participant_id":"P07","condition":"B","inhale_duration":5,"post_inhale_hold_duration":0,"exhale_duration":5,"post_exhale_hold_duration":0,"drift":1,"randomized_timing_inhale":0,"randomized_timing_post_inhale_hold":0,"randomized_timing_exhale":0,"randomized_timing_post_exhale_hold":0,"auto_stop_minutes":5,"reminder_interval_minutes":0,"inhale_color":[1,0,0,1],"exhale_color":[0,0,1,1],"background_color":[0,0,0,0],"overlay_opacity":0.25,"shape":"rectangle","color_fill_gradient":"on","animation_mode":"sinusoidal","hold_ripple_mode":"gradient","display_mode":"overlay"}
{"t":1758400430000,"mono_ms":429880,"event":"app_stop"}
{"t":1758400000130,"mono_ms":10,"event":"animation_start","inhale_duration":5,"post_inhale_hold_duration":0,"exhale_duration":5,"post_exhale_hold_duration":0,"drift":1}
{"t":1758400300141,"mono_ms":300021,"event":"animation_stop"}
{"t":1758400090000,"mono_ms":89880,"event":"pause"}
{"t":1758400095000,"mono_ms":94880,"event":"resume"}
{"t":1758400250000,"mono_ms":249880,"event":"reset"}
{"t":1758400150000,"mono_ms":149880,"event":"reminder"}
{"t":1758400300140,"mono_ms":300020,"event":"auto_stop"}
{"t":1758400005134,"mono_ms":5014,"event":"phase_end","phase":"inhale","cycle":0,"scheduled_ms":5000,"actual_ms":5003.9}
{"t":1758400420000,"mono_ms":419880,"event":"export","path":"exhale-log-P07-1758400420.jsonl"}
```

`actual_ms` always includes the delay between when a phase was due to end and when the
background controller thread's tick loop notices, so it's never less than `scheduled_ms`. That
delay isn't corrected for afterward: each phase is scheduled from a deadline set when the
previous one ended, so a late tick pushes every later phase's start back by the same amount, and
the cumulative drift row further down adds those delays up.

### What the log can miss

- `app_stop` is best effort. The app writes it on every normal quit: the Quit button, the tray's
  Quit, the Linux window's close box, and on macOS Cmd-Q, the app menu's Quit and the Dock's Quit.
  It's the last line the log gets, so no `phase_end` follows it. It's missing after a crash, a
  force quit, a logoff or shutdown on Windows, and SIGTERM on macOS and Linux. An `app_start` with
  no `app_stop` before the next `app_start` marks a session that ended one of those ways.
- A process killed while it writes can leave the last line cut short. The next launch starts a
  new line, and `pacing-validation.py` skips the cut line with a warning.
- Computer sleep changes the clocks. `t` is the wall clock and runs through sleep on every
  platform. On macOS and Linux `mono_ms`, `actual_ms` and the auto-stop countdown pause while the
  computer sleeps, so a sleep shows as a gap in `t` that `mono_ms` doesn't share. On Windows they
  count the sleep: a phase that was running when the computer slept logs an `actual_ms` that
  includes it, and an auto-stop that came due during the sleep fires at wake.

### Export and clear

Press Export log in the Study card to copy the live log to a new file on the same machine. The
app tries the folders the operating system names for the participant, in this order, and uses the
first one that exists and takes the copy: Desktop, Documents, Downloads, then the home folder. The
app doesn't send the copy anywhere, so the participant hands it to the lab. The card then shows
"Exported to" and the copy's full path. The copy is named `exhale-log-<id>-<unix_secs>.jsonl`,
with `<id>` cleaned as in the live log's name and `<unix_secs>` the time of the export. A name
that already exists gets `-2`, `-3` and so on, so an export never replaces a file. Exporting also
appends one `export` event to the live log after the copy is made, so the exported file never
contains a record of its own export. The event's `path` is the copy's file name with no folder,
which keeps the account's name out of the log.

The operating system can sync the Desktop to a cloud account, such as OneDrive on Windows or
iCloud Drive on macOS. A synced Desktop uploads the copy, which holds the participant id, the
condition code and the pace. exhale doesn't control that, so check the participant's machine
before the study if the log has to stay local.

The Mac App Store build's sandbox blocks Export to the Desktop, so a macOS study uses a build from
source (see [Obtaining the build](#obtaining-the-build)). The Snap exports normally.

Clear log empties the live log in place and records a `log_cleared` line with the same fields
`app_start` carries, so a cleared log still opens with a valid header. The button appears only
while the log holds nothing that an export didn't copy, the export's own line included. When a
line has arrived since the last export, the app hides the button and the card says "New lines
arrived since the export. Stop, then export again to clear the log." A running animation writes a
line at the end of every phase, so the order is Stop, Export, Clear. A reminder or an auto-stop
that lands between Export and Clear hides the button too, and another export brings it back. A
Clear leaves its header as the log's only line, and no export holds that line yet, so the next
Clear needs a new export first.

Clear leaves the clock `mono_ms` is measured from alone, so a `log_cleared` line and everything
after it keeps counting from the original app launch.

### The debug log

Every build, study or not, writes a diagnostic log named `exhale.log`. It lives beside the program
and falls back to the system's temporary folder when the program's folder isn't writable. The app
empties it at each launch, so it covers the current run only, and Export never copies it. It
holds startup steps, window and monitor changes, hotkey registration, warnings and errors. It
holds no participant id and no folder path. It does hold the full detail of a `study.toml` error,
and that can quote values the file set, a pace included. When a participant reports a "Study
can't start" card, ask for `exhale.log` before they launch the app again.

## Pacing validation

To check exhale's timing against its own schedule: write a `study.toml` with
`auto_stop_minutes = 30.0` and the default `5.0 / 0.0 / 5.0 / 0.0` durations. Any `condition`
works, since this run isn't measuring a participant, but use a `participant_id` that no
participant has, because each id gets a log file of its own. Keep the computer awake, since sleep
changes the clocks (see [What the log can miss](#what-the-log-can-miss)). Launch the build, leave
it running until auto-stop fires, open Settings, press Export log, then run:

```sh
uv run --no-project scripts/pacing-validation.py <exported file>
```

The script scores the log's last session, which is everything after the final `app_start` or
`log_cleared` line. It splits that session into runs. A run ends at the next `animation_start`
line, and wherever the cycle number and the phase order stop increasing, which marks a restart
that wrote no line. A restart can write the last `phase_end` line of the run it cuts short just
after the new `animation_start`. Until the first line with `cycle` 0, a line with a `cycle` above 0
stays with the run before, so it doesn't count as a run of its own. The script prints one Markdown
table per run. A log with one run prints one table. A log with more prints a table for each run
under a "Run N of M" heading. Each table lists platform and version, the configured cycle length,
how many complete cycles it measured, the mean cycle length and its error, the single worst cycle
and its error, cumulative drift across the run, and the mean overshoot of each phase in
milliseconds.

The script compares each cycle with its scheduled length, the sum of the `scheduled_ms` of that
cycle's own lines. Drift and randomized timing change what the app schedules, so neither counts as
timing error. When the header sets either one, the configured-cycle row says so.

The script skips a line that isn't valid JSON, such as a line cut short when the app was killed,
and a `phase_end` line that lacks a whole-number `cycle`, a known `phase`, or a number in `mono_ms`,
`scheduled_ms` or `actual_ms`. It prints a warning on stderr with a count of the skipped lines, and
a cycle that lost a line counts as incomplete.

`--self-check` runs the script's own tests on built-in logs and needs no log file. It prints
`pacing-validation self-check passed` when the tests pass. The `citations` job in CI runs it beside
the repo's other script self-checks.

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
  macOS download as not shipped. The Mac App Store build's sandbox blocks Export to the Desktop,
  so a Mac study uses a build from source. `cargo build --release -p exhale-app`, run on a Mac,
  writes `target/release/exhale`. That binary isn't an app bundle, so it drops reminder
  notifications, and the log still records each `reminder` line. A second launch of the bare
  binary can't bring the Settings window back after a participant closes it. Reopen it from the
  Preferences item in the menu-bar menu or with the Preferences hotkey, Ctrl+Shift+, unless the
  participant rebound it.

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
