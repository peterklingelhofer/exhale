# Citation corpus

42 sources: 40 Crossref-verified, 2 verified against Open Library. 5 are cited from their abstract only and say so, and 2 are not peer-reviewed and are tiered E so they can back lineage but never a claim.

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

### Counts

| Verification | n |
|---|---|
| crossref-verified | 40 |
| openlibrary-verified | 2 |
| **total** | **42** |

| Access level | n |
|---|---|
| open-access | 17 |
| paywalled | 25 |
| **total** | **42** |

| Evidence tier | n |
|---|---|
| A | 7 |
| B | 7 |
| C | 16 |
| D | 9 |
| E | 2 |
| null (not a study) | 1 |
| **total** | **42** |

---

## Why a breathing reminder next to a screen

11 sources.

#### `albulescu2022-micro-breaks`

Albulescu, Patricia; Macsinga, Irina; Rusu, Andrei; Sulea, Coralia; Bodnaru, Alexandra; Tulbure, Bogdan Tudor. (2022). *"Give me a break!" A systematic review and meta-analysis on the efficacy of micro-breaks for increasing well-being and performance*. PLOS ONE 17(8): e0272460

- DOI: [10.1371/journal.pone.0272460](https://doi.org/10.1371/journal.pone.0272460)
- Verification: crossref-verified | Access: open-access | evidence tier **A**
- Backs:
  - short breaks of ten minutes or less from a work task reduce fatigue and increase vigor
  - an interruption long enough to run one breath cycle is a plausible unit of recovery
- Caveat: 22 studies, 19 manuscripts. The performance effect was smaller and less consistent than the well-being effect, and the authors report it depended on task type. None of the included studies used a breathing exercise as the break activity, so this supports the general shape of exhale's reminder timer, not its specific content.

#### `deniz2024-forward-head-lung-volumes`

Deniz, Yasemin; Ertekın, Damla; Cokar, Dılek. (2024). *The effect of forward head posture on dynamic lung volumes in young adults: a systematic review*. Bulletin of Faculty of Physical Therapy 29(1): 15

- DOI: [10.1186/s43161-024-00186-7](https://doi.org/10.1186/s43161-024-00186-7)
- Verification: crossref-verified | Access: open-access | evidence tier **B**
- Backs:
  - across four comparison studies totalling 115 participants, forward head posture was associated with FVC reductions of 0.25 to 0.81 L and FEV1 reductions of 0.16 to 0.93 L
  - craniovertebral angle correlates positively with dynamic pulmonary volumes
- Caveat: Systematic review without meta-analysis; the authors phrase the conclusion as FHP 'can potentially cause' pulmonary abnormalities. Searched ResearchGate alongside PubMed and Google Scholar, which is an unusual database choice. This is the posture half of the screen-breathing argument: it is about head position, not about screens, and the link to screens comes from jung2016-smartphone-posture-respiration.

#### `grassmann2016-cognitive-load-respiration`

Grassmann, Mariel; Vlemincx, Elke; von Leupoldt, Andreas; Mittelstädt, Justin M.; Van den Bergh, Omer. (2016). *Respiratory Changes in Response to Cognitive Load: A Systematic Review*. Neural Plasticity 2016: 8146809

- DOI: [10.1155/2016/8146809](https://doi.org/10.1155/2016/8146809)
- Verification: crossref-verified | Access: open-access | evidence tier **A**
- Backs:
  - mentally demanding work is reliably marked by faster breathing, with medium to large effects
  - respiratory amplitude stays roughly stable under cognitive load rather than shrinking
  - cognitive load lowers end-tidal CO2, meaning ventilation runs ahead of metabolic need
- Caveat: 54 experiments across 53 articles. Careful with this one: it establishes that cognitive load raises rate while leaving amplitude roughly stable, so it does NOT support 'shallow' if shallow means less air moved. Minute ventilation goes up. For the diaphragmatic-to-thoracic shift that 'shallow' actually names, cite schleifer2002-hyperventilation-job-stress; for the same effect measured at a real keyboard, cite schleifer1994-vdt-petco2 and schleifer2008-emg-gaps-computer-work. This entry is the general cognitive-load backdrop.

#### `johnson2023-20-20-20`

Johnson, Sophia; Rosenfield, Mark. (2023). *20-20-20 Rule: Are These Numbers Justified?* Optometry and Vision Science 100(1): 52-56

- DOI: [10.1097/OPX.0000000000001971](https://doi.org/10.1097/OPX.0000000000001971)
- Verification: crossref-verified | Access: paywalled | evidence tier **C**
- Backs:
  - the widely repeated 20-20-20 rule has little peer-reviewed support
  - scheduled 20-second breaks every 5, 10 or 20 minutes produced no significant effect on ocular symptoms, reading speed or task accuracy
- Caveat: n = 30, 40-minute tablet reading task, four break schedules. Symptoms rose significantly in all four conditions including the most frequent break schedule. Included here as a caution against exhale's own genre: a periodic on-screen nudge is not self-evidently effective just because it is popular and plausible.

#### `jung2016-smartphone-posture-respiration`

Jung, Sang In; Lee, Na Kyung; Kang, Kyung Woo; Kim, Kyoung; Lee, Do Youn. (2016). *The effect of smartphone usage time on posture and respiratory function*. Journal of Physical Therapy Science 28(1): 186-189

- DOI: [10.1589/jpts.28.186](https://doi.org/10.1589/jpts.28.186)
- Verification: crossref-verified | Access: open-access | evidence tier **C**
- Backs:
  - people using smartphones more than four hours a day had significantly worse craniovertebral angle, worse scapular index and lower peak expiratory flow than people using them less than four hours a day
- Caveat: n = 50, cross-sectional, two groups split on self-reported usage. Correlational: heavy users may differ in many ways besides screen time, and peak expiratory flow was the only respiratory measure that separated (FVC and FEV1 did not). Crossref carries no license; access level taken from J Phys Ther Sci being fully open access.

#### `rosenfield2011-computer-vision-syndrome`

Rosenfield, Mark. (2011). *Computer vision syndrome: a review of ocular causes and potential treatments*. Ophthalmic and Physiological Optics 31(5): 502-515

- DOI: [10.1111/j.1475-1313.2011.00834.x](https://doi.org/10.1111/j.1475-1313.2011.00834.x)
- Verification: crossref-verified | Access: paywalled | evidence tier **B**
- Backs:
  - blink rate falls substantially during display work relative to other tasks
  - reduced and incomplete blinking is a principal mechanism of computer vision syndrome
- Caveat: Narrative review of the ocular literature. Load-bearing for the blink half of exhale's opening claim only. exhale does nothing about blinking; see the gaps ledger.

#### `schleifer1994-vdt-petco2`

Schleifer, Lawrence M.; Ley, Ronald. (1994). *End-tidal PCO2 as an index of psychophysiological activity during VDT data-entry work and relaxation*. Ergonomics 37(2): 245-254

- DOI: [10.1080/00140139408963642](https://doi.org/10.1080/00140139408963642)
- Verification: crossref-verified | Access: paywalled | evidence tier **C**
- Backs:
  - during computer data-entry work, end-tidal CO2 was significantly lower and respiration frequency significantly higher than during either baseline relaxation or progressive muscle relaxation
  - breathing changes measurably at a screen, and end-tidal CO2 discriminates the state
- Caveat: n = 11 data-entry operators monitored continuously across three consecutive six-hour work days. Small sample, but this is real screen work over real working days, not a lab proxy. This entry retracts an earlier claim in this repo that no peer-reviewed study had measured respiration during ordinary screen use. It had; this is it, from 1994.

#### `schleifer2002-hyperventilation-job-stress`

Schleifer, Lawrence M.; Ley, Ronald; Spalding, Thomas W.. (2002). *A hyperventilation theory of job stress and musculoskeletal disorders*. American Journal of Industrial Medicine 41(5): 420-432

- DOI: [10.1002/ajim.10061](https://doi.org/10.1002/ajim.10061)
- Verification: crossref-verified | Access: paywalled | evidence tier **D**
- Backs:
  - hyperventilation is often characterised by a shift from a diaphragmatic to a thoracic breathing pattern
  - thoracic breathing recruits sternocleidomastoid, scalene and trapezius muscles, imposing biomechanical stress on the neck and shoulder region
  - breathing training and rest breaks are a rationale-backed response to this pattern at work
- Caveat: Theory paper, not an experiment, hence tier D. It is nonetheless the closest thing in the peer-reviewed literature to the folk claim about 'shallow' breathing at a screen: the diaphragmatic-to-thoracic shift IS what people mean by shallow. Cite it for the pattern, never for a measured tidal volume, and note it theorises the shift rather than demonstrating it in screen users.

#### `schleifer2008-emg-gaps-computer-work`

Schleifer, Lawrence M.; Spalding, Thomas W.; Kerick, Scott E.; Cram, Jeffrey R.; Ley, Ronald; Hatfield, Bradley D.. (2008). *Mental stress and trapezius muscle activation under psychomotor challenge: A focus on EMG gaps during computer work*. Psychophysiology 45(3): 356-365

- DOI: [10.1111/j.1469-8986.2008.00645.x](https://doi.org/10.1111/j.1469-8986.2008.00645.x)
- Verification: crossref-verified | Access: paywalled | evidence tier **C**
- Backs:
  - end-tidal CO2 was lower under high mental workload than low during computer data entry, replicating the over-breathing finding in a second sample
  - lower end-tidal CO2 tracked reduced trapezius EMG-gap frequency, suggesting over-breathing mediates muscle tension at the keyboard
- Caveat: n = 23. The second independent measurement by this group of the same effect, fourteen years after schleifer1994-vdt-petco2. Two small samples pointing the same way is what the screen-breathing claim actually rests on.

#### `sheppard2018-digital-eye-strain`

Sheppard, Amy L.; Wolffsohn, James S.. (2018). *Digital eye strain: prevalence, measurement and amelioration*. BMJ Open Ophthalmology 3(1): e000146

- DOI: [10.1136/bmjophth-2018-000146](https://doi.org/10.1136/bmjophth-2018-000146)
- Verification: crossref-verified | Access: open-access | evidence tier **B**
- Backs:
  - digital eye strain is common among heavy display users
  - reduced blink rate and incomplete blinks during screen work are among its established contributors
- Caveat: Crossref carries no license field for this record; access level is taken from BMJ Open Ophthalmology being a fully open-access title.

#### `tsubota1993-vdt-blink`

Tsubota, Kazuo; Nakamori, Katsu. (1993). *Dry Eyes and Video Display Terminals*. New England Journal of Medicine 328(8): 584

- DOI: [10.1056/NEJM199302253280817](https://doi.org/10.1056/NEJM199302253280817)
- Verification: crossref-verified | Access: paywalled | evidence tier **C**
- Backs:
  - blink rate during display work is far below blink rate at rest
- Caveat: NUMBERS NOT READ 2026-08-28. This is a one-page correspondence item, not a full paper, and it is paywalled to automated fetch. It is the origin of the widely quoted '22 blinks/min at rest, 7 during screen work' figures, which circulate almost entirely by secondary citation. Cite it for the direction of the effect. For a magnitude you can defend, use rosenfield2011-computer-vision-syndrome or sheppard2018-digital-eye-strain instead.

---

## Whether slow paced breathing does anything

7 sources.

#### `chaddha2019-slow-breathing-bp`

Chaddha, Ashish; Modaff, Daniel; Hooper-Lane, Christopher; Feldstein, David A.. (2019). *Device and non-device-guided slow breathing to reduce blood pressure: A systematic review and meta-analysis*. Complementary Therapies in Medicine 45: 179-184

- DOI: [10.1016/j.ctim.2019.03.005](https://doi.org/10.1016/j.ctim.2019.03.005)
- Verification: crossref-verified | Access: paywalled | evidence tier **A**
- Backs:
  - sustained slow breathing programmes lower systolic blood pressure by about 5.6 mmHg and diastolic by about 3.0 mmHg in hypertensive and prehypertensive adults
- Caveat: 17 RCTs. Heterogeneity was high for every analysis, and the authors say so. Inclusion required at least 5 minutes of breathing at 10 breaths/min or slower, on at least 3 days a week, for at least 4 weeks. exhale asks for none of that and measures none of it, so this describes a dose exhale does not deliver. Read alongside vandijk2018-close-the-book.

#### `fincham2023-breathwork-meta`

Fincham, Guy William; Strauss, Clara; Montero-Marin, Jesus; Cavanagh, Kate. (2023). *Effect of breathwork on stress and mental health: A meta-analysis of randomised-controlled trials*. Scientific Reports 13(1): 432

- DOI: [10.1038/s41598-022-27247-y](https://doi.org/10.1038/s41598-022-27247-y)
- Verification: crossref-verified | Access: open-access | evidence tier **A**
- Backs:
  - breathwork lowers self-reported stress against control conditions, g = -0.35 (95% CI -0.55 to -0.14), 12 RCTs, 785 adults
  - comparable small-to-medium effects for anxiety (g = -0.32, k = 20) and depressive symptoms (g = -0.40, k = 18)
- Caveat: Most included studies were rated at moderate risk of bias. Effects are small-to-medium and self-reported. This is the strongest single warrant for the claim that a breathing practice does something, and it is still a g of about a third of a standard deviation.

#### `laborde2022-vsb-meta`

Laborde, S.; Allen, M. S.; Borges, U.; Dosseville, F.; Hosang, T. J.; Iskra, M.; Mosley, E.; Salvotti, C.; Spolverato, L.; Zammit, N.; Javelle, F.. (2022). *Effects of voluntary slow breathing on heart rate and heart rate variability: A systematic review and a meta-analysis*. Neuroscience & Biobehavioral Reviews 138: 104711

- DOI: [10.1016/j.neubiorev.2022.104711](https://doi.org/10.1016/j.neubiorev.2022.104711)
- Verification: crossref-verified | Access: paywalled | evidence tier **A**
- Backs:
  - voluntary slow breathing raises vagally-mediated HRV during the session, immediately after a single session, and after a multi-session intervention
  - few adverse effects are expected from slow breathing practice
- Caveat: 223 studies from 1842 screened abstracts (172 during, 16 immediately-after, 49 after-intervention). This is the central warrant for exhale's whole premise. Note what it establishes: an effect on a cardiac index of parasympathetic activity, not an effect on how anyone feels or works.

#### `little2025-a52-breath-method`

Little, Abbie L.. (2025). *The A52 Breath Method: A Narrative Review of Breathwork for Mental Health and Stress Resilience*. Stress and Health 41(4): e70098

- DOI: [10.1002/smi.70098](https://doi.org/10.1002/smi.70098)
- Verification: crossref-verified | Access: open-access | evidence tier **D**
- Backs:
  - a 5 s inhale, 5 s exhale, 2 s post-exhale hold at five breaths per minute is a protocol a recent review argues is representative of the effective literature
  - 23 of 30 reviewed studies reported significant HRV improvement; 10 reported anxiety reduction and 9 reported reduced perceived stress
  - benefits appear larger in people with elevated baseline distress
- Caveat: Narrative review, single author, 465 abstracts screened and 30 full texts analysed, with no meta-analysis and no risk-of-bias assessment. It proposes the protocol it reviews, which is a conflict of framing worth naming. Its A52 shape maps onto exhale's four sliders as 5 / 0 / 5 / 2, which is 5 breaths per minute and inside the tested band, unlike exhale's shipped default. Note that marchant2025-square-478-six found no-hold 6 bpm outperformed both hold-bearing patterns it tested, so the 2 s retention here is the least supported part of the protocol.

#### `russo2017-slow-breathing-physiology`

Russo, Marc A.; Santarelli, Danielle M.; O'Rourke, Dean. (2017). *The physiological effects of slow breathing in the healthy human*. Breathe 13(4): 298-309

- DOI: [10.1183/20734735.009817](https://doi.org/10.1183/20734735.009817)
- Verification: crossref-verified | Access: open-access | evidence tier **D**
- Backs:
  - slow breathing improves ventilation efficiency and alters cardiorespiratory coupling, respiratory sinus arrhythmia and sympathovagal balance
- Caveat: Narrative review. The authors close by calling explicitly for further research, and describe the health claims as potential rather than demonstrated. Use it to explain a mechanism, not to assert an outcome.

#### `vandijk2018-close-the-book`

van Dijk, Peter R.; van Hateren, Kornelis J. J.; Kleefstra, Nanne; Landman, Gijs W. D.. (2018). *It is time to close the book on device-guided slow breathing*. Blood Pressure 27(3): 181-182

- DOI: [10.1080/08037051.2018.1435260](https://doi.org/10.1080/08037051.2018.1435260)
- Verification: crossref-verified | Access: paywalled | no evidence tier (not a study)
- Backs:
  - a body of specialist opinion holds that the blood-pressure case for device-guided slow breathing is weak and should be considered closed
- Caveat: Editorial, two pages, not a study, hence no evidence tier. Carried deliberately: it is the strongest published dissent against the cardiovascular claims exhale sits next to, and a corpus that omitted it would be advocacy rather than provenance.

#### `zaccaro2018-slow-breathing-review`

Zaccaro, Andrea; Piarulli, Andrea; Laurino, Marco; Garbella, Erika; Menicucci, Danilo; Neri, Bruno; Gemignani, Angelo. (2018). *How Breath-Control Can Change Your Life: A Systematic Review on Psycho-Physiological Correlates of Slow Breathing*. Frontiers in Human Neuroscience 12: 353

- DOI: [10.3389/fnhum.2018.00353](https://doi.org/10.3389/fnhum.2018.00353)
- Verification: crossref-verified | Access: open-access | evidence tier **B**
- Backs:
  - slow breathing, conventionally defined as under 10 breaths per minute, is associated with increased HRV and shifts in central and autonomic measures
  - reported psychological correlates include increased comfort and relaxation and reduced anxiety and arousal
- Caveat: Systematic review without meta-analysis. The included studies vary widely in technique, rate and duration, so the pooled picture is directional rather than dose-specific.

---

## What the numbers should be

10 sources.

#### `bae2021-exhalation-inhalation-ratio`

Bae, Dalbyeol; Matthews, Jacob J. L.; Chen, J. Jean; Mah, Linda. (2021). *Increased exhalation to inhalation ratio during breathing enhances high-frequency heart rate variability in healthy adults*. Psychophysiology 58(11): e13905

- DOI: [10.1111/psyp.13905](https://doi.org/10.1111/psyp.13905)
- Verification: crossref-verified | Access: paywalled | evidence tier **C**
- Backs:
  - a 2:1 exhale-to-inhale cue raised RMSSD and HF-HRV relative to a 1:1 cue at the participant's own breathing rate
  - the HF-HRV elevation persisted about four minutes after the 2:1 block ended
- Caveat: n = 28 (16 young, 12 older). Note the manipulation check: the achieved ratios were 1.08 and 1.33, nowhere near the instructed 1:1 and 2:1, and pacing was at each participant's spontaneous rate rather than in the resonance range. One of four studies that disagree about the ratio; see vandiest2014-ie-ratio-relaxation (also positive), lin2014-equal-ratio-hrv (favours the equal ratio) and meehan2024-longer-exhalations (null). Gap 4 in the ledger tabulates all four.

#### `balban2023-cyclic-sighing`

Balban, Melis Yilmaz; Neri, Eric; Kogon, Manuela M.; Weed, Lara; Nouriani, Bita; Jo, Booil; Holl, Gary; Zeitzer, Jamie M.; Spiegel, David; Huberman, Andrew D.. (2023). *Brief structured respiration practices enhance mood and reduce physiological arousal*. Cell Reports Medicine 4(1): 100895

- DOI: [10.1016/j.xcrm.2022.100895](https://doi.org/10.1016/j.xcrm.2022.100895)
- Verification: crossref-verified | Access: open-access | evidence tier **A**
- Backs:
  - five minutes a day of exhale-emphasising cyclic sighing improved mood and lowered respiratory rate more than an equal period of mindfulness meditation over one month
  - box breathing, equal inhale / hold / exhale, was tested head-to-head and was not the best-performing arm
- Caveat: Remote randomised controlled study, pre-registered as NCT05304000. The primary comparator is mindfulness meditation, not a sham, so the arms differ in more than breath ratio. The mood difference reached p < 0.05 in a mixed-effects model; this is a real but modest separation. Directly relevant to exhale twice over: it is the best evidence that a longer exhale is the right emphasis, and it is the reason the README should stop presenting box breathing as equivalent.

#### `laborde2021-ie-ratio-pauses`

Laborde, Sylvain; Iskra, Maša; Zammit, Nina; Borges, Uirassu; You, Min; Sevoz-Couche, Caroline; Dosseville, Fabrice. (2021). *Slow-Paced Breathing: Influence of Inhalation/Exhalation Ratio and of Respiratory Pauses on Cardiac Vagal Activity*. Sustainability 13(14): 7775

- DOI: [10.3390/su13147775](https://doi.org/10.3390/su13147775)
- Verification: crossref-verified | Access: open-access | evidence tier **C**
- Backs:
  - inhalation/exhalation ratio and the presence of respiratory pauses were manipulated directly against cardiac vagal activity as the outcome
- Caveat: NUMBERS NOT READ 2026-08-28. Carried for the design question it asks, which is exactly the question exhale's four sliders pose. It is the fifth study bearing on the ratio disagreement tabulated in gap 4, and the only one that also manipulates respiratory pauses, which makes it the highest-value full text still unread in this corpus: it bears on both gap 4 and gap 11. Read it before quoting any effect from it.

#### `laborde2021-spb-6cpm-biofeedback`

Laborde, Sylvain; Allen, Mark S.; Borges, Uirassu; Iskra, Maša; Zammit, Nina; You, Min; Hosang, Thomas; Mosley, Emma; Dosseville, Fabrice. (2021). *Psychophysiological effects of slow-paced breathing at six cycles per minute with or without heart rate variability biofeedback*. Psychophysiology 59(1): e13952

- DOI: [10.1111/psyp.13952](https://doi.org/10.1111/psyp.13952)
- Verification: crossref-verified | Access: open-access | evidence tier **C**
- Backs:
  - slow-paced breathing at six cycles per minute raised RMSSD relative to control in every condition tested
  - adding heart-rate-variability biofeedback on top of the six-per-minute pace did not add a further benefit
- Caveat: Crossref records this as issued 2021, appearing in the January 2022 issue (59:1). Together with tabor2022-guided-breathing-design this is why exhale needs no sensor: the pace is doing the work, not the feedback loop.

#### `lin2014-equal-ratio-hrv`

Lin, I. M.; Tai, L. Y.; Fan, S. Y.. (2014). *Breathing at a rate of 5.5 breaths per minute with equal inhalation-to-exhalation ratio increases heart rate variability*. International Journal of Psychophysiology 91(3): 206-211

- DOI: [10.1016/j.ijpsycho.2013.12.006](https://doi.org/10.1016/j.ijpsycho.2013.12.006)
- Verification: crossref-verified | Access: paywalled | evidence tier **C**
- Backs:
  - 5.5 breaths per minute with an equal 5:5 inhale-to-exhale ratio produced higher SDNN and LF power than 6 bpm or than a 4:6 ratio
  - all four slow-breathing patterns increased self-reported relaxation relative to spontaneous breathing
- Caveat: n = 47, Latin-square counterbalanced. The clearest published result AGAINST a longer exhale on HRV grounds, and it is why exhale's README no longer asserts the 1:2 mechanism. Note the second claim carefully: every pattern tested increased relaxation, so the ratio argument is about which slow pattern is best, not about whether slow breathing works.

#### `marchant2025-square-478-six`

Marchant, Joshua; Khazan, Inna; Cressman, Mikel; Steffen, Patrick. (2025). *Comparing the Effects of Square, 4-7-8, and 6 Breaths-per-Minute Breathing Conditions on Heart Rate Variability, CO2 Levels, and Mood*. Applied Psychophysiology and Biofeedback 50(2): 261-276

- DOI: [10.1007/s10484-025-09688-z](https://doi.org/10.1007/s10484-025-09688-z)
- Verification: crossref-verified | Access: paywalled | evidence tier **B**
- Backs:
  - breathing at 6 breaths per minute raised HRV more than either square (box) breathing or 4-7-8 breathing, with small to medium effects
  - square and 4-7-8 breathing are popularly promoted but have little empirical support
  - none of the four conditions produced meaningful changes in blood pressure or mood
  - breathing at 6 breaths per minute unexpectedly produced mild over-breathing
- Caveat: n = 84 college students, within-subjects, four conditions: square, 4-7-8, 6 bpm at 4:6, and 6 bpm at 5:5. Single session, so it speaks to acute effect only. This is the head-to-head that answers exhale's default-preset question directly, and it answers against box breathing and against 4-7-8. The over-breathing finding is the uncomfortable one: see gap 5 in the ledger.

#### `meehan2024-longer-exhalations`

Meehan, Zachary M.; Shaffer, Fred. (2024). *Do Longer Exhalations Increase HRV During Slow-Paced Breathing?* Applied Psychophysiology and Biofeedback 49(3): 407-417

- DOI: [10.1007/s10484-024-09637-2](https://doi.org/10.1007/s10484-024-09637-2)
- Verification: crossref-verified | Access: open-access | evidence tier **B**
- Backs:
  - at 6 breaths per minute, a 1:2 inhale-to-exhale ratio produced no HRV advantage over 1:1 in either an original experiment or its replication
  - the finding held across time-domain, frequency-domain and nonlinear HRV metrics
- Caveat: Original n = 26, replication n = 16; both undergraduate samples, both within-subjects with manipulation checks. Small, but it is the only entry in this corpus that ran its own replication. Its scope condition matters: it holds rate fixed inside the resonance range, so it does not rule out a ratio effect at a person's spontaneous rate, which is what bae2021-exhalation-inhalation-ratio measured. One of four disagreeing studies; see also vandiest2014-ie-ratio-relaxation and lin2014-equal-ratio-hrv. Critically, all four measured HRV; none of them measured how participants felt except vandiest2014-ie-ratio-relaxation, which is why exhale's recommendation now rests on the subjective outcome rather than on this dispute.

#### `sevozcouche2022-coherence-resonance`

Sevoz-Couche, Caroline; Laborde, Sylvain. (2022). *Heart rate variability and slow-paced breathing: when coherence meets resonance*. Neuroscience & Biobehavioral Reviews 135: 104576

- DOI: [10.1016/j.neubiorev.2022.104576](https://doi.org/10.1016/j.neubiorev.2022.104576)
- Verification: crossref-verified | Access: paywalled | evidence tier **D**
- Backs:
  - coherence and resonance are distinct phenomena that are frequently conflated in the slow-breathing literature
- Caveat: Narrative review. Carried as a terminology guard: much of the popular writing exhale competes with uses 'coherence' loosely, and this is the paper to check before adopting that vocabulary in the app or the README.

#### `vandiest2014-ie-ratio-relaxation`

Van Diest, Ilse; Verstappen, Karen; Aubert, André E.; Widjaja, Devy; Vansteenwegen, Debora; Vlemincx, Elke. (2014). *Inhalation/Exhalation Ratio Modulates the Effect of Slow Breathing on Heart Rate Variability and Relaxation*. Applied Psychophysiology and Biofeedback 39(3-4): 171-180

- DOI: [10.1007/s10484-014-9253-x](https://doi.org/10.1007/s10484-014-9253-x)
- Verification: crossref-verified | Access: paywalled | evidence tier **C**
- Backs:
  - participants reported more relaxation, more stress reduction, more mindfulness and more positive energy breathing with a low inhale/exhale ratio (longer exhale) than a high one
  - a low inhale/exhale ratio also produced more HF-HRV power, but only in the slow breathing condition
  - slowing the rate on its own improved only self-reported positive energy
- Caveat: n = 30, four patterns crossing rate (6 or 12 breaths/min) with i/e ratio (0.42 or 2.33). This is the single most important entry for exhale's longer-exhale recommendation, and for a reason easy to miss: it is the only study in this corpus that measured how people FELT across ratios, and on that outcome the longer exhale clearly won. The HRV studies that disagree about ratio were measuring something else.

#### `you2023-respiratory-frequency`

You, Min; Laborde, Sylvain; Ackermann, Stefan; Borges, Uirassu; Dosseville, Fabrice; Mosley, Emma. (2023). *Influence of Respiratory Frequency of Slow-Paced Breathing on Vagally-Mediated Heart Rate Variability*. Applied Psychophysiology and Biofeedback 49(1): 133-143

- DOI: [10.1007/s10484-023-09605-2](https://doi.org/10.1007/s10484-023-09605-2)
- Verification: crossref-verified | Access: paywalled | evidence tier **C**
- Backs:
  - five minutes of slow-paced breathing at 5, 5.5, 6, 6.5 and 7 cycles per minute all raised cardiac vagal activity above spontaneous breathing
  - LF-HRV discriminated between the tested frequencies more sensitively than RMSSD
  - the band actually tested and supported is 5 to 7 cycles per minute
- Caveat: Crossref records this as issued 2023 (online 8 December); it appears in the March 2024 issue, 49(1), which is how it is usually cited. The citekey follows the Crossref issued year, as everywhere else in this corpus. n = 75, all athletes aged 19-31, single lab session. Generalisation to a desk worker is an assumption, not a finding. This is the source that puts a number on exhale's default: 5 s in and 10 s out is 4.0 cycles per minute, below the bottom of the band this study tested.

---

## Where the practice came from

2 sources.

#### `muktibodhananda1998-hatha-yoga-pradipika`

Muktibodhananda, Swami. (1998). *Hatha Yoga Pradipika*. Munger, Bihar, India: Bihar School of Yoga

- ISBN: 9788185787381 | [Open Library record](https://openlibrary.org/search?q=hatha+yoga+pradipika+muktibodhananda)
- Verification: openlibrary-verified | Access: paywalled | evidence tier **E**
- Backs:
  - the classical hatha text and commentary in which timed inhale, retention and exhale ratios are set out is the historical origin of the ratio instructions modern breathing apps repeat
- Caveat: NOT READ 2026-08-28. Bibliographic record checked against Open Library (Bihar School of Yoga, 1998 record). Not peer reviewed, and a commentary on a fifteenth-century text rather than a primary source in any modern sense. Carried because the 'longer exhale' instruction exhale ships did not come from a laboratory: it came from here, centuries before anyone measured HRV. Naming that is more honest than retrofitting a citation to psychophysiology. It must never back a physiological claim.

#### `satyananda2008-apmb`

Saraswati, Swami Satyananda. (2008). *Asana Pranayama Mudra Bandha*. Munger, Bihar, India: Yoga Publications Trust 553 pp

- ISBN: 9788186336144 | [Open Library record](https://openlibrary.org/books/OL22138410M/Asana_pranayama_mudra_bandha)
- Verification: openlibrary-verified | Access: paywalled | evidence tier **E**
- Backs:
  - the systematic pranayama tradition from which exhale's controllable inhale / retention / exhale / retention structure descends is documented in a standard modern reference manual
- Caveat: NOT READ 2026-08-28. Bibliographic record checked against Open Library, which returns the Yoga Publications Trust edition at 553 pages under this ISBN but dates its record 1999, while the printing in question is described as the 2008 Fourth Revised Edition revised under Swami Niranjanananda Saraswati; the edition history of this title is genuinely tangled and this entry does not resolve it. Not peer reviewed. Cited ONLY for lineage: it is where exhale's four-phase structure comes from, and pretending the design was derived from 2020s psychophysiology would be revisionist. It must never back a physiological claim.

---

## Whether an on-screen visual pacer works

3 sources.

#### `moraveji2011-peripheral-paced-respiration`

Moraveji, Neema; Olson, Ben; Nguyen, Truc; Saadat, Mahmoud; Khalighi, Yaser; Pea, Roy; Heer, Jeffrey. (2011). *Peripheral paced respiration: influencing user physiology during information work*. Proceedings of the 24th Annual ACM Symposium on User Interface Software and Technology (UIST '11) 423-428

- DOI: [10.1145/2047196.2047250](https://doi.org/10.1145/2047196.2047250)
- Verification: crossref-verified | Access: paywalled | evidence tier **C**
- Backs:
  - a translucent animated bar spanning the screen, running in the periphery of attention during normal information work, significantly lowered participants' breathing rate
  - peripheral pacing does not require the user's full attention to change breathing
- Caveat: This is the closest published analogue to exhale that exists, and its limitation is the important part: the reduction occurred while the pacing feedback was active and did not persist as a lasting change in respiratory pattern. An always-on overlay should be understood as an effect that lasts as long as it is on. A free author copy is posted at vis.stanford.edu; the ACM Digital Library version is paywalled.

#### `tabor2022-guided-breathing-design`

Tabor, Aaron; Bateman, Scott; Scheme, Erik J.; schraefel, m.c.. (2022). *Comparing heart rate variability biofeedback and simple paced breathing to inform the design of guided breathing technologies*. Frontiers in Computer Science 4: 926649

- DOI: [10.3389/fcomp.2022.926649](https://doi.org/10.3389/fcomp.2022.926649)
- Verification: crossref-verified | Access: open-access | evidence tier **C**
- Backs:
  - an expanding and contracting circle pacing 6 breaths per minute produced HRV amplitude gains statistically indistinguishable from sensor-driven HRV biofeedback
  - both conditions took roughly two minutes for effects to appear
  - paced breathing needs no sensor, no real-time processing and no sustained attention, which suits it to use as a secondary task
- Caveat: Between-subjects, n = 28 (14 per group), single 10-minute session. This is the strongest published warrant for exhale's specific design choices: an expanding/contracting shape, no hardware, no account, watchable while doing something else.

#### `wongsuphasawat2012-cant-force-calm`

Wongsuphasawat, Kanit; Gamburg, Alex; Moraveji, Neema. (2012). *You can't force calm: designing and evaluating respiratory regulating interfaces for calming technology*. Adjunct Proceedings of the 25th Annual ACM Symposium on User Interface Software and Technology (UIST '12 Adjunct) 69-70

- DOI: [10.1145/2380296.2380326](https://doi.org/10.1145/2380296.2380326)
- Verification: crossref-verified | Access: paywalled | evidence tier **C**
- Backs:
  - visual pacing produced more measured respiratory change than auditory pacing, but auditory pacing was rated as subjectively more calming
- Caveat: Two-page adjunct paper, effectively a poster; treat the finding as a design signal, not a result. It is carried anyway because it is the one published comparison that puts exhale's visual-only design at a disadvantage on the outcome most users actually care about, which is whether they feel calmer.

---

## Physiology and neuroscience

7 sources.

#### `lehrer2014-hrv-biofeedback`

Lehrer, Paul M.; Gevirtz, Richard. (2014). *Heart rate variability biofeedback: how and why does it work?* Frontiers in Psychology 5: 756

- DOI: [10.3389/fpsyg.2014.00756](https://doi.org/10.3389/fpsyg.2014.00756)
- Verification: crossref-verified | Access: open-access | evidence tier **D**
- Backs:
  - maximum heart-rate oscillation is usually reached breathing at approximately 0.1 Hz, six breaths per minute
  - refined measurement puts the average resonance frequency nearer 0.09 Hz, about 5.5 breaths per minute, a breath lasting roughly 11 seconds
  - resonance frequency is individual: taller people and men tend to have lower resonance frequencies
  - baroreflex gain increases substantially during HRV biofeedback
- Caveat: Narrative mechanistic review by the technique's principal developers. Read as the mechanism argument, not as independent evidence. The individual-variation point is the honest reason exhale's timing sliders are user-editable at all: there is no single correct number to hardcode.

#### `li2016-sigh-circuit`

Li, Peng; Janczewski, Wiktor A.; Yackle, Kevin; Kam, Kaiwen; Pagliardini, Silvia; Krasnow, Mark A.; Feldman, Jack L.. (2016). *The peptidergic control circuit for sighing*. Nature 530(7590): 293-297

- DOI: [10.1038/nature16964](https://doi.org/10.1038/nature16964)
- Verification: crossref-verified | Access: paywalled | evidence tier **D**
- Backs:
  - sighing is generated by a dedicated peptidergic circuit projecting onto the preBotzinger complex, not as a byproduct of ordinary breathing rhythm
- Caveat: Mouse work. It establishes that the sigh is a distinct, hardwired respiratory behaviour, which is the mechanistic backdrop for the cyclic-sighing result in balban2023-cyclic-sighing. It says nothing about humans deliberately performing sighs, and must not be cited as if it did.

#### `vlemincx2013-sigh-reset-model`

Vlemincx, Elke; Abelson, James L.; Lehrer, Paul M.; Davenport, Paul W.; Van Diest, Ilse; Van den Bergh, Omer. (2013). *Respiratory variability and sighing: A psychophysiological reset model*. Biological Psychology 93(1): 24-32

- DOI: [10.1016/j.biopsycho.2012.12.001](https://doi.org/10.1016/j.biopsycho.2012.12.001)
- Verification: crossref-verified | Access: paywalled | evidence tier **D**
- Backs:
  - sighs act as resetters of respiratory variability: random variability is elevated before a sigh and correlated variability increases after one
- Caveat: Theoretical model paper with supporting data. Carried because it is the honest place to point when someone asks why a deliberately irregular breath might be useful, which is the only literature adjacent to exhale's randomised-timing sliders. It is not evidence that those sliders help; see the gaps ledger.

#### `vlemincx2016-sigh-relief`

Vlemincx, Elke; Van Diest, Ilse; Van den Bergh, Omer. (2016). *A sigh of relief or a sigh to relieve: The psychological and physiological relief effect of deep breaths*. Physiology & Behavior 165: 127-135

- DOI: [10.1016/j.physbeh.2016.07.004](https://doi.org/10.1016/j.physbeh.2016.07.004)
- Verification: crossref-verified | Access: paywalled | evidence tier **C**
- Backs:
  - self-reported relief was higher after an instructed deep breath than before it
  - spontaneous sighs were followed by a gradual fall in muscle tension, most clearly in people high in anxiety sensitivity
- Caveat: NUMBERS NOT READ 2026-08-28; paywalled to automated fetch, findings taken from the abstract. The instructed-breath result is the closest thing in this corpus to evidence that being told to take one deliberate breath, which is precisely what exhale's reminder does, changes how a person feels.

#### `yackle2017-breathing-arousal-neurons`

Yackle, Kevin; Schwarz, Lindsay A.; Kam, Kaiwen; Sorokin, Jordan M.; Huguenard, John R.; Feldman, Jack L.; Luo, Liqun; Krasnow, Mark A.. (2017). *Breathing control center neurons that promote arousal in mice*. Science 355(6332): 1411-1415

- DOI: [10.1126/science.aai7984](https://doi.org/10.1126/science.aai7984)
- Verification: crossref-verified | Access: paywalled | evidence tier **D**
- Backs:
  - a small preBotzinger subpopulation projects to and positively regulates noradrenergic locus coeruleus neurons, giving breathing a direct anatomical route to arousal state
  - ablating those roughly 175 neurons left breathing intact but increased calm behaviours
- Caveat: Mouse work, and the direction is breathing pattern to arousal rather than voluntarily slow breathing to calm. It is the single best answer to 'why would breathing slowly change how I feel at all', and it is still an inference from mice. Do not cite it for a human effect.

#### `yasuma2004-rsa`

Yasuma, Fumihiko; Hayano, Jun-ichiro. (2004). *Respiratory Sinus Arrhythmia: Why Does the Heartbeat Synchronize With Respiratory Rhythm?* Chest 125(2): 683-690

- DOI: [10.1378/chest.125.2.683](https://doi.org/10.1378/chest.125.2.683)
- Verification: crossref-verified | Access: paywalled | evidence tier **D**
- Backs:
  - heart rate rises on inhalation and falls on exhalation, and this respiratory sinus arrhythmia is the coupling that HRV-based breathing claims rest on
- Caveat: Review. This is the physiological fact underneath every 'longer exhale calms you' claim in this corpus, which is why the claim is so intuitive and why meehan2024-longer-exhalations failing to find a ratio effect is worth taking seriously: a real beat-to-beat mechanism does not guarantee a measurable session-level outcome.

#### `zelano2016-nasal-respiration-limbic`

Zelano, Christina; Jiang, Heidi; Zhou, Guangyu; Arora, Nikita; Schuele, Stephan; Rosenow, Joshua; Gottfried, Jay A.. (2016). *Nasal Respiration Entrains Human Limbic Oscillations and Modulates Cognitive Function*. The Journal of Neuroscience 36(49): 12448-12467

- DOI: [10.1523/JNEUROSCI.2586-16.2016](https://doi.org/10.1523/JNEUROSCI.2586-16.2016)
- Verification: crossref-verified | Access: open-access | evidence tier **C**
- Backs:
  - nasal breathing entrains oscillations in human piriform cortex, amygdala and hippocampus, and the effect is specific to the nasal route rather than to breathing as such
- Caveat: Human intracranial recordings in a small epilepsy-surgery cohort plus behavioural experiments. Carried because it is the reason nose-versus-mouth is a real variable and not folklore. exhale gives no nasal-breathing guidance at all, which is a defensible omission for a wordless overlay but should be a conscious one.

---

## Limits, harms and adherence

2 sources.

#### `fincham2024-high-ventilation-rct`

Fincham, Guy W.; Epel, Elissa; Colasanti, Alessandro; Strauss, Clara; Cavanagh, Kate. (2024). *Effects of brief remote high ventilation breathwork with retention on mental health and wellbeing: a randomised placebo-controlled trial*. Scientific Reports 14(1): 16893

- DOI: [10.1038/s41598-024-64254-7](https://doi.org/10.1038/s41598-024-64254-7)
- Verification: crossref-verified | Access: open-access | evidence tier **B**
- Backs:
  - high-ventilation breathwork with retention is a distinct practice from slow-paced breathing and warrants a placebo-controlled evaluation of its own
- Caveat: NUMBERS NOT READ 2026-08-28; carried for the distinction it draws rather than for its effect estimate. It matters to exhale because exhale's sliders can be set to fast, hold-heavy patterns that leave the slow-breathing evidence base entirely. Adverse-event reporting across this field is sparse: reviews note that only a minority of breathwork trials report on adverse events at all, which is the honest basis for the README's advice to stop if intense feelings arise.

#### `linardon2020-app-attrition`

Linardon, Jake; Fuller-Tyszkiewicz, Matthew. (2020). *Attrition and adherence in smartphone-delivered interventions for mental health problems: A systematic and meta-analytic review*. Journal of Consulting and Clinical Psychology 88(1): 1-13

- DOI: [10.1037/ccp0000459](https://doi.org/10.1037/ccp0000459)
- Verification: crossref-verified | Access: paywalled | evidence tier **A**
- Backs:
  - dropout and non-adherence are the dominant practical failure mode of smartphone-delivered mental health interventions, even where efficacy trials are positive
- Caveat: NUMBERS NOT READ 2026-08-28. Carried as the reality check on every other entry in this corpus: an effect measured in a supervised session says little about a tool someone installs and forgets. exhale has no telemetry and therefore no idea whether anyone keeps it running, which is stated plainly in the gaps ledger rather than hidden.

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

### 6. `drift` has no literature behind it at all

exhale defaults `drift` to `1.01`, making each cycle 1% longer than the last. Over 30 cycles that is
a 1.35x stretch, taking a 4.0 breaths/min pace down to roughly 3.0 and further outside any tested
range. No study in this corpus examines a progressively lengthening pace. It is an invented feature.
It may be a pleasant one. It is not evidence-based, and the default is not zero.

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
