# Citation corpus

48 sources: 45 Crossref-verified, 2 verified against Open Library, 1 verified against PubMed. 41 were read in full, 4 from the abstract only, and 3 are catalogue records only; each entry says which. 2 are not peer-reviewed and are tiered E so they can back lineage but never a claim.

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

### Counts

| Verification | n |
|---|---|
| crossref-verified | 45 |
| openlibrary-verified | 2 |
| pubmed-verified | 1 |
| **total** | **48** |

| Access level | n |
|---|---|
| open-access | 21 |
| paywalled | 27 |
| **total** | **48** |

| Read depth | n |
|---|---|
| abstract | 4 |
| full-text | 41 |
| record | 3 |
| **total** | **48** |

| Evidence tier | n |
|---|---|
| A | 6 |
| B | 4 |
| C | 23 |
| D | 12 |
| E | 2 |
| null (not a study) | 1 |
| **total** | **48** |

---

## Why a breathing reminder next to a screen

11 sources.

#### `albulescu2022-micro-breaks`

Albulescu, Patricia; Macsinga, Irina; Rusu, Andrei; Sulea, Coralia; Bodnaru, Alexandra; Tulbure, Bogdan Tudor. (2022). *"Give me a break!" A systematic review and meta-analysis on the efficacy of micro-breaks for increasing well-being and performance*. PLOS ONE 17(8): e0272460

- DOI: [10.1371/journal.pone.0272460](https://doi.org/10.1371/journal.pone.0272460)
- Verification: crossref-verified | Access: open-access | Read: full text | evidence tier **A**
- Backs:
  - short breaks of ten minutes or less from a work task reduce fatigue and increase vigor
  - an interruption long enough to run one breath cycle is a plausible unit of recovery
- Caveat: 22 studies, 19 manuscripts. The performance effect was smaller and less consistent than the well-being effect, and the authors report it depended on task type. None of the included studies used a breathing exercise as the break activity, so this supports the general shape of exhale's reminder timer, not its specific content.

#### `deniz2024-forward-head-lung-volumes`

Deniz, Yasemin; Ertekin, Damla; Çokar, Dilek. (2024). *The effect of forward head posture on dynamic lung volumes in young adults: a systematic review*. Bulletin of Faculty of Physical Therapy 29(1): 15

- DOI: [10.1186/s43161-024-00186-7](https://doi.org/10.1186/s43161-024-00186-7)
- Verification: crossref-verified | Access: open-access | Read: full text | evidence tier **C**
- Backs:
  - across four comparison studies totalling 115 participants, forward head posture was associated with FVC reductions of 0.25 to 0.81 L and FEV1 reductions of 0.16 to 0.93 L
  - craniovertebral angle correlates positively with dynamic pulmonary volumes
- Caveat: Systematic review without meta-analysis of four small comparison studies and two correlation studies; the authors phrase the conclusion as forward head posture 'can potentially cause' pulmonary abnormalities. The search included ResearchGate alongside PubMed and Google Scholar, which is an unusual choice. This is the posture half of the screen-breathing argument: it is about head position, not about screens, and the link to screens comes from jung2016-smartphone-posture-respiration. The Crossref deposit carries the second and third authors' names with a dotless i, a transliteration artefact; they are printed here in standard Turkish orthography.

#### `grassmann2016-cognitive-load-respiration`

Grassmann, Mariel; Vlemincx, Elke; von Leupoldt, Andreas; Mittelstädt, Justin M.; Van den Bergh, Omer. (2016). *Respiratory Changes in Response to Cognitive Load: A Systematic Review*. Neural Plasticity 2016: 8146809

- DOI: [10.1155/2016/8146809](https://doi.org/10.1155/2016/8146809)
- Verification: crossref-verified | Access: open-access | Read: full text | evidence tier **B**
- Backs:
  - mentally demanding work is reliably marked by faster breathing, with medium to large effects
  - respiratory amplitude stays roughly stable under cognitive load rather than shrinking
  - cognitive load lowers end-tidal CO2, meaning ventilation runs ahead of metabolic need
- Caveat: 54 experiments across 53 articles. Note the direction of the amplitude finding: cognitive load raises rate while leaving amplitude roughly stable, so this does not support 'shallow' if shallow means less air moved. Minute ventilation goes up. For the diaphragmatic-to-thoracic shift that 'shallow' usually names, see schleifer2002-hyperventilation-job-stress; for the same over-breathing measured at a keyboard, see schleifer1994-vdt-petco2 and schleifer2008-emg-gaps-computer-work. This entry is the general cognitive-load backdrop, and it is also the reason the keyboard cannot be singled out as the cause: any demanding task produces the pattern.

#### `johnson2023-20-20-20`

Johnson, Sophia; Rosenfield, Mark. (2023). *20-20-20 Rule: Are These Numbers Justified?* Optometry and Vision Science 100(1): 52-56

- DOI: [10.1097/OPX.0000000000001971](https://doi.org/10.1097/OPX.0000000000001971)
- Verification: crossref-verified | Access: paywalled | Read: full text | evidence tier **C**
- Backs:
  - the widely repeated 20-20-20 rule has little peer-reviewed support
  - scheduled 20-second breaks every 5, 10 or 20 minutes produced no significant effect on ocular symptoms, reading speed or task accuracy
- Caveat: n = 30, 40-minute tablet reading task, four break schedules. Symptoms rose significantly in all four conditions including the most frequent break schedule. Included here as a caution against exhale's own genre: a periodic on-screen nudge is not self-evidently effective just because it is popular and plausible.

#### `jung2016-smartphone-posture-respiration`

Jung, Sang In; Lee, Na Kyung; Kang, Kyung Woo; Kim, Kyoung; Lee, Do Youn. (2016). *The effect of smartphone usage time on posture and respiratory function*. Journal of Physical Therapy Science 28(1): 186-189

- DOI: [10.1589/jpts.28.186](https://doi.org/10.1589/jpts.28.186)
- Verification: crossref-verified | Access: open-access | Read: full text | evidence tier **C**
- Backs:
  - people using smartphones more than four hours a day had significantly worse craniovertebral angle, worse scapular index and lower peak expiratory flow than people using them less than four hours a day
- Caveat: n = 50, cross-sectional, two groups split on self-reported usage. Correlational: heavy users may differ in many ways besides screen time, and peak expiratory flow was the only respiratory measure that separated (FVC and FEV1 did not). Crossref carries no license; access level taken from J Phys Ther Sci being fully open access.

#### `rosenfield2011-computer-vision-syndrome`

Rosenfield, Mark. (2011). *Computer vision syndrome: a review of ocular causes and potential treatments*. Ophthalmic and Physiological Optics 31(5): 502-515

- DOI: [10.1111/j.1475-1313.2011.00834.x](https://doi.org/10.1111/j.1475-1313.2011.00834.x)
- Verification: crossref-verified | Access: paywalled | Read: full text | evidence tier **D**
- Backs:
  - blink rate falls substantially during display work relative to other tasks
  - reduced and incomplete blinking is a principal mechanism of computer vision syndrome
- Caveat: Narrative review of the ocular literature. Load-bearing for the blink half of exhale's opening claim only. exhale does nothing about blinking; see the gaps ledger.

#### `schleifer1994-vdt-petco2`

Schleifer, Lawrence M.; Ley, Ronald. (1994). *End-tidal PCO2 as an index of psychophysiological activity during VDT data-entry work and relaxation*. Ergonomics 37(2): 245-254

- DOI: [10.1080/00140139408963642](https://doi.org/10.1080/00140139408963642)
- Verification: crossref-verified | Access: paywalled | Read: full text | evidence tier **C**
- Backs:
  - during computer data-entry work, end-tidal CO2 was significantly lower and respiration frequency significantly higher than during either baseline relaxation or progressive muscle relaxation
  - breathing changes measurably at a screen, and end-tidal CO2 discriminates the state
- Caveat: n = 11 data-entry operators monitored continuously across three consecutive six-hour work days. Small sample, but this is real work over real working days, not a lab proxy. Note the comparison: data entry against two relaxation conditions, so it shows what demanding work at a keyboard does, not that the screen itself is the cause; grassmann2016-cognitive-load-respiration finds the same pattern under cognitive load generally. Along with schleifer2008-emg-gaps-computer-work it is the direct evidence that keyboard work changes respiration, and it dates from 1994, well before the topic reached breathwork writing.

#### `schleifer2002-hyperventilation-job-stress`

Schleifer, Lawrence M.; Ley, Ronald; Spalding, Thomas W. (2002). *A hyperventilation theory of job stress and musculoskeletal disorders*. American Journal of Industrial Medicine 41(5): 420-432

- DOI: [10.1002/ajim.10061](https://doi.org/10.1002/ajim.10061)
- Verification: crossref-verified | Access: paywalled | Read: full text | evidence tier **D**
- Backs:
  - hyperventilation is often characterised by a shift from a diaphragmatic to a thoracic breathing pattern
  - thoracic breathing recruits sternocleidomastoid, scalene and trapezius muscles, imposing biomechanical stress on the neck and shoulder region
  - breathing training and rest breaks are a rationale-backed response to this pattern at work
- Caveat: Theory paper, not an experiment, hence tier D. It is nonetheless the closest thing in the peer-reviewed literature to the folk claim about 'shallow' breathing at a screen: the diaphragmatic-to-thoracic shift is what people mean by shallow. Cite it for the pattern, never for a measured tidal volume, and note it theorises the shift rather than demonstrating it in screen users.

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
- Caveat: Crossref carries no license field for this record; access level is taken from BMJ Open Ophthalmology being a fully open-access title.

#### `tsubota1993-vdt-blink`

Tsubota, Kazuo; Nakamori, Katsu. (1993). *Dry Eyes and Video Display Terminals*. New England Journal of Medicine 328(8): 584

- DOI: [10.1056/NEJM199302253280817](https://doi.org/10.1056/NEJM199302253280817)
- Open copy: <https://www.nejm.org/doi/pdf/10.1056/NEJM199302253280817>
- Verification: crossref-verified | Access: paywalled | Read: catalogue record only | evidence tier **C**
- Backs:
  - blink rate during display work is far below blink rate at rest
- Caveat: Not read. This is a one-page correspondence item rather than a full paper. It is the origin of the widely quoted '22 blinks a minute at rest, 7 during screen work' figures, which circulate almost entirely by secondary citation. Cite it for the direction of the effect. For a magnitude, use rosenfield2011-computer-vision-syndrome or sheppard2018-digital-eye-strain instead.

---

## Whether slow paced breathing does anything

7 sources.

#### `chaddha2019-slow-breathing-bp`

Chaddha, Ashish; Modaff, Daniel; Hooper-Lane, Christopher; Feldstein, David A. (2019). *Device and non-device-guided slow breathing to reduce blood pressure: A systematic review and meta-analysis*. Complementary Therapies in Medicine 45: 179-184

- DOI: [10.1016/j.ctim.2019.03.005](https://doi.org/10.1016/j.ctim.2019.03.005)
- Verification: crossref-verified | Access: paywalled | Read: full text | evidence tier **A**
- Backs:
  - sustained slow breathing programmes lower systolic blood pressure by about 5.6 mmHg and diastolic by about 3.0 mmHg in hypertensive and prehypertensive adults
- Caveat: 17 RCTs. Heterogeneity was high for every analysis, and the authors say so. Inclusion required at least 5 minutes of breathing at 10 breaths/min or slower, on at least 3 days a week, for at least 4 weeks. exhale asks for none of that and measures none of it, so this describes a dose exhale does not deliver. Read alongside vandijk2018-close-the-book.

#### `fincham2023-breathwork-meta`

Fincham, Guy William; Strauss, Clara; Montero-Marin, Jesus; Cavanagh, Kate. (2023). *Effect of breathwork on stress and mental health: A meta-analysis of randomised-controlled trials*. Scientific Reports 13(1): 432

- DOI: [10.1038/s41598-022-27247-y](https://doi.org/10.1038/s41598-022-27247-y)
- Verification: crossref-verified | Access: open-access | Read: full text | evidence tier **A**
- Backs:
  - breathwork lowers self-reported stress against control conditions, g = -0.35 (95% CI -0.55 to -0.14), 12 RCTs, 785 adults
  - comparable small-to-medium effects for anxiety (g = -0.32, k = 20) and depressive symptoms (g = -0.40, k = 18)
  - adverse-event reporting in breathwork trials is sparse: four of the twelve primary-outcome trials reported on it, and none attributed lasting harm to breathwork
- Caveat: Most included studies were rated at moderate risk of bias. Effects are small-to-medium and self-reported. Only four of the twelve primary-outcome trials reported on adverse events, and none attributed lasting harm to breathwork; the authors call for better reporting, particularly for fast-paced techniques. This is the strongest single warrant for the claim that a breathing practice does something, and it is still an effect of about a third of a standard deviation.

#### `laborde2022-vsb-meta`

Laborde, S.; Allen, M. S.; Borges, U.; Dosseville, F.; Hosang, T. J.; Iskra, M.; Mosley, E.; Salvotti, C.; Spolverato, L.; Zammit, N.; Javelle, F. (2022). *Effects of voluntary slow breathing on heart rate and heart rate variability: A systematic review and a meta-analysis*. Neuroscience & Biobehavioral Reviews 138: 104711

- DOI: [10.1016/j.neubiorev.2022.104711](https://doi.org/10.1016/j.neubiorev.2022.104711)
- Open copy: <https://pure.solent.ac.uk/en/publications/74890190-b567-4d13-b1d6-e0bd6b06431f>
- Verification: crossref-verified | Access: paywalled | Read: full text | evidence tier **A**
- Backs:
  - voluntary slow breathing raises vagally-mediated HRV during the session, immediately after a single session, and after a multi-session intervention
  - few adverse effects are expected from slow breathing practice
- Caveat: 223 studies from 1842 screened abstracts (172 during, 16 immediately-after, 49 after-intervention). This is the central warrant for exhale's whole premise. Note what it establishes: an effect on a cardiac index of parasympathetic activity, not an effect on how anyone feels or works.

#### `little2025-a52-breath-method`

Little, Abbie L. (2025). *The A52 Breath Method: A Narrative Review of Breathwork for Mental Health and Stress Resilience*. Stress and Health 41(4): e70098

- DOI: [10.1002/smi.70098](https://doi.org/10.1002/smi.70098)
- Verification: crossref-verified | Access: open-access | Read: full text | evidence tier **D**
- Backs:
  - a 5 s inhale, 5 s exhale, 2 s post-exhale hold at five breaths per minute is a protocol a recent review argues is representative of the effective literature
  - 23 of 30 reviewed studies reported significant HRV improvement; 10 reported anxiety reduction and 9 reported reduced perceived stress
  - benefits appear larger in people with elevated baseline distress
- Caveat: Narrative review, single author, 465 abstracts screened and 30 full texts analysed, with no meta-analysis and no risk-of-bias assessment. It proposes the protocol it reviews, which is a conflict of framing worth naming. Its A52 shape maps onto exhale's four sliders as 5 / 0 / 5 / 2, which is 5 breaths per minute and inside the tested band. marchant2025-square-478-six found that no-hold 6 bpm outperformed both hold-bearing patterns it tested, so the 2 s retention is the least supported part of the protocol.

#### `russo2017-slow-breathing-physiology`

Russo, Marc A.; Santarelli, Danielle M.; O'Rourke, Dean. (2017). *The physiological effects of slow breathing in the healthy human*. Breathe 13(4): 298-309

- DOI: [10.1183/20734735.009817](https://doi.org/10.1183/20734735.009817)
- Verification: crossref-verified | Access: open-access | Read: full text | evidence tier **D**
- Backs:
  - slow breathing improves ventilation efficiency and alters cardiorespiratory coupling, respiratory sinus arrhythmia and sympathovagal balance
- Caveat: Narrative review. The authors close by calling explicitly for further research, and describe the health claims as potential rather than demonstrated. Use it to explain a mechanism, not to assert an outcome.

#### `vandijk2018-close-the-book`

van Dijk, Peter R.; van Hateren, Kornelis J. J.; Kleefstra, Nanne; Landman, Gijs W. D. (2018). *It is time to close the book on device-guided slow breathing*. Blood Pressure 27(3): 181-182

- DOI: [10.1080/08037051.2018.1435260](https://doi.org/10.1080/08037051.2018.1435260)
- Open copy: <https://www.tandfonline.com/doi/pdf/10.1080/08037051.2018.1435260>
- Verification: crossref-verified | Access: paywalled | Read: full text | no evidence tier (not a study)
- Backs:
  - a body of specialist opinion holds that the blood-pressure case for device-guided slow breathing is weak and should be considered closed
- Caveat: Letter, two pages, not a study, hence no evidence tier. Carried deliberately: it is the strongest published dissent against the cardiovascular claims exhale sits next to, and a corpus that omitted it would be advocacy rather than provenance.

#### `zaccaro2018-slow-breathing-review`

Zaccaro, Andrea; Piarulli, Andrea; Laurino, Marco; Garbella, Erika; Menicucci, Danilo; Neri, Bruno; Gemignani, Angelo. (2018). *How Breath-Control Can Change Your Life: A Systematic Review on Psycho-Physiological Correlates of Slow Breathing*. Frontiers in Human Neuroscience 12: 353

- DOI: [10.3389/fnhum.2018.00353](https://doi.org/10.3389/fnhum.2018.00353)
- Verification: crossref-verified | Access: open-access | Read: full text | evidence tier **B**
- Backs:
  - slow breathing, conventionally defined as under 10 breaths per minute, is associated with increased HRV and shifts in central and autonomic measures
  - reported psychological correlates include increased comfort and relaxation and reduced anxiety and arousal
- Caveat: Systematic review without meta-analysis. The included studies vary widely in technique, rate and duration, so the pooled picture is directional rather than dose-specific.

---

## What the numbers should be

14 sources.

#### `bae2021-exhalation-inhalation-ratio`

Bae, Dalbyeol; Matthews, Jacob J. L.; Chen, J. Jean; Mah, Linda. (2021). *Increased exhalation to inhalation ratio during breathing enhances high-frequency heart rate variability in healthy adults*. Psychophysiology 58(11): e13905

- DOI: [10.1111/psyp.13905](https://doi.org/10.1111/psyp.13905)
- Verification: crossref-verified | Access: paywalled | Read: full text | evidence tier **C**
- Backs:
  - a 2:1 exhale-to-inhale cue raised RMSSD and HF-HRV relative to a 1:1 cue at the participant's own breathing rate
  - the HF-HRV elevation persisted about four minutes after the 2:1 block ended
- Caveat: n = 28 (16 young, 12 older). Note the manipulation check: the achieved ratios were 1.08 and 1.33, well short of the instructed 1:1 and 2:1, and pacing was at each participant's spontaneous rate rather than in the resonance range. One of five studies in this corpus that disagree about the ratio; see vandiest2014-ie-ratio-relaxation and laborde2021-ie-ratio-pauses (also positive), lin2014-equal-ratio-hrv (favours the equal ratio) and meehan2024-longer-exhalations (null). Gap 4 in the ledger tabulates all five.

#### `balban2023-cyclic-sighing`

Balban, Melis Yilmaz; Neri, Eric; Kogon, Manuela M.; Weed, Lara; Nouriani, Bita; Jo, Booil; Holl, Gary; Zeitzer, Jamie M.; Spiegel, David; Huberman, Andrew D. (2023). *Brief structured respiration practices enhance mood and reduce physiological arousal*. Cell Reports Medicine 4(1): 100895

- DOI: [10.1016/j.xcrm.2022.100895](https://doi.org/10.1016/j.xcrm.2022.100895)
- Verification: crossref-verified | Access: open-access | Read: full text | evidence tier **B**
- Backs:
  - five minutes a day of exhale-emphasising cyclic sighing improved mood and lowered respiratory rate more than an equal period of mindfulness meditation over one month
  - box breathing, equal inhale / hold / exhale, was tested head-to-head and was not the best-performing arm
- Caveat: Remote randomised controlled study, pre-registered as NCT05304000, with 108 participants: 24 in the mindfulness-meditation control, 30 cyclic sighing, 21 box breathing and 33 cyclic hyperventilation. The comparator is mindfulness meditation rather than a sham, so the arms differ in more than breath ratio. In the mixed-effects model, cyclic sighing separated from the control on positive affect; box breathing and cyclic hyperventilation did not, and the breathwork arms were not tested against one another. Daily positive-affect gains were 1.89 points for cyclic sighing and 1.84 for box breathing. This is the best evidence in the corpus that emphasising the exhale is the right emphasis, and it is weaker than the abstract suggests: the box arm is small and the difference between the two patterns is not established. The cyclic-sighing protocol also adds a double inhale, so its effect cannot be attributed to exhale length alone.

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
- Caveat: Conducted at high altitude in a hypoxic state, so it does not transfer to a desk at sea level and must not be cited as if it did. It is carried for two narrow purposes: it is a second study placing the controlled literature at 6 breaths per minute, and its dead-space mechanism is the reason slow breathing is not simply less breathing.

#### `laborde2021-ie-ratio-pauses`

Laborde, Sylvain; Iskra, Maša; Zammit, Nina; Borges, Uirassu; You, Min; Sevoz-Couche, Caroline; Dosseville, Fabrice. (2021). *Slow-Paced Breathing: Influence of Inhalation/Exhalation Ratio and of Respiratory Pauses on Cardiac Vagal Activity*. Sustainability 13(14): 7775

- DOI: [10.3390/su13147775](https://doi.org/10.3390/su13147775)
- Verification: crossref-verified | Access: open-access | Read: abstract only | evidence tier **C**
- Backs:
  - at six cycles per minute, RMSSD was higher when the exhalation was longer than the inhalation, across inhalation/exhalation ratios of 0.8, 1.0 and 1.2
  - brief 0.4 s pauses after inhalation and after exhalation did not further change RMSSD
- Caveat: n = 64 athletes, within-subjects, six 5-minute conditions in one session. Findings taken from the abstract; effect sizes not read. This is the fifth study bearing on the ratio disagreement tabulated in gap 4, on the side of the longer exhale, and the only one that also manipulates respiratory pauses, which is why it also settles gap 11 as far as brief pauses go. Published in Sustainability, an MDPI journal outside the field, which is worth knowing when weighing it against the other four.

#### `laborde2021-spb-6cpm-biofeedback`

Laborde, Sylvain; Allen, Mark S.; Borges, Uirassu; Iskra, Maša; Zammit, Nina; You, Min; Hosang, Thomas; Mosley, Emma; Dosseville, Fabrice. (2021). *Psychophysiological effects of slow-paced breathing at six cycles per minute with or without heart rate variability biofeedback*. Psychophysiology 59(1): e13952

- DOI: [10.1111/psyp.13952](https://doi.org/10.1111/psyp.13952)
- Verification: crossref-verified | Access: open-access | Read: full text | evidence tier **C**
- Backs:
  - slow-paced breathing at six cycles per minute raised RMSSD relative to control in every condition tested
  - adding heart-rate-variability biofeedback on top of the six-per-minute pace did not add a further RMSSD benefit, though that condition reported more positive emotional valence
- Caveat: n = 112, single session. Crossref records this as issued 2021, appearing in the January 2022 issue (59:1). Both conditions raised RMSSD and lowered arousal; the biofeedback condition additionally reported more positive emotional valence, so 'no added benefit' is specific to the cardiac measure. Together with tabor2022-guided-breathing-design this is why exhale ships without a sensor: for the physiological effect of a single session, the pace does the work rather than the feedback loop.

#### `lin2014-equal-ratio-hrv`

Lin, I. M.; Tai, L. Y.; Fan, S. Y. (2014). *Breathing at a rate of 5.5 breaths per minute with equal inhalation-to-exhalation ratio increases heart rate variability*. International Journal of Psychophysiology 91(3): 206-211

- DOI: [10.1016/j.ijpsycho.2013.12.006](https://doi.org/10.1016/j.ijpsycho.2013.12.006)
- Verification: crossref-verified | Access: paywalled | Read: full text | evidence tier **C**
- Backs:
  - 5.5 breaths per minute with an equal 5:5 inhale-to-exhale ratio produced higher SDNN and LF power than 6 bpm or than a 4:6 ratio
  - all four slow-breathing patterns increased self-reported relaxation relative to spontaneous breathing
- Caveat: n = 47, Latin-square counterbalanced. The clearest published result against a longer exhale on HRV grounds, and one reason exhale makes no mechanism claim for the 1:2 ratio. It also measured relaxation and anxiety across the four patterns and reports that all four increased relaxation over baseline, with no ratio-specific advantage; that is one of three subjective measurements bearing on gap 4. Note the second claim carefully: every pattern tested increased relaxation, so the ratio argument is about which slow pattern is best, not about whether slow breathing works.

#### `marchant2025-square-478-six`

Marchant, Joshua; Khazan, Inna; Cressman, Mikel; Steffen, Patrick. (2025). *Comparing the Effects of Square, 4-7-8, and 6 Breaths-per-Minute Breathing Conditions on Heart Rate Variability, CO2 Levels, and Mood*. Applied Psychophysiology and Biofeedback 50(2): 261-276

- DOI: [10.1007/s10484-025-09688-z](https://doi.org/10.1007/s10484-025-09688-z)
- Verification: crossref-verified | Access: paywalled | Read: full text | evidence tier **C**
- Backs:
  - breathing at 6 breaths per minute raised HRV more than either square (box) breathing or 4-7-8 breathing, with small to medium effects
  - square and 4-7-8 breathing are popularly promoted but have little empirical support
  - none of the four conditions produced meaningful changes in blood pressure or mood
  - breathing at 6 breaths per minute unexpectedly produced mild over-breathing
- Caveat: n = 84 college students, within-subjects, four conditions: square, 4-7-8, 6 bpm at 4:6, and 6 bpm at 5:5. Single session, so it speaks to acute effect only, and it is the largest ratio comparison in this corpus. This is the head-to-head that ranks exhale's presets, and it ranks against box breathing and 4-7-8. Two of its findings cut against arguments made elsewhere in this corpus: mood did not change meaningfully in any condition, which is the largest subjective null in gap 4, and 6 bpm produced mild over-breathing, which is gap 5.

#### `meehan2024-longer-exhalations`

Meehan, Zachary M.; Shaffer, Fred. (2024). *Do Longer Exhalations Increase HRV During Slow-Paced Breathing?* Applied Psychophysiology and Biofeedback 49(3): 407-417

- DOI: [10.1007/s10484-024-09637-2](https://doi.org/10.1007/s10484-024-09637-2)
- Verification: crossref-verified | Access: open-access | Read: full text | evidence tier **B**
- Backs:
  - at 6 breaths per minute, a 1:2 inhale-to-exhale ratio produced no HRV advantage over 1:1 in either an original experiment or its replication
  - the finding held across time-domain, frequency-domain and nonlinear HRV metrics
- Caveat: Original n = 26, replication n = 16; both undergraduate samples, both within-subjects with manipulation checks. Small, but it is the only entry in this corpus that ran its own replication, and its introduction tallies the older literature: three further nulls and one result favouring the longer inhale. Its scope condition matters: it holds rate fixed inside the resonance range, so it does not rule out a ratio effect at a person's spontaneous rate, which is what bae2021-exhalation-inhalation-ratio measured. One of five disagreeing studies; see also vandiest2014-ie-ratio-relaxation, laborde2021-ie-ratio-pauses and lin2014-equal-ratio-hrv. All five measured HRV; vandiest2014-ie-ratio-relaxation, lin2014-equal-ratio-hrv and marchant2025-square-478-six also measured how participants felt, and those three do not agree either.

#### `sevozcouche2022-coherence-resonance`

Sevoz-Couche, Caroline; Laborde, Sylvain. (2022). *Heart rate variability and slow-paced breathing: when coherence meets resonance*. Neuroscience & Biobehavioral Reviews 135: 104576

- DOI: [10.1016/j.neubiorev.2022.104576](https://doi.org/10.1016/j.neubiorev.2022.104576)
- Open copy: <https://hal.sorbonne-universite.fr/hal-03578368>
- Verification: crossref-verified | Access: paywalled | Read: full text | evidence tier **D**
- Backs:
  - coherence and resonance are distinct phenomena that are frequently conflated in the slow-breathing literature
- Caveat: Narrative review. Carried as a terminology guard: much of the popular writing exhale competes with uses 'coherence' loosely, and this is the paper to check before adopting that vocabulary in the app or the README.

#### `shaffer2020-resonance-frequency-assessment`

Shaffer, Fred; Meehan, Zachary M. (2020). *A Practical Guide to Resonance Frequency Assessment for Heart Rate Variability Biofeedback*. Frontiers in Neuroscience 14: 570400

- DOI: [10.3389/fnins.2020.570400](https://doi.org/10.3389/fnins.2020.570400)
- Verification: crossref-verified | Access: open-access | Read: full text | evidence tier **D**
- Backs:
  - the relationship between breathing rate and heart-rate-variability amplitude is an inverted U with a peak, not a slope
  - resonance frequency ranges from 4.5 to 6.5 breaths per minute in adults, and 6.5 to 9.5 in children
  - breathing in a narrow band around the resonance frequency stimulates the baroreflex better than breathing across a wider range
- Caveat: Methods guide rather than an experiment, hence tier D. It bears on exhale's drift setting: if HRV amplitude peaks at the resonance frequency, breathing progressively slower moves away from that peak once past it, so 'slower is always better' is false for HRV amplitude. Gap 6 notes that the peak is in HRV amplitude rather than in relaxation or comfort, so this bounds the HRV argument for drift without settling the question. Shaffer is also an author of meehan2024-longer-exhalations, so this corpus leans on one group twice; the resonance-frequency model itself is not a contested finding.

#### `szulczewski2019-training-relaxation`

Szulczewski, Mikołaj Tytus. (2019). *Training of paced breathing at 0.1 Hz improves CO2 homeostasis and relaxation during a paced breathing task*. PLOS ONE 14(6): e0218550

- DOI: [10.1371/journal.pone.0218550](https://doi.org/10.1371/journal.pone.0218550)
- Verification: crossref-verified | Access: open-access | Read: full text | evidence tier **C**
- Backs:
  - across seven consecutive days of ten-minute paced breathing, self-reported task pleasantness rose significantly and unpleasant arousal fell, with the affective gains emerging by mid-training rather than on day one
  - the end-tidal CO2 drop caused by paced breathing shrank with practice: 37.5% of participants dropped below 30 mmHg on day one against 6.3% on day seven
- Caveat: n = 16, single group, no control, one week. Small and uncontrolled, so treat the magnitudes as indicative. It is carried because it is the closest thing in this corpus to evidence for the pranayama proposition that the practice improves with practice: relaxation was not immediate, it accrued. Note what it does not show. Training was at a fixed 0.1 Hz throughout; it is evidence that repeated practice at one rate gets better, not that progressively slowing within a session helps. That second proposition remains untested. See also joshi1992-pranayam-training.

#### `vandiest2014-ie-ratio-relaxation`

Van Diest, Ilse; Verstappen, Karen; Aubert, André E.; Widjaja, Devy; Vansteenwegen, Debora; Vlemincx, Elke. (2014). *Inhalation/Exhalation Ratio Modulates the Effect of Slow Breathing on Heart Rate Variability and Relaxation*. Applied Psychophysiology and Biofeedback 39(3-4): 171-180

- DOI: [10.1007/s10484-014-9253-x](https://doi.org/10.1007/s10484-014-9253-x)
- Open copy: <https://lirias.kuleuven.be/retrieve/843e9b82-a9b0-42e6-897e-9177c10d71b1>
- Verification: crossref-verified | Access: paywalled | Read: full text | evidence tier **C**
- Backs:
  - participants reported more relaxation, more stress reduction, more mindfulness and more positive energy breathing with a low inhale/exhale ratio (longer exhale) than a high one
  - a low inhale/exhale ratio also produced more HF-HRV power, but only in the slow breathing condition
  - slowing the rate on its own improved only self-reported positive energy
- Caveat: n = 30, four patterns crossing rate (6 or 12 breaths/min) with i/e ratio (0.42 or 2.33). This is the single most important entry for exhale's longer-exhale preference: it is the one study in this corpus in which the longer exhale won on how people felt. It is not the only study that measured that. lin2014-equal-ratio-hrv found every slow pattern raised relaxation with no ratio-specific edge, and marchant2025-square-478-six, at nearly three times the sample, found no meaningful mood change in any condition. Gap 4 weighs the three.

#### `you2023-respiratory-frequency`

You, Min; Laborde, Sylvain; Ackermann, Stefan; Borges, Uirassu; Dosseville, Fabrice; Mosley, Emma. (2023). *Influence of Respiratory Frequency of Slow-Paced Breathing on Vagally-Mediated Heart Rate Variability*. Applied Psychophysiology and Biofeedback 49(1): 133-143

- DOI: [10.1007/s10484-023-09605-2](https://doi.org/10.1007/s10484-023-09605-2)
- Verification: crossref-verified | Access: paywalled | Read: full text | evidence tier **C**
- Backs:
  - five minutes of slow-paced breathing at 5, 5.5, 6, 6.5 and 7 cycles per minute all raised cardiac vagal activity above spontaneous breathing
  - LF-HRV discriminated between the tested frequencies more sensitively than RMSSD
  - the band actually tested and supported is 5 to 7 cycles per minute
- Caveat: Crossref records this as issued 2023 (online 8 December); it appears in the March 2024 issue, 49(1), which is how it is usually cited. The citekey follows the Crossref issued year, as everywhere else in this corpus. n = 75, all athletes aged 19-31, single lab session. Generalisation to a desk worker is an assumption, not a finding. This is the source that fixes the tested band the settings panel reports: 5 s in and 5 s out is 6 cycles per minute, inside it; the earlier 5 s in and 10 s out is 4.0, below it.

---

## Where the practice came from

3 sources.

#### `joshi1992-pranayam-training`

Joshi, L. N.; Joshi, V. D.; Gokhale, L. V. (1992). *Effect of short term 'Pranayam' practice on breathing rate and ventilatory functions of lung*. Indian Journal of Physiology and Pharmacology 36(2): 105-108

- PMID: [1506070](https://pubmed.ncbi.nlm.nih.gov/1506070/) (no DOI exists)
- Verification: pubmed-verified | Access: paywalled | Read: abstract only | evidence tier **C**
- Backs:
  - six weeks of pranayama practice in 75 young adults lowered resting respiratory rate and prolonged breath-holding time
  - the same training raised forced vital capacity, FEV1, maximum voluntary ventilation and peak expiratory flow rate
- Caveat: Findings taken from the abstract. No DOI exists, and the record was verified against the NCBI E-utilities API by PMID rather than Crossref. Uncontrolled before-and-after design in a 1992 regional journal, so treat the effect sizes as unusable. It is carried because it is the only entry in this corpus that speaks to graded extension as a training progression: capacity grew over six weeks of practice. That is a claim about adaptation across sessions, not evidence for extending the breath without limit inside a single sitting, which is a different proposition that shaffer2020-resonance-frequency-assessment bears on.

#### `muktibodhananda1998-hatha-yoga-pradipika`

Muktibodhananda, Swami. (1998). *Hatha Yoga Pradipika*, 3rd ed. Munger, Bihar, India: Bihar School of Yoga 642 pp

- ISBN: 9788185787381 | [Open Library record](https://openlibrary.org/books/OL9083573M/Hatha_Yoga_Pradipika)
- Verification: openlibrary-verified | Access: paywalled | Read: catalogue record only | evidence tier **E**
- Backs:
  - the classical hatha text and commentary in which timed inhale, retention and exhale ratios are set out is the historical origin of the ratio instructions modern breathing apps repeat
- Caveat: Contents not consulted; the bibliographic record was checked against Open Library, which lists this ISBN as the third edition, Bihar School of Yoga, 1998, 642 pages. Not peer reviewed, and a commentary on a fifteenth-century text rather than a primary source in any modern sense. Carried because the longer-exhale instruction exhale ships did not come from a laboratory: it came from this tradition, centuries before anyone measured HRV. Naming that is more accurate than retrofitting a citation to psychophysiology. It must never back a physiological claim.

#### `satyananda1999-apmb`

Satyananda Saraswati, Swami. (1999). *Asana Pranayama Mudra Bandha*, 3rd rev. ed. Munger, Bihar, India: Yoga Publications Trust 553 pp

- ISBN: 9788186336144 | [Open Library record](https://openlibrary.org/books/OL22138410M/Asana_pranayama_mudra_bandha)
- Verification: openlibrary-verified | Access: paywalled | Read: catalogue record only | evidence tier **E**
- Backs:
  - the systematic pranayama tradition from which exhale's controllable inhale / retention / exhale / retention structure descends is documented in a standard modern reference manual
- Caveat: Contents not consulted; the bibliographic record was checked against Open Library, which lists this ISBN as the third revised edition, Yoga Publications Trust, 1999, 553 pages. A 2008 fourth revised edition exists under the same imprint; this entry cites the edition the ISBN resolves to. Not peer reviewed. Cited only for lineage: it is where exhale's four-phase structure comes from, and presenting the design as derived from 2020s psychophysiology would be revisionist. It must never back a physiological claim.

---

## Whether an on-screen visual pacer works

3 sources.

#### `moraveji2011-peripheral-paced-respiration`

Moraveji, Neema; Olson, Ben; Nguyen, Truc; Saadat, Mahmoud; Khalighi, Yaser; Pea, Roy; Heer, Jeffrey. (2011). *Peripheral paced respiration: influencing user physiology during information work*. Proceedings of the 24th Annual ACM Symposium on User Interface Software and Technology (UIST '11) 423-428

- DOI: [10.1145/2047196.2047250](https://doi.org/10.1145/2047196.2047250)
- Verification: crossref-verified | Access: paywalled | Read: full text | evidence tier **C**
- Backs:
  - a translucent animated bar spanning the screen, running in the periphery of attention during normal information work, significantly lowered participants' breathing rate
  - peripheral pacing does not require the user's full attention to change breathing
- Caveat: This is the closest published analogue to exhale that exists, and its limitation is the important part: the reduction occurred while the pacing feedback was active and did not persist as a lasting change in respiratory pattern. An always-on overlay should be understood as an effect that lasts as long as it is on. An author copy has been distributed from vis.stanford.edu; the ACM Digital Library version is paywalled.

#### `tabor2022-guided-breathing-design`

Tabor, Aaron; Bateman, Scott; Scheme, Erik J.; schraefel, m.c. (2022). *Comparing heart rate variability biofeedback and simple paced breathing to inform the design of guided breathing technologies*. Frontiers in Computer Science 4: 926649

- DOI: [10.3389/fcomp.2022.926649](https://doi.org/10.3389/fcomp.2022.926649)
- Verification: crossref-verified | Access: open-access | Read: full text | evidence tier **C**
- Backs:
  - an expanding and contracting circle pacing 6 breaths per minute produced HRV amplitude gains statistically indistinguishable from sensor-driven HRV biofeedback
  - both conditions took roughly two minutes for effects to appear
  - paced breathing needs no sensor, no real-time processing and no sustained attention, which suits it to use as a secondary task
- Caveat: Between-subjects, n = 28 (14 per group), single 10-minute session. This is the strongest published warrant for exhale's specific design choices: an expanding/contracting shape, no hardware, no account, watchable while doing something else.

#### `wongsuphasawat2012-cant-force-calm`

Wongsuphasawat, Kanit; Gamburg, Alex; Moraveji, Neema. (2012). *You can't force calm: designing and evaluating respiratory regulating interfaces for calming technology*. Adjunct Proceedings of the 25th Annual ACM Symposium on User Interface Software and Technology (UIST '12 Adjunct) 69-70

- DOI: [10.1145/2380296.2380326](https://doi.org/10.1145/2380296.2380326)
- Verification: crossref-verified | Access: paywalled | Read: full text | evidence tier **C**
- Backs:
  - visual pacing produced more measured respiratory change than auditory pacing, but auditory pacing was rated as subjectively more calming
- Caveat: Two-page adjunct paper, effectively a poster; treat the finding as a design signal, not a result. It is carried anyway because it is the one published comparison that puts exhale's visual-only design at a disadvantage on the outcome most users actually care about, which is whether they feel calmer.

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
- Caveat: Narrative mechanistic review by the technique's principal developers. Read as the mechanism argument, not as independent evidence. The individual-variation point is the honest reason exhale's timing sliders are user-editable at all: there is no single correct number to hardcode.

#### `li2016-sigh-circuit`

Li, Peng; Janczewski, Wiktor A.; Yackle, Kevin; Kam, Kaiwen; Pagliardini, Silvia; Krasnow, Mark A.; Feldman, Jack L. (2016). *The peptidergic control circuit for sighing*. Nature 530(7590): 293-297

- DOI: [10.1038/nature16964](https://doi.org/10.1038/nature16964)
- Open copy: <https://www.ncbi.nlm.nih.gov/pmc/articles/PMC4852886/>
- Verification: crossref-verified | Access: paywalled | Read: full text | evidence tier **D**
- Backs:
  - sighing is generated by a dedicated peptidergic circuit projecting onto the preBotzinger complex, not as a byproduct of ordinary breathing rhythm
- Caveat: Mouse work. It establishes that the sigh is a distinct, hardwired respiratory behaviour, which is the mechanistic backdrop for the cyclic-sighing result in balban2023-cyclic-sighing. It says nothing about humans deliberately performing sighs, and must not be cited as if it did.

#### `vlemincx2013-sigh-reset-model`

Vlemincx, Elke; Abelson, James L.; Lehrer, Paul M.; Davenport, Paul W.; Van Diest, Ilse; Van den Bergh, Omer. (2013). *Respiratory variability and sighing: A psychophysiological reset model*. Biological Psychology 93(1): 24-32

- DOI: [10.1016/j.biopsycho.2012.12.001](https://doi.org/10.1016/j.biopsycho.2012.12.001)
- Verification: crossref-verified | Access: paywalled | Read: full text | evidence tier **D**
- Backs:
  - sighs act as resetters of respiratory variability: random variability is elevated before a sigh and correlated variability increases after one
- Caveat: Theoretical model paper with supporting data. Carried because it is the honest place to point when someone asks why a deliberately irregular breath might be useful, which is the only literature adjacent to exhale's randomised-timing sliders. It is not evidence that those sliders help; see the gaps ledger.

#### `vlemincx2016-sigh-relief`

Vlemincx, Elke; Van Diest, Ilse; Van den Bergh, Omer. (2016). *A sigh of relief or a sigh to relieve: The psychological and physiological relief effect of deep breaths*. Physiology & Behavior 165: 127-135

- DOI: [10.1016/j.physbeh.2016.07.004](https://doi.org/10.1016/j.physbeh.2016.07.004)
- Verification: crossref-verified | Access: paywalled | Read: abstract only | evidence tier **C**
- Backs:
  - self-reported relief was higher after an instructed deep breath than before it
  - spontaneous sighs were followed by a gradual fall in muscle tension, most clearly in people high in anxiety sensitivity
- Caveat: Findings taken from the abstract; the version of record is paywalled. The instructed-breath result is the closest thing in this corpus to evidence that being told to take one deliberate breath, which is what exhale's reminder does, changes how a person feels.

#### `yackle2017-breathing-arousal-neurons`

Yackle, Kevin; Schwarz, Lindsay A.; Kam, Kaiwen; Sorokin, Jordan M.; Huguenard, John R.; Feldman, Jack L.; Luo, Liqun; Krasnow, Mark A. (2017). *Breathing control center neurons that promote arousal in mice*. Science 355(6332): 1411-1415

- DOI: [10.1126/science.aai7984](https://doi.org/10.1126/science.aai7984)
- Open copy: <https://www.ncbi.nlm.nih.gov/pmc/articles/PMC5505554/>
- Verification: crossref-verified | Access: paywalled | Read: full text | evidence tier **D**
- Backs:
  - a small preBotzinger subpopulation projects to and positively regulates noradrenergic locus coeruleus neurons, giving breathing a direct anatomical route to arousal state
  - ablating those roughly 175 neurons left breathing intact but increased calm behaviours
- Caveat: Mouse work, and the direction is breathing pattern to arousal rather than voluntarily slow breathing to calm. It is the single best answer to 'why would breathing slowly change how I feel at all', and it is still an inference from mice. Do not cite it for a human effect.

#### `yasuma2004-rsa`

Yasuma, Fumihiko; Hayano, Jun-ichiro. (2004). *Respiratory Sinus Arrhythmia: Why Does the Heartbeat Synchronize With Respiratory Rhythm?* Chest 125(2): 683-690

- DOI: [10.1378/chest.125.2.683](https://doi.org/10.1378/chest.125.2.683)
- Verification: crossref-verified | Access: paywalled | Read: full text | evidence tier **D**
- Backs:
  - heart rate rises on inhalation and falls on exhalation, and this respiratory sinus arrhythmia is the coupling that HRV-based breathing claims rest on
- Caveat: Review. This is the physiological fact underneath every 'longer exhale calms you' claim in this corpus, which is why the claim is so intuitive and why meehan2024-longer-exhalations failing to find a ratio effect is worth taking seriously: a real beat-to-beat mechanism does not guarantee a measurable session-level outcome.

#### `zelano2016-nasal-respiration-limbic`

Zelano, Christina; Jiang, Heidi; Zhou, Guangyu; Arora, Nikita; Schuele, Stephan; Rosenow, Joshua; Gottfried, Jay A. (2016). *Nasal Respiration Entrains Human Limbic Oscillations and Modulates Cognitive Function*. The Journal of Neuroscience 36(49): 12448-12467

- DOI: [10.1523/JNEUROSCI.2586-16.2016](https://doi.org/10.1523/JNEUROSCI.2586-16.2016)
- Verification: crossref-verified | Access: open-access | Read: full text | evidence tier **C**
- Backs:
  - nasal breathing entrains oscillations in human piriform cortex, amygdala and hippocampus, and the effect is specific to the nasal route rather than to breathing as such
- Caveat: Human intracranial recordings in a small epilepsy-surgery cohort plus behavioural experiments. Carried because it is the reason nose-versus-mouth is a real variable and not folklore. exhale gives no nasal-breathing guidance at all, which is a defensible omission for a wordless overlay but should be a conscious one.

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
- Caveat: Pre-registered as NCT06064474, 200 healthy young adults, blinded against an active comparator; the largest trial of this technique to date. The primary finding is a null: high-ventilation breathwork did not outperform the comparator on stress. It is carried for the distinction it draws and for its tolerability data. It matters to exhale because the sliders can be set to fast, hold-heavy patterns that leave the slow-breathing evidence base entirely, and this is where the documented short-term effects of those patterns are recorded.

#### `linardon2020-app-attrition`

Linardon, Jake; Fuller-Tyszkiewicz, Matthew. (2020). *Attrition and adherence in smartphone-delivered interventions for mental health problems: A systematic and meta-analytic review*. Journal of Consulting and Clinical Psychology 88(1): 1-13

- DOI: [10.1037/ccp0000459](https://doi.org/10.1037/ccp0000459)
- Open copy: <https://figshare.com/articles/journal_contribution/Attrition_and_adherence_in_smartphone-delivered_interventions_for_mental_health_problems_a_systematic_and_meta-analytic_review/20730262>
- Verification: crossref-verified | Access: paywalled | Read: abstract only | evidence tier **A**
- Backs:
  - dropout and non-adherence are the dominant practical failure mode of smartphone-delivered mental health interventions, even where efficacy trials are positive
- Caveat: Findings taken from the abstract: mean attrition of 24.1% at short-term and 35.5% at longer-term follow-up across 70 trials. Carried as the reality check on every other entry in this corpus: an effect measured in a supervised session says little about a tool someone installs and forgets. exhale has no telemetry and therefore no measure of whether anyone keeps it running, which the gaps ledger states plainly.

#### `szulczewski2019-antihyperventilation-instruction`

Szulczewski, Mikołaj Tytus. (2019). *An Anti-hyperventilation Instruction Decreases the Drop in End-tidal CO2 and Symptoms of Hyperventilation During Breathing at 0.1 Hz*. Applied Psychophysiology and Biofeedback 44(3): 247-256

- DOI: [10.1007/s10484-019-09438-y](https://doi.org/10.1007/s10484-019-09438-y)
- Verification: crossref-verified | Access: open-access | Read: full text | evidence tier **C**
- Backs:
  - one sentence of instruction cut the end-tidal CO2 drop during 6-per-minute paced breathing from 5.21 to 2.7 mmHg
  - hyperventilation symptoms rose 0.63 points on a 7-point scale without the instruction and did not rise significantly with it
  - the instruction used was to avoid excessively deep breathing and to breathe shallowly and naturally
- Caveat: Randomised, two groups, n = 46 aged 19-26, single session. This is the mitigation for gap 5 and it is unusually cheap: the problem with slow pacing is depth, not rate, and a single sentence removes most of it. exhale paces rate and says nothing about depth, so this is the one instruction in the corpus with a safety rationale for appearing in the app rather than a persuasive one.

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
