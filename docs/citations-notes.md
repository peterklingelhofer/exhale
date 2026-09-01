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
either claims more than the literature supports, or ships a default that the literature does not
back. Nothing here is a reason not to use the app. It is a list of things that are currently
believed rather than known.

### 1. Shallow breathing at screens: a retraction, and what the evidence actually supports

**An earlier revision of this file said no peer-reviewed study had measured respiration during
ordinary screen use. That was wrong, and it was wrong because the first search was not deep enough.**
The literature exists, it is just old and small and filed under ergonomics rather than under
breathwork.

- [`schleifer1994-vdt-petco2`](#schleifer1994-vdt-petco2): eleven data-entry operators, monitored
  continuously across three consecutive six-hour work days. During computer work, end-tidal CO2 was
  significantly **lower** and respiration frequency significantly **higher** than during either
  baseline relaxation or progressive muscle relaxation.
- [`schleifer2008-emg-gaps-computer-work`](#schleifer2008-emg-gaps-computer-work): the same group,
  fourteen years later, n = 23. Lower end-tidal CO2 under high mental workload during computer data
  entry, and it tracked reduced trapezius EMG gaps.
- [`schleifer2002-hyperventilation-job-stress`](#schleifer2002-hyperventilation-job-stress): the
  theory paper tying it together, which states that hyperventilation "is often characterised by a
  shift from a **diaphragmatic to a thoracic** breathing pattern," recruiting sternocleidomastoid,
  scalene and trapezius.

That diaphragmatic-to-thoracic shift is what people mean by "shallow." It is chest breathing instead
of belly breathing. So the folk claim has a real mechanism in the peer-reviewed record.

There is a second, independent line of support through posture.
[`jung2016-smartphone-posture-respiration`](#jung2016-smartphone-posture-respiration) found that
people using smartphones more than four hours a day had significantly worse craniovertebral angle
and lower peak expiratory flow than lighter users, and
[`deniz2024-forward-head-lung-volumes`](#deniz2024-forward-head-lung-volumes) found forward head
posture associated with FVC reductions of 0.25 to 0.81 L across 115 participants.

**What is still not supported** is "shallow" meaning reduced *volume of air moved*.
[`grassmann2016-cognitive-load-respiration`](#grassmann2016-cognitive-load-respiration), 54
experiments, finds respiratory amplitude roughly stable and minute ventilation **up** under
cognitive load. Both things are true at once: more air per minute, moved by the wrong muscles, from
a slumped posture with less capacity available.

So the precise, defensible statement is: at a screen people breathe **faster, higher in the chest,
and slightly over-ventilated**, from a posture that reduces how much the diaphragm can do. That is
the claim the README now makes.

Separately, and still true: the specific popular framing of "screen apnea" or "email apnea," meaning
outright breath-*holding* at a screen, traces to unpublished observations by Linda Stone from 2007,
tested informally on acquaintances with no protocol, no published data and no replication. Nothing
found in this pass measures breath-holding during screen use. The over-breathing finding above is
close to the opposite of breath-holding, and the two claims should not be run together.

### 2. exhale's default breathing rate is below the band anyone has tested

exhale ships 5 s inhale and 10 s exhale
([`rust/crates/exhale-core/src/settings.rs`](../rust/crates/exhale-core/src/settings.rs)). That is a
15-second cycle, or **4.0 breaths per minute**.

[`you2023-respiratory-frequency`](#you2023-respiratory-frequency) tested 5, 5.5, 6, 6.5 and 7 cycles
per minute and found all of them raised cardiac vagal activity above spontaneous breathing. It did
not test 4. [`lehrer2014-hrv-biofeedback`](#lehrer2014-hrv-biofeedback) puts the average resonance
frequency at about 5.5 breaths per minute and notes it varies by individual.
[`lin2014-equal-ratio-hrv`](#lin2014-equal-ratio-hrv) found 5.5 bpm outperformed 6.

So exhale's out-of-the-box default sits below the bottom of the range with direct support. It may be
fine, or better, for a given person; nobody has measured it.

The shipped default has deliberately **not** been changed. Silently moving the pace under people who
already have exhale installed is worse than documenting where the number came from. Changing it is a
decision to make on purpose, in a release note, not as a side effect of writing this file.

**What shipped instead: the app says it out loud.** The Timing card in the settings window now
computes the current rate from the four duration fields and prints it next to the range above, so
the default discloses its own coverage on first open rather than only here. It renders unprompted,
not behind a hover or a disclosure triangle, for a specific reason: a disclosure reaches the people
who go looking, and the default is imposed on everyone who never opens the panel at all. The copy
lives in [`rust/crates/exhale-core/src/pacing.rs`](../rust/crates/exhale-core/src/pacing.rs) with a
test per line, including one asserting that no line names an effect, a benefit or a condition. The
binary states arithmetic and one range; everything evidentiary stays in this document, which can be
corrected in an afternoon rather than in a store-review cycle.

### 3. Box breathing is not the beginner-friendly default it looks like

A natural thought is that `4` / `4` / `4` / `4` box breathing would be a gentler default than the
current 5 / 10. Two problems.

First, arithmetic: 4+4+4+4 is a 16-second cycle, or **3.75 breaths per minute**. That is *slower*
than the current default and further below the tested band, not closer to it. The holds hide the
rate.

Second, and more decisively, box breathing has been tested head-to-head and lost twice:

- [`marchant2025-square-478-six`](#marchant2025-square-478-six), n = 84, compared square breathing,
  4-7-8 breathing, and 6 breaths per minute at two ratios. Breathing at 6 bpm raised HRV **more than
  either square or 4-7-8**, with small to medium effects. The paper opens by stating flatly that
  square and 4-7-8 "are popularly promoted by psychotherapists but have little empirical support."
- [`balban2023-cyclic-sighing`](#balban2023-cyclic-sighing), a pre-registered RCT, tested box
  breathing against cyclic sighing and cyclic hyperventilation over a month. Box breathing was not
  the best-performing arm; the exhale-emphasising one was.

The same evidence rules out 4-7-8, sometimes attributed to Andrew Weil, for the same reason: it lost
in Marchant, and at 4+7+8 = 19 s it is 3.16 breaths per minute, slower still.

**The evidence-based beginner default is `5` / `0` / `5` / `0`**: 6 breaths per minute, no holds,
a 10-second cycle. It sits inside the tested band, it is the condition that won in Marchant, and it
is genuinely easier for a beginner than box breathing, because holding the breath is the hard part,
not the slow part. Box breathing remains a perfectly reasonable thing to want and exhale still
supports it. It is just not the choice the evidence points at.

**What shipped.** Both patterns are offered as one-click presets in the Timing card, `5` / `0` / `5`
/ `0` first and `4` / `4` / `4` / `4` fourth, and neither carries a badge, a rank or a caption
claiming anything. Ordering is the only editorial signal, and it is a weak one on purpose. What does
the work instead is that selecting box breathing makes the readout directly below say *"Now: 3.8
breaths a minute, a 16 s cycle"* and *"This one is slower than any of them"*, computed rather than
written. A caption asserting the same thing would have to be maintained against the corpus; the
arithmetic maintains itself, and it appears for every pattern rather than only the ones somebody
remembered to annotate. The presets are listed in
[`rust/crates/exhale-core/src/presets.rs`](../rust/crates/exhale-core/src/presets.rs), where each
carries a citekey that never reaches the screen and exists only so
[`scripts/generate-citations.py`](../scripts/generate-citations.py) can fail the build if the record
behind a shipped pattern is retracted, downgraded to tier E, or marked `inAppCitable: false`.

Four records carry that flag today: [`chaddha2019-slow-breathing-bp`](#chaddha2019-slow-breathing-bp),
[`fincham2023-breathwork-meta`](#fincham2023-breathwork-meta),
[`balban2023-cyclic-sighing`](#balban2023-cyclic-sighing) and
[`little2025-a52-breath-method`](#little2025-a52-breath-method). The flag is not a quality judgement
and reads oddly without that said out loud: `fincham2023` is one of the strongest warrants in this
corpus. It marks records whose claims are about blood pressure, anxiety, depression or mood, which a
store-reviewed binary should not be leaning on whatever their quality, because a claim compiled into
a signed app cannot be withdrawn at the speed evidence changes.

### 4. The 1:2 ratio: what it does and does not support

The README used to say to make the exhale twice as long as the inhale "to engage the parasympathetic
nervous system." That mechanism claim is not supportable as stated. But the practice is *not*
unsupported, and the reason is a distinction worth getting right: **the HRV studies and the
how-do-you-feel studies disagree with each other, and they disagree in a consistent direction.**

On HRV, four results, and they do not line up:

| Source | n | Design | Result on ratio |
|---|---|---|---|
| [`bae2021-exhalation-inhalation-ratio`](#bae2021-exhalation-inhalation-ratio) | 28 | 2:1 vs 1:1 cue at spontaneous rate | Longer exhale raised RMSSD and HF-HRV |
| [`vandiest2014-ie-ratio-relaxation`](#vandiest2014-ie-ratio-relaxation) | 30 | i/e 0.42 vs 2.33, at 6 and 12 bpm | Longer exhale raised HF-HRV, but only at the slow rate |
| [`lin2014-equal-ratio-hrv`](#lin2014-equal-ratio-hrv) | 47 | 5:5 vs 4:6 at 5.5 and 6 bpm | **Equal** ratio won on SDNN and LF |
| [`meehan2024-longer-exhalations`](#meehan2024-longer-exhalations) | 26 + 16 replication | 1:1 vs 1:2 at 6 bpm | No difference, in the original *and* the replication |

Two for, one against, one null. Nobody should be asserting a mechanism from that.

On how people actually felt, the picture is cleaner.
[`vandiest2014-ie-ratio-relaxation`](#vandiest2014-ie-ratio-relaxation) is the only study here that
measured subjective state across ratios, and participants reported more relaxation, more stress
reduction, more mindfulness and more positive energy with the longer exhale. Slowing the rate alone
moved only one of those four. [`balban2023-cyclic-sighing`](#balban2023-cyclic-sighing) points the
same way over a month, on mood.

So: **keep the recommendation, drop the mechanism.** A longer exhale is worth preferring because
people report feeling better doing it, which is the outcome anyone installing exhale actually cares
about. It should not be sold as a way to "engage the parasympathetic nervous system," because the
cardiac measures that would show that are split. Note also
[`lin2014-equal-ratio-hrv`](#lin2014-equal-ratio-hrv)'s second finding: *every* slow pattern it
tested increased relaxation over baseline. Rate is doing most of the work. Ratio is a preference
with a modest, contested edge.

### 5. Pacing someone slowly at a screen may push them further into over-breathing

This one is a genuine tension between two entries and nobody has looked at it.

[`marchant2025-square-478-six`](#marchant2025-square-478-six) reports, as an unexpected finding, that
breathing at 6 breaths per minute produced **mild over-breathing**: HRV went up and end-tidal CO2
went down. Meanwhile [`schleifer1994-vdt-petco2`](#schleifer1994-vdt-petco2) and
[`schleifer2008-emg-gaps-computer-work`](#schleifer2008-emg-gaps-computer-work) show that a person
at a keyboard is *already* mildly hypocapnic.

exhale paces rate and says nothing about depth. A user who slows to 6 breaths per minute while
taking large breaths moves more air per minute, not less. The whole benefit of slow breathing is
usually framed as calming, and the CO2 direction here runs the other way. No study in this corpus
tests a slow pacer on an already-hypocapnic screen worker, which is exactly exhale's user.

Nothing actionable follows yet, and this is not a safety warning: the effect Marchant reports is
mild and was measured in a single session. It is recorded because it is the most interesting
unanswered question this corpus turned up, and because an app that paces breathing should know that
pacing rate is not the same as pacing volume.

### 6. `drift`: the tradition is uncontradicted, and the app should not pretend otherwise

**Two corrections in this entry, 2026-08-31.** Both were caught by the maintainer pushing back, and
both are the same mistake in different clothes: treating the edge of the research literature as if it
were the edge of legitimate practice.

**Correction 1 (the overclaim).** An earlier revision said the inverted-U relationship between
breathing rate and HRV meant progressively slower breathing "moves away from the optimum."
[`shaffer2020-resonance-frequency-assessment`](#shaffer2020-resonance-frequency-assessment)
describes a peak in **HRV amplitude**, not in relaxation, comfort or benefit. Sliding from one to the
other is exactly what this corpus's tier-D rule forbids, and it was done here to justify a code
change that had already been decided on.

**Correction 2 (the cap).** On the strength of that overclaim, a ceiling was added capping `drift` at
three breaths a minute. It has been **removed**. The argument against it is simple and correct: a
10-second inhale with a 20-second exhale is unremarkable in pranayama, absence of research is not
evidence of harm, and an app has no business preventing an advanced practitioner from configuring
what they actually practise. The ceiling was paternalism wearing a citation.

**What actually counters the tradition on elongating the breath: nothing found in this review.**

- No study located here tests subjective relaxation below about 5 breaths a minute. The literature
  stops; it does not turn around.
- The subjective evidence that exists **supports** the tradition.
  [`vandiest2014-ie-ratio-relaxation`](#vandiest2014-ie-ratio-relaxation) found the longer exhale
  produced more relaxation, stress reduction and positive energy;
  [`lin2014-equal-ratio-hrv`](#lin2014-equal-ratio-hrv) found every slow pattern beat baseline on
  relaxation; [`szulczewski2019-training-relaxation`](#szulczewski2019-training-relaxation) found
  relaxation *accrued over a week of practice* rather than arriving on day one, which is the
  tradition's own claim about training; and
  [`joshi1992-pranayam-training`](#joshi1992-pranayam-training) found six weeks of practice lowered
  resting respiratory rate and lengthened breath-holding time.
- The one real caution is about **depth, not rate**:
  [`szulczewski2019-antihyperventilation-instruction`](#szulczewski2019-antihyperventilation-instruction)
  shows paced breathing at 6 a minute drops end-tidal CO2 by 5.21 mmHg, and that one sentence of
  instruction cuts that to 2.7 mmHg. See gap 5.

**What shipped instead of a cap.** `drift` compounds without limit, as it always did. Two changes
address the real problem, which was never that slow breathing is bad but that the *control was too
coarse to ask for anything gentle*:

1. **The stepper step is now 0.1 percentage points, was 1.0.** Compounding makes whole percents
   enormous. From a 15 s cycle, 1 % doubles the breath in 70 cycles (~25 min); 0.1 % takes 693 cycles
   (~4.2 h); 0.01 % about 41 h. The entire useful range sat below the old minimum step, so the only
   drift a user could previously select was one that ran away inside a single sitting. Display is
   capped at three decimals, so 0.001 % is the finest value the field round-trips, and anything
   below the step is typed rather than clicked. The settings window reports the doubling point
   whenever drift is on, because "0.1 % per cycle" tells nobody whether they have chosen something
   gentle or something that runs away before lunch.

   **Counted in breaths, not minutes.** The doubling *time* depends on the cycle you start from, so
   the same 1 % reads as 17 minutes from a 10 s cycle and 25 minutes from a 15 s one; the first
   version of the readout said exactly that and looked like it could not do arithmetic. The doubling
   *count* has no such dependence: cycle `k` lasts `c · dᵏ`, so `dᵏ = 2` at `k = ln2 / ln d` and the
   starting length cancels. One per cent is 70 breaths from anywhere, 0.1 % is 693, 0.001 % is
   69,315. It is also a true repeat interval rather than a first milestone, since the same count
   takes the cycle from double to quadruple. The figures quoted in this entry are in minutes because
   they are anchored to the 15 s shipped default; the app cannot assume that.
2. **`drift` defaults to 1.0, off.** This is a coverage-and-consent argument, not a claim that drift
   is harmful: on by default it moved every new user out of the region anyone has measured, within
   minutes, without asking. It is one field away for anyone who wants it.

**Still unsupported, and worth stating.** No study examines a *progressively lengthening* pace at
all. `szulczewski2019-training-relaxation` trained at a fixed rate, so it supports "keep practising,"
not "keep slowing down within a session." The compounding ramp remains an invention of this app. That
is a reason to describe it honestly, which is what this entry is for, and not a reason to forbid it.

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
where transient tetany, light-headedness and distress are documented. This is the honest basis for
the README's existing advice to take breaks if intense feelings arise, and for it staying there.

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
   points at. This is a release-note decision, not a silent one.
3. Ship named presets with the citation attached to each, instead of asking users to invent numbers:
   6 breaths/min (`5` / `0` / `5` / `0`), A52 (`5` / `0` / `5` / `2`), box (`4` / `4` / `4` / `4`),
   and the current default as "extended exhale." A preset carrying its own evidence tier is more
   honest than a slider that implies every value is equally supported.
4. **Queued for the next release, no release needed of its own:** a "Research & Citations" item in
   the tray menu that opens this file in the browser. Roughly ten lines: one `MenuItem` built and
   appended in [`tray.rs`](../rust/crates/exhale-app/src/tray.rs) alongside `preferences_item`, and one
   arm in the `MenuEvent` loop at [`main.rs:1066`](../rust/crates/exhale-app/src/main.rs). It rides
   along with whatever is tagged next rather than justifying a build, re-sign and three store
   resubmissions on its own. A link out is deliberately preferred over rendering this corpus inside
   the settings window: exhale ships through Apple, Microsoft and Snap review, and putting prose about
   parasympathetic activation inside a health-adjacent binary invites scrutiny that a repo link does
   not.
5. Nobody has tested a slow visual pacer on an already-hypocapnic screen worker (gap 5). That is a
   real, publishable question that exhale is unusually well placed to ask.
