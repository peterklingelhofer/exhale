# Microsoft Store listing copy

The Partner Center form field values for the exhale listing (Store ID `9P79Z1NJMZB3`). Build and
upload mechanics are in [DEPLOYMENT.md](../../../DEPLOYMENT.md#windows-microsoft-store).

---

## Description

```
A minimal breathing overlay: a friendly indicator and reminder to take full, deep breaths while looking at screens.

Demanding work at a keyboard measurably changes how you breathe. In small studies, people breathed faster and mildly over-breathed during data entry, and forward head posture is associated with reduced lung volumes. Slow paced breathing is the breathing practice with the most published evidence behind it. Whether it counters that over-breathing is untested, and one study found it can add to it. exhale paces slow breathing for you with no sensor, no account and no telemetry.

The overlay is a translucent window that gently expands on inhale and contracts on exhale. It stays on top of other windows and lets clicks pass through, so it never interrupts whatever you're doing. On some Windows 10 graphics drivers it opens as a regular window instead.

Inhale, post-inhale hold, exhale, and post-exhale hold durations are all configurable. The default is 5 seconds in and 5 out with no holds, which is 6 breaths a minute, where most of the direct evidence was gathered. The range tested directly runs from 5 to 7 a minute. Box breathing (4 / 4 / 4 / 4) is supported too, though at 3.75 breaths a minute it falls below that range. Making the exhale longer than the inhale is a matter of preference, since the studies comparing ratios are split.

Every research claim above is sourced, alongside a list of what the research doesn't support, at https://github.com/peterklingelhofer/exhale/blob/main/docs/CITATIONS.md

exhale runs from the system tray, and its Preferences window is fully keyboard-navigable.

Few side effects are expected from slow breathing. The sliders also reach fast, hold-heavy patterns, where brief light-headedness and muscle cramps have been reported, so slow down or stop if you feel either.

Disclaimer: The information and guidance provided by this app are for general informational purposes only and aren't medical advice. The creator isn't a medical professional. Always seek the advice of a qualified healthcare provider with any questions about your health, and don't disregard or delay professional medical advice because of this app. Use is at your own risk.
```

The Store MSIX is built with `--no-default-features`, so global hotkeys are compiled out. Don't
list them here.

## Product features

```
Click-through full-screen overlay that never interrupts your work
Customizable inhale, exhale, and hold durations for any breathing cadence
Inhale, exhale, and background colors with optional gradient or constant fill
Multiple shapes: fullscreen, circle, or rectangle
Sinusoidal or linear animation modes
Adjustable overlay opacity to blend with any desktop
Drift factor to gradually lengthen or shorten breath cycles over time
Lives in the system tray: start, stop, or reset without leaving your workflow
Five breathing patterns as one-click presets, with the current rate shown against the tested range
```

## Short description (150 char max)

```
A translucent breathing overlay that gently expands on inhale and contracts on exhale: a friendly reminder to breathe fully while staring at screens.
```

## Search terms (keywords)

Partner Center takes up to 7.

```
mindfulness, meditation, box breathing, mental health, relaxation, wellness
```

## What's new in this version

Per-release, so it's not pinned here. Take it from the release notes for the tag being
submitted. The v2.0.25 text, kept as a shape reference:

```
No longer needs the Microsoft Visual C++ Redistributable.
New one-click breathing presets, with the current pace shown in breaths per minute.
The default pattern is now 5 seconds in and 5 out, six breaths a minute.
A Research item in the tray menu opens the sources behind these choices.
Holds set to 0 take no time, and timing randomization is now a percentage of each phase.
Fixed: the Linear animation option now takes effect, a settings file missing a value no longer resets everything, and invalid durations no longer freeze the animation.
```

## Restricted capability justification (runFullTrust)

Submission options asks for this on every submission that uploads a package.

```
exhale is a Win32 desktop app, a single Rust binary packaged as MSIX, so it declares runFullTrust to run as a regular desktop process. It needs that to draw its translucent breathing overlay as a layered, click-through, always-on-top window above other apps, and to show its system tray icon and menu. It makes no network connections, collects no data, and only reads and writes its own settings and log files.
```

## Copyright and trademark info

```
© Peter Klingelhofer
```

## Support contact info

- **Support email**: `peterklingelhofer@gmail.com`
- **Support URL**: `https://github.com/peterklingelhofer/exhale/issues`
- **Website**: `https://github.com/peterklingelhofer/exhale`

## Privacy policy URL

```
https://github.com/peterklingelhofer/exhale/blob/main/PRIVACY.md
```

## Category / Subcategory

- Primary category: **Health & fitness** (matches the Mac App Store category)
- Subcategory: **Fitness**

## System requirements

- **Minimum OS**: Windows 10 version 1809 (build 17763), matching `MinVersion` in
  [AppxManifest.xml](AppxManifest.xml)
- **Recommended architecture**: x64
- No special hardware requirements

## Pricing and availability

- **Markets**: All (default for free apps)
- **Price**: Free
- **Visibility**: Public
- **Schedule**: Release as soon as possible after certification

## Age ratings

Every answer for exhale is **No**: violence, sexual content, controlled substances, gambling,
in-app purchases, user-generated content. Result should come back everyone-friendly
(ESRB "Everyone" / PEGI 3 / USK 0). The questionnaire also asks whether the app contains medical or
treatment information. The answer is no, since the binary carries no health claims and no
disclaimer. It shows an overlay, timing controls and a readout of the breathing rate, and its
Research item opens the corpus page.
