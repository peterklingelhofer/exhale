# Citation corpus

<!-- SUMMARY -->

Machine-readable companion: [`CITATIONS.csl.json`](./CITATIONS.csl.json) (CSL-JSON).
Verification pass completed 2026-08-28 against the Crossref REST API.

> **This file is generated.** Edit [`CITATIONS.csl.json`](./CITATIONS.csl.json) for records and
> [`citations-notes.md`](./citations-notes.md) for the prose, then run
> `uv run --no-project scripts/generate-citations.py`. Editing `CITATIONS.md` directly will be
> overwritten. `--check` fails if the two drift apart.

exhale is a breathing overlay, not a medical device, and nothing here is medical advice. See the
[disclaimer](../README.md#disclaimer). This corpus exists so that anyone, including the author, can
check which of exhale's claims and defaults rest on published evidence and which do not. The
[gaps ledger](#gaps-and-unsupported-choices) at the end is the more useful half.

## How to read this

### Verification status

This says how the *bibliographic record* was checked. It says nothing about whether the finding is
true, and nothing about whether the full text was read.

| Status | Meaning |
|---|---|
| `crossref-verified` | The DOI resolves in Crossref, and the title, authors, year, journal, volume and pages printed here are the ones Crossref returned, not the ones a search result claimed. |
| `openlibrary-verified` | No DOI exists because the source is a book. The title, author, publisher and page count printed here were checked against the Open Library record for the stated ISBN. Edition ambiguities are written into the caveat. |
| `unverified` | No resolvable DOI and no catalogue record. Bibliographic details are inherited from secondary citation and may be wrong. |

A `crossref-verified` record can still carry a loud caveat. Several entries here are Crossref-verified
but were paywalled to full-text fetch, meaning we confirmed the paper exists and is what we say it is
but never read its numbers. Those carry `NUMBERS NOT READ` in the caveat. Verification confirms the
*citation*, not the *claim*.

### Access level

`open-access` (a CC licence is registered with Crossref, or the title is fully open access) |
`paywalled`. Where a Crossref `license` field was present, the access level is taken from it rather
than guessed; where it was absent the basis is stated in the entry's caveat.

### Evidence tier

Applied to sources making a claim about what happens to a human being. Physiological and animal
mechanism papers are tier D by definition: they explain why something might work, they do not
establish that it does.

| Tier | Definition | How exhale is allowed to use it |
|---|---|---|
| A | Systematic review or meta-analysis of controlled trials, or a large pre-registered RCT | May be cited for an outcome claim |
| B | Controlled experiment with an internal replication, or a systematic review of controlled experiments | May be cited for an outcome claim, with its scope conditions stated |
| C | Single controlled experiment, small n, or lab-only | Cite as suggestive; never as "research shows" |
| D | Narrative review, mechanism, or animal work | Cite for *why*, never for *whether* |
| E | Not peer reviewed, or contradicted by better evidence | May be cited for provenance, meaning where a practice came from. Never for whether it works |

<!-- COUNTS -->

<!-- CORPUS -->

---

## Gaps and unsupported choices

Written by hand. This section is the point of the exercise: everything below is a place where exhale
ships something the literature does not settle, or where the evidence is thinner or more divided
than a bare citation would suggest. Nothing here is a reason not to use the app. It is a list of
things that are currently believed rather than known.

### 1. What is actually measured about breathing at a screen

The relevant literature is old, small, and filed under ergonomics rather than breathwork.

- [`schleifer1994-vdt-petco2`](#schleifer1994-vdt-petco2): eleven data-entry operators, monitored
  continuously across three consecutive six-hour work days. During computer work, end-tidal CO2 was
  significantly **lower** and respiration frequency significantly **higher** than during either
  baseline relaxation or progressive muscle relaxation.
- [`schleifer2008-emg-gaps-computer-work`](#schleifer2008-emg-gaps-computer-work): the same group,
  fourteen years later, n = 23. Lower end-tidal CO2 under high mental workload during computer data
  entry, tracking reduced trapezius EMG gaps.
- [`schleifer2002-hyperventilation-job-stress`](#schleifer2002-hyperventilation-job-stress): the
  theory paper tying it together, which states that hyperventilation "is often characterised by a
  shift from a **diaphragmatic to a thoracic** breathing pattern," recruiting sternocleidomastoid,
  scalene and trapezius.

That diaphragmatic-to-thoracic shift is what people mean by "shallow": chest breathing instead of
belly breathing.

A second, independent line runs through posture.
[`jung2016-smartphone-posture-respiration`](#jung2016-smartphone-posture-respiration) found that
people using smartphones more than four hours a day had significantly worse craniovertebral angle
and lower peak expiratory flow than lighter users, and
[`deniz2024-forward-head-lung-volumes`](#deniz2024-forward-head-lung-volumes) found forward head
posture associated with FVC reductions of 0.25 to 0.81 L across 115 participants.

**What the evidence does not support** is "shallow" meaning a reduced *volume of air moved*.
[`grassmann2016-cognitive-load-respiration`](#grassmann2016-cognitive-load-respiration), 54
experiments, finds respiratory amplitude roughly stable and minute ventilation **up** under
cognitive load. Both things hold at once: more air per minute, moved by the wrong muscles, from a
slumped posture with less capacity available.

The defensible statement is therefore that at a screen people breathe **faster, higher in the chest,
and slightly over-ventilated**, from a posture that reduces how much the diaphragm can do. That is
the claim the README makes.

It is not the same claim as "screen apnea" or "email apnea," meaning outright breath-*holding* at a
screen. That framing traces to unpublished observations by Linda Stone from 2007, tested informally
on acquaintances with no protocol, no published data and no replication. Nothing found in this pass
measures breath-holding during screen use, and the over-breathing finding above points the other
way. The two claims should not be run together.

### 2. The shipped default sits below the band with direct support

exhale ships 5 s inhale and 10 s exhale
([`rust/crates/exhale-core/src/settings.rs`](../rust/crates/exhale-core/src/settings.rs)): a
15-second cycle, or **4.0 breaths per minute**.

[`you2023-respiratory-frequency`](#you2023-respiratory-frequency) tested 5, 5.5, 6, 6.5 and 7 cycles
per minute and found all of them raised cardiac vagal activity above spontaneous breathing. It did
not test 4. [`lehrer2014-hrv-biofeedback`](#lehrer2014-hrv-biofeedback) puts average resonance
frequency at about 5.5 breaths per minute and notes it varies by individual.
[`lin2014-equal-ratio-hrv`](#lin2014-equal-ratio-hrv) found 5.5 outperformed 6.

So 4.0 sits below the bottom of the directly tested range. It may be fine, or better, for a given
person; nobody has measured it. The default is kept for continuity with existing installs, and the
Timing panel computes the current rate and states it against that range on open, so the coverage is
visible where the choice is made. `5` / `0` / `5` / `0`, which is 6 a minute and inside the range,
is the first of the one-click presets.

### 3. Box breathing is slower than it looks

`4` / `4` / `4` / `4` is a 16-second cycle, or **3.75 breaths per minute**: *slower* than exhale's
default and further below the tested band, not closer to it. The holds hide the rate, which is why
the settings panel computes it.

Box breathing has also been tested head-to-head twice and did not win either time:

- [`marchant2025-square-478-six`](#marchant2025-square-478-six), n = 84, compared square breathing,
  4-7-8 breathing, and 6 breaths per minute at two ratios. Breathing at 6 raised HRV **more than
  either square or 4-7-8**, with small to medium effects. The paper opens by stating that square and
  4-7-8 "are popularly promoted by psychotherapists but have little empirical support."
- [`balban2023-cyclic-sighing`](#balban2023-cyclic-sighing), a pre-registered RCT, tested box
  breathing against cyclic sighing and cyclic hyperventilation over a month. Box breathing was not
  the best-performing arm; the exhale-emphasising one was.

The same evidence applies to 4-7-8, sometimes attributed to Andrew Weil: it lost in Marchant, and at
4+7+8 = 19 s it is 3.16 breaths per minute, slower still.

The pattern with the best direct support for someone starting out is `5` / `0` / `5` / `0`: 6 breaths
per minute, no holds, a 10-second cycle. It sits inside the tested band, it is the condition that won
in Marchant, and holding the breath is the harder part for a beginner rather than the slow part. Box
breathing remains a reasonable thing to want, and exhale offers it as a preset. It is simply not the
pattern the evidence points at.

### 4. Longer exhale: supported on how people feel, contested on the heart

On HRV, four results, and they do not line up:

| Source | n | Design | Result on ratio |
|---|---|---|---|
| [`bae2021-exhalation-inhalation-ratio`](#bae2021-exhalation-inhalation-ratio) | 28 | 2:1 vs 1:1 cue at spontaneous rate | Longer exhale raised RMSSD and HF-HRV |
| [`vandiest2014-ie-ratio-relaxation`](#vandiest2014-ie-ratio-relaxation) | 30 | i/e 0.42 vs 2.33, at 6 and 12 bpm | Longer exhale raised HF-HRV, but only at the slow rate |
| [`lin2014-equal-ratio-hrv`](#lin2014-equal-ratio-hrv) | 47 | 5:5 vs 4:6 at 5.5 and 6 bpm | **Equal** ratio won on SDNN and LF |
| [`meehan2024-longer-exhalations`](#meehan2024-longer-exhalations) | 26 + 16 replication | 1:1 vs 1:2 at 6 bpm | No difference, in the original *and* the replication |

Two for, one against, one null. No mechanism claim survives that split, which is why exhale does not
make one.

On how people reported feeling, the picture is cleaner.
[`vandiest2014-ie-ratio-relaxation`](#vandiest2014-ie-ratio-relaxation) is the only study here that
measured subjective state across ratios, and participants reported more relaxation, more stress
reduction, more mindfulness and more positive energy with the longer exhale. Slowing the rate alone
moved only one of those four. [`balban2023-cyclic-sighing`](#balban2023-cyclic-sighing) points the
same way over a month, on mood.

So a longer exhale is worth preferring because people report feeling better doing it, which is the
outcome anyone installing exhale actually cares about. Note also
[`lin2014-equal-ratio-hrv`](#lin2014-equal-ratio-hrv)'s second finding: *every* slow pattern it
tested increased relaxation over baseline. Rate does most of the work; ratio is a preference with a
modest, contested edge.

### 5. Pacing someone slowly at a screen may push them further into over-breathing

A genuine tension between two entries that nobody has looked at.

[`marchant2025-square-478-six`](#marchant2025-square-478-six) reports, as an unexpected finding, that
breathing at 6 breaths per minute produced **mild over-breathing**: HRV went up and end-tidal CO2
went down. Meanwhile [`schleifer1994-vdt-petco2`](#schleifer1994-vdt-petco2) and
[`schleifer2008-emg-gaps-computer-work`](#schleifer2008-emg-gaps-computer-work) show that a person at
a keyboard is *already* mildly hypocapnic.

exhale paces rate and says nothing about depth. A user who slows to 6 breaths per minute while taking
large breaths moves more air per minute, not less. No study in this corpus tests a slow pacer on an
already-hypocapnic screen worker, which is exactly exhale's user.

This is not a safety warning: the effect Marchant reports is mild and was measured in a single
session. It is recorded because it is the most interesting unanswered question this corpus turned up,
and because an app that paces breathing should know that pacing rate is not the same as pacing
volume. The one mitigation with evidence behind it is an instruction rather than a setting:
[`szulczewski2019-antihyperventilation-instruction`](#szulczewski2019-antihyperventilation-instruction)
shows one sentence of anti-hyperventilation guidance cuts the end-tidal CO2 drop from 5.21 mmHg to
2.7.

### 6. `drift` is an invention of this app

`drift` lengthens every cycle by a fixed percentage, compounding, so the breath extends gradually
across a session. Graded extension of the breath is a long-standing pranayama practice
([`satyananda2008-apmb`](#satyananda2008-apmb)), but **no study in this corpus examines a
progressively lengthening pace at all.** [`szulczewski2019-training-relaxation`](#szulczewski2019-training-relaxation)
trained at a fixed rate and found relaxation accrued over a week of practice, which supports "keep
practising" rather than "keep slowing down within a session."

Nothing in this review contradicts the tradition either. The literature simply stops below about 5
breaths a minute; it does not turn around and report worse outcomes there. What subjective evidence
exists points the tradition's way:
[`vandiest2014-ie-ratio-relaxation`](#vandiest2014-ie-ratio-relaxation) found the longer exhale
produced more relaxation, stress reduction and positive energy;
[`lin2014-equal-ratio-hrv`](#lin2014-equal-ratio-hrv) found every slow pattern beat baseline on
relaxation; and [`joshi1992-pranayam-training`](#joshi1992-pranayam-training) found six weeks of
practice lowered resting respiratory rate and lengthened breath-holding time. The inverted-U in
[`shaffer2020-resonance-frequency-assessment`](#shaffer2020-resonance-frequency-assessment) is worth
reading alongside these, but it describes a peak in **HRV amplitude**, not in relaxation or comfort,
and does not transfer to one.

The one documented caution is about **depth, not rate**. See gap 5.

`drift` is therefore unbounded and **defaults to 0, off**. Off by default is a coverage argument
rather than a claim of harm: on by default it would move every new user out of the region anyone has
measured, within minutes, without asking. Unbounded because a 10-second inhale with a 20-second
exhale is unremarkable in pranayama, and absence of research is not evidence of harm.

The stepper moves in 0.1 percentage points, and values below that can be typed; display is capped at
three decimals, so 0.001 % is the finest value the field round-trips. Compounding is steep enough
that whole percents are unusable: from a 15 s cycle, 1 % doubles the breath in about 25 minutes,
0.1 % in about 4.2 hours, 0.01 % in about 41. The settings panel reports the doubling point in
**breaths** rather than minutes, because cycle `k` lasts `c · dᵏ` and so `dᵏ = 2` at
`k = ln2 / ln d`, with the starting cycle length cancelling out: 1 % is 70 breaths from any starting
pace, 0.1 % is 693, 0.001 % is 69,315. A doubling time would depend on where the user started and
would disagree with the minute figures quoted above, which are anchored to the 15 s default.

### 7. Randomised timing has no literature behind it either

The four randomisation sliders inject per-phase jitter. Every pacing study in this corpus uses a
fixed rate; that is what "paced" means. The nearest adjacent literature is
[`vlemincx2013-sigh-reset-model`](#vlemincx2013-sigh-reset-model), on natural respiratory
variability and sighs, which is about spontaneous breathing rather than about deliberately
destabilising a pacer. Defaults are 0, which is the right default. Treat the sliders as an
aesthetic option.

### 8. Nothing about exhale itself has ever been measured

No study in this corpus is about exhale. The closest published analogue is
[`moraveji2011-peripheral-paced-respiration`](#moraveji2011-peripheral-paced-respiration): a
translucent animated bar across the screen, running in the periphery during real information work,
which significantly lowered participants' breathing rate. Its limitation is the one that matters
here. The reduction happened **while the pacing was active** and did not persist as a lasting change
in respiratory pattern. An always-on overlay should be understood as an effect that lasts as long as
it is on.

The strongest design warrant is [`tabor2022-guided-breathing-design`](#tabor2022-guided-breathing-design):
an expanding and contracting circle at 6 breaths/min matched sensor-driven HRV biofeedback on HRV
amplitude, with effects appearing in about two minutes and no hardware needed. That is exhale's
Circle mode, and it is why exhale needs no sensor, no account and no telemetry. It is still n = 28
in one session.

### 9. Visual-only guidance is the weaker modality for the outcome users care about

[`wongsuphasawat2012-cant-force-calm`](#wongsuphasawat2012-cant-force-calm) found visual pacing
produced more measured respiratory change than auditory pacing, but auditory was rated more calming.
exhale is visual-only by design, because it is meant to sit silently in the corner of a working
screen. That is a real trade-off against felt calm, made deliberately. The source is a two-page
adjunct paper, so it is a signal rather than a result.

### 10. exhale cites blink research and does nothing about blinking

The blink-rate literature is well supported, and exhale does not act on it. The overlay paces
breathing; it does not prompt a blink, does not detect blinks, and does not implement anything from
the digital eye strain literature. The blink finding is context for why screens deserve a nudge, not
a description of what this app does.

Also relevant to exhale's whole genre: [`johnson2023-20-20-20`](#johnson2023-20-20-20) found that
scheduled 20-second breaks at any of three intervals produced no significant effect on symptoms,
reading speed or accuracy. A periodic on-screen nudge is not effective merely because it is popular.

### 11. Both hold sliders default to 0, with no cited basis either way

`post_inhale_hold_duration` and `post_exhale_hold_duration` both default to 0.
[`laborde2021-ie-ratio-pauses`](#laborde2021-ie-ratio-pauses) is the study that manipulates
respiratory pauses directly, and it was paywalled to full-text fetch in this pass, so its numbers
have not been read. [`little2025-a52-breath-method`](#little2025-a52-breath-method) argues for a 2 s
post-exhale hold, while [`marchant2025-square-478-six`](#marchant2025-square-478-six) found the two
hold-heavy patterns it tested underperformed a no-hold 6 bpm pace. On current evidence, 0 is a
defensible default.

### 12. Nobody knows whether anyone keeps using it

exhale has no telemetry, by design and stated in [PRIVACY.md](../PRIVACY.md). The consequence is
that the single biggest determinant of whether a tool like this does anything, namely whether people
keep it running, is unmeasured and unmeasurable here.
[`linardon2020-app-attrition`](#linardon2020-app-attrition) is the reality check: dropout and
non-adherence dominate outcomes for app-delivered interventions even where efficacy trials are
positive. Every effect size in this corpus was measured in a supervised session with a compliant
participant. That is not the same population as someone who installed a menu-bar app in March.

### 13. Adverse-event reporting in this field is thin

Reviews of breathwork note that only a minority of trials report on adverse events at all.
[`laborde2022-vsb-meta`](#laborde2022-vsb-meta) concludes that few adverse effects are expected from
*slow* breathing specifically, which is the mode exhale is built around. exhale's sliders can also
be set to fast, hold-heavy patterns that leave that evidence base entirely; those belong to the
high-ventilation literature ([`fincham2024-high-ventilation-rct`](#fincham2024-high-ventilation-rct)),
where transient tetany, light-headedness and distress are documented. This is the basis for the
README's advice to take breaks if intense feelings arise.

### 14. The tradition sources are lineage, not evidence, and are tiered accordingly

exhale's four-phase structure, inhale / retention / exhale / retention, is pranayama. It did not come
from psychophysiology, and the corpus says so: [`satyananda2008-apmb`](#satyananda2008-apmb) and
[`muktibodhananda1998-hatha-yoga-pradipika`](#muktibodhananda1998-hatha-yoga-pradipika) are carried
at tier **E**, meaning they may be cited for where a practice came from and never for whether it
works.

This is a deliberate inclusion rather than an endorsement, for two reasons. Retrofitting a 2020s HRV
citation onto an instruction that is centuries older would be revisionist about the app's actual
design history. And the "longer exhale" idea specifically entered modern breathing apps through this
tradition, not through a laboratory, which is worth knowing when weighing how much of the supporting
literature was designed to test a pre-existing belief rather than to discover something.

Both entries are marked `NOT READ`. Their bibliographic records were checked against Open Library;
their contents were not consulted, and no claim in this repo rests on them. The Satyananda edition
history is genuinely tangled: Open Library returns the Yoga Publications Trust 553-page edition under
the cited ISBN but dates its record 1999, while the printing generally referred to is the 2008 Fourth
Revised Edition. That is not resolved here.

## What would close the biggest gaps

In rough order of value per unit effort:

1. Read the `NUMBERS NOT READ` entries and either promote or downgrade the claims resting on them.
   [`laborde2021-ie-ratio-pauses`](#laborde2021-ie-ratio-pauses) is the highest value: it bears
   directly on gaps 4 and 11.
2. Decide, deliberately, whether the shipped default should move from 4.0 breaths/min to
   `5` / `0` / `5` / `0`, which is 6 breaths/min and is what the head-to-head evidence in gap 3
   points at. This is a release-note decision, not a silent one, because it would change the pace
   under everyone who has never opened the settings panel.
3. Nobody has tested a slow visual pacer on an already-hypocapnic screen worker (gap 5). That is a
   real, publishable question that exhale is unusually well placed to ask.
4. Settle whether graded extension does anything, which would put a floor under gap 6. No study in
   this corpus varies the pace *within* a session, so the question is open in both directions.
