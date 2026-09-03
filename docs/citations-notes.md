# Citation corpus

<!-- SUMMARY -->

Machine-readable companion: [`CITATIONS.csl.json`](./CITATIONS.csl.json) (CSL-JSON).
Bibliographic records were verified on 2026-08-28 (Crossref, Open Library) and 2026-08-30 (PubMed).
Every claim was re-checked against the abstracts and open full texts on 2026-09-02.

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
| `openlibrary-verified` | No DOI exists because the source is a book. The title, author, publisher, edition and page count printed here were checked against the Open Library record for the stated ISBN. |
| `pubmed-verified` | No DOI exists, but the article is indexed in PubMed. The title, authors, journal, year, volume and pages were checked against the NCBI E-utilities record for the stated PMID. |
| `unverified` | No resolvable DOI and no catalogue record. Bibliographic details are inherited from secondary citation and may be wrong. |

Author names are printed as the registry holds them, which is why a few records carry initials
where others carry full given names.

A verified record can still carry a loud caveat. Verification confirms the *citation*, not the
*claim*, so each entry also states how deeply the source was read before its claims were written
down: in full, from the abstract only, or as a catalogue record only.

### Access level

`open-access` (a CC licence is registered with Crossref, or the title is fully open access) |
`paywalled`. Where a Crossref `license` field was present, the access level is taken from it rather
than guessed; where it was absent the basis is stated in the entry's caveat. `paywalled` describes
the version of record. Where a legal open copy exists in a repository or free at the publisher, the
entry links it as an open copy.

### Evidence tier

Applied to sources making a claim about what happens to a human being. Physiological and animal
mechanism papers are tier D by definition: they explain why something might work, they do not
establish that it does.

| Tier | Definition | How exhale is allowed to use it |
|---|---|---|
| A | Systematic review or meta-analysis of controlled trials, or a pre-registered RCT with 200 or more participants | May be cited for an outcome claim |
| B | Pre-registered or internally replicated controlled experiment, or a systematic review of experiments without meta-analysis | May be cited for an outcome claim, with its scope conditions stated |
| C | Single controlled experiment, small n, lab-only, or observational work, including systematic reviews of observational studies | Cite as suggestive; never as "research shows" |
| D | Narrative review, theory, mechanism, or animal work | Cite for *why*, never for *whether* |
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
belly breathing. It is a theory of mechanism. No study in this corpus measures chest against
diaphragm breathing in screen users.

Two things limit how much the keyboard can be blamed. Both Schleifer studies compare work with
relaxation, and [`grassmann2016-cognitive-load-respiration`](#grassmann2016-cognitive-load-respiration)
finds the same faster, over-ventilated pattern under any demanding task, so the keyboard is where
the effect was measured rather than shown to be its cause. And the two samples total 34 people
from one research group.

A second, independent line runs through posture.
[`jung2016-smartphone-posture-respiration`](#jung2016-smartphone-posture-respiration) found that
people using smartphones more than four hours a day had significantly worse craniovertebral angle
and lower peak expiratory flow than lighter users, and
[`deniz2024-forward-head-lung-volumes`](#deniz2024-forward-head-lung-volumes) found forward head
posture associated with FVC reductions of 0.25 to 0.81 L across 115 participants.

**What the evidence does not support** is "shallow" meaning a reduced *volume of air moved*.
[`grassmann2016-cognitive-load-respiration`](#grassmann2016-cognitive-load-respiration), 54
experiments, finds respiratory amplitude roughly stable and minute ventilation **up** under
cognitive load. Both things can hold at once: more air per minute, on the theory above moved by
the wrong muscles, from a slumped posture associated with less capacity.

The defensible statement is therefore that during demanding work at a keyboard people breathe
**faster and slightly over-ventilated**, from a posture associated with reduced lung volumes; the
shift to chest breathing is the theory of why, not a measurement. That is the claim the README
makes.

It is not the same claim as "screen apnea" or "email apnea," meaning outright breath-*holding* at a
screen. That framing traces to unpublished observations by Linda Stone from 2007, tested informally
on acquaintances with no protocol, no published data and no replication. Nothing found in this pass
measures breath-holding during screen use, and the over-breathing finding above points the other
way. The two claims should not be run together.

### 2. The default is inside the tested band, but the band is narrow and resonance frequency is individual

exhale ships 5 s inhale and 5 s exhale, no holds
([`rust/crates/exhale-core/src/settings.rs`](../rust/crates/exhale-core/src/settings.rs)): a
10-second cycle, or **6.0 breaths per minute**.

[`you2023-respiratory-frequency`](#you2023-respiratory-frequency) tested 5, 5.5, 6, 6.5 and 7 cycles
per minute and found all of them raised cardiac vagal activity above spontaneous breathing.
[`marchant2025-square-478-six`](#marchant2025-square-478-six) ran 6 a minute against square and
4-7-8 breathing head-to-head, n = 84, and 6 won. So the default is the pace with the most direct
support of anything exhale could have shipped.

That is a weaker statement than it sounds, for two reasons.

**The tested band is five values wide.** Nobody has compared 6 against 3, or against 8, in this
corpus. "Inside the range that has been tested" is a statement about coverage, not about optimality.

**Resonance frequency is individual.** [`lehrer2014-hrv-biofeedback`](#lehrer2014-hrv-biofeedback)
puts the average at about 5.5 breaths per minute and is explicit that it varies from person to
person; [`lin2014-equal-ratio-hrv`](#lin2014-equal-ratio-hrv) found 5.5 outperformed 6. A single
shipped number cannot be right for everyone, and finding a person's own resonance frequency takes an
assessment protocol and a sensor, neither of which exhale has. The default is a reasonable starting
point, not a personalised one.

Anyone who already has exhale installed keeps whatever they had: the timing fields carry no
`#[serde(default)]`, so an existing `settings.toml` is untouched and the change reaches only fresh
installs and Reset to Defaults. The previous default, `5` / `0` / `10` / `0` at 4 a minute, is still
offered as a one-click preset. Nobody has measured 4 a minute in this corpus, which is why it is no
longer what a new user gets without asking.

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
  breathing, cyclic sighing and cyclic hyperventilation over a month against a
  mindfulness-meditation control. Cyclic sighing, the exhale-emphasising arm, separated from the
  control on positive affect; box breathing did not. The box arm had 21 people, the arms were not
  tested against each other, and their daily gains were 1.84 and 1.89 points, so this is a
  difference in reaching significance rather than a demonstrated gap.

The same evidence applies to 4-7-8, sometimes attributed to Andrew Weil: it lost in Marchant, and at
4+7+8 = 19 s it is 3.16 breaths per minute, slower still.

The pattern with the best direct support is `5` / `0` / `5` / `0`: 6 breaths per minute, no holds, a
10-second cycle. It sits inside the tested band, it is the condition that won in Marchant, and
holding the breath is the harder part for a beginner rather than the slow part. It is what exhale
defaults to. Box breathing remains a reasonable thing to want, and exhale offers it as a preset. It
is simply not the pattern the evidence points at.

### 4. Longer exhale: contested on the heart, thin on how people feel

On HRV, five results, and they do not line up:

| Source | n | Design | Result on ratio |
|---|---|---|---|
| [`bae2021-exhalation-inhalation-ratio`](#bae2021-exhalation-inhalation-ratio) | 28 | 2:1 vs 1:1 cue at spontaneous rate | Longer exhale raised RMSSD and HF-HRV |
| [`vandiest2014-ie-ratio-relaxation`](#vandiest2014-ie-ratio-relaxation) | 30 | i/e 0.42 vs 2.33, at 6 and 12 bpm | Longer exhale raised HF-HRV, but only at the slow rate |
| [`lin2014-equal-ratio-hrv`](#lin2014-equal-ratio-hrv) | 47 | 5:5 vs 4:6 at 5.5 and 6 bpm | **Equal** ratio won on SDNN and LF |
| [`laborde2021-ie-ratio-pauses`](#laborde2021-ie-ratio-pauses) | 64 | i/e 0.8, 1.0, 1.2 at 6 bpm, with and without 0.4 s pauses | Longer exhale raised RMSSD; pauses changed nothing |
| [`meehan2024-longer-exhalations`](#meehan2024-longer-exhalations) | 26 + 16 replication | 1:1 vs 1:2 at 6 bpm | No difference, in the original *and* the replication |

Three for, one against, one null inside this corpus.
[`meehan2024-longer-exhalations`](#meehan2024-longer-exhalations)'s introduction tallies the older
literature as three further nulls and one result favouring the longer inhale. No mechanism claim
survives that split, which is why exhale does not make one.

On how people reported feeling, the picture is not cleaner. Three studies here measured subjective
state across ratios. [`vandiest2014-ie-ratio-relaxation`](#vandiest2014-ie-ratio-relaxation), n = 30,
found more relaxation, stress reduction, mindfulness and positive energy with the longer exhale, and
slowing the rate alone moved only one of those four.
[`lin2014-equal-ratio-hrv`](#lin2014-equal-ratio-hrv), n = 47, found every slow pattern raised
relaxation over baseline, with no ratio-specific advantage.
[`marchant2025-square-478-six`](#marchant2025-square-478-six), n = 84 and the largest of the three,
found no meaningful mood change in any condition, including its two 6-per-minute ratios.
[`balban2023-cyclic-sighing`](#balban2023-cyclic-sighing) points toward exhale emphasis over a
month, but its cyclic-sighing arm also adds a double inhale, so the ratio cannot be isolated.

So a longer exhale is offered as a preference. One study of thirty people found it felt better, a
larger one found no difference, and rate does most of the work either way: every slow pattern in
[`lin2014-equal-ratio-hrv`](#lin2014-equal-ratio-hrv) beat baseline on relaxation.

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
([`satyananda1999-apmb`](#satyananda1999-apmb)), but **no study in this corpus examines a
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

### 11. Both hold sliders default to 0; brief pauses are neutral and longer holds are untested

`post_inhale_hold_duration` and `post_exhale_hold_duration` both default to 0.
[`laborde2021-ie-ratio-pauses`](#laborde2021-ie-ratio-pauses) is the study that manipulates
respiratory pauses directly: at 6 cycles per minute, adding 0.4 s pauses after inhalation and after
exhalation did not change RMSSD. [`little2025-a52-breath-method`](#little2025-a52-breath-method)
argues for a 2 s post-exhale hold, and [`marchant2025-square-478-six`](#marchant2025-square-478-six)
found the two hold-heavy patterns it tested underperformed a no-hold 6 bpm pace. Brief pauses are
neutral and longer holds are untested at exhale's rates, so 0 is a defensible default.

### 12. Nobody knows whether anyone keeps using it

exhale has no telemetry, by design and stated in [PRIVACY.md](../PRIVACY.md). The consequence is
that the single biggest determinant of whether a tool like this does anything, namely whether people
keep it running, is unmeasured and unmeasurable here.
[`linardon2020-app-attrition`](#linardon2020-app-attrition) is the reality check: dropout and
non-adherence dominate outcomes for app-delivered interventions even where efficacy trials are
positive. Every effect size in this corpus was measured in a supervised session with a compliant
participant. That is not the same population as someone who installed a menu-bar app in March.

### 13. Adverse-event reporting in this field is thin

[`fincham2023-breathwork-meta`](#fincham2023-breathwork-meta) found that only four of its twelve
primary-outcome trials reported on adverse events at all, none attributing lasting harm to
breathwork. [`laborde2022-vsb-meta`](#laborde2022-vsb-meta) concludes that few adverse effects are expected from
*slow* breathing specifically, which is the mode exhale is built around. exhale's sliders can also
be set to fast, hold-heavy patterns that leave that evidence base entirely; those belong to the
high-ventilation literature ([`fincham2024-high-ventilation-rct`](#fincham2024-high-ventilation-rct)),
where transient tetany, light-headedness and distress are documented. This is the basis for the
README's advice to take breaks if intense feelings arise.

### 14. The tradition sources are lineage, not evidence, and are tiered accordingly

exhale's four-phase structure, inhale / retention / exhale / retention, is pranayama. It did not come
from psychophysiology, and the corpus says so: [`satyananda1999-apmb`](#satyananda1999-apmb) and
[`muktibodhananda1998-hatha-yoga-pradipika`](#muktibodhananda1998-hatha-yoga-pradipika) are carried
at tier **E**, meaning they may be cited for where a practice came from and never for whether it
works.

This is a deliberate inclusion rather than an endorsement, for two reasons. Retrofitting a 2020s HRV
citation onto an instruction that is centuries older would be revisionist about the app's actual
design history. And the "longer exhale" idea specifically entered modern breathing apps through this
tradition, not through a laboratory, which is worth knowing when weighing how much of the supporting
literature was designed to test a pre-existing belief rather than to discover something.

Both entries are catalogue records only: checked against Open Library, contents not consulted, and
no claim in this repository rests on them. The Satyananda entry cites the 1999 third revised edition
its ISBN resolves to; a 2008 fourth revised edition exists under the same imprint.

## What would close the biggest gaps

In rough order of value per unit effort:

1. Read the four entries cited from their abstract only, and the full text of
   [`laborde2021-ie-ratio-pauses`](#laborde2021-ie-ratio-pauses), whose effect sizes would sharpen
   gaps 4 and 11.
2. Nobody has tested a slow visual pacer on an already-hypocapnic screen worker (gap 5). That is a
   real, publishable question that exhale is unusually well placed to ask.
3. Settle whether graded extension does anything, which would put a floor under gap 6. No study in
   this corpus varies the pace *within* a session, so the question is open in both directions.
4. Resonance frequency is individual (gap 2) and exhale ships one number for everyone. Whether a
   sensorless app can help someone find their own, by any method better than trying a few and
   noticing, is unresolved and would matter more than the default ever will.
