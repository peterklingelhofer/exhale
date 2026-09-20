# Citation corpus

49 sources: 46 Crossref-verified, 2 verified against Open Library, 1 verified against PubMed. 45 were read in full, 2 from the abstract only, and 2 are catalogue records only; each entry says which. 2 are not peer-reviewed and are tiered E so they can back lineage but never a claim.

Machine-readable companion: [`CITATIONS.csl.json`](./CITATIONS.csl.json) (CSL-JSON).
Bibliographic records were verified on 2026-08-28 (Crossref, Open Library) and 2026-08-30 (PubMed)
and re-verified against all three registries on 2026-09-20. Every claim was re-checked against the
abstracts and open full texts on 2026-09-02, and again on 2026-09-20 against the full text of 35
entries and the abstract of 12, with the two books unread as before.

> **This file is generated.** Edit [`CITATIONS.csl.json`](./CITATIONS.csl.json) for records and
> [`citations-notes.md`](./citations-notes.md) for the prose, then run
> `uv run --no-project scripts/generate-citations.py`. Editing `CITATIONS.md` directly will be
> overwritten. `--check` fails if the two drift apart.

exhale is a breathing overlay with no medical purpose, and nothing here is medical advice. See the
[disclaimer](../README.md#disclaimer). This corpus exists so that anyone, including the author, can
check which of exhale's claims and defaults rest on published evidence and which don't. The
[gaps ledger](#gaps-and-unsupported-choices) at the end is the more useful half.

## How to read this

### Verification status

This says how the *bibliographic record* was checked. It says nothing about whether the finding is
true, and nothing about whether the full text was read.

| Status | Meaning |
|---|---|
| `crossref-verified` | The DOI resolves in Crossref, and the title, authors, year, journal, volume and pages printed here are the ones Crossref returned rather than the ones a search result claimed. |
| `openlibrary-verified` | No DOI exists because the source is a book. The title, author, publisher, edition and page count printed here were checked against the Open Library record for the stated ISBN. |
| `pubmed-verified` | No DOI exists, but the article is indexed in PubMed. The title, authors, journal, year, volume and pages were checked against the NCBI E-utilities record for the stated PMID. |
| `unverified` | No resolvable DOI and no catalogue record. Bibliographic details are inherited from secondary citation and may be wrong. |

Author names are printed as the registry holds them, which is why a few records carry initials
where others carry full given names.

A verified record can still carry a loud caveat. Verification confirms the *citation* rather than the
*claim*, so each entry also states to what depth the source was read before its claims were written
down: in full, from the abstract only, or as a catalogue record only.

### Access level

`open-access` (a CC licence is registered with Crossref, or the title is fully open access) |
`paywalled`. Where a Crossref `license` field was present, the access level is taken from it rather
than guessed; where it was absent the basis is stated in the entry's caveat. `paywalled` describes
the version of record. Where a legal open copy exists in a repository or free at the publisher, the
entry links it as an open copy.

### Evidence tier

Applied to sources making a claim about what happens to a human being. Physiological and animal
mechanism papers are tier D by definition: they explain why something might work, they don't
establish that it does.

| Tier | Definition | How exhale is allowed to use it |
|---|---|---|
| A | Systematic review or meta-analysis of controlled trials, or a pre-registered RCT with 200 or more participants | May be cited for an outcome claim |
| B | Pre-registered or internally replicated controlled experiment, or a systematic review of experiments without meta-analysis | May be cited for an outcome claim, with its scope conditions stated |
| C | Single controlled experiment, small n, lab-only, or observational work, including systematic reviews of observational studies | Cite as suggestive; never as "research shows" |
| D | Narrative review, theory, mechanism, or animal work | Cite for mechanism only; it can't establish that a practice works |
| E | Not peer reviewed, or contradicted by better evidence | May be cited for provenance, meaning where a practice came from. Never for whether it works |

### Counts

| Verification | n |
|---|---|
| crossref-verified | 46 |
| openlibrary-verified | 2 |
| pubmed-verified | 1 |
| **total** | **49** |

| Access level | n |
|---|---|
| open-access | 21 |
| paywalled | 28 |
| **total** | **49** |

| Read depth | n |
|---|---|
| abstract | 2 |
| full-text | 45 |
| record | 2 |
| **total** | **49** |

| Evidence tier | n |
|---|---|
| A | 6 |
| B | 3 |
| C | 24 |
| D | 13 |
| E | 2 |
| null (not a study) | 1 |
| **total** | **49** |

---

## Why a breathing reminder next to a screen

11 sources.

#### `albulescu2022-micro-breaks`

Albulescu, Patricia; Macsinga, Irina; Rusu, Andrei; Sulea, Coralia; Bodnaru, Alexandra; Tulbure, Bogdan Tudor. (2022). *"Give me a break!" A systematic review and meta-analysis on the efficacy of micro-breaks for increasing well-being and performance*. PLOS ONE 17(8): e0272460

- DOI: [10.1371/journal.pone.0272460](https://doi.org/10.1371/journal.pone.0272460)
- Verification: crossref-verified | Access: open-access | Read: full text | evidence tier **A**
- Backs:
  - short breaks of ten minutes or less from a work task reduce fatigue and increase vigor
  - break duration didn't moderate the vigor or fatigue effects, but the studies that measured them used breaks of 2 to 10 minutes, so a one-breath interruption sits below the shortest break tested for well-being and inside the studied range only for performance, where longer breaks did better
- Caveat: 22 independent study samples from 19 publications. Break duration moderated the performance effect (longer was better) but not vigor or fatigue. The pooled breaks ran from 8 seconds to 10 minutes, but every sub-minute break came from a performance-only study, the vigor and fatigue studies used breaks of 2 to 10 minutes, and the break-duration tests rest on nine studies each for vigor and fatigue against 15 for performance. The performance effect was smaller and less consistent than the well-being effect, and the authors report it depended on task type. The paper reports break activities only by category and describes none as a breathing exercise, so this supports the general shape of exhale's reminder timer; the breathing content itself is untested here.

#### `deniz2024-forward-head-lung-volumes`

Deniz, Yasemin; Ertekin, Damla; Çokar, Dilek. (2024). *The effect of forward head posture on dynamic lung volumes in young adults: a systematic review*. Bulletin of Faculty of Physical Therapy 29(1): 15

- DOI: [10.1186/s43161-024-00186-7](https://doi.org/10.1186/s43161-024-00186-7)
- Verification: crossref-verified | Access: open-access | Read: full text | evidence tier **C**
- Backs:
  - across four comparison studies totalling 115 participants, forward head posture was associated with FVC reductions of 0.25 to 0.81 L and FEV1 reductions of 0.16 to 0.93 L
  - craniovertebral angle correlates positively with dynamic pulmonary volumes
- Caveat: Systematic review without meta-analysis of four small comparison studies and two correlation studies; the authors phrase the conclusion as forward head posture 'can potentially cause' pulmonary abnormalities. The search included ResearchGate alongside PubMed and Google Scholar, which is an unusual choice. This is the posture half of the screen-breathing argument: it measures head position, and the link to screens comes from jung2016-smartphone-posture-respiration. The Crossref deposit carries the second and third authors' names with a dotless i, a transliteration artefact; they're printed here in standard Turkish orthography.

#### `grassmann2016-cognitive-load-respiration`

Grassmann, Mariel; Vlemincx, Elke; von Leupoldt, Andreas; Mittelstädt, Justin M.; Van den Bergh, Omer. (2016). *Respiratory Changes in Response to Cognitive Load: A Systematic Review*. Neural Plasticity 2016: 8146809

- DOI: [10.1155/2016/8146809](https://doi.org/10.1155/2016/8146809)
- Verification: crossref-verified | Access: open-access | Read: full text | evidence tier **B**
- Backs:
  - mentally demanding work is reliably marked by faster breathing, with mostly medium to large effects
  - respiratory amplitude stays roughly stable under cognitive load
  - in the four experiments that measured it, cognitive load lowered end-tidal CO2, which the authors read as possible overbreathing while noting that CO2 production rose too
- Caveat: 54 experiments across 53 articles. The rate finding is the well-supported one: 20 studies with large effects. The end-tidal CO2 finding rests on four of the 54 experiments, with one computed effect size of d = -0.26, and the authors hedge it as 'may lead to overbreathing'. Note the direction of the amplitude finding: cognitive load raises rate while leaving amplitude roughly stable, so this doesn't support 'shallow' if shallow means less air moved. Minute ventilation goes up. For the diaphragmatic-to-thoracic shift that 'shallow' usually names, see schleifer2002-hyperventilation-job-stress; for the same over-breathing measured at a keyboard, see schleifer1994-vdt-petco2 and schleifer2008-emg-gaps-computer-work. This entry is the general cognitive-load backdrop, and it's also the reason the keyboard can't be singled out as the cause: any demanding task produces the pattern.

#### `johnson2023-20-20-20`

Johnson, Sophia; Rosenfield, Mark. (2023). *20-20-20 Rule: Are These Numbers Justified?* Optometry and Vision Science 100(1): 52-56

- DOI: [10.1097/OPX.0000000000001971](https://doi.org/10.1097/OPX.0000000000001971)
- Verification: crossref-verified | Access: paywalled | Read: full text | evidence tier **C**
- Backs:
  - the widely repeated 20-20-20 rule has little peer-reviewed support
  - scheduled 20-second breaks every 5, 10 or 20 minutes produced no significant effect on ocular symptoms, reading speed or task accuracy
- Caveat: n = 30, 40-minute tablet reading task, four break schedules. Symptoms rose significantly in all four conditions including the most frequent break schedule. Included here as a caution against exhale's own genre: a periodic on-screen nudge isn't self-evidently effective just because it's popular and plausible.

#### `jung2016-smartphone-posture-respiration`

Jung, Sang In; Lee, Na Kyung; Kang, Kyung Woo; Kim, Kyoung; Lee, Do Youn. (2016). *The effect of smartphone usage time on posture and respiratory function*. Journal of Physical Therapy Science 28(1): 186-189

- DOI: [10.1589/jpts.28.186](https://doi.org/10.1589/jpts.28.186)
- Verification: crossref-verified | Access: open-access | Read: full text | evidence tier **C**
- Backs:
  - people using smartphones more than four hours a day had significantly worse craniovertebral angle, worse scapular index and lower peak expiratory flow than people using them less than four hours a day
- Caveat: n = 50, cross-sectional, two groups split on self-reported usage. Correlational: heavy users may differ in many ways besides screen time, and peak expiratory flow was the only respiratory measure that separated (FVC and FEV1 didn't). Crossref carries no license; access level taken from J Phys Ther Sci being fully open access.

#### `rosenfield2011-computer-vision-syndrome`

Rosenfield, Mark. (2011). *Computer vision syndrome: a review of ocular causes and potential treatments*. Ophthalmic and Physiological Optics 31(5): 502-515

- DOI: [10.1111/j.1475-1313.2011.00834.x](https://doi.org/10.1111/j.1475-1313.2011.00834.x)
- Open copy: <https://onlinelibrary.wiley.com/doi/full/10.1111/j.1475-1313.2011.00834.x>
- Verification: crossref-verified | Access: paywalled | Read: full text | evidence tier **D**
- Backs:
  - blink rate falls substantially during display work relative to other tasks
  - reduced and incomplete blinking, with greater corneal exposure, is the review's probable mechanism for the dry-eye half of computer vision syndrome
- Caveat: Narrative review of the ocular literature. The version of record is served free at the publisher, which labels it Free Access, and the only licence Crossref registers for it is Wiley's own terms rather than a CC licence, so it's marked paywalled here with the publisher page as the open copy. Load-bearing for the blink half of exhale's opening claim only. exhale does nothing about blinking; see the gaps ledger.

#### `schleifer1994-vdt-petco2`

Schleifer, Lawrence M.; Ley, Ronald. (1994). *End-tidal PCO2 as an index of psychophysiological activity during VDT data-entry work and relaxation*. Ergonomics 37(2): 245-254

- DOI: [10.1080/00140139408963642](https://doi.org/10.1080/00140139408963642)
- Open copy: <https://stacks.cdc.gov/view/cdc/204606>
- Verification: crossref-verified | Access: paywalled | Read: full text | evidence tier **C**
- Backs:
  - during computer data-entry work, end-tidal CO2 was significantly lower and respiration frequency significantly higher than during either baseline relaxation or progressive muscle relaxation
  - breathing changes measurably at a screen, and end-tidal CO2 discriminates the state
- Caveat: n = 11 female data-entry operators, temps from a clerical agency, tested over three consecutive six-hour days in the NIOSH work-stress laboratory, with the first two days given to practice and only the third day analysed. End-tidal CO2 averaged 41.8 mmHg during data entry against 42.9 during baseline relaxation and 44.0 during progressive muscle relaxation. The authors call the change small, 1 to 2 mmHg, and a hyperventilatory stress effect, and all three means sit inside the 35 to 45 mmHg range that marchant2025-square-478-six treats as healthy. The paper reports end-tidal CO2 lower than at rest and never reports hypocapnia. A public-domain copy is on CDC Stacks: the article states it was prepared under US Government sponsorship. Small sample, and a laboratory simulation (entering tax-form data on a numeric keypad) rather than the operators' own jobs, though run over full working days rather than a short lab session. Note the comparison: data entry against two relaxation conditions, so it shows what demanding work at a keyboard does. It doesn't show that the screen itself is the cause, and grassmann2016-cognitive-load-respiration finds the same pattern under cognitive load generally. Along with schleifer2008-emg-gaps-computer-work it's the direct evidence that keyboard work changes respiration, and it dates from 1994, well before the topic reached breathwork writing.

#### `schleifer2002-hyperventilation-job-stress`

Schleifer, Lawrence M.; Ley, Ronald; Spalding, Thomas W. (2002). *A hyperventilation theory of job stress and musculoskeletal disorders*. American Journal of Industrial Medicine 41(5): 420-432

- DOI: [10.1002/ajim.10061](https://doi.org/10.1002/ajim.10061)
- Verification: crossref-verified | Access: paywalled | Read: full text | evidence tier **D**
- Backs:
  - hyperventilation is often characterised by a shift from a diaphragmatic to a thoracic breathing pattern
  - thoracic breathing recruits sternocleidomastoid, scalene and trapezius muscles, imposing biomechanical stress on the neck and shoulder region
  - breathing training and rest breaks are a rationale-backed response to this pattern at work
- Caveat: Theory paper rather than an experiment, hence tier D. It's nonetheless the closest thing in the peer-reviewed literature to the folk claim about 'shallow' breathing at a screen: the diaphragmatic-to-thoracic shift is what people mean by shallow. Cite it for the pattern, never for a measured tidal volume, and note that it theorises the shift; no study demonstrates it in screen users.

#### `schleifer2008-emg-gaps-computer-work`

Schleifer, Lawrence M.; Spalding, Thomas W.; Kerick, Scott E.; Cram, Jeffrey R.; Ley, Ronald; Hatfield, Bradley D. (2008). *Mental stress and trapezius muscle activation under psychomotor challenge: A focus on EMG gaps during computer work*. Psychophysiology 45(3): 356-365

- DOI: [10.1111/j.1469-8986.2008.00645.x](https://doi.org/10.1111/j.1469-8986.2008.00645.x)
- Verification: crossref-verified | Access: paywalled | Read: full text | evidence tier **C**
- Backs:
  - end-tidal CO2 was lower under high mental workload than low during computer data entry, replicating the over-breathing finding in a second sample
  - lower end-tidal CO2 tracked reduced trapezius EMG-gap frequency, suggesting over-breathing mediates muscle tension at the keyboard
- Caveat: n = 23. The second independent measurement by this group of the same effect, fourteen years after schleifer1994-vdt-petco2. Two small samples pointing the same way is what the screen-breathing claim actually rests on.

#### `sheppard2018-digital-eye-strain`

Sheppard, Amy L.; Wolffsohn, James S. (2018). *Digital eye strain: prevalence, measurement and amelioration*. BMJ Open Ophthalmology 3(1): e000146

- DOI: [10.1136/bmjophth-2018-000146](https://doi.org/10.1136/bmjophth-2018-000146)
- Verification: crossref-verified | Access: open-access | Read: full text | evidence tier **D**
- Backs:
  - digital eye strain is common among heavy display users
  - reduced blink rate and incomplete blinks during screen work are among its established contributors
- Caveat: The article itself is published under CC BY-NC 4.0. Crossref carries no license field for the record, so access level is taken from BMJ Open Ophthalmology being a fully open-access title.

#### `tsubota1993-vdt-blink`

Tsubota, Kazuo; Nakamori, Katsu. (1993). *Dry Eyes and Video Display Terminals*. New England Journal of Medicine 328(8): 584

- DOI: [10.1056/NEJM199302253280817](https://doi.org/10.1056/NEJM199302253280817)
- Open copy: <https://www.nejm.org/doi/pdf/10.1056/NEJM199302253280817>
- Verification: crossref-verified | Access: paywalled | Read: full text | evidence tier **C**
- Backs:
  - blink rate during display work is far below blink rate at rest: 22 per minute relaxed, 10 reading a book, 7 viewing a display, in 104 office workers
- Caveat: A one-page letter, read in full. 104 healthy office workers, about half of them using computers for around three hours a day. Mean blink rate 22 +/- 9 per minute under relaxed conditions, 10 +/- 6 reading a book at table level, 7 +/- 7 viewing text on a display. The letter also reports a larger exposed ocular surface at the display (2.3 cm2 against 1.2 reading) and a higher tear-evaporation rate. These are the '22 at rest, 7 at a screen' figures that circulate by secondary citation. Note the spread on the display figure (7 +/- 7) and that the letter reports no statistics. For a fuller treatment, use rosenfield2011-computer-vision-syndrome or sheppard2018-digital-eye-strain.

---

## Whether slow paced breathing does anything

7 sources.

#### `chaddha2019-slow-breathing-bp`

Chaddha, Ashish; Modaff, Daniel; Hooper-Lane, Christopher; Feldstein, David A. (2019). *Device and non-device-guided slow breathing to reduce blood pressure: A systematic review and meta-analysis*. Complementary Therapies in Medicine 45: 179-184

- DOI: [10.1016/j.ctim.2019.03.005](https://doi.org/10.1016/j.ctim.2019.03.005)
- Verification: crossref-verified | Access: paywalled | Read: full text | evidence tier **A**
- Backs:
  - sustained slow breathing programmes lower systolic blood pressure by about 5.6 mmHg and diastolic by about 3.0 mmHg in hypertensive and prehypertensive adults
- Caveat: 17 trials, each a randomised controlled trial or the first phase of a randomised cross-over study. Heterogeneity was high for every analysis, and the authors say so. Inclusion required at least 5 minutes of breathing at 10 breaths/min or slower, on at least 3 days a week, for at least 4 weeks. exhale asks for none of that and measures none of it, so this describes a dose exhale doesn't deliver. Read alongside vandijk2018-close-the-book.

#### `fincham2023-breathwork-meta`

Fincham, Guy William; Strauss, Clara; Montero-Marin, Jesus; Cavanagh, Kate. (2023). *Effect of breathwork on stress and mental health: A meta-analysis of randomised-controlled trials*. Scientific Reports 13(1): 432

- DOI: [10.1038/s41598-022-27247-y](https://doi.org/10.1038/s41598-022-27247-y)
- Verification: crossref-verified | Access: open-access | Read: full text | evidence tier **A**
- Backs:
  - breathwork lowers self-reported stress against control conditions, g = -0.35 (95% CI -0.55 to -0.14), 12 RCTs, 785 adults
  - comparable small-to-medium effects for anxiety (g = -0.32, k = 20) and depressive symptoms (g = -0.40, k = 18)
  - adverse-event reporting in breathwork trials is sparse: four of the twelve primary-outcome trials reported on it, and none attributed lasting harm to breathwork
- Caveat: Most included studies were rated at moderate risk of bias. Effects are small-to-medium and self-reported. Only four of the twelve primary-outcome trials reported on adverse events, and none attributed lasting harm to breathwork; the authors call for better reporting, particularly for fast-paced techniques. This is the strongest single warrant for the claim that a breathing practice does something, and it's still an effect of about a third of a standard deviation.

#### `laborde2022-vsb-meta`

Laborde, S.; Allen, M. S.; Borges, U.; Dosseville, F.; Hosang, T. J.; Iskra, M.; Mosley, E.; Salvotti, C.; Spolverato, L.; Zammit, N.; Javelle, F. (2022). *Effects of voluntary slow breathing on heart rate and heart rate variability: A systematic review and a meta-analysis*. Neuroscience & Biobehavioral Reviews 138: 104711

- DOI: [10.1016/j.neubiorev.2022.104711](https://doi.org/10.1016/j.neubiorev.2022.104711)
- Open copy: <https://pure.solent.ac.uk/en/publications/74890190-b567-4d13-b1d6-e0bd6b06431f>
- Verification: crossref-verified | Access: paywalled | Read: full text | evidence tier **A**
- Backs:
  - voluntary slow breathing raises vagally-mediated HRV during the session, immediately after a single session, and after a multi-session intervention
  - few adverse effects are expected from slow breathing practice
- Caveat: 223 studies from 1842 screened abstracts (172 during, 16 immediately-after, 49 after-intervention). This is the central warrant for exhale's whole premise. Note what it establishes: an effect on a cardiac index of parasympathetic activity; it says nothing about how anyone feels or works.

#### `little2025-a52-breath-method`

Little, Abbie L. (2025). *The A52 Breath Method: A Narrative Review of Breathwork for Mental Health and Stress Resilience*. Stress and Health 41(4): e70098

- DOI: [10.1002/smi.70098](https://doi.org/10.1002/smi.70098)
- Verification: crossref-verified | Access: open-access | Read: full text | evidence tier **D**
- Backs:
  - a 5 s inhale, 5 s exhale, 2 s post-exhale hold at five breaths per minute is a protocol a recent review argues is representative of the effective literature
  - 23 of 30 reviewed studies reported significant HRV improvement; 10 reported anxiety reduction and 9 reported reduced perceived stress
  - benefits appear larger in people with elevated baseline distress
- Caveat: Narrative review, single author, 465 records returned, 269 screened by title and abstract after de-duplication, 93 full texts assessed and 30 analysed, with no meta-analysis and no risk-of-bias assessment. It proposes the protocol it reviews, which is a conflict of framing worth naming, and it states that no study to date has directly tested the 5-5-2 sequence. Its A52 shape maps onto exhale's four sliders as 5 / 0 / 5 / 2, which is 5 breaths per minute and inside the tested band. marchant2025-square-478-six found that no-hold 6 bpm outperformed both hold-bearing patterns it tested, so the 2 s retention is the least supported part of the protocol.

#### `russo2017-slow-breathing-physiology`

Russo, Marc A.; Santarelli, Danielle M.; O'Rourke, Dean. (2017). *The physiological effects of slow breathing in the healthy human*. Breathe 13(4): 298-309

- DOI: [10.1183/20734735.009817](https://doi.org/10.1183/20734735.009817)
- Verification: crossref-verified | Access: open-access | Read: full text | evidence tier **D**
- Backs:
  - slow breathing improves ventilation efficiency and alters cardiorespiratory coupling, respiratory sinus arrhythmia and sympathovagal balance
- Caveat: Narrative review. The authors close by calling explicitly for further research, and describe the health claims as potential rather than demonstrated. Use it to explain a mechanism rather than to assert an outcome.

#### `vandijk2018-close-the-book`

van Dijk, Peter R.; van Hateren, Kornelis J. J.; Kleefstra, Nanne; Landman, Gijs W. D. (2018). *It is time to close the book on device-guided slow breathing*. Blood Pressure 27(3): 181-182

- DOI: [10.1080/08037051.2018.1435260](https://doi.org/10.1080/08037051.2018.1435260)
- Open copy: <https://www.tandfonline.com/doi/pdf/10.1080/08037051.2018.1435260>
- Verification: crossref-verified | Access: paywalled | Read: full text | no evidence tier (not a study)
- Backs:
  - a body of specialist opinion holds that the blood-pressure case for device-guided slow breathing is weak and should be considered closed
- Caveat: Letter, two pages, so no evidence tier applies. Carried deliberately: it's the strongest published dissent against the cardiovascular claims exhale sits next to, and a corpus that omitted it would be advocacy rather than provenance.

#### `zaccaro2018-slow-breathing-review`

Zaccaro, Andrea; Piarulli, Andrea; Laurino, Marco; Garbella, Erika; Menicucci, Danilo; Neri, Bruno; Gemignani, Angelo. (2018). *How Breath-Control Can Change Your Life: A Systematic Review on Psycho-Physiological Correlates of Slow Breathing*. Frontiers in Human Neuroscience 12: 353

- DOI: [10.3389/fnhum.2018.00353](https://doi.org/10.3389/fnhum.2018.00353)
- Verification: crossref-verified | Access: open-access | Read: full text | evidence tier **B**
- Backs:
  - slow breathing, conventionally defined as under 10 breaths per minute, is associated with increased HRV and shifts in central and autonomic measures
  - reported psychological correlates include increased comfort and relaxation and reduced anxiety and arousal
- Caveat: Systematic review without meta-analysis: 15 articles from 2,461 abstracts. The authors intended a meta-analysis and found pooling infeasible, and they describe the HF and LF outcomes across the included studies as heterogeneous and contradictory. The included studies vary widely in technique, rate and duration, so the overall picture is directional rather than dose-specific.

---

## What the numbers should be

15 sources.

#### `bae2021-exhalation-inhalation-ratio`

Bae, Dalbyeol; Matthews, Jacob J. L.; Chen, J. Jean; Mah, Linda. (2021). *Increased exhalation to inhalation ratio during breathing enhances high-frequency heart rate variability in healthy adults*. Psychophysiology 58(11): e13905

- DOI: [10.1111/psyp.13905](https://doi.org/10.1111/psyp.13905)
- Verification: crossref-verified | Access: paywalled | Read: full text | evidence tier **C**
- Backs:
  - a 2:1 exhale-to-inhale cue raised RMSSD and HF-HRV relative to a 1:1 cue at the participant's own breathing rate
  - the HF-HRV elevation persisted about four minutes after the 2:1 block ended
- Caveat: n = 28 (16 young, 12 older). Note the manipulation check: the achieved ratios were 1.08 under the 1:1 cue and 1.33 under the 2:1 cue, so the longer-exhale condition fell well short of the instructed 2:1, and pacing was at each participant's spontaneous rate rather than in the resonance range. One of five studies in this corpus that disagree about the ratio; see vandiest2014-ie-ratio-relaxation and laborde2021-ie-ratio-pauses (also positive), lin2014-equal-ratio-hrv (favours the equal ratio) and meehan2024-longer-exhalations (null). Gap 4 in the ledger tabulates all five.

#### `balban2023-cyclic-sighing`

Balban, Melis Yilmaz; Neri, Eric; Kogon, Manuela M.; Weed, Lara; Nouriani, Bita; Jo, Booil; Holl, Gary; Zeitzer, Jamie M.; Spiegel, David; Huberman, Andrew D. (2023). *Brief structured respiration practices enhance mood and reduce physiological arousal*. Cell Reports Medicine 4(1): 100895

- DOI: [10.1016/j.xcrm.2022.100895](https://doi.org/10.1016/j.xcrm.2022.100895)
- Verification: crossref-verified | Access: open-access | Read: full text | evidence tier **C**
- Backs:
  - five minutes a day of exhale-emphasising cyclic sighing raised positive affect and lowered respiratory rate more than an equal period of mindfulness meditation over one month
  - box breathing, equal inhale / hold / exhale, was tested head-to-head and wasn't the best-performing arm
- Caveat: Remote randomised controlled study with 108 participants: 24 in the mindfulness-meditation control, 30 cyclic sighing, 21 box breathing and 33 cyclic hyperventilation. It was registered retrospectively as NCT05304000. The authors state it was intended as an exploratory study and 'was not pre-registered as a clinical trial'. The comparator is mindfulness meditation rather than a sham, so the arms differ in more than breath ratio. In the mixed-effects model, cyclic sighing separated from the control on positive affect. Box breathing and cyclic hyperventilation didn't, and the breathwork arms weren't tested against one another. Daily positive-affect gains were 1.89 points for cyclic sighing and 1.84 for box breathing. This is the best evidence in the corpus that emphasising the exhale is the right emphasis, and it's weaker than the abstract suggests: the box arm is small and the difference between the two patterns isn't established. The cyclic-sighing protocol also adds a double inhale, so its effect can't be attributed to exhale length alone. Heart rate variability and resting heart rate slopes didn't differ between groups. Respiratory rate did. In raw daily means the mindfulness arm's reductions in negative affect and state anxiety were the larger ones (1.62 against 0.98 points, and 3.95 against 3.03), and the mixed-effects model found no group difference on either, so the breathwork advantage is specific to positive affect.

#### `bernardi2001-slow-breathing-chemoreflex`

Bernardi, Luciano; Gabutti, Alessandra; Porta, Cesare; Spicuzza, Lucia. (2001). *Slow breathing reduces chemoreflex response to hypoxia and hypercapnia, and increases baroreflex sensitivity*. Journal of Hypertension 19(12): 2221-2229

- DOI: [10.1097/00004872-200112000-00016](https://doi.org/10.1097/00004872-200112000-00016)
- Verification: crossref-verified | Access: paywalled | Read: full text | evidence tier **C**
- Backs:
  - breathing at 6 per minute depressed both hypoxic and hypercapnic chemoreflex responses compared with spontaneous or 15 per minute breathing
  - baroreflex sensitivity was greater during slow breathing
- Caveat: n = 15 healthy individuals, comparing 6 breaths per minute with spontaneous and 15-per-minute breathing. Carried to mark where the tested territory ends: 6 per minute is the slowest rate here, and nothing in this corpus tests slower than 5. Gap 6 discusses what that leaves unsupported. See also bilo2012-slow-breathing-altitude.

#### `bilo2012-slow-breathing-altitude`

Bilo, Grzegorz; Revera, Miriam; Bussotti, Maurizio; Bonacina, Daniele; Styczkiewicz, Katarzyna; Caldara, Gianluca; Giglio, Alessia; Faini, Andrea; Giuliano, Andrea; Lombardi, Carolina; Kawecka-Jaszcz, Kalina; Mancia, Giuseppe; Agostoni, Piergiuseppe; Parati, Gianfranco. (2012). *Effects of Slow Deep Breathing at High Altitude on Oxygen Saturation, Pulmonary and Systemic Hemodynamics*. PLoS ONE 7(11): e49074

- DOI: [10.1371/journal.pone.0049074](https://doi.org/10.1371/journal.pone.0049074)
- Verification: crossref-verified | Access: open-access | Read: full text | evidence tier **C**
- Backs:
  - fifteen minutes of paced breathing at 6 per minute raised arterial oxygen saturation and lowered systemic and pulmonary arterial pressure at high altitude
  - the proposed mechanism is a larger tidal volume reducing the proportion of each breath spent on anatomical dead space
- Caveat: n = 39 (Study A, 4559 m for two to three days) and 28 (Study B, 5400 m for twelve to sixteen days). Conducted at high altitude in a hypoxic state, so it doesn't transfer to a desk at sea level and must not be cited as if it did. It's carried for two narrow purposes: it's a second study placing the controlled literature at 6 breaths per minute, and its dead-space mechanism is the reason slow breathing isn't simply less breathing.

#### `laborde2021-ie-ratio-pauses`

Laborde, Sylvain; Iskra, Maša; Zammit, Nina; Borges, Uirassu; You, Min; Sevoz-Couche, Caroline; Dosseville, Fabrice. (2021). *Slow-Paced Breathing: Influence of Inhalation/Exhalation Ratio and of Respiratory Pauses on Cardiac Vagal Activity*. Sustainability 13(14): 7775

- DOI: [10.3390/su13147775](https://doi.org/10.3390/su13147775)
- Verification: crossref-verified | Access: open-access | Read: full text | evidence tier **C**
- Backs:
  - at six cycles per minute, RMSSD was higher when the exhalation was longer than the inhalation, across inhalation/exhalation ratios of 0.8, 1.0 and 1.2
  - brief 0.4 s pauses after inhalation and after exhalation didn't further change RMSSD
- Caveat: n = 64 athletes, within-subjects, six 5-minute conditions in one session. 66 were recruited and 2 excluded for technical faults. Effect sizes from the full text: ratio main effect partial eta-squared 0.12. The longer exhale beat the shorter exhale at d = 0.51 and the equal ratio at d = 0.37. Equal against shorter exhale was d = 0.14, not significant. The pause main effect was F(1, 63) = 2.495, p = 0.119. Only the breathing rate was verified, so whether participants actually produced the 0.4 s pauses or the exact ratios wasn't measured. Every condition raised RMSSD over baseline at d = 1.05 to 1.36. The authors note that their ratio range, 0.8 to 1.2, is narrower than other studies'. This is one of six studies bearing on the ratio disagreement tabulated in gap 4, on the side of the longer exhale, and the only one that also manipulates respiratory pauses, which is why it also settles gap 11 as far as brief pauses go. Published in Sustainability, an MDPI journal outside the field, which matters when weighing it against the other five.

#### `laborde2021-spb-6cpm-biofeedback`

Laborde, Sylvain; Allen, Mark S.; Borges, Uirassu; Iskra, Maša; Zammit, Nina; You, Min; Hosang, Thomas; Mosley, Emma; Dosseville, Fabrice. (2021). *Psychophysiological effects of slow-paced breathing at six cycles per minute with or without heart rate variability biofeedback*. Psychophysiology 59(1): e13952

- DOI: [10.1111/psyp.13952](https://doi.org/10.1111/psyp.13952)
- Verification: crossref-verified | Access: open-access | Read: full text | evidence tier **C**
- Backs:
  - slow-paced breathing at six cycles per minute raised RMSSD from the pre-session baseline, with and without biofeedback
  - adding heart-rate-variability biofeedback on top of the six-per-minute pace didn't add a further RMSSD benefit, though that condition reported more positive emotional valence
- Caveat: n = 112, single session, within-subjects, no no-breathing control: the comparison is PRE against DURING and POST. The pacer was a ball moving up and down at 6 cpm (EZ-Air), not an expanding shape. Valence was lower during breathing than before it in both conditions (d = 0.37), which is the abstract's 'more negative emotional valence'. The condition main effect on valence was partial eta-squared 0.06. Crossref records this as issued 2021, appearing in the January 2022 issue (59:1). Both conditions raised RMSSD and lowered arousal; the biofeedback condition also reported more positive emotional valence, so 'no added benefit' is specific to the cardiac measure. Together with tabor2022-guided-breathing-design this is why exhale ships without a sensor: for the physiological effect of a single session, the pace does the work. RMSSD was back at baseline across the five-minute recovery recording that followed the session (PRE against POST d = 0.22, not significant), and so was arousal (d = 0.15). The authors call it a switch-on, switch-off effect, which is a second source for the 'lasts as long as it's on' point in gap 8.

#### `lehrer2022-my-life-hrvb`

Lehrer, Paul. (2022). *My Life in HRV Biofeedback Research*. Applied Psychophysiology and Biofeedback 47(4): 289-298

- DOI: [10.1007/s10484-022-09535-5](https://doi.org/10.1007/s10484-022-09535-5)
- Verification: crossref-verified | Access: paywalled | Read: full text | evidence tier **D**
- Backs:
  - the resonance-frequency figure of about 6 breaths per minute came out of Vaschillo's 2002 biofeedback work, where each person's frequency fell between 4.5 and 6.5 cycles per minute, and Song and Lehrer's 2003 pacing study, which paced people from 3 to 14 breaths per minute and found RSA amplitude highest at 4 and 6 and lower at 3
  - induced HRV amplitude is at a minimum around 2 to 3 cycles per minute, which the author attributes to negative resonance
  - each person's resonance frequency is related to height and gender, with slower frequencies for taller people and men, unrelated to age or weight, and appeared unchangeable even after several months of biofeedback practice, with an average of 5.5 breaths per minute
  - paced breathing at a rate above resonance frequency, used as a control condition in several clinical studies, still seems to produce clinically significant effects, which the author lists as an open question, alongside some evidence that breathing slightly away from exact resonance frequency has a slightly smaller clinical effect on blood pressure and anxiety
- Caveat: A first-person retrospective by one of the technique's principal developers, so tier D and read as history rather than evidence: the findings carried here are the author's summary of his own group's published papers, cited rather than re-analysed, and the primary sources (Vaschillo et al. 2002, Song and Lehrer 2003, Vaschillo et al. 2006) aren't in this corpus. Lehrer is also an author of lehrer2014-hrv-biofeedback, so the 5.5 figure and the individual-variation point come from one group twice. Carried for two reasons. It's the clearest account of where the 6-a-minute figure came from, and its report that RSA amplitude was highest at 4 and 6 a minute and fell at 3 is why gaps 2 and 6 say that no primary study in this corpus has measured 4 a minute or reported worse outcomes below 5. Those are HRV-amplitude findings, the same kind of peak shaffer2020-resonance-frequency-assessment describes, and they say nothing about relaxation or comfort at slower rates. Read from a copy posted on academia.edu. The version of record is paywalled at Springer.

#### `lin2014-equal-ratio-hrv`

Lin, I. M.; Tai, L. Y.; Fan, S. Y. (2014). *Breathing at a rate of 5.5 breaths per minute with equal inhalation-to-exhalation ratio increases heart rate variability*. International Journal of Psychophysiology 91(3): 206-211

- DOI: [10.1016/j.ijpsycho.2013.12.006](https://doi.org/10.1016/j.ijpsycho.2013.12.006)
- Verification: crossref-verified | Access: paywalled | Read: full text | evidence tier **C**
- Backs:
  - 5.5 breaths per minute with an equal 5:5 inhale-to-exhale ratio produced higher SDNN and LF power than 6 bpm or than a 4:6 ratio
  - all four slow-breathing patterns increased self-reported relaxation relative to spontaneous breathing
- Caveat: n = 47, Latin-square counterbalanced. The clearest published result against a longer exhale on HRV grounds, and one reason exhale makes no mechanism claim for the 1:2 ratio. It also measured relaxation and anxiety across the four patterns and reports that all four increased relaxation over baseline. The abstract doesn't say whether the patterns differed from each other on relaxation, so that's the extent of the subjective finding this corpus can carry from it, one of three bearing on gap 4. Note the second claim carefully: every pattern tested increased relaxation, so the ratio argument is about which slow pattern is best rather than about whether slow breathing works.

#### `marchant2025-square-478-six`

Marchant, Joshua; Khazan, Inna; Cressman, Mikel; Steffen, Patrick. (2025). *Comparing the Effects of Square, 4-7-8, and 6 Breaths-per-Minute Breathing Conditions on Heart Rate Variability, CO2 Levels, and Mood*. Applied Psychophysiology and Biofeedback 50(2): 261-276

- DOI: [10.1007/s10484-025-09688-z](https://doi.org/10.1007/s10484-025-09688-z)
- Verification: crossref-verified | Access: paywalled | Read: full text | evidence tier **C**
- Backs:
  - breathing at 6 breaths per minute raised LF-HRV more than either square (box) breathing or 4-7-8 breathing at both ratios tested, with small to medium effects, and on RMSSD only the 4:6 ratio's advantage over the hold patterns was significant
  - square and 4-7-8 breathing are popularly promoted but have little empirical support
  - blood pressure didn't change significantly in any condition after Scheffe correction, neither 6 bpm ratio changed mood, and the two hold-based patterns showed small decreases in positive mood (0.19 and 0.24 SD) of which only the 4:6 against 4-7-8 contrast survived the same correction
  - breathing at 6 breaths per minute unexpectedly produced mild over-breathing
- Caveat: n = 84 college students, within-subjects, four conditions: square, 4-7-8, 6 bpm at 4:6, and 6 bpm at 5:5. Single session, so it speaks to acute effect only, and it's the largest ratio comparison in this corpus. This is the head-to-head that ranks exhale's presets, and it ranks against box breathing and 4-7-8. RMSSD gains over baseline in standard-deviation units: 5:5 0.45, 4:6 0.65, square 0.25, 4-7-8 0.27, all significant. Scheffe-corrected pairwise: 4:6 beat square and 4-7-8 (p < 0.001). 5:5's edge over them, 0.19 and 0.18, didn't reach significance (p = 0.10 and 0.15). 4:6 against 5:5 was 0.20, p = 0.07. On LF-HRV both 6 bpm conditions beat both hold patterns. So '6 bpm won' is right and 'the 5:5 condition won' isn't: the point estimates favour 4:6, and only 4:6 separated from the hold patterns on RMSSD. The authors' own conclusion is that no outcome differed significantly by ratio, that the study wasn't powered below 0.17, and that if a longer exhale has an effect it is small. Adherence within a second of the target pace was 92.9 percent for 5:5, 86.9 percent for 4:6 and 73.8 percent for both hold patterns, and participants generally breathed faster than paced. End-tidal CO2 fell by 0.56 SD (5:5) and 0.65 SD (4:6), which on average put participants in the 30 to 35 mmHg range the authors call mild over-breathing. They also note the capnometer's baseline zeroing may have been off. Two of its findings cut against arguments made elsewhere in this corpus: mood didn't change at either 6 bpm ratio, which is the largest subjective null in gap 4, and 6 bpm produced mild over-breathing, which is gap 5.

#### `meehan2024-longer-exhalations`

Meehan, Zachary M.; Shaffer, Fred. (2024). *Do Longer Exhalations Increase HRV During Slow-Paced Breathing?* Applied Psychophysiology and Biofeedback 49(3): 407-417

- DOI: [10.1007/s10484-024-09637-2](https://doi.org/10.1007/s10484-024-09637-2)
- Verification: crossref-verified | Access: open-access | Read: full text | evidence tier **B**
- Backs:
  - at 6 breaths per minute, a 1:2 inhale-to-exhale ratio produced no HRV advantage over 1:1 in either an original experiment or its replication
  - the finding held across time-domain, frequency-domain and nonlinear HRV metrics
- Caveat: Original n = 26, replication n = 16; both undergraduate samples, both within-subjects with manipulation checks. Small, but it's the only entry in this corpus that ran its own replication, and its introduction tallies the older literature: three further nulls and one result favouring the longer inhale. Its scope condition matters: it holds rate fixed inside the resonance range, so it doesn't rule out a ratio effect at a person's spontaneous rate, which is what bae2021-exhalation-inhalation-ratio measured. One of six disagreeing studies. See also vandiest2014-ie-ratio-relaxation, laborde2021-ie-ratio-pauses, lin2014-equal-ratio-hrv and marchant2025-square-478-six. All six measured HRV; vandiest2014-ie-ratio-relaxation, lin2014-equal-ratio-hrv and marchant2025-square-478-six also measured how participants felt, and those three don't agree either.

#### `sevozcouche2022-coherence-resonance`

Sevoz-Couche, Caroline; Laborde, Sylvain. (2022). *Heart rate variability and slow-paced breathing: when coherence meets resonance*. Neuroscience & Biobehavioral Reviews 135: 104576

- DOI: [10.1016/j.neubiorev.2022.104576](https://doi.org/10.1016/j.neubiorev.2022.104576)
- Open copy: <https://hal.sorbonne-universite.fr/hal-03578368>
- Verification: crossref-verified | Access: paywalled | Read: full text | evidence tier **D**
- Backs:
  - temporal coherence of respiratory, blood-pressure and cardiac oscillations is reached only when breathing at the baroreflex resonant frequency of about 0.1 Hz
  - the baroreflex loop's delay of roughly five seconds varies between about 4 and 6.5 seconds across people, putting the natural frequency between 0.075 and 0.12 Hz, roughly 4.5 to 7 breaths per minute
- Caveat: Narrative review, hence tier D. It defines its terms: 'resonance' is the roughly 0.1 Hz oscillation the baroreflex loop's delay produces, and 'coherence' the phase alignment of breathing, blood pressure and heart rate that only breathing at that frequency produces. Carried as a terminology reference: 'cardiac coherence' also circulates as a product name in the market exhale sits in, and this is the paper to check before adopting that vocabulary in the app or the README. Its conclusion notes that some studies report no effects and asks for placebo comparisons such as pacing at a spontaneous rate. The delay figures, which the review takes from Vaschillo et al. 2006, are a second citation, with lehrer2014-hrv-biofeedback, for resonance frequency being individual, and the review notes that exact coherence at 0.1 Hz is especially obtained in young participants.

#### `shaffer2020-resonance-frequency-assessment`

Shaffer, Fred; Meehan, Zachary M. (2020). *A Practical Guide to Resonance Frequency Assessment for Heart Rate Variability Biofeedback*. Frontiers in Neuroscience 14: 570400

- DOI: [10.3389/fnins.2020.570400](https://doi.org/10.3389/fnins.2020.570400)
- Verification: crossref-verified | Access: open-access | Read: full text | evidence tier **D**
- Backs:
  - the relationship between breathing rate and heart-rate-variability amplitude is an inverted U with a peak
  - resonance frequency ranges from 4.5 to 6.5 breaths per minute in adults, and 6.5 to 9.5 in children
  - breathing in a narrow band around the resonance frequency stimulates the baroreflex better than breathing across a wider range
- Caveat: Methods guide rather than an experiment, hence tier D. It bears on exhale's drift setting: if HRV amplitude peaks at the resonance frequency, breathing progressively slower moves away from that peak once past it, so 'slower is always better' is false for HRV amplitude. Gap 6 notes that the peak is in HRV amplitude rather than in relaxation or comfort, so this bounds the HRV argument for drift without settling the question. Shaffer is also an author of meehan2024-longer-exhalations, so this corpus leans on one group twice; the resonance-frequency model itself isn't a contested finding.

#### `szulczewski2019-training-relaxation`

Szulczewski, Mikołaj Tytus. (2019). *Training of paced breathing at 0.1 Hz improves CO2 homeostasis and relaxation during a paced breathing task*. PLOS ONE 14(6): e0218550

- DOI: [10.1371/journal.pone.0218550](https://doi.org/10.1371/journal.pone.0218550)
- Verification: crossref-verified | Access: open-access | Read: full text | evidence tier **C**
- Backs:
  - across seven consecutive days of ten-minute paced breathing, self-reported task pleasantness rose significantly and unpleasant arousal fell, with the affective gains emerging by mid-training rather than on day one
  - the end-tidal CO2 drop caused by paced breathing shrank with practice: 37.5% of participants dropped below 30 mmHg on day one against 6.3% on day seven
- Caveat: n = 16, single group, no control, one week. Small and uncontrolled, so treat the magnitudes as indicative. It's carried because it's the closest thing in this corpus to evidence for the pranayama proposition that the practice improves with practice: relaxation wasn't immediate, it accrued. Note what it doesn't show. Training was at a fixed 0.1 Hz throughout; it's evidence that repeated practice at one rate gets better. Whether progressively slowing within a session helps is a second proposition, and it remains untested. See also joshi1992-pranayam-training.

#### `vandiest2014-ie-ratio-relaxation`

Van Diest, Ilse; Verstappen, Karen; Aubert, André E.; Widjaja, Devy; Vansteenwegen, Debora; Vlemincx, Elke. (2014). *Inhalation/Exhalation Ratio Modulates the Effect of Slow Breathing on Heart Rate Variability and Relaxation*. Applied Psychophysiology and Biofeedback 39(3-4): 171-180

- DOI: [10.1007/s10484-014-9253-x](https://doi.org/10.1007/s10484-014-9253-x)
- Open copy: <https://lirias.kuleuven.be/retrieve/843e9b82-a9b0-42e6-897e-9177c10d71b1>
- Verification: crossref-verified | Access: paywalled | Read: full text | evidence tier **C**
- Backs:
  - participants reported more relaxation, more stress reduction, more mindfulness and more positive energy breathing with a low inhale/exhale ratio (longer exhale) than a high one
  - a low inhale/exhale ratio also produced more HF-HRV power, but only in the slow breathing condition
  - of the four relaxation dimensions, slowing the rate on its own improved only positive energy, though it also raised pleasantness and lowered arousal on the SAM
- Caveat: 30 recruited, 23 analysed after five exclusions on criteria and two for not following the pacing, and every F-test carries df (1, 22). Four patterns crossing rate (6 or 12 breaths/min) with i/e ratio (0.42 or 2.33). The achieved ratios at 6 bpm were 0.49 and 1.44. The achieved rates in those conditions were 7.32 and 7.69 breaths a minute rather than 6, so the slow-rate contrast actually tested was about 0.5 against 1.4 at a little over 7 a minute. The authors note that minute ventilation during instructed breathing was high compared with baseline and that participants may have been hyperventilating, which they offer as one reason the subjective effects were small. Heart rate was higher with the longer exhale, which the authors call contrary to expectation. This is the single most important entry for exhale's longer-exhale preference: it's the one study in this corpus in which the longer exhale won on how people felt. It isn't the only study that measured that. lin2014-equal-ratio-hrv found every slow pattern raised relaxation with no ratio-specific edge, and marchant2025-square-478-six, at nearly three times the sample, found no meaningful mood change in any condition. Gap 4 weighs the three.

#### `you2023-respiratory-frequency`

You, Min; Laborde, Sylvain; Ackermann, Stefan; Borges, Uirassu; Dosseville, Fabrice; Mosley, Emma. (2023). *Influence of Respiratory Frequency of Slow-Paced Breathing on Vagally-Mediated Heart Rate Variability*. Applied Psychophysiology and Biofeedback 49(1): 133-143

- DOI: [10.1007/s10484-023-09605-2](https://doi.org/10.1007/s10484-023-09605-2)
- Verification: crossref-verified | Access: paywalled | Read: full text | evidence tier **C**
- Backs:
  - five minutes of slow-paced breathing at 5, 5.5, 6, 6.5 and 7 cycles per minute all raised cardiac vagal activity above spontaneous breathing
  - LF-HRV discriminated between the tested frequencies more sensitively than RMSSD
  - the band actually tested and supported is 5 to 7 cycles per minute
- Caveat: Crossref records this as issued 2023 (online 8 December); it appears in the March 2024 issue, 49(1), which is how it's usually cited. The citekey follows the Crossref issued year, as everywhere else in this corpus. n = 75, all athletes aged 19-31, single lab session. Generalisation to a desk worker is an assumption rather than a finding. This is the source that fixes the tested band the settings panel reports: 5 s in and 5 s out is 6 cycles per minute, inside it; the earlier 5 s in and 10 s out is 4.0, below it.

---

## Where the practice came from

3 sources.

#### `joshi1992-pranayam-training`

Joshi, L. N.; Joshi, V. D.; Gokhale, L. V. (1992). *Effect of short term 'Pranayam' practice on breathing rate and ventilatory functions of lung*. Indian Journal of Physiology and Pharmacology 36(2): 105-108

- PMID: [1506070](https://pubmed.ncbi.nlm.nih.gov/1506070/) (no DOI exists)
- Open copy: <https://ijpp.com/IJPP%20archives/1992_36_2/105-108.pdf>
- Verification: pubmed-verified | Access: paywalled | Read: full text | evidence tier **C**
- Backs:
  - six weeks of pranayama practice in 75 young adults lowered resting respiratory rate and prolonged breath-holding time
  - the same training raised forced vital capacity, maximum voluntary ventilation and peak expiratory flow rate in both sexes. FEV1, which the paper reports as a percentage, rose in the women and fell in the men
- Caveat: Read in full from the journal's own archive PDF. No DOI exists, and the record was verified against the NCBI E-utilities API by PMID rather than Crossref. 33 male and 42 female medical students, mean age 18.5, each their own control with no separate control group. The protocol was 20 minutes twice a day on weekdays and once on Saturdays for six weeks: two minutes of slow maximal breaths at 5 s in and 5 s out (6 a minute, exhale's default rate), sixteen minutes of 5 s maximal inhale, 17 s hold and 8 s maximal exhale (a 30-second cycle, 2 a minute), then two minutes at 5 s in and out again. Uncontrolled before-and-after design in a 1992 regional journal, so treat the effect sizes as unusable. Per-measure n in the tables runs from 23 to 42 per sex, so the respiratory-rate, FVC and FEV1% results rest on 68, 62 and 58 people rather than 75, and Table III's FEV1% differences don't match Tables I and II. No Crossref record exists to carry a licence and the journal's archive serves the PDF free without stating one, so the access level stays paywalled with the archive PDF as the open copy. It's carried because it's the only entry in this corpus that speaks to graded extension as a training progression: capacity grew over six weeks of practice. That's a claim about adaptation across sessions. Extending the breath without limit inside a single sitting is a different proposition, and shaffer2020-resonance-frequency-assessment bears on it. The trained pattern was hold-heavy, which bears on gap 11: six weeks of a 17-second retention with no adverse effects reported, in an uncontrolled study that wasn't looking for them.

#### `muktibodhananda1998-hatha-yoga-pradipika`

Muktibodhananda, Swami. (1998). *Hatha Yoga Pradipika*, 3rd ed. Munger, Bihar, India: Bihar School of Yoga 642 pp

- ISBN: 9788185787381 | [Open Library record](https://openlibrary.org/books/OL9083573M/Hatha_Yoga_Pradipika)
- Verification: openlibrary-verified | Access: paywalled | Read: catalogue record only | evidence tier **E**
- Backs:
  - the classical hatha text and commentary in which timed inhale, retention and exhale ratios are set out is the historical origin of the ratio instructions modern breathing apps repeat
- Caveat: Contents not consulted; the bibliographic record was checked against Open Library, which lists this ISBN as the third edition, Bihar School of Yoga, 1998, 642 pages. Not peer reviewed, and a commentary on a fifteenth-century text rather than a primary source in any modern sense. Carried because the longer-exhale instruction exhale ships didn't come from a laboratory: it came from this tradition, centuries before anyone measured HRV. Naming that is more accurate than retrofitting a citation to psychophysiology. It must never back a physiological claim.

#### `satyananda1999-apmb`

Satyananda Saraswati, Swami. (1999). *Asana Pranayama Mudra Bandha*, 3rd rev. ed. Munger, Bihar, India: Yoga Publications Trust 553 pp

- ISBN: 9788186336144 | [Open Library record](https://openlibrary.org/books/OL22138410M/Asana_pranayama_mudra_bandha)
- Verification: openlibrary-verified | Access: paywalled | Read: catalogue record only | evidence tier **E**
- Backs:
  - the systematic pranayama tradition from which exhale's controllable inhale / retention / exhale / retention structure descends is documented in a standard modern reference manual
- Caveat: Contents not consulted; the bibliographic record was checked against Open Library, which lists this ISBN as the third revised edition, Yoga Publications Trust, 1999, 553 pages. A 2008 fourth revised edition exists under the same imprint; this entry cites the edition the ISBN resolves to. Not peer reviewed. Cited only for lineage: it's where exhale's four-phase structure comes from, and presenting the design as derived from 2020s psychophysiology would be revisionist. It must never back a physiological claim.

---

## Whether an on-screen visual pacer works

3 sources.

#### `moraveji2011-peripheral-paced-respiration`

Moraveji, Neema; Olson, Ben; Nguyen, Truc; Saadat, Mahmoud; Khalighi, Yaser; Pea, Roy; Heer, Jeffrey. (2011). *Peripheral paced respiration: influencing user physiology during information work*. Proceedings of the 24th Annual ACM Symposium on User Interface Software and Technology (UIST '11) 423-428

- DOI: [10.1145/2047196.2047250](https://doi.org/10.1145/2047196.2047250)
- Open copy: <https://idl.cs.washington.edu/files/2011-PPR-UIST.pdf>
- Verification: crossref-verified | Access: paywalled | Read: full text | evidence tier **C**
- Backs:
  - a translucent animated bar spanning the screen, running in the periphery of attention during normal information work, significantly lowered participants' breathing rate
  - peripheral pacing doesn't require the user's full attention to change breathing
- Caveat: n = 13 (9 men, 4 women, mean age 25.5), students from computer science and related fields doing their own work, two 20-minute conditions. The pacer was sensor-triggered and intermittent: it ran in 2-minute bursts when the wearer's breath rate rose 20% above their resting baseline or at least once every six minutes, and was on for about 60% of the pacer condition. It paced to 20% below that baseline, which was taken during three minutes of eyes-closed relaxation (a pre-survey for three of the thirteen) and averaged 9.3 a minute, so the target was near 7.5 a minute, close to exhale's 6. On average they didn't get there: the rate while a burst was on averaged 15.0 (SD 4.4), and the paper gives no per-person figures. Mean rate was 15.7 a minute in the pacer condition against 17.6 without, a difference the paper gives as 1.8 bpm in the results and 1.9 in the discussion, and between bursts the rate returned to 17.1. This is the closest published analogue to exhale that exists, and its limitation is the important part: the reduction occurred while the pacing feedback was active and didn't persist as a lasting change in respiratory pattern. An always-on overlay should be understood as an effect that lasts as long as it's on. An author copy is served from idl.cs.washington.edu. The ACM Digital Library version is paywalled.

#### `tabor2022-guided-breathing-design`

Tabor, Aaron; Bateman, Scott; Scheme, Erik J.; schraefel, m.c. (2022). *Comparing heart rate variability biofeedback and simple paced breathing to inform the design of guided breathing technologies*. Frontiers in Computer Science 4: 926649

- DOI: [10.3389/fcomp.2022.926649](https://doi.org/10.3389/fcomp.2022.926649)
- Verification: crossref-verified | Access: open-access | Read: full text | evidence tier **C**
- Backs:
  - an expanding and contracting circle pacing 6 breaths per minute reduced breathing rate and raised LF power as much as sensor-driven HRV biofeedback, with no significant group difference, and its HRV amplitude gain was larger (4.2 to 8.0 against 5.0 to 7.0, phase by protocol interaction p = 0.03), which the authors still summarise as comparable
  - both conditions took roughly two minutes for effects to appear
  - paced breathing needs no sensor, no real-time processing and no sustained attention, which suits it to use as a secondary task
- Caveat: Between-subjects, n = 28 (14 per group), single 10-minute session. Paced breathing was run as a focused task in the study. The secondary-task suitability in the third claim is the paper's design argument, which the experiment didn't test. Fourteen per group can fail to find a difference without establishing equivalence. This is the strongest published warrant for exhale's specific design choices: an expanding/contracting shape, no hardware, no account, watchable while doing something else.

#### `wongsuphasawat2012-cant-force-calm`

Wongsuphasawat, Kanit; Gamburg, Alex; Moraveji, Neema. (2012). *You can't force calm: designing and evaluating respiratory regulating interfaces for calming technology*. Adjunct Proceedings of the 25th Annual ACM Symposium on User Interface Software and Technology (UIST '12 Adjunct) 69-70

- DOI: [10.1145/2380296.2380326](https://doi.org/10.1145/2380296.2380326)
- Open copy: <https://hci.stanford.edu/publications/2012/CantForceCalmUIST2012.pdf>
- Verification: crossref-verified | Access: paywalled | Read: full text | evidence tier **C**
- Backs:
  - auditory pacing was rated more calming than visual pacing (t = 1.92, df = 26, reported as p < 0.05), while the visual guide's larger reduction in breathing rate was a non-significant trend (t = 0.66, p = 0.26)
- Caveat: n = 14, phone-based, 1-minute drills at 6.4 breaths per minute, each followed by 3 minutes of reading, both modalities intermittent. The visual guide showed the user's own stomach expansion against a target, so it was biofeedback, which exhale's visual pacer isn't. Both modes raised self-reported calm over reading. Two-page adjunct paper, effectively a poster: the calm difference clears p < 0.05 only one-tailed, the breathing-rate p of 0.256 is one-tailed too (two-tailed about 0.51), and the between-modality tests were run as two-sample tests on paired data. Treat the finding as a design signal rather than a result. It's carried anyway because it's the one published comparison that puts exhale's visual-only design at a disadvantage on the outcome most users actually care about, which is whether they feel calmer. An author copy is served from hci.stanford.edu.

---

## Physiology and neuroscience

7 sources.

#### `lehrer2014-hrv-biofeedback`

Lehrer, Paul M.; Gevirtz, Richard. (2014). *Heart rate variability biofeedback: how and why does it work?* Frontiers in Psychology 5: 756

- DOI: [10.3389/fpsyg.2014.00756](https://doi.org/10.3389/fpsyg.2014.00756)
- Verification: crossref-verified | Access: open-access | Read: full text | evidence tier **D**
- Backs:
  - maximum heart-rate oscillation is usually reached breathing at approximately 0.1 Hz, six breaths per minute
  - refined measurement puts the average resonance frequency nearer 0.09 Hz, about 5.5 breaths per minute, a breath lasting roughly 11 seconds
  - resonance frequency is individual: taller people and men tend to have lower resonance frequencies
  - baroreflex gain increases substantially during HRV biofeedback
- Caveat: Narrative mechanistic review by the technique's principal developers. Read as the mechanism argument rather than as independent evidence. The individual-variation point is the honest reason exhale's timing sliders are user-editable at all: there's no single correct number to hardcode.

#### `li2016-sigh-circuit`

Li, Peng; Janczewski, Wiktor A.; Yackle, Kevin; Kam, Kaiwen; Pagliardini, Silvia; Krasnow, Mark A.; Feldman, Jack L. (2016). *The peptidergic control circuit for sighing*. Nature 530(7590): 293-297

- DOI: [10.1038/nature16964](https://doi.org/10.1038/nature16964)
- Open copy: <https://www.ncbi.nlm.nih.gov/pmc/articles/PMC4852886/>
- Verification: crossref-verified | Access: paywalled | Read: full text | evidence tier **D**
- Backs:
  - sighing is generated by a dedicated peptidergic circuit projecting onto the preBotzinger complex rather than as a byproduct of ordinary breathing rhythm
- Caveat: Mouse and rat work. It establishes that the sigh is a distinct, hardwired respiratory behaviour, which is the mechanistic backdrop for the cyclic-sighing result in balban2023-cyclic-sighing. It says nothing about humans deliberately performing sighs, and must not be cited as if it did.

#### `vlemincx2013-sigh-reset-model`

Vlemincx, Elke; Abelson, James L.; Lehrer, Paul M.; Davenport, Paul W.; Van Diest, Ilse; Van den Bergh, Omer. (2013). *Respiratory variability and sighing: A psychophysiological reset model*. Biological Psychology 93(1): 24-32

- DOI: [10.1016/j.biopsycho.2012.12.001](https://doi.org/10.1016/j.biopsycho.2012.12.001)
- Verification: crossref-verified | Access: paywalled | Read: full text | evidence tier **D**
- Backs:
  - the model proposes that sighs act as resetters that restore the balance between components of respiratory variability and cause relief
- Caveat: Review and theoretical model paper. Carried because it's the honest place to point when someone asks why a deliberately irregular breath might be useful, which is the only literature adjacent to exhale's randomised-timing sliders. It's not evidence that those sliders help; see the gaps ledger.

#### `vlemincx2016-sigh-relief`

Vlemincx, Elke; Van Diest, Ilse; Van den Bergh, Omer. (2016). *A sigh of relief or a sigh to relieve: The psychological and physiological relief effect of deep breaths*. Physiology & Behavior 165: 127-135

- DOI: [10.1016/j.physbeh.2016.07.004](https://doi.org/10.1016/j.physbeh.2016.07.004)
- Verification: crossref-verified | Access: paywalled | Read: abstract only | evidence tier **C**
- Backs:
  - self-reported relief was higher after an instructed deep breath than before it
  - frontalis muscle tension fell after a spontaneous sigh in people high in anxiety sensitivity, and after a spontaneous breath hold in people low in it
- Caveat: Findings taken from the abstract; the version of record is paywalled. The instructed-breath result is the closest thing in this corpus to evidence that being told to take one deliberate breath, which is what exhale's reminder does, changes how a person feels.

#### `yackle2017-breathing-arousal-neurons`

Yackle, Kevin; Schwarz, Lindsay A.; Kam, Kaiwen; Sorokin, Jordan M.; Huguenard, John R.; Feldman, Jack L.; Luo, Liqun; Krasnow, Mark A. (2017). *Breathing control center neurons that promote arousal in mice*. Science 355(6332): 1411-1415

- DOI: [10.1126/science.aai7984](https://doi.org/10.1126/science.aai7984)
- Open copy: <https://www.ncbi.nlm.nih.gov/pmc/articles/PMC5505554/>
- Verification: crossref-verified | Access: paywalled | Read: full text | evidence tier **D**
- Backs:
  - a small preBotzinger subpopulation projects to and positively regulates noradrenergic locus coeruleus neurons, giving breathing a direct anatomical route to arousal state
  - ablating those roughly 175 neurons left breathing intact but increased calm behaviours
- Caveat: Mouse work, and the direction is breathing pattern to arousal rather than voluntarily slow breathing to calm. It's the single best answer to 'why would breathing slowly change how I feel at all', and it's still an inference from mice. Don't cite it for a human effect.

#### `yasuma2004-rsa`

Yasuma, Fumihiko; Hayano, Jun-ichiro. (2004). *Respiratory Sinus Arrhythmia: Why Does the Heartbeat Synchronize With Respiratory Rhythm?* Chest 125(2): 683-690

- DOI: [10.1378/chest.125.2.683](https://doi.org/10.1378/chest.125.2.683)
- Verification: crossref-verified | Access: paywalled | Read: full text | evidence tier **D**
- Backs:
  - heart rate rises on inhalation and falls on exhalation, and this respiratory sinus arrhythmia is the coupling that HRV-based breathing claims rest on
- Caveat: Opinion and hypothesis article, filed by Chest under Opinions/Hypotheses. The abstract argues that RSA serves pulmonary gas exchange and notes evidence of a possible dissociation between RSA and vagal control of heart rate, a caution against reading RSA as a pure index of vagal tone. This is the physiological fact underneath every 'longer exhale calms you' claim in this corpus, which is why the claim is so intuitive and why meehan2024-longer-exhalations failing to find a ratio effect is worth taking seriously: a real beat-to-beat mechanism doesn't guarantee a measurable session-level outcome.

#### `zelano2016-nasal-respiration-limbic`

Zelano, Christina; Jiang, Heidi; Zhou, Guangyu; Arora, Nikita; Schuele, Stephan; Rosenow, Joshua; Gottfried, Jay A. (2016). *Nasal Respiration Entrains Human Limbic Oscillations and Modulates Cognitive Function*. The Journal of Neuroscience 36(49): 12448-12467

- DOI: [10.1523/JNEUROSCI.2586-16.2016](https://doi.org/10.1523/JNEUROSCI.2586-16.2016)
- Verification: crossref-verified | Access: open-access | Read: full text | evidence tier **C**
- Backs:
  - nasal breathing entrains oscillations in human piriform cortex, amygdala and hippocampus, and the effect is specific to the nasal route rather than to breathing as such
- Caveat: Human intracranial recordings in a small epilepsy-surgery cohort plus behavioural experiments. Carried because it's the reason nose-versus-mouth is a real variable and not folklore. exhale gives no nasal-breathing guidance at all, which is a defensible omission for a wordless overlay but should be a conscious one.

---

## Limits, harms and adherence

3 sources.

#### `fincham2024-high-ventilation-rct`

Fincham, Guy W.; Epel, Elissa; Colasanti, Alessandro; Strauss, Clara; Cavanagh, Kate. (2024). *Effects of brief remote high ventilation breathwork with retention on mental health and wellbeing: a randomised placebo-controlled trial*. Scientific Reports 14(1): 16893

- DOI: [10.1038/s41598-024-64254-7](https://doi.org/10.1038/s41598-024-64254-7)
- Verification: crossref-verified | Access: open-access | Read: full text | evidence tier **A**
- Backs:
  - high-ventilation breathwork with retention is a distinct practice from slow-paced breathing and warrants a placebo-controlled evaluation of its own
  - short-term effects reported by participants in the high-ventilation arm included light-headedness, dizziness and tetany, with no lasting adverse effects reported
- Caveat: Pre-registered as NCT06064474, 200 healthy young adults, blinded against an active comparator; the largest trial of this technique to date. The primary finding is a null: high-ventilation breathwork didn't outperform the comparator on stress. It's carried for the distinction it draws and for its tolerability data. It matters to exhale because the sliders can be set to fast, hold-heavy patterns that leave the slow-breathing evidence base entirely, and this is where the documented short-term effects of those patterns are recorded.

#### `linardon2020-app-attrition`

Linardon, Jake; Fuller-Tyszkiewicz, Matthew. (2020). *Attrition and adherence in smartphone-delivered interventions for mental health problems: A systematic and meta-analytic review*. Journal of Consulting and Clinical Psychology 88(1): 1-13

- DOI: [10.1037/ccp0000459](https://doi.org/10.1037/ccp0000459)
- Verification: crossref-verified | Access: paywalled | Read: abstract only | evidence tier **A**
- Backs:
  - dropout and non-adherence are common in trials of smartphone-delivered mental health interventions and may undermine the validity of their findings, and usage declines over the course of a trial
- Caveat: Findings taken from the abstract: mean attrition of 24.1% at short-term and 35.5% at longer-term follow-up across 70 trials. In trials reporting both per-protocol and intention-to-treat analyses the effect sizes differed by only a d of 0.18. Carried as the reality check on every other entry in this corpus: an effect measured in a supervised session says little about a tool someone installs and forgets. exhale has no telemetry and therefore no measure of whether anyone keeps it running, which the gaps ledger states plainly.

#### `szulczewski2019-antihyperventilation-instruction`

Szulczewski, Mikołaj Tytus. (2019). *An Anti-hyperventilation Instruction Decreases the Drop in End-tidal CO2 and Symptoms of Hyperventilation During Breathing at 0.1 Hz*. Applied Psychophysiology and Biofeedback 44(3): 247-256

- DOI: [10.1007/s10484-019-09438-y](https://doi.org/10.1007/s10484-019-09438-y)
- Verification: crossref-verified | Access: open-access | Read: full text | evidence tier **C**
- Backs:
  - a two-sentence instruction cut the end-tidal CO2 drop during 6-per-minute paced breathing from 5.21 to 2.7 mmHg
  - hyperventilation symptoms rose 0.63 points on a 7-point scale without the instruction and didn't rise significantly with it
  - the instruction used was to avoid excessively deep breathing and to breathe shallowly and naturally
- Caveat: Randomised, two groups, n = 46 aged 19-26, single session. This is the mitigation for gap 5 and it's unusually cheap: the problem with slow pacing is depth rather than rate, and a two-sentence instruction roughly halves it, cutting the drop from 5.21 to 2.7 mmHg. 36.4% of the instructed group still dropped below 30 mmHg against 43.5% of controls, and baseline end-tidal CO2 was about 36 mmHg in both groups. exhale paces rate and says nothing about depth, so this is the one instruction in the corpus with a safety rationale for appearing in the app.

---

## Gaps and unsupported choices

Written by hand. This section is the point of the exercise: everything below is a place where exhale
ships something the literature doesn't settle, or where the evidence is thinner or more divided
than a bare citation would suggest. Nothing here is a reason not to use the app. It's a list of
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
belly breathing. It's a theory of mechanism. No study in this corpus measures chest against
diaphragm breathing in screen users.

Two things limit how much the keyboard can be blamed. The 1994 study compares work with
relaxation and the 2008 study compares high with low mental workload during data entry, and [`grassmann2016-cognitive-load-respiration`](#grassmann2016-cognitive-load-respiration)
finds the same faster, over-ventilated pattern under any demanding task, so the keyboard is where
the effect was measured rather than shown to be its cause. And the two samples total 34 people
from one research group.

A second, independent line runs through posture.
[`jung2016-smartphone-posture-respiration`](#jung2016-smartphone-posture-respiration) found that
people using smartphones more than four hours a day had significantly worse craniovertebral angle
and lower peak expiratory flow than lighter users, and
[`deniz2024-forward-head-lung-volumes`](#deniz2024-forward-head-lung-volumes) found forward head
posture associated with FVC reductions of 0.25 to 0.81 L across 115 participants.

**What the evidence doesn't support** is "shallow" meaning a reduced *volume of air moved*.
[`grassmann2016-cognitive-load-respiration`](#grassmann2016-cognitive-load-respiration), 54
experiments, finds respiratory amplitude roughly stable and minute ventilation **up** under
cognitive load. Both things can hold at once: more air per minute, on the theory above moved by
the wrong muscles, from a slumped posture associated with less capacity.

The defensible statement is therefore that during demanding work at a keyboard people breathe
**faster and slightly over-ventilated**, from a posture associated with reduced lung volumes; the
shift to chest breathing is the theory of why; no study measures it. That's the claim the README
makes.

It isn't the same claim as "screen apnea" or "email apnea," meaning outright breath-*holding* at a
screen. That framing traces to unpublished observations by Linda Stone from 2007, tested informally
on acquaintances with no protocol, no published data and no replication. Nothing found in this pass
measures breath-holding during screen use, and the over-breathing finding above points the other
way. The two claims shouldn't be run together.

### 2. The default is inside the tested band, but the band is narrow and resonance frequency is individual

exhale ships 5 s inhale and 5 s exhale, no holds
([`rust/crates/exhale-core/src/settings.rs`](../rust/crates/exhale-core/src/settings.rs)): a
10-second cycle, or **6.0 breaths per minute**.

[`you2023-respiratory-frequency`](#you2023-respiratory-frequency) tested 5, 5.5, 6, 6.5 and 7 cycles
per minute and found all of them raised cardiac vagal activity above spontaneous breathing.
[`marchant2025-square-478-six`](#marchant2025-square-478-six) ran 6 a minute against square and
4-7-8 breathing head-to-head, n = 84, and 6 won. So the default is the pace with the most direct
support of anything exhale could have shipped.

That's a weaker statement than it sounds, for two reasons.

**The tested band is five values wide.** Nobody has compared 6 against 3, or against 8, in this
corpus. "Inside the range that has been tested" is a statement about coverage; it says nothing about which rate is best.

**Resonance frequency is individual.** [`lehrer2014-hrv-biofeedback`](#lehrer2014-hrv-biofeedback)
puts the average at about 5.5 breaths per minute and is explicit that it varies from person to
person; [`lin2014-equal-ratio-hrv`](#lin2014-equal-ratio-hrv) found 5.5 outperformed 6. A single
shipped number can't be right for everyone, and finding a person's own resonance frequency takes an
assessment protocol and a sensor, neither of which exhale has. The default is a reasonable starting
point rather than a personalised one.

Anyone who already has exhale installed keeps whatever they had: the timing fields carry no
`#[serde(default)]`, so an existing `settings.toml` is untouched and the change reaches only fresh
installs and Reset to Defaults. The previous default, `5` / `0` / `10` / `0` at 4 a minute, is still
offered as a one-click preset. No primary study in this corpus has measured 4 a minute, which is why
it's no longer what a new user gets without asking. The one report is secondhand:
[`lehrer2022-my-life-hrvb`](#lehrer2022-my-life-hrvb) recalls Song and Lehrer 2003 pacing people from 3
to 14 a minute and finding RSA amplitude highest at 4 and 6, with a decrease at 3. That paper isn't in
the corpus, and RSA amplitude is a heart-rate measure rather than a report of how anyone felt.

### 3. Box breathing is slower than it looks

`4` / `4` / `4` / `4` is a 16-second cycle, or **3.75 breaths per minute**: *slower* than exhale's
default and further below the tested band. The holds hide the rate, which is why
the settings panel computes it.

Box breathing has also been tested head-to-head twice and didn't win either time:

- [`marchant2025-square-478-six`](#marchant2025-square-478-six), n = 84, compared square breathing,
  4-7-8 breathing, and 6 breaths per minute at two ratios. Breathing at 6 raised HRV **more than
  either square or 4-7-8**, with small to medium effects. The paper opens by stating that square and
  4-7-8 "are popularly promoted by psychotherapists but have little empirical support."
- [`balban2023-cyclic-sighing`](#balban2023-cyclic-sighing), a randomised trial registered after
  the fact rather than pre-registered, tested box breathing, cyclic sighing and cyclic
  hyperventilation over a month against a mindfulness-meditation control. Cyclic sighing, the exhale-emphasising arm, separated from the
  control on positive affect; box breathing didn't. The box arm had 21 people, the arms weren't
  tested against each other, and their daily gains were 1.84 and 1.89 points, so this is a
  difference in reaching significance rather than a demonstrated gap.

The same evidence applies to 4-7-8, sometimes attributed to Andrew Weil: it lost in Marchant, and at
4+7+8 = 19 s it's 3.16 breaths per minute, slower still.

The pattern with the best direct support is `5` / `0` / `5` / `0`: 6 breaths per minute, no holds, a
10-second cycle. It sits inside the tested band, it's at the rate that won in Marchant, and
holding the breath is the harder part for a beginner rather than the slow part. One honest
qualification: Marchant ran 6 a minute at two ratios, and on RMSSD the 4:6 ratio carried the
significant differences over the hold patterns while the 5:5 ratio's edge (0.19 and 0.18 SD)
wasn't significant after Scheffé correction (p = 0.10 and 0.15). The two ratios didn't differ significantly from each other (p = 0.07),
so 6 a minute is what won, and the choice of 5:5 over 4:6 within it rests on gap 4, where the
ratio evidence splits. It's what exhale
defaults to. Box breathing remains a reasonable thing to want, and exhale offers it as a preset. It's
simply not the pattern the evidence points at.

### 4. Longer exhale: contested on the heart, thin on how people feel

On HRV, six results, and they don't line up:

| Source | n | Design | Result on ratio |
|---|---|---|---|
| [`bae2021-exhalation-inhalation-ratio`](#bae2021-exhalation-inhalation-ratio) | 28 | 2:1 vs 1:1 cue at spontaneous rate | Longer exhale raised RMSSD and HF-HRV |
| [`vandiest2014-ie-ratio-relaxation`](#vandiest2014-ie-ratio-relaxation) | 23 (30 recruited) | i/e 0.42 vs 2.33 targets (0.49 vs 1.44 achieved), at 6 and 12 bpm (7.3 to 7.7 achieved at the slow rate) | Longer exhale raised HF-HRV, but only at the slow rate |
| [`lin2014-equal-ratio-hrv`](#lin2014-equal-ratio-hrv) | 47 | 5:5 vs 4:6 at 5.5 and 6 bpm | **Equal** ratio won on SDNN and LF |
| [`laborde2021-ie-ratio-pauses`](#laborde2021-ie-ratio-pauses) | 64 | i/e 0.8, 1.0, 1.2 at 6 bpm, with and without 0.4 s pauses | Longer exhale raised RMSSD; pauses changed nothing |
| [`meehan2024-longer-exhalations`](#meehan2024-longer-exhalations) | 26 + 16 replication | 1:1 vs 1:2 at 6 bpm | No difference, in the original *and* the replication |
| [`marchant2025-square-478-six`](#marchant2025-square-478-six) | 84 | 5:5 vs 4:6 at 6 bpm | No significant ratio difference on RMSSD (4:6 ahead by 0.20 SD, p = 0.07), LF-HRV or end-tidal CO2 |

Three for, one against, two nulls inside this corpus.
[`meehan2024-longer-exhalations`](#meehan2024-longer-exhalations)'s introduction tallies the older
literature as three further nulls and one result favouring the longer inhale. No mechanism claim
survives that split, which is why exhale doesn't make one.

On how people reported feeling, the picture isn't cleaner. Three studies here measured subjective
state across ratios. [`vandiest2014-ie-ratio-relaxation`](#vandiest2014-ie-ratio-relaxation), n = 23,
found more relaxation, stress reduction, mindfulness and positive energy with the longer exhale, and
slowing the rate alone moved only one of those four.
[`lin2014-equal-ratio-hrv`](#lin2014-equal-ratio-hrv), n = 47, found every slow pattern raised
relaxation over baseline, and its abstract doesn't say whether the patterns differed from each other.
[`marchant2025-square-478-six`](#marchant2025-square-478-six), n = 84 and the largest of the three,
found no mood change at either of its two 6-per-minute ratios, and only uncorrected small
decreases in positive mood after square and 4-7-8.
[`balban2023-cyclic-sighing`](#balban2023-cyclic-sighing) points toward exhale emphasis over a
month, but its cyclic-sighing arm also adds a double inhale, so the ratio can't be isolated.

So a longer exhale is offered as a preference. One study of 23 people found it felt better, a
larger one found no difference, and rate does most of the work either way: every slow pattern in
[`lin2014-equal-ratio-hrv`](#lin2014-equal-ratio-hrv) beat baseline on relaxation.

### 5. Pacing someone slowly at a screen may push them further into over-breathing

A genuine tension between two entries that nobody has looked at.

[`marchant2025-square-478-six`](#marchant2025-square-478-six) reports, as an unexpected finding, that
breathing at 6 breaths per minute produced **mild over-breathing**: HRV went up and end-tidal CO2
went down, into the 30 to 35 mmHg range.
[`vandiest2014-ie-ratio-relaxation`](#vandiest2014-ie-ratio-relaxation) reports the same from the other
direction: tidal volume rose from 532 ml at baseline to 1262 and 1383 ml in its slow conditions, and
the authors suspect their participants were hyperventilating. Meanwhile [`schleifer1994-vdt-petco2`](#schleifer1994-vdt-petco2) and
[`schleifer2008-emg-gaps-computer-work`](#schleifer2008-emg-gaps-computer-work) show that a person at
a keyboard *already* runs end-tidal CO2 lower than at rest: one to two mmHg lower in the 1994
study, a change its authors call relatively small and a hyperventilatory stress effect, and lower
again under higher mental workload in the 2008 study, so over-breathing relative to rest rather than
hypocapnia.

exhale paces rate and says nothing about depth. A user who slows to 6 breaths per minute while taking
large breaths moves more air per minute. No study in this corpus tests a slow pacer on a screen
worker who is already over-breathing, which is exactly exhale's user.

This isn't a safety warning: the effect Marchant reports is mild and was measured in a single
session. It's recorded because it's the most interesting unanswered question this corpus turned up,
and because an app that paces breathing should know that pacing rate isn't the same as pacing
volume. The one mitigation with evidence behind it is an instruction rather than a setting:
[`szulczewski2019-antihyperventilation-instruction`](#szulczewski2019-antihyperventilation-instruction)
shows a two-sentence anti-hyperventilation instruction cuts the end-tidal CO2 drop from 5.21 mmHg to
2.7, which is roughly half rather than all of it.

### 6. `drift` is an invention of this app

`drift` lengthens every cycle by a fixed percentage, compounding, so the breath extends gradually
across a session. Graded extension of the breath is a long-standing pranayama practice
([`satyananda1999-apmb`](#satyananda1999-apmb)), but **no study in this corpus examines a
progressively lengthening pace at all.** [`szulczewski2019-training-relaxation`](#szulczewski2019-training-relaxation)
trained at a fixed rate and found relaxation accrued over a week of practice, which supports "keep
practising" rather than "keep slowing down within a session."

Nothing in this review contradicts the tradition either. The primary studies in this corpus stop below
about 5 breaths a minute and don't report worse outcomes there. The one report from slower pacing is
secondhand and about HRV amplitude: [`lehrer2022-my-life-hrvb`](#lehrer2022-my-life-hrvb) recalls RSA
amplitude lower at 3 a minute than at 4 and 6, and induced HRV amplitude at a minimum around 2 to 3.
Both are the same kind of measure as the inverted-U below, and neither says anything about relaxation
or comfort. What subjective evidence
exists points the tradition's way:
[`vandiest2014-ie-ratio-relaxation`](#vandiest2014-ie-ratio-relaxation) found the longer exhale
produced more relaxation, stress reduction and positive energy;
[`lin2014-equal-ratio-hrv`](#lin2014-equal-ratio-hrv) found every slow pattern beat baseline on
relaxation; and [`joshi1992-pranayam-training`](#joshi1992-pranayam-training) found six weeks of
practice lowered resting respiratory rate and lengthened breath-holding time. The inverted-U in
[`shaffer2020-resonance-frequency-assessment`](#shaffer2020-resonance-frequency-assessment) is worth
reading alongside these, but it describes a peak in **HRV amplitude** rather than in relaxation or comfort,
and doesn't transfer to one.

The one documented caution is about **depth rather than rate**. See gap 5.

`drift` is therefore unbounded and **defaults to 0, off**. Off by default is a coverage argument
rather than a claim of harm: on by default it would move every new user out of the region anyone has
measured, within minutes, without asking. Unbounded because a 10-second inhale with a 20-second
exhale is unremarkable in pranayama, and absence of research isn't evidence of harm.

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
fixed rate; that's what "paced" means. The nearest adjacent literature is
[`vlemincx2013-sigh-reset-model`](#vlemincx2013-sigh-reset-model), on natural respiratory
variability and sighs, which is about spontaneous breathing rather than about deliberately
destabilising a pacer. Defaults are 0, which is the right default. Treat the sliders as an
aesthetic option.

### 8. Nothing about exhale itself has ever been measured

No study in this corpus is about exhale. The closest published analogue is
[`moraveji2011-peripheral-paced-respiration`](#moraveji2011-peripheral-paced-respiration): a
translucent animated bar across the screen, running in the periphery during real information work,
which significantly lowered participants' breathing rate. Two things about it differ from exhale:
it was sensor-triggered and intermittent, running in two-minute bursts when the wearer's breathing
rose 20 percent above their resting rate or at least once every six minutes, and its target was set
per person at 20 percent below an eyes-closed resting rate that averaged 9.3 a minute, so near 7.5
rather than 6. On average they didn't get there: breathing averaged 15 a minute while the pacer was
on. n = 13, and the drop was 1.9 breaths a minute. Its limitation is the one that
matters here. The reduction happened **while the pacing was active** and didn't persist as a lasting change
in respiratory pattern. An always-on overlay should be understood as an effect that lasts as long as
it's on.

The strongest design warrant is [`tabor2022-guided-breathing-design`](#tabor2022-guided-breathing-design):
an expanding and contracting circle at 6 breaths/min matched sensor-driven HRV biofeedback on
breathing rate and LF power, and its HRV amplitude gain was larger (interaction p = 0.03) though the
authors call the two comparable, with effects appearing in about two minutes and no hardware needed. That's exhale's
Circle mode, and it's why exhale needs no sensor, no account and no telemetry. It's still n = 28
in one session.

### 9. Visual-only guidance is the weaker modality for the outcome users care about

[`wongsuphasawat2012-cant-force-calm`](#wongsuphasawat2012-cant-force-calm) found auditory pacing
was rated more calming than visual, while the visual guide's larger change in breathing rate was
only a non-significant trend (p = 0.26). Its visual arm was a biofeedback display of the user's own
breathing, which exhale isn't.
exhale is visual-only by design, because it's meant to sit silently in the corner of a working
screen. That's a real trade-off against felt calm, made deliberately. The source is a two-page
adjunct paper with 14 participants, so it's a signal rather than a result.

### 10. exhale cites blink research and does nothing about blinking

The blink-rate literature is well supported, and exhale doesn't act on it. The overlay paces
breathing; it doesn't prompt a blink, doesn't detect blinks, and doesn't implement anything from
the digital eye strain literature. The blink finding is context for why screens deserve a nudge rather than
a description of what this app does.

Also relevant to exhale's whole genre: [`johnson2023-20-20-20`](#johnson2023-20-20-20) found that
scheduled 20-second breaks at any of three intervals produced no significant effect on symptoms,
reading speed or accuracy. A periodic on-screen nudge isn't effective merely because it's popular.

### 11. Both hold sliders default to 0; brief pauses are neutral and longer holds are untested

`post_inhale_hold_duration` and `post_exhale_hold_duration` both default to 0.
[`laborde2021-ie-ratio-pauses`](#laborde2021-ie-ratio-pauses) is the study that manipulates
respiratory pauses directly: at 6 cycles per minute, adding 0.4 s pauses after inhalation and after
exhalation didn't change RMSSD. [`little2025-a52-breath-method`](#little2025-a52-breath-method)
argues for a 2 s post-exhale hold, and [`marchant2025-square-478-six`](#marchant2025-square-478-six)
found the two hold-heavy patterns it tested underperformed a no-hold 6 bpm pace. Brief pauses are
neutral and longer holds are untested at exhale's rates, so 0 is a defensible default.

### 12. Nobody knows whether anyone keeps using it

exhale has no telemetry, by design and stated in [PRIVACY.md](../PRIVACY.md). The consequence is
that the single biggest determinant of whether a tool like this does anything, namely whether people
keep it running, is unmeasured and unmeasurable here.
[`linardon2020-app-attrition`](#linardon2020-app-attrition) is the reality check: across 70
trials of smartphone-delivered mental health interventions, attrition averaged 24.1% at short-term
and 35.5% at longer-term follow-up, usage consistently declined over the course of the trials, and
the authors say this may undermine the validity of the findings. Every effect size in this corpus was measured in a supervised session with a compliant
participant. That's not the same population as someone who installed a menu-bar app in March.

### 13. Adverse-event reporting in this field is thin

[`fincham2023-breathwork-meta`](#fincham2023-breathwork-meta) found that only four of its twelve
primary-outcome trials reported on adverse events at all, none attributing lasting harm to
breathwork. [`laborde2022-vsb-meta`](#laborde2022-vsb-meta) concludes that few adverse effects are expected from
*slow* breathing specifically, which is the mode exhale is built around. exhale's sliders can also
be set to fast, hold-heavy patterns that leave that evidence base entirely; those belong to the
high-ventilation literature ([`fincham2024-high-ventilation-rct`](#fincham2024-high-ventilation-rct)),
where transient tetany, light-headedness and distress are documented. This is the basis for the
README's advice to take breaks if intense feelings arise.

### 14. The tradition sources are lineage rather than evidence, and are tiered accordingly

exhale's four-phase structure, inhale / retention / exhale / retention, is pranayama. It didn't come
from psychophysiology, and the corpus says so: [`satyananda1999-apmb`](#satyananda1999-apmb) and
[`muktibodhananda1998-hatha-yoga-pradipika`](#muktibodhananda1998-hatha-yoga-pradipika) are carried
at tier **E**, meaning they may be cited for where a practice came from and never for whether it
works.

This is a deliberate inclusion rather than an endorsement, for two reasons. Retrofitting a 2020s HRV
citation onto an instruction that's centuries older would be revisionist about the app's actual
design history. And the "longer exhale" idea specifically entered modern breathing apps through this
tradition rather than through a laboratory, which matters when weighing how much of the supporting
literature was designed to test a pre-existing belief rather than to discover something.

Both entries are catalogue records only: checked against Open Library, contents not consulted, and
no claim in this repository rests on them. The Satyananda entry cites the 1999 third revised edition
its ISBN resolves to; a 2008 fourth revised edition exists under the same imprint.

## What would close the biggest gaps

In rough order of value per unit effort:

1. Read the two entries still cited from their abstract only,
   [`linardon2020-app-attrition`](#linardon2020-app-attrition) and
   [`vlemincx2016-sigh-relief`](#vlemincx2016-sigh-relief), and re-read in full the ten entries
   whose full text couldn't be reached on the 2026-09-20 pass and were checked against their
   abstracts instead. The effect sizes of
   [`laborde2021-ie-ratio-pauses`](#laborde2021-ie-ratio-pauses) are now in its entry.
2. Nobody has tested a slow visual pacer on a screen worker who is already over-breathing (gap 5). That's a
   real, publishable question that exhale is unusually well placed to ask.
3. Settle whether graded extension does anything, which would put a floor under gap 6. No study in
   this corpus varies the pace *within* a session, so the question is open in both directions.
4. Resonance frequency is individual (gap 2) and exhale ships one number for everyone. Whether a
   sensorless app can help someone find their own, by any method better than trying a few and
   noticing, is unresolved and would matter more than the default ever will.
