# Microsoft Store listing copy

The Partner Center form field values for the exhale listing (Store ID `9P79Z1NJMZB3`).

**Why this is tracked.** These strings are promotional copy, and under MDR/FTC reasoning promotional
copy is what establishes a product's intended purpose. They are also the surface that has already
rotted once: this listing and `snap/snapcraft.yaml` went on asserting two claims for a further two
days after [`docs/citations-notes.md`](../../../docs/citations-notes.md) retracted them, in gaps
ledger items 1 and 4. Living in an untracked scratch file, they were invisible to CI and to review.

(Those phrases are deliberately not quoted here. This file is a surface the denylist scans for
assertions, and it cannot tell a quotation from a claim; only `docs/` and `README.md` are exempt,
because they are the retraction record itself.)

`scripts/generate-citations.py --check` scans this file against the retracted-phrase denylist, so a
withdrawn claim surviving here now fails the build. Keep it that way: if the listing copy moves,
move the entry in `ASSERTING_SURFACES` with it.

Build and upload mechanics are in [DEPLOYMENT.md](../../../DEPLOYMENT.md#windows--microsoft-store),
not here. This file is only the words.

---

## Description

```
A minimal cross-platform breathing overlay: a friendly indicator and reminder to take full, deep breaths while looking at screens. Screen work measurably changes how you breathe: at a keyboard, people breathe faster and higher in the chest, and slumped screen posture reduces how much the diaphragm can do. Slow paced breathing is the countermeasure with the most evidence behind it, and exhale is a way to run one with no sensor, no account and no telemetry.

The overlay is a translucent always-on-top window that gently expands on inhale and contracts on exhale. Inhale, post-inhale hold, exhale, and post-exhale hold durations are all configurable. A good place to start is 5 seconds in and 5 seconds out, which is 6 breaths a minute. Rates from 5 to 7 a minute are the ones that have been tested directly. Box breathing (4 / 4 / 4 / 4) is also supported.

Every claim above is sourced, alongside a ledger of what the research does not support, at https://github.com/peterklingelhofer/exhale/blob/main/docs/CITATIONS.md

Every action (Start, Stop, Reset, Quit, Preferences) is rebindable to a global keyboard shortcut. Fully keyboard-navigable Preferences panel. Runs as a menu-bar / system-tray app; the overlay itself is click-through so it never interrupts whatever you're doing.

Take breaks if intense feelings arise; it's important not to overdo it.

---
Disclaimer: The information and guidance provided by this app are intended for general informational purposes only and are not medical advice. The creator is not a medical professional. Always seek the advice of a qualified healthcare provider with any questions about your health, and do not disregard or delay professional medical advice because of this app. Use is at your own risk.
```

## Short description (150 char max)

```
A translucent breathing overlay that gently expands on inhale and contracts on exhale: a friendly reminder to breathe fully while staring at screens.
```

## Search terms (keywords)

```
breathing, mindfulness, focus, breath, reminder, meditation, productivity, calm, relaxation, mental health, box breathing, pranayama, wellness, overlay, screen, blink
```

## What's new in this version

Per-release, so it is **not** pinned here. Take it from the release notes for the tag being
submitted. The v2.0.21 text, kept as a shape reference:

```
Rebuilt from the ground up in Rust. ~36% lower CPU vs the prior build. Every action has a customizable global keyboard shortcut, full keyboard navigation in Preferences, inline reset confirmation, smoother Start/Stop transitions, near-zero CPU when all four breath durations are set to zero (treated as a static tint).
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
(ESRB "Everyone" / PEGI 3 / USK 0).

**Re-check this if a research surface ever ships inside the binary.** The questionnaire asks whether
the app contains medical or treatment information, and today the honest answer is no: the binary
carries no health claims and no disclaimer, only a wordless overlay and four numeric sliders. Adding
in-app evidentiary text could change that answer and re-trigger review. See
[RESEARCH-SURFACE-HANDOFF.md](../../../RESEARCH-SURFACE-HANDOFF.md).
