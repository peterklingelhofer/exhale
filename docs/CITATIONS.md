# Citation corpus

49 sources: 46 Crossref-verified, 2 verified against Open Library, 1 verified against PubMed. 35 were read in full, 12 from the abstract only, and 2 are catalog records only. Each entry says which. 2 aren't peer-reviewed and are tiered E so they can back lineage but never a claim. 568 quotes back 98 of the 100 claims, and all 568 were found in the source's own text.

Machine-readable companion: [`CITATIONS.csl.json`](./CITATIONS.csl.json) (CSL-JSON).
Bibliographic records were verified on 2026-08-28 (Crossref, Open Library) and 2026-08-30 (PubMed)
and re-verified against all three registries on 2026-09-20 and 2026-09-26. Every claim was re-checked
against the abstracts and open full texts on 2026-09-02, and again on 2026-09-20 against the full text
of 35 entries and the abstract of 12, with the two books unread as before. On 2026-09-26 every
statement was checked again against the same texts, and the ten entries that had been marked as read
in full but could only be checked against their abstracts were relabeled as read from the abstract.
On 2026-10-18 each claim of a source that was read gained the words in the source that back it, and
every one of those quotes was found in the source's text.

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
| `crossref-verified` | The DOI resolves in Crossref, and the title, authors, year, journal, volume and pages printed here are the ones Crossref returned. |
| `openlibrary-verified` | No DOI exists because the source is a book. The title, author, publisher, edition and page count printed here were checked against the Open Library record for the stated ISBN. |
| `pubmed-verified` | No DOI exists, but the article is indexed in PubMed. The title, authors, journal, year, volume and pages were checked against the NCBI E-utilities record for the stated PMID. |
| `unverified` | No resolvable DOI and no catalog record. Bibliographic details are inherited from secondary citation and may be wrong. |

Author names are printed as the registry holds them, which is why a few records carry initials
where others carry full given names.

A verified record can still carry a loud caveat. Verification confirms only the *citation*, so each
entry also states to what depth the source was read before its claims were written down: in full,
from the abstract only, or as a catalog record only.

### Access level

`open-access` (a CC license is registered with Crossref, or the title is fully open access) |
`paywalled`. Where a Crossref `license` field was present, the access level is taken from it.
Where it was absent, the basis is stated in the entry's caveat. `paywalled` describes
the version of record. Where a legal open copy exists in a repository or free at the publisher, the
entry links it as an open copy.

### Evidence tier

Applied to sources making a claim about what happens to a human being. Physiological and animal
mechanism papers are tier D by definition: they explain why something might work without
establishing that it does.

| Tier | Definition | How exhale is allowed to use it |
|---|---|---|
| A | Systematic review or meta-analysis of controlled trials, or a pre-registered RCT with 200 or more participants | May be cited for an outcome claim |
| B | Pre-registered or internally replicated controlled experiment, or a systematic review of experiments without meta-analysis | May be cited for an outcome claim, with its scope conditions stated |
| C | Single controlled experiment, small n, lab-only, or observational work, including systematic reviews of observational studies | Cite as suggestive and never as "research shows" |
| D | Narrative review, theory, mechanism, or animal work | Cite for mechanism only: it can't establish that a practice works |
| E | Not peer reviewed, or contradicted by better evidence | May be cited for provenance, meaning where a practice came from, and never for whether it works |

### Quotes

Each claim is followed by the words in the source that back it, and by where they sit: `Abstract`,
a section heading as the source prints it, a table, or a printed page. Quotes under a caveat back a
statement in the caveat. `[...]` marks words left out, and nothing else inside the quote marks
differs from the source. Where a passage in this file, the README or the gaps ledger links an entry,
quote marks hold that entry's words, and the generator stops when they surround words that no quote
of the linked entry has. A word discussed as a word is set in italics.

### Receipts

Most of the source texts are copyrighted, so they stay out of the repository.
[`CITATIONS.receipts.json`](./CITATIONS.receipts.json) carries a receipt for each quote instead: a
hash of its words, the copy of the source it was found in, a hash of that copy's text and the date.
A quote edited after it was checked no longer matches its receipt, and the generator stops until the
quote is found in the source again. Each entry's *Quotes checked* line counts the quotes that hold a
receipt and names the copies they were found in.

Matching forgives typography only: curly or straight quote marks, dash forms, line breaks, spacing
around punctuation, and a hyphen or dash between two letters, since PDFs break and join words at
line ends. Words, case, digits and units have to match.

The receipts also hold the read-depth label to account. An entry marked as read in full has at least
one quote found outside its abstract, and an entry marked as read from the abstract quotes only its
abstract. A copy is used only when it carries the entry's title, so a landing page or a different
paper can't stand in for the source.

A receipt shows that the words were in that copy on that date. It doesn't show that the copy is the
version of record, since an author's manuscript can differ from the published article in small ways,
or that the words back the claim in the way the claim says. The caveats carry that judgment.

### Registry and link checks

A scheduled job asks Crossref, Open Library and PubMed about every entry each week, fetches every
open copy, and checks the quotes found in Europe PMC against its current copy.
[`CITATIONS.registry.json`](./CITATIONS.registry.json) keeps the answers. A verification label stands
only while the registry agrees with the title, year, journal, volume, issue, pages and first author
printed here, and an edit to any of those fields stops the generator until the registry has been
asked again. A retraction or an expression of concern posted against a source stops it too, and a
correction is listed under the entry it corrects. Open copies and catalog pages link a Wayback
Machine capture, so the copy that was quoted can still be read if the page moves.

### Counts that only go down

[`CITATIONS.baseline.json`](./CITATIONS.baseline.json) holds five counts: unverified entries, quotes
without a receipt, claims without a quote, figures in the README that no quote of the cited entry
prints, and links without an archived copy. `--check` fails when a count changes and the baseline
doesn't change with it, and the generator raises a count only when told to with `--raise`.

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
| abstract | 12 |
| full-text | 35 |
| record | 2 |
| **total** | **49** |

| Evidence tier | n |
|---|---|
| A | 6 |
| B | 4 |
| C | 22 |
| D | 14 |
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
  - short breaks of ten minutes or less from a task had small effects, reducing self-reported fatigue (d = .35) and boosting self-reported vigor (d = .36)
    - "For the purpose of this paper, we adopted a general definition of micro-breaks as short discontinuities in one’s tasks of no longer than 10 minutes" (Introduction)
    - "Random-effects meta-analyses shown statistically significant but small effects of micro-breaks in boosting vigor (d = .36, p < .001; k = 9, n = 913), reducing fatigue (d = .35, p < .001; k = 9, n = 803), and a non-significant effect on increasing overall performance (d = .16, p = .116; k = 15, n = 1132)." (Abstract)
    - "Considering the outcomes’ operationalizations, vigor and fatigue were exclusively assessed with self-report scales." (Results, Description of the included studies)
  - break duration didn't moderate the vigor or fatigue effects, and the studies behind those effects used breaks of 2 to 10 minutes, so a one-breath interruption sits below the shortest break tested for well-being, while the performance studies went as short as 8 seconds and found that longer breaks did better
    - "For vigor and fatigue, the results show that none of the considered moderators impacted the efficacy of micro-breaks (all ps > .050)." (Results, Moderator analyses)
    - "Break duration (minutes) 9 0.01 .909 9 0.61 .436 15 7.50 .006" (Table 4)
    - "Break duration varied significantly between studies, ranging between 8 seconds and 10 minutes." (Results, Description of the included studies)
    - "Rieger et al. (2017) [ 59] [...] Cognitive 2 Vigor Performance" (Table 2)
    - "For overall performance, however, two moderators were found. Break duration was revealed to be one of the significant results (b = .07, p = .006, R2 = .34). Indicating that the longer the break, the more micro-break leads to a performance increase." (Results, Moderator analyses)
    - "Finstad et al. (2006) [ 91] Study 1 Non work Cognitive Irrelevant Prospective memory test (Thorndike & Lorge, 1944) 30 [...] 0.133 Performance" (Table 2)
- Caveat: 22 independent study samples from 19 publications. Break duration moderated the performance effect (longer was better) but not vigor or fatigue. The pooled breaks ran from 8 seconds to 10 minutes, but every sub-minute break came from a performance-only study, the vigor and fatigue studies used breaks of 2 to 10 minutes, and the break-duration tests rest on nine studies each for vigor and fatigue against 15 for performance. The authors rate the risk of bias in the overall sample as somewhat unclear, with four of the 22 studies at low risk. The performance effect was smaller and less consistent than the well-being effect, and the authors report it depended on task type. The paper reports break activities only by category and describes none as a breathing exercise, so this supports the general shape of exhale's reminder timer. The breathing content itself is untested here.
  - "Hence, the final sample included 22 independent study samples that resulted from 19 publications" (Results, Selection and inclusion of studies)
  - "However, there was significant unexplained variance in the true effect sizes for performance (Q(14) = 34.52, p = .002; 95% prediction interval: [-.53, .86])." (Results, Effectiveness of micro-breaks)
  - "The second significant effect was the type of task performed before the break (Q(2) = 6.53, p = .011)." (Results, Moderator analyses)
  - "Overall, only four out of twenty-two studies were assessed with a low risk of bias, whereas one presented a high risk for at least half of the criteria. Thus, we are inclined to consider the risk of bias in our overall sample as being somewhat unclear." (Results, Quality of the included studies)
- Quotes checked: 13 of 13 against the Europe PMC full text and the Europe PMC abstract, last on 2026-10-18

#### `deniz2024-forward-head-lung-volumes`

Deniz, Yasemin; Ertekin, Damla; Çokar, Dilek. (2024). *The effect of forward head posture on dynamic lung volumes in young adults: a systematic review*. Bulletin of Faculty of Physical Therapy 29(1): 15

- DOI: [10.1186/s43161-024-00186-7](https://doi.org/10.1186/s43161-024-00186-7)
- Verification: crossref-verified | Access: open-access | Read: full text | evidence tier **C**
- Backs:
  - across four comparison studies totaling 115 participants, three reported significant FVC and FEV1 decreases with forward head posture, and the reductions ranged from 0.25 to 0.81 L for FVC and 0.16 to 0.93 L for FEV1
    - "A total of four comparison studies and two correlation studies met the inclusion criteria for this study." (Abstract)
    - "Across these four studies, encompassing a total of 115 participants, the observed reduction range for FVC was between -0.25 L and -0.81 L, while the reduction values for FEV1 ranged from -0.16 to -0.93 L." (Abstract)
    - "The reduction in FVC was measured to range between 0.25 to 0.81 L, while the reduction in FEV1 was reported to potentially range between 0.16 to 0.93 L." (Results, Outcomes)
    - "In three comparative studies, significant statistical decreases in FVC and FEV1 values among individuals with FHP have been reported [28, 31, 32]. In another study, while a significant decrease in dynamic lung volume was not observed, significant changes in predicted parameter values were reported [33]." (Results, Outcomes)
  - in the two included correlation studies, craniovertebral angle correlated positively with FVC (r = 0.53 and 0.63) and FEV1 (r = 0.31 and 0.54)
    - "According to the results of the correlation studies, a positive correlation was observed between craniovertebral angle (CVA) and dynamic pulmonary volumes." (Abstract)
    - "The correlation coefficient between CVA and FVC was reported as r = 0.63 and r = 0.53 in two studies, while the correlation coefficient between CVA and FEV1 was reported as r = 0.31 and r = 0.54" (Results, Outcomes)
- Caveat: Systematic review without meta-analysis of four small comparison studies and two correlation studies. The abstract phrases the conclusion as forward head posture 'can potentially cause' pulmonary abnormalities, and the main-text Conclusion says the findings suggest forward head posture 'exerts a negative impact on dynamic lung volumes'. The 0.25 L low end of the FVC range comes from Koseki's study, which measured healthy males after altering their head position. The Discussion cites a further study whose lack of a significant correlation between craniovertebral angle and FVC or FEV1 contradicts the included studies, and the authors suggest milder forward head posture in its sample may explain it. The search included ResearchGate alongside PubMed and Google Scholar, which is an unusual choice. This is the posture half of the screen-breathing argument: it measures head position, and the link to screens comes from jung2016-smartphone-posture-respiration. The Crossref deposit and the publisher's page carry the second and third authors' names with a dotless i, a transliteration artifact. They're printed here in standard Turkish orthography.
  - "Among these, two studies were correlation studies, and therefore, for the purpose of observing inter-group volume differences, only four studies were incorporated." (Discussion)
  - "The existing evidence indicates that FHP can potentially cause abnormalities in pulmonary function." (Abstract)
  - "A comprehensive review of the electronic literature was conducted by 2 investigators (YD, DE) using databases including ResearchGate, PubMed, and Google Scholar." (Methods, Literature search strategy)
  - "The study aimed to investigate whether there is a difference in pulmonary function between individuals with forward head posture and those with neutral head posture." (Abstract)
  - "The findings suggest that the FHP exerts a negative impact on dynamic lung volumes by altering the biomechanics of the diaphragm and accessory muscles." (Conclusion)
  - "In Köseki's study, only healthy males were included, and measurements were taken by altering the head position of these healthy males. An average loss of 0.25 L was reported." (Discussion)
  - "However, the lack of a statistically significant correlation between FEV1, FVC values, and CVA contradicted the findings of the three studies included in our research. [...] we interpreted the absence of a significant correlation between dynamic lung volume and CVA as potentially stemming from the participants' lower severity of FHP compared to other studies [39]." (Discussion)
- Quotes checked: 13 of 13 against the publisher's page and the Crossref abstract, last on 2026-10-18

#### `grassmann2016-cognitive-load-respiration`

Grassmann, Mariel; Vlemincx, Elke; von Leupoldt, Andreas; Mittelstädt, Justin M.; Van den Bergh, Omer. (2016). *Respiratory Changes in Response to Cognitive Load: A Systematic Review*. Neural Plasticity 2016: 8146809

- DOI: [10.1155/2016/8146809](https://doi.org/10.1155/2016/8146809)
- Verification: crossref-verified | Access: open-access | Read: full text | evidence tier **B**
- Backs:
  - mentally demanding work is reliably marked by faster breathing, with medium to large effects in 48% of the reviewed studies
    - "While mentally demanding episodes are clearly marked by faster breathing and higher minute ventilation" (Abstract)
    - "The present findings show that cognitive load was accompanied by a clear increase in RR. Of note, 48% of the reviewed studies indicated medium to large effects for the increase from baseline to task." (4.1. Respiratory Responses to Cognitive Load)
  - respiratory amplitude appears to stay roughly stable under cognitive load, though tidal volume findings are inconsistent and tidal volume mostly fell in tasks with performance feedback
    - "respiratory amplitude appears to remain rather stable" (Abstract)
    - "respiratory amplitude which corresponds to tidal volume (TV), the amount of air that is inspired during one respiratory cycle" (1. Introduction)
    - "While the studies including TV are rather inconsistent and mainly reported no significant changes from baseline to task" (3.3. Empirical Findings on Respiration and Cognitive Load)
    - "Interestingly, the implementation of feedback was mostly associated with a decrease in TV [...] whereas the experiments not providing feedback did not show significant changes in TV." (3.3. Empirical Findings on Respiration and Cognitive Load)
  - in the four experiments that measured it, cognitive load lowered end-tidal CO2, which the authors read as possible overbreathing while noting that CO2 production rose too, though the CO2 production findings didn't all point the same way
    - "In addition, we found that cognitive load may lead to overbreathing as indicated by decreased end-tidal CO2 but is also accompanied by elevated oxygen consumption and CO2 release." (Abstract)
    - "Number of studies % of studies [...] Partial pressure of end-tidal CO2 (petCO2) 4 7.41" (Table 1)
    - "However, petCO2 was consistently reported to decrease whereas findings on VCO2 did not entirely point in the same direction." (4.1. Respiratory Responses to Cognitive Load)
- Caveat: 54 experiments across 53 articles. The rate finding is the well-supported one: 20 studies with large effects. The end-tidal CO2 finding rests on four of the 54 experiments, with one computed effect size of d = -0.26, and the authors hedge it as 'may lead to overbreathing'. The amplitude finding is that cognitive load raises rate while amplitude appears to stay roughly stable, though tidal volume mostly fell in tasks with performance feedback. Minute ventilation rose in every study, so this doesn't support *shallow* if shallow means less air moved. For the diaphragmatic-to-thoracic shift that *shallow* usually names, see schleifer2002-hyperventilation-job-stress. For the same over-breathing measured at a keyboard, see schleifer1994-vdt-petco2 and schleifer2008-emg-gaps-computer-work. This entry is the general cognitive-load backdrop, and it's also the reason the keyboard can't be singled out as the cause: the review finds faster breathing under cognitive load in general, and the authors say the size of the respiratory effects may vary with the type of cognitive demand.
  - "Table 1 summarizes the study characteristics of 54 experiments which are reported in 53 articles." (3.1. Study Characteristics)
  - "As illustrated in Figure 1, the most robust measure is RR which is based on a total sample of 930 individuals." (3.2. Coding and Integration of Effects)
  - "Following the guideline by Cohen [92], five studies suggest small effects (.20 ≤ d < .50 [...] ), six studies suggest medium effects (.50 ≤ d < .80 [...] ), and 20 studies suggest large effects (d ≥ .80 [...] (two studies reported in [73])) for the increase in RR." (3.3. Empirical Findings on Respiration and Cognitive Load)
  - "Capnographic measures show that petCO2 levels were lower during task performance (d = −.26; [58])" (3.3. Empirical Findings on Respiration and Cognitive Load)
  - "MV increased in all studies" (3.3. Empirical Findings on Respiration and Cognitive Load)
  - "The 11 studies including more than one type of cognitive task in their experimental design (see Table 2) imply that the magnitude of respiratory effects may vary depending on the given types of cognitive demands." (4.2. Methodological Evaluation)
- Quotes checked: 15 of 15 against the Europe PMC full text and the Europe PMC abstract, last on 2026-10-18

#### `johnson2023-20-20-20`

Johnson, Sophia; Rosenfield, Mark. (2023). *20-20-20 Rule: Are These Numbers Justified?* Optometry and Vision Science 100(1): 52-56

- DOI: [10.1097/OPX.0000000000001971](https://doi.org/10.1097/OPX.0000000000001971)
- Verification: crossref-verified | Access: paywalled | Read: abstract only | evidence tier **C**
- Backs:
  - the widely repeated 20-20-20 rule has relatively little peer-reviewed support
    - "The so-called 20-20-20 rule, [...] is widely cited. Unfortunately, there is relatively little peer-reviewed evidence to support this rule." (Abstract)
  - scheduled 20-second breaks every 5, 10 or 20 minutes produced no significant effect on ocular symptoms, reading speed or task accuracy
    - "The task was undertaken on four separate occasions, with 20-second breaks being allowed every 5, 10, 20, or 40 minutes (i.e., no break), respectively. Both before and after each trial, subjects completed a questionnaire regarding ocular and visual symptoms experienced during the session." (Abstract)
    - "However, there was no significant effect of scheduled breaks on reported symptoms ( P = .70), reading speed ( P = .93), or task accuracy ( P = .55)." (Abstract)
- Caveat: n = 30, 40-minute tablet reading task, four break schedules. Symptoms rose significantly in all four conditions including the most frequent break schedule. Included here as a caution against exhale's own genre: a periodic on-screen nudge isn't self-evidently effective just because it's popular and plausible.
  - "The study was carried out on 30 young subjects who performed a 40-minute, cognitively demanding reading task from a tablet computer." (Abstract)
  - "A significant increase in post-task symptoms (with respective to the pre-task value) was observed for all four trials ( P < .001)." (Abstract)
  - "Although widely cited as a treatment option, these results do not support the proposal of using 20-second scheduled breaks as a therapeutic intervention for digital eye strain." (Abstract)
- Quotes checked: 6 of 6 against the Europe PMC abstract, last on 2026-10-18

#### `jung2016-smartphone-posture-respiration`

Jung, Sang In; Lee, Na Kyung; Kang, Kyung Woo; Kim, Kyoung; Lee, Do Youn. (2016). *The effect of smartphone usage time on posture and respiratory function*. Journal of Physical Therapy Science 28(1): 186-189

- DOI: [10.1589/jpts.28.186](https://doi.org/10.1589/jpts.28.186)
- Verification: crossref-verified | Access: open-access | Read: full text | evidence tier **C**
- Backs:
  - people using smartphones more than four hours a day had lower craniovertebral angle (more forward head posture), lower scapular index (more rounded shoulders) and lower peak expiratory flow than people using them less than four hours a day, differences the authors report as significant
    - "group 1 for those that used smartphones less than 4 hours per day (3.0 ± 1.0 hours), and group 2 for those that used smartphones for more than 4 hours each day (5.5 ± 1.0 hours)" (SUBJECTS AND METHODS)
    - "Significant differences were noted for CVA, scapular index, and PEF of the 2 groups (p < 0.05)." (RESULTS)
    - "As stated earlier, a lower CVA indicates greater FHP. In our study, subjects who use smartphone more than 4 hours per day had lower CVAs." (DISCUSSION)
    - "A lower scapular index indicates a higher degree of rounded shoulders. In our study, subjects who used smartphones more than 4 hours per day, had lower scapular index." (DISCUSSION)
    - "CVA (°) 54.5 (± 4.2) 53.0 (± 6.3)\* Round shoulder Scapular index 67.5 (± 4.2) 65.5 (± 6.5)\* Respiratory function test FVC (l) 3.3 (± 0.9) 2.8 (± 0.9) FEV1 (l) 3.0 (± 0.8) 2.5 (± 0.8) FEV1/FVC (%) 91.9 (± 9.9) 91.1 (± 8.6) PEF (l/sec) 6.2 (± 2.3) 4.3 (± 1.5)\*" (Table 1)
- Caveat: n = 50, cross-sectional, two groups split on self-reported usage. The abstract describes the groups as randomly allocated, which can't hold for groups defined by their own usage. Correlational: heavy users may differ in many ways besides screen time, and the authors report peak expiratory flow as the only respiratory measure that differed significantly, with FVC and FEV1 also lower in the heavier-use group. Recomputed with the independent t-test the Methods name, Table 1's means and SDs give p of about 0.33 for craniovertebral angle, 0.20 for scapular index, 0.001 for peak expiratory flow and 0.03 for FEV1. Only the significance mark on peak expiratory flow holds up, and FEV1, which carries none, would pass. The authors call the sample relatively small and didn't test whether posture and respiratory function were related. Crossref carries no license, so access level is taken from J Phys Ther Sci being fully open access.
  - "However, other variables (FVC, FEV1, FEV1/FVC) also showed lower values like PEF in subjects using smartphone for prolonged duration." (DISCUSSION)
  - "A total of 50 subjects participated in the study, with 25 subjects in each group." (RESULTS)
  - "The subjects were divided in to 2 groups according to the duration of smartphone usage reported by each individual" (SUBJECTS AND METHODS)
  - "In respiratory function, Only PEF showed significant difference between the two groups." (DISCUSSION)
  - "The number of subjects was relatively small and we did not investigate the statistical relationship between posture and respiratory function." (DISCUSSION)
  - "This is an open-access article distributed under the terms of the Creative Commons Attribution Non-Commercial No Derivatives (by-nc-nd) License" (License note, first page)
  - "Participants were randomly allocated to 2 groups: group 1 (subjects who used smartphones for <4 hours/day, n=25) and group 2 (subjects who used smartphones for >4 hours/day, n=25)." (Abstract)
  - "Data were analyzed by using the independent t-test to examine for differences in results between the 2 groups." (SUBJECTS AND METHODS)
- Quotes checked: 13 of 13 against the Europe PMC full text and the Europe PMC abstract, last on 2026-10-18

#### `rosenfield2011-computer-vision-syndrome`

Rosenfield, Mark. (2011). *Computer vision syndrome: a review of ocular causes and potential treatments*. Ophthalmic and Physiological Optics 31(5): 502-515

- DOI: [10.1111/j.1475-1313.2011.00834.x](https://doi.org/10.1111/j.1475-1313.2011.00834.x)
- Open copy: <https://onlinelibrary.wiley.com/doi/full/10.1111/j.1475-1313.2011.00834.x>
- Verification: crossref-verified | Access: paywalled | Read: full text | evidence tier **D**
- Backs:
  - the review reports that blink rate falls during computer operation, citing means of 22 per minute relaxed, 10 reading a book and 7 viewing a VDT
    - "Several investigations have shown that blink rate is reduced during computer operation. [...] Mean blink rates were 22 per min while relaxed, but only 10 and 7 per min when viewing the book or VDT, respectively." (CVS and dry eye, Reduced blink rate)
  - reduced and incomplete blinking, with greater corneal exposure, is the review's probable mechanism for the dry-eye half of computer vision syndrome
    - "In considering ocular factors that may lead to CVS, two primary areas have been identified namely: (1) inappropriate oculomotor responses and (2) dry eye." (Ocular causes of CVS)
    - "Accommodation and vergence responses to electronic screens appear to be similar to those found when viewing printed materials, whereas the prevalence of dry eye symptoms is greater during computer operation. The latter is probably due to a decrease in blink rate and blink amplitude, as well as increased corneal exposure resulting from the monitor frequently being positioned in primary gaze." (Abstract)
    - "In contrast, symptoms of dry eye do appear to be directly related to computer use due to the position of the monitor (producing increased corneal exposure), reduced blink rate, increased partial blinking and other environmental factors." (Conclusions)
- Caveat: Narrative review of the ocular literature. The version of record is served free at the publisher, which labels it Free Access, and the only license Crossref registers for it is Wiley's own terms, so it's marked paywalled here with the publisher page as the open copy. Load-bearing for the blink half of exhale's opening claim only. exhale does nothing about blinking. See the gaps ledger.
- Quotes checked: 4 of 4 against the publisher's page and the Europe PMC abstract, last on 2026-10-18

#### `schleifer1994-vdt-petco2`

Schleifer, Lawrence M.; Ley, Ronald. (1994). *End-tidal PCO2 as an index of psychophysiological activity during VDT data-entry work and relaxation*. Ergonomics 37(2): 245-254

- DOI: [10.1080/00140139408963642](https://doi.org/10.1080/00140139408963642)
- Open copy: <https://stacks.cdc.gov/view/cdc/204606> ([archived](https://web.archive.org/web/20261018202730/https://stacks.cdc.gov/view/cdc/204606))
- Verification: crossref-verified | Access: paywalled | Read: full text | evidence tier **C**
- Backs:
  - during computer data-entry work, end-tidal CO2 was significantly lower and respiration frequency significantly higher than during either baseline relaxation or progressive muscle relaxation
    - "Consistent with an increase in psychophysiological arousal, end-tidal PCO2, cardiac inter-beat interval, and relaxation ratings during data-entry work were significantly lower than during either baseline relaxation or progressive muscle relaxation, while respiration frequency and tension ratings were higher." (Abstract)
    - "However, the increase from the progressive relaxation period to the total work period [...] was significant [...] as was the difference between the total work and baseline relaxation periods" (3.1.2. Respiration rate, p. 249)
  - breathing changed measurably during repetitive VDT data-entry work compared with relaxation, and end-tidal CO2 was the only physiological measure that discriminated among all three conditions
    - "The findings indicate that end-tidal PCO2 discriminates among different psychophysiological states, and that end-tidal PCO2 may be useful in indexing the stress-health effects of human-computer interactions." (Abstract)
    - "Moreover, the small change in end-tidal [...] from the progressive relaxation condition to the total work condition was accompanied by a 26% increase in respiration rate." (4. Discussion, p. 251)
    - "The VDT data-entry condition provided for a highly repetitive, monotonous task" (1. Introduction, p. 246)
    - "End-tidal [...] was the only physiological indicator to discriminate among all of the three test conditions." (4. Discussion, p. 251)
- Caveat: n = 11 female data-entry operators recruited from a clerical-secretarial agency, tested over three consecutive six-hour days in the NIOSH work-stress laboratory, with the first two days given to practice and only the third day analyzed. End-tidal CO2 averaged 41.8 mmHg during data entry against 42.9 during baseline relaxation and 44.0 during progressive muscle relaxation. The authors call the change small, 1 to 2 mmHg, and a hyperventilatory stress effect, and all three means sit inside the 35 to 45 mmHg range that marchant2025-square-478-six treats as healthy. The paper reports end-tidal CO2 lower than at rest and never reports hypocapnia. A public-domain copy is on CDC Stacks: the article states it was prepared under US Government sponsorship. Small sample, and a laboratory simulation (entering tax-form data on a numeric keypad), though run over six-hour days. The comparison is data entry against two relaxation conditions, so it shows what demanding work at a keyboard does. It doesn't show that the screen itself is the cause, and grassmann2016-cognitive-load-respiration finds the same pattern under cognitive load generally. The authors leave for further research whether end-tidal CO2 can tell apart the cognitive and emotional demands of a task at different levels of difficulty, or of different tasks such as programming and data entry. Along with schleifer2008-emg-gaps-computer-work it's the direct evidence that keyboard work changes respiration, and it dates from 1994, well before the topic reached breathwork writing.
  - "Eleven data-entry operators were monitored continuously for three consecutive, 6 hour work days under the following conditions: (a) during a self-relaxation baseline period; (b) during an abbreviated progressive muscle relaxation period; and (c) during a period of computer-based data-entry work." (Abstract)
  - "Eleven female subjects (mean age = 25, range 18-40) with primarily a vocational and business school background were recruited from a clerical-secretarial agency" (2.1. Subjects, p. 246)
  - "The subjects performed the data-entry task at VDT workstations in the NIOSH work-stress laboratory." (2.2.1. Workstation configuration, p. 247)
  - "The task consisted of entering numerical data from US Internal Revenue Service tax forms using the numeric keypad" (2.3. Design and procedure, p. 248)
  - "Since the first two days of the study were devoted to practising the data-entry task and adapting the subjects to the psychophysiological monitoring systems, the analyses focused on data from the third day of the study only." (2.4. Data analyses, p. 249)
  - "from the baseline relaxation period (M = 42.94 mm Hg [...] to the progressive relaxation period (M = 43.97 mm Hg" (3.1.1. End-tidal PCO2, p. 249)
  - "while during the work period, breathing exceeded metabolic demands [...] denoting a hyperventilatory stress effect" (4. Discussion, p. 251)
  - "Although the mean change in end-tidal [...] is relatively small (1-2 mm Hg), it is comparable with those of other studies of the effects of psychological stress on respiration." (4. Discussion, p. 251)
  - "Further research is needed to determine the utility of this index in discriminating the cognitive and emotional demands of the same task performed at different levels of difficulty, as well as in differentiating between the cognitive and emotional demands of various types of tasks (e.g., programming vs. data entry)." (5. Conclusion, p. 252)
  - "Prepared under US Government sponsorship. This article is in the public domain." (Title footnote, p. 245)
- Quotes checked: 16 of 16 against the CDC Stacks scan and the Europe PMC abstract, last on 2026-10-18

#### `schleifer2002-hyperventilation-job-stress`

Schleifer, Lawrence M.; Ley, Ronald; Spalding, Thomas W. (2002). *A hyperventilation theory of job stress and musculoskeletal disorders*. American Journal of Industrial Medicine 41(5): 420-432

- DOI: [10.1002/ajim.10061](https://doi.org/10.1002/ajim.10061)
- Verification: crossref-verified | Access: paywalled | Read: abstract only | evidence tier **D**
- Backs:
  - hyperventilation is often characterized by a shift from a diaphragmatic to a thoracic breathing pattern
    - "Hyperventilation is often characterized by a shift from a diaphragmatic to a thoracic breathing pattern" (Abstract)
  - thoracic breathing recruits sternocleidomastoid, scalene and trapezius muscles, imposing biomechanical stress on the neck and shoulder region
    - "a thoracic breathing pattern, which imposes biomechanical stress on the neck/shoulder region due to the ancillary recruitment of sternocelidomastoid, scalene, and trapezius muscles in support of thoraci breathing" (Abstract)
  - the theory offers a rationale for coping with job stress and musculoskeletal discomfort through breathing training, light physical exercise and rest breaks
    - "A hyperventilation theory also provides a unique rationale for coping with job stress and musculoskeletal discomfort through breathing training, light physical exercise, and rest breaks." (Abstract)
- Caveat: Theory paper with no experiment of its own, hence tier D. It's nonetheless the closest thing in the peer-reviewed literature to the folk claim about *shallow* breathing at a screen: the diaphragmatic-to-thoracic shift is what people mean by shallow. Cite it for the pattern, never for a measured tidal volume. It theorizes the shift, and no study in this corpus measures it in screen users.
  - "Based on established principles of breathing and job stress and the relevant empirical literatures, a hyperventilation theory of job stress and work-related musculoskeletal disorders was developed." (Abstract)
- Quotes checked: 4 of 4 against the Europe PMC abstract, last on 2026-10-18

#### `schleifer2008-emg-gaps-computer-work`

Schleifer, Lawrence M.; Spalding, Thomas W.; Kerick, Scott E.; Cram, Jeffrey R.; Ley, Ronald; Hatfield, Bradley D. (2008). *Mental stress and trapezius muscle activation under psychomotor challenge: A focus on EMG gaps during computer work*. Psychophysiology 45(3): 356-365

- DOI: [10.1111/j.1469-8986.2008.00645.x](https://doi.org/10.1111/j.1469-8986.2008.00645.x)
- Verification: crossref-verified | Access: paywalled | Read: abstract only | evidence tier **C**
- Backs:
  - end-tidal CO2 was lower during high mental workload than during low mental workload in computer data entry, in 23 participants
    - "EMG activity and gaps in the left and right trapezii were monitored in 23 participants under low and high mental workload (LMW and HMW) demands during computer data entry." (Abstract)
    - "In addition, heart period and end-tidal CO2 were lower during HMW, whereas self-reported mood states were higher during HMW." (Abstract)
  - lower end-tidal CO2 corresponded with fewer trapezius EMG gaps, which the authors say suggests overbreathing may mediate trapezius muscle activation
    - "The correspondence between lower end-tidal CO2 and lower EMG-gap frequencies suggests that hyperventilation (overbreathing) may mediate trapezius muscle activation." (Abstract)
    - "Increases in EMG activity and decreases in EMG-gap frequencies in both left and right trapezius muscles were greater during HMW than LMW." (Abstract)
- Caveat: n = 23. A second sample, fourteen years after schleifer1994-vdt-petco2, from a team that includes both of its authors. The contrast differs: this study compares high with low mental workload during data entry, and schleifer1994-vdt-petco2 compared data entry with relaxation. Both found end-tidal CO2 lower in the more demanding condition, so the screen-breathing claim rests on two small samples pointing the same way.
  - "monitored in 23 participants" (Abstract)
- Quotes checked: 5 of 5 against the Europe PMC abstract, last on 2026-10-18

#### `sheppard2018-digital-eye-strain`

Sheppard, Amy L.; Wolffsohn, James S. (2018). *Digital eye strain: prevalence, measurement and amelioration*. BMJ Open Ophthalmology 3(1): e000146

- DOI: [10.1136/bmjophth-2018-000146](https://doi.org/10.1136/bmjophth-2018-000146)
- Verification: crossref-verified | Access: open-access | Read: full text | evidence tier **D**
- Backs:
  - digital eye strain is common among heavy display users
    - "Digital eye strain (DES), also known as computer vision syndrome, encompasses a range of ocular and visual symptoms, and estimates suggest its prevalence may be 50% or more among computer users." (Abstract)
    - "With six or more hours of computer use, contact lens wearers were more likely to be affected than non-wearers, with prevalences of 65% and 50%, respectively." (Symptoms and prevalence)
  - reduced blink rate and incomplete blinks during screen work may be relevant to its dry eye symptoms, though blink-rate reductions reported in older studies may not reflect the effect of modern displays, and incomplete blinking may be the more relevant of the two
    - "Reduced blink rate with computer use has been observed in numerous studies 33–35 and may be relevant to dry eye symptoms that frequently occur with DES." (Blinking and squinting)
    - "It is possible that technological improvements in digital displays mean that they are more similar to printed materials, so the substantial reductions in blink rate with computer use reported in older studies may not be indicative of modern effects." (Blinking and squinting)
    - "Incomplete blinking, where the upper eyelid does not cover the entire corneal surface, may be more relevant to dry eye than blink rate as tear film stability can be maintained with a reduced blink rate, providing that most blinks are complete." (Blinking and squinting)
    - "However, reading from a hard copy was associated with a significantly lower proportion of incomplete blinks (0%–5%) compared with reading from a tablet (14.5%), expanded computer display (13.5%) or electronic reading (9%; see  table 2). The specific influence of digital devices on incomplete blinks is unclear, and further research is needed to address this issue, along with the possible benefits of blink training." (Blinking and squinting)
- Caveat: The article itself is published under CC BY-NC 4.0. Crossref carries no license field for the record, so access level is taken from BMJ Open Ophthalmology being a fully open-access title.
  - "This is an Open Access article distributed in accordance with the Creative Commons Attribution Non Commercial (CC BY-NC 4.0) license" (License statement)
- Quotes checked: 7 of 7 against the Europe PMC full text and the Europe PMC abstract, last on 2026-10-18

#### `tsubota1993-vdt-blink`

Tsubota, Kazuo; Nakamori, Katsu. (1993). *Dry Eyes and Video Display Terminals*. New England Journal of Medicine 328(8): 584

- DOI: [10.1056/NEJM199302253280817](https://doi.org/10.1056/NEJM199302253280817)
- Open copy: <https://www.nejm.org/doi/pdf/10.1056/NEJM199302253280817>
- Verification: crossref-verified | Access: paywalled | Read: full text | evidence tier **C**
- Backs:
  - blink rate during display work is far below blink rate at rest: 22 per minute relaxed, 10 reading a book, 7 viewing a display, in 104 office workers
    - "We studied several factors affecting tear dynamics in 104 healthy office workers (45 men and 59 women; age range, 20 to 69 years), about half of whom used computers for an average of three hours per day." (p. 584, paragraph beginning The ocular surface is normally covered)
    - "The mean (±SD) rate of blinking, which was measured at 22.5 °C and 40 percent humidity, was 22 ±9 per minute under relaxed conditions, 10 ±6 per minute while the subjects were reading a book at table level, and 7 ±7 per minute while the subjects were viewing text on a video display terminal." (p. 584, paragraph beginning The ocular surface is normally covered)
- Caveat: A one-page letter, read in full. 104 healthy office workers, about half of them using computers for around three hours a day. Mean blink rate 22 +/- 9 per minute under relaxed conditions, 10 +/- 6 reading a book at table level, 7 +/- 7 viewing text on a display. The letter also reports a larger exposed ocular surface at the display than when reading (2.3 cm2 against 1.2, with 2.2 relaxed). It found that tear evaporation rose with exposed surface area and concludes that display use is associated with a higher evaporation rate. It reports no evaporation rate measured at the display. rosenfield2011-computer-vision-syndrome and sheppard2018-digital-eye-strain repeat its figures of 22 at rest and 7 at a screen. The display figure has a wide spread (7 +/- 7), and the letter reports means and SDs but no significance tests. For a fuller treatment, use rosenfield2011-computer-vision-syndrome or sheppard2018-digital-eye-strain.
  - "Ocular surface, calculated by measuring the width of the palpebral fissure, [...] was 2.2 ±0.4 cm2 under relaxed conditions, 1.2 ±0.4 cm2 while the subjects were reading, and 2.3 ±0.5 cm2 while the subjects were working at the video display terminal." (p. 584, paragraph beginning The ocular surface is normally covered)
  - "These results indicate that the use of video display terminals is associated with a decreased frequency of blinking and an increased rate of tear evaporation, each of which can contribute to dry eyes." (p. 584, paragraph beginning These results indicate)
  - "The rate of evaporation increased as the area of exposed ocular surface increased, and the rate of increase was greater with larger surface areas." (p. 584, paragraph beginning To evaluate the effect)
- Quotes checked: 5 of 5 against the publisher's page, last on 2026-10-18

---

## Whether slow paced breathing does anything

7 sources.

#### `chaddha2019-slow-breathing-bp`

Chaddha, Ashish; Modaff, Daniel; Hooper-Lane, Christopher; Feldstein, David A. (2019). *Device and non-device-guided slow breathing to reduce blood pressure: A systematic review and meta-analysis*. Complementary Therapies in Medicine 45: 179-184

- DOI: [10.1016/j.ctim.2019.03.005](https://doi.org/10.1016/j.ctim.2019.03.005)
- Verification: crossref-verified | Access: paywalled | Read: abstract only | evidence tier **A**
- Backs:
  - slow breathing practiced for at least four weeks lowered systolic blood pressure by 5.62 mmHg and diastolic by 2.97 mmHg, pooled across trials of people with hypertension or prehypertension or on antihypertensive medication
    - "Overall, slow breathing decreased SBP by -5.62 mmHg [-7.86, -3.38] and DBP by -2.97 mmHg [-4.28, -1.66]." (Abstract)
    - "Interest is increasing in nonpharmacological interventions to treat blood pressure in hypertensive and prehypertensive patients at low cardiac risk. This meta-analysis of randomized controlled trials assesses the impact of device-guided and non-device-guided (pranayama) slow breathing on blood pressure reduction in these patient populations." (Abstract)
    - "randomized controlled trial or first phase of a randomized cross-over study; subjects with hypertension, prehypertension or on antihypertensive medication; intervention consisting of slow breathing at ≤10 breaths/minute for ≥5 min on ≥3 days/week; total intervention duration of ≥4 weeks; follow-up for ≥4 weeks; and a control group" (Abstract)
- Caveat: 17 trials, each a randomized controlled trial or the first phase of a randomized cross-over study. Heterogeneity was high for every analysis, and the authors say so. Inclusion required at least 5 minutes of breathing at 10 breaths/min or slower, on at least 3 days a week, for at least 4 weeks. exhale asks for none of that and measures none of it, so this describes a dose exhale doesn't deliver. Read alongside vandijk2018-close-the-book.
  - "Of 103 citations eligible for full-text review, 17 studies were included in the meta-analysis." (Abstract)
  - "studies meeting these inclusion criteria: randomized controlled trial or first phase of a randomized cross-over study" (Abstract)
  - "intervention consisting of slow breathing at ≤10 breaths/minute for ≥5 min on ≥3 days/week; total intervention duration of ≥4 weeks" (Abstract)
  - "Heterogeneity was high for all analyses." (Abstract)
- Quotes checked: 7 of 7 against the Europe PMC abstract, last on 2026-10-18

#### `fincham2023-breathwork-meta`

Fincham, Guy William; Strauss, Clara; Montero-Marin, Jesus; Cavanagh, Kate. (2023). *Effect of breathwork on stress and mental health: A meta-analysis of randomised-controlled trials*. Scientific Reports 13(1): 432

- DOI: [10.1038/s41598-022-27247-y](https://doi.org/10.1038/s41598-022-27247-y)
- Verification: crossref-verified | Access: open-access | Read: full text | evidence tier **A**
- Backs:
  - breathwork is associated with lower self-reported stress than control conditions, g = -0.35 (95% CI -0.55 to -0.14), 12 RCTs, 785 adults
    - "The primary outcome self-reported/subjective stress included 12 randomised-controlled trials (k = 12) with a total of 785 adult participants." (Abstract)
    - "The random-effects analysis yielded a significant small-to-medium mean effect size, g = - 0.35 [95% CI - 0.55, - 0.14], z = 3.32, p = 0.0009, showing breathwork was associated with lower levels of stress than control conditions." (Abstract)
  - comparable small-to-medium effects for anxiety (g = -0.32, k = 20) and depressive symptoms (g = -0.40, k = 18)
    - "Meta-analyses for secondary outcomes of self-reported/subjective anxiety (k = 20) and depressive symptoms (k = 18) showed similar significant effect sizes: g = - 0.32, p < 0.0001, and g = - 0.40, p < 0.0001, respectively." (Abstract)
    - "This meta-analysis found significant small-medium effects of breathwork on self-reported/subjective stress, anxiety and depression compared to non-breathwork control conditions." (Conclusions)
  - adverse-event reporting in breathwork trials is sparse: four of the twelve primary-outcome trials reported on it, none attributed lasting harm to breathwork, and in one fast-paced trial a small subgroup reported side effects such as hot flushes, shortness of breath or sweating and chose to continue
    - "Risk of bias scoring for the 12 studies on the primary outcome is reported using RoB 2 in Fig. 2. [...] We did not find reported adverse events or lasting bad effects directly attributed to breathwork interventions; four studies (six in total including secondary outcome studies) actively reported on this." (Results, Risk of bias for stress)
    - "Nonetheless, regarding safety and tolerability, a small subgroup of participants in Ravindran et al.’s study [...] focusing on fast-paced breathwork in unipolar and bipolar depression reported side effects such as hot flushes, shortness of breath and/or sweating. However, these participants opted to continue the intervention and no participants dropped out of the breathwork group due to adverse effects." (Results, Risk of bias for stress)
- Caveat: Most included studies were rated at moderate risk of bias. Effects are small-to-medium and self-reported. The authors conclude that breathwork may be effective for stress and mental health and urge caution, asking for low risk-of-bias designs to avoid a miscalibration between hype and evidence. Only four of the twelve primary-outcome trials reported on adverse events, none attributed lasting harm to breathwork, and in one fast-paced trial a small subgroup reported side effects such as hot flushes, shortness of breath or sweating. The authors call for better reporting, particularly for fast-paced techniques. This is the strongest single warrant for the claim that a breathing practice does something, and it's still an effect of about a third of a standard deviation. The first author declares training as a Breath Teacher with The Breath-Body-Mind Foundation. The stress effect was significant for nonclinical samples and not for the mental-health (one study) or physical-health subgroups, though the subgroups didn't differ significantly from one another and the authors say those analyses may have been underpowered.
  - "Most studies were deemed as being at moderate risk of bias." (Abstract)
  - "Lastly, more studies should report on adverse events and lasting bad effects, with further research needed to gauge the safety profile of fast-paced breathwork in particular, so it not administered blindly to potentially vulnerable populations." (Strengths, limitations and future directions)
  - "G.W.F. has trained as a Breath Teacher with The Breath-Body-Mind Foundation, New York." (Competing interests)
  - "There was a significant effect of breathwork on stress in nonclinical samples, but not in mental (only one study) or physical health populations." (Results, Subgroup analyses for stress)
  - "While nonclinical samples showed a significant effect on self-reported/subjective stress outcomes and physical and mental health samples did not, between-subgroup differences were non-significant and the point estimates for these subgroups were similar (ranging from ES = 0.26–0.38). These findings could mean that breathwork is not effective for physical/mental health populations, however, it is also possible that this analysis was underpowered to detect effects given the relatively small number of studies contributing to the subgroups, as we have already mentioned." (Discussion, Breathwork and stress)
  - "For the between-group effect sizes (ESs) we computed Hedges’ g, based on the standardised between-group difference at post-intervention considering sampling variance among groups; an ES of 0.2 is classed as small, 0.5 medium and 0.8 large" (Methods, Data synthesis and analysis)
  - "Overall, results showed that breathwork may be effective for improving stress and mental health. However, we urge caution and advocate for nuanced research approaches with low risk-of-bias study designs to avoid a miscalibration between hype and evidence." (Abstract)
- Quotes checked: 13 of 13 against the Europe PMC full text and the Europe PMC abstract, last on 2026-10-18

#### `laborde2022-vsb-meta`

Laborde, S.; Allen, M. S.; Borges, U.; Dosseville, F.; Hosang, T. J.; Iskra, M.; Mosley, E.; Salvotti, C.; Spolverato, L.; Zammit, N.; Javelle, F. (2022). *Effects of voluntary slow breathing on heart rate and heart rate variability: A systematic review and a meta-analysis*. Neuroscience & Biobehavioral Reviews 138: 104711

- DOI: [10.1016/j.neubiorev.2022.104711](https://doi.org/10.1016/j.neubiorev.2022.104711)
- Open copy: <https://hal.science/hal-04925897> ([archived](https://web.archive.org/web/20261018205231/https://hal.science/hal-04925897))
- Verification: crossref-verified | Access: paywalled | Read: full text | evidence tier **A**
- Backs:
  - voluntary slow breathing raises vagally-mediated HRV during the session, immediately after a single session, and after a multi-session intervention
    - "Specifically, we focus on HRV parameters indexing PNS activity regulating cardiac functioning, referred to as vagally-mediated (vm)HRV: (1) during the breathing session (i.e., DURING), (2) immediately after one training session (i.e., IM-AFTER1), as well as (3) after a multi-session intervention (i.e., AFTER-INT). [...] Results indicate increases in vmHRV with VSB, DURING, IM-AFTER1, and AFTER-INT." (Abstract)
    - "To conclude, this meta-analysis shows that voluntary slow breathing leads to an increase in the parasympathetic nervous control of the heart, underlined by an increase in RMSSD in all three time points (DURING, IM-AFTER1, AFTER-INT) and LFabs DURING." (5 CONCLUSION)
  - the authors remark in their conclusion that few adverse effects were reported across the included studies, giving grade 1 lightheadedness as an example, though the review's data extraction didn't include adverse events
    - "Given the involvement of the PNS in a large range of health-related outcomes and conditions, VSB exercises could be advised as a low-tech and low-cost technique to use in prevention and adjunct treatment purposes, with few adverse effects expected." (Abstract)
    - "Moreover, very few adverse effects are reported throughout the studies (e.g., GRADE 1 lightheadedness), providing further support for their use." (5 CONCLUSION)
    - "For each study, the following characteristics were extracted: study ID, year of publication, study design (withinvs. between-subject design), sample size, sample characteristics (healthy vs. patients; age; gender); characteristics of voluntary slow breathing intervention and control: nose vs. mouth breathing, breathing depth, main locus of breathing (chest vs. abdomen), respiratory frequency, inhalation/exhalation ratio, use of biofeedback, type of breathing pacer (e.g., visual, audio, kinesthetic), intervention length, as well as HRV parameters." (2.2 Data Collection and Data Items)
- Caveat: 223 studies from 1842 selected abstracts (172 during, 16 immediately-after, 49 after-intervention). This is the central warrant for exhale's whole premise. It finds an effect on RMSSD, a cardiac index of parasympathetic activity, that is medium during the session (effect size 0.530) and small immediately after one session (0.140) and after a multi-session intervention (0.320), and it says nothing about how anyone feels or works. The authors rated 215 studies at high risk of bias, 5 with some concerns and 1 at low risk, and say the findings need careful interpretation.
  - "From the 1842 selected abstracts, 223 studies were suitable for inclusion (172 DURING, 16 IM-AFTER1, and 49 AFTER-INT)." (Abstract)
  - "The primary outcomes were HRV parameters related to cardiac vagal activity: RMSSD and LFabs for DURING, and RMSSD for both IM-AFTER1 and AFTER-INT. [...] The secondary outcomes were heart rate and the remaining HRV parameters." (2.2 Data Collection and Data Items)
  - "Results from the main analysis after outliers’ exclusion show a medium effect size 0.530 (95% CI, 0.430 to 0.620) with large general heterogeneity (I² = 81%) and moderate variance of the true effect (tau² = 0.140) for RMSSD (k = 71)" (3.3 Effects DURING)
  - "Results from the main-analysis after outlier’s exclusion for IM-AFTER1 show a small effect size of 0.140 (95% CI, 0.030 to 0.240) with no heterogeneity (I² = 0%; tau² = 0) for RMSSD." (3.4 Effects IM-AFTER1)
  - "Results from the main-analysis for AFTER-INT show a small effect size of 0.320 (95% CI, 0.080 to 0.560) with large heterogeneity (I² = 79%; tau² = 0.219) for RMSSD (k = 22)." (3.5 Effects AFTER-INT)
  - "Overall Risk of Bias was high in 215 studies, with some concerns in 5 studies, and low in 1" (3.2 Risk of Bias)
  - "However, the large majority of studies included present a high risk of bias, which highlights the need for careful interpretation of findings." (5 CONCLUSION)
- Quotes checked: 12 of 12 against the HAL repository copy and the Europe PMC abstract, last on 2026-10-18

#### `little2025-a52-breath-method`

Little, Abbie L. (2025). *The A52 Breath Method: A Narrative Review of Breathwork for Mental Health and Stress Resilience*. Stress and Health 41(4): e70098

- DOI: [10.1002/smi.70098](https://doi.org/10.1002/smi.70098)
- Verification: crossref-verified | Access: open-access | Read: full text | evidence tier **D**
- Backs:
  - a 5 s inhale, 5 s exhale, 2 s post-exhale hold at five breaths per minute is a protocol a recent review proposes and argues aligns closely with the most effective protocols in the literature, while stating that no study to date has directly tested the 5-5-2 sequence
    - "This review analyses the existing literature on slow, diaphragmatic, nasal breathing and breath-holding techniques, to propose the A52 Breath Method-a theoretically grounded approach for enhancing stress resilience." (Abstract)
    - "The method involves a 5‐s nasal inhalation, 5‐s nasal or mouth exhalation, and a 2‐s post‐exhalation hold—repeated for 10 min to achieve five‐breaths‐per‐minute (5‐5‐2 configuration)." (1.1 Introducing the A52 Breath Method)
    - "The A52 Breath Method, with its 5‐5‐2 pacing, bridges theory with real‐world applicability, aligning closely with the most effective protocols in the literature." (5.1 Conclusion)
    - "While no studies to date have directly tested the 5‐5‐2 breathing sequence, converging evidence from slow, paced breathing, extended exhalation, nasal inhalation, and rhythm consistency protocols, including those incorporating brief pauses (e.g. Sevoz‐Couche and Laborde 2022), suggests this sequence may have advantageous effects." (1.1 Introducing the A52 Breath Method)
  - 23 of 30 reviewed studies reported significant HRV improvement, 10 reported anxiety reduction and 9 reported reduced perceived stress
    - "Across the 30 included studies, HRV emerged as the most frequently reported physiological outcome, with 23 studies noting significant improvements in at least one HRV parameter (e.g. RMSSD, HF‐HRV, LF/HF ratio). [...] Psychologically, anxiety reduction was reported in 10 studies. Additionally, decreased perceived stress (n = 9), PTSD (n = 2), and enhanced emotional regulation (n = 2) was reported across clinical and healthy populations." (3 Results and Overview of Included Studies)
  - in one study the review cites, a single 2-minute session of slow-paced breathing gave the largest HRV gains to people with high distress or a faster resting respiratory rate
    - "Moreover, greater physiological and psychological gains were found in those with higher resting respiratory rates and psychological distress, as well as those who practiced their technique more frequently (Balban et al. 2023; Pozzato et al. 2025)." (4.1 Physiological Effects of Breathwork)
    - "Slow‐paced breathing (6 bpm, 5:5 I:E ratio); single 2‐min session; visual pacing" (Table 2, Pozzato et al. 2025)
    - "Individuals with high distress or faster resting respiratory rate showed the largest HRV gains." (Table 2, Pozzato et al. 2025)
- Caveat: Narrative review, single author, 465 records returned, 269 screened by title and abstract after de-duplication, 93 full texts assessed and 30 analyzed, with no meta-analysis and no risk-of-bias assessment. The paper's counts don't reconcile: 269 screened less 174 marked irrelevant leaves 95, and its abstract says 30 studies underwent full-text review. It proposes the protocol it reviews and grades the included techniques by their similarity to it, which is a conflict of framing, and it states that no study to date has directly tested the 5-5-2 sequence. Its A52 shape maps onto exhale's four sliders as 5 / 0 / 5 / 2, which is 5 breaths per minute and inside the tested band. The review says limited research exists on brief breath holds after the exhale, and it cites a suggestion that a pause longer than 1 s may help mitigate stress. marchant2025-square-478-six found that no-hold 6 bpm outperformed both hold-bearing patterns it tested, so in this corpus the 2 s retention has the least direct support.
  - "A narrative review of breathwork literature was conducted that focused on the physiological and psychological mechanisms underpinning stress reduction." (Abstract)
  - "Abbie L. Little: conceptualization, methodology, data curation, formal analysis, investigation, writing – original draft, and writing – review and editing." (Author Contributions)
  - "The search identification query returned 465 articles: 196 of these were removed as duplicates and 269 studies were independently screened based on titles and abstracts. From these, 174 studies were marked irrelevant and 93 full‐text studies were screened for eligibility; then, 63 studies were further excluded as they did not meet all of the inclusion criteria, leaving 30 studies to undergo analysis (see Figure 1)." (2 Methodology)
  - "The A52 Breath Method is a new unvalidated breathing protocol developed by the author, Abbie Little, through theoretical and practitioner insights while being specifically designed for mental resilience and stress reduction." (1.1 Introducing the A52 Breath Method)
  - "Figure 4 classifications were determined on the degree of similarity to the novel A52 breath method, which emphasizes slow nasal diaphragmatic breathing (5‐s inhale, 5‐s exhale), with a post‐exhalation hold and a focus on simple, repeatable daily practice." (Figure 4 caption)
  - "Limited research exists on brief breath hold retention, especially following the exhalation." (5 Gaps and Future Research Directions)
  - "Medical databases were searched: 465 articles were screened and 30 studies underwent full-text review." (Abstract)
  - "While the optimal pattern and frequency of slow paced breathing remains undecided, and limited studies have isolated the effects of a 2‐s pause, researchers have suggested that a delay of greater than 1 s may optimize the mitigation of stress" (1.1 Introducing the A52 Breath Method)
- Quotes checked: 16 of 16 against the Europe PMC full text and the Europe PMC abstract, last on 2026-10-18

#### `russo2017-slow-breathing-physiology`

Russo, Marc A.; Santarelli, Danielle M.; O'Rourke, Dean. (2017). *The physiological effects of slow breathing in the healthy human*. Breathe 13(4): 298-309

- DOI: [10.1183/20734735.009817](https://doi.org/10.1183/20734735.009817)
- Verification: crossref-verified | Access: open-access | Read: full text | evidence tier **D**
- Backs:
  - slow breathing improves ventilation efficiency and alters cardiorespiratory coupling, respiratory sinus arrhythmia and sympathovagal balance
    - "Key findings include effects on respiratory muscle activity, ventilation efficiency, chemoreflex and baroreflex sensitivity, heart rate variability, blood flow dynamics, respiratory sinus arrhythmia, cardiorespiratory coupling, and sympathovagal balance." (Key points)
    - "Enhances ventilation efficiency and arterial oxygenation via alveolar recruitment, and distension and reduction of alveolar dead space" (Table 1)
    - "Increases RSA (maximises around 6 breaths per min (resonant frequency))" (Table 1)
    - "Clustering of heartbeats within inspiratory phase (cardiorespiratory coupling)" (Table 1)
    - "Shift towards parasympathetic dominance" (Table 1)
- Caveat: Narrative review. The authors close by calling explicitly for further research, and describe the health claims as potential without calling them demonstrated. Use it to explain a mechanism, and don't cite it for an outcome. Crossref registers no license for this DOI, and the article itself carries a CC BY-NC 4.0 license, as every Breathe article does.
  - "We initially performed a Medline search via PubMed for articles reviewing or reporting on the effects of breathing at 4–10 breaths per min or 0.07–0.16 Hz in humans. [...] The Medline search expanded during the writing of the manuscript to incorporate literature pertaining to the normal physiology of the respiratory, cardiovascular, cardiorespiratory and autonomic nervous systems, and other topics relevant to the review." (Review methods)
  - "Slow breathing practices have been adopted in the modern world across the globe due to their claimed health benefits." (Abstract)
  - "There appears to be potential for use of controlled slow breathing techniques as a means of optimising physiological parameters that appear to be associated with health and longevity, and that may extend to disease states; however, there is a dire need for further research into the area." (Key points)
  - "A hierarchy of research will provide a foundation upon which any therapeutic claims can be tested and validated." (Future research directions)
  - "To review and discuss the evidence and hypotheses regarding the mechanisms underlying slow breathing physiological effects in humans." (Educational aims)
- Quotes checked: 10 of 10 against the Europe PMC full text and the Europe PMC abstract, last on 2026-10-18

#### `vandijk2018-close-the-book`

van Dijk, Peter R.; van Hateren, Kornelis J. J.; Kleefstra, Nanne; Landman, Gijs W. D. (2018). *It is time to close the book on device-guided slow breathing*. Blood Pressure 27(3): 181-182

- DOI: [10.1080/08037051.2018.1435260](https://doi.org/10.1080/08037051.2018.1435260)
- Open copy: <https://www.tandfonline.com/doi/pdf/10.1080/08037051.2018.1435260>
- Verification: crossref-verified | Access: paywalled | Read: full text | no evidence tier (not a study)
- Backs:
  - a letter to the editor argues that device-guided slow breathing has no beneficial effect on blood pressure and that the question should be closed
    - "It is time to close the book on device-guided slow breathing" (Title)
    - "When considering the methodological quality, the highest quality trials do not show any effect of DGB on blood pressure as was found in the only double-blind (and sham-controlled) trial [...] Furthermore, the absence of any possible relevant effect has been confirmed in meta-analyses of trials using individual patient data of different trials and aggregated data of more trials without using individual patient data" (Letter, paragraph starting Firstly)
    - "In addition, evidence from meta-analyses of randomized trials and systematic reviews have clearly demonstrated, as this study does (!), that DBG has no beneficial effects on reducing blood pressure." (Letter, paragraph starting In conclusion)
- Caveat: Letter, two pages, so no evidence tier applies. Its evidence includes the only double-blind, sham-controlled trial and an individual-patient-data meta-analysis, and both share authors with the letter. Carried because it's the strongest published dissent against the cardiovascular claims exhale sits next to, and leaving it out would make the corpus one-sided.
  - "Letter to the Editor [...] Pages 181-182" (Article header)
  - "Peter R. van Dijk,Kornelis J.J. van Hateren,Nanne Kleefstra &Gijs W.D. Landman" (Article header)
  - "Landman GW, van Hateren KJ, van Dijk PR, et al. Efficacy of device-guided breathing for hypertension in blinded, randomized, active-controlled trials: a meta-analysis of individual patient data." (References)
  - "Landman GW, Drion I, van Hateren KJ, et al. Device-guided breathing as treatment for hypertension in type 2 diabetes mellitus: a randomized, double-blind, sham-controlled trial." (References)
- Quotes checked: 7 of 7 against the publisher's page, last on 2026-10-18

#### `zaccaro2018-slow-breathing-review`

Zaccaro, Andrea; Piarulli, Andrea; Laurino, Marco; Garbella, Erika; Menicucci, Danilo; Neri, Bruno; Gemignani, Angelo. (2018). *How Breath-Control Can Change Your Life: A Systematic Review on Psycho-Physiological Correlates of Slow Breathing*. Frontiers in Human Neuroscience 12: 353

- DOI: [10.3389/fnhum.2018.00353](https://doi.org/10.3389/fnhum.2018.00353)
- Verification: crossref-verified | Access: open-access | Read: full text | evidence tier **B**
- Backs:
  - in healthy adults aged 18 to 65, slow breathing, defined in this review as 10 breaths per minute or fewer, is associated with increased HRV and shifts in central and autonomic measures
    - "This systematic review is aimed at unveiling psychophysiological mechanisms underlying slow breathing techniques (<10 breaths/minute) and their effects on healthy subjects." (Abstract)
    - "Slow breathing techniques promote autonomic changes increasing Heart Rate Variability and Respiratory Sinus Arrhythmia paralleled by Central Nervous System (CNS) activity modifications." (Abstract)
    - "Slow breathing techniques (related both to slow paced breathing and to HRV Biofeedback) seem to interact with the cardio-respiratory system by increasing HRV and RSA, suggesting thus a strong involvement of the parasympathetic nervous system (Reyes del Paso et al., 1993; Berntson et al., 1997)." (Discussion, Summary of main findings)
    - "Studies identified from the literature search were included if: [...] They were conducted on healthy humans (both expert or naïve for breathing techniques) [...] Any technique of breath control that directly slows the breath down to 10 breaths per minute was used" (Methods, Study design)
    - "Studies identified from the literature search were excluded if: [...] Young (<18 years) and/or old (>65 years) subjects were recruited" (Methods, Study design)
  - reported psychological outcomes include increased comfort and relaxation and reduced anxiety and arousal
    - "Psychological/behavioral outputs related to the abovementioned changes are increased comfort, relaxation, pleasantness, vigor and alertness, and reduced symptoms of arousal, anxiety, depression, anger, and confusion." (Abstract)
- Caveat: Systematic review without meta-analysis: 15 articles from 2,461 abstracts. The authors intended a meta-analysis and found pooling infeasible, and they describe the HF and LF outcomes across the included studies as heterogeneous and contradictory. The included studies vary widely in technique, rate and duration, and none used a randomized controlled or longitudinal design. The review runs no dose-response analysis, though its conclusions single out 6 breaths per minute. No included study estimated correlations between the physiological changes and the psychological outcomes, apart from one that correlated physiology with stable personality traits.
  - "From a pool of 2,461 abstracts only 15 articles met eligibility criteria and were included in the review." (Abstract)
  - "The initial aim of this work was to conduct a meta-analysis of the existent literature, but due to the heterogeneity of the selected experimental groups, of the interventions, and of the outcomes, a statistical pooling was infeasible." (Limitations)
  - "At variance, when considering HF and LF power, a heterogeneous and contradictory set of outcomes was found, mainly depending on the breathing frequency: Park and Park (2012), and Stark et al. (2000) observed HF power increases (slow breathing techniques vs. control condition), while other studies found no changes (Siepmann et al., 2008; Kharya et al., 2014; Lin et al., 2014) or even HF power decreases (Lehrer et al., 2003)." (Discussion, Summary of main findings)
  - "We must underline that the paucity of collected evidence is mostly ascribable to the heterogeneity of the investigated techniques and of the participants selection criteria." (Discussion, Summary of main findings)
  - "No studies adopted longitudinal or randomized controlled designs." (Risk of bias)
  - "In particular, we found reliable associations between increase of HRV power and of LF power, increase of EEG alpha and decrease of EEG theta power, induced by slow breathing techniques at 6 b/min, and positive psychological/behavioral effects." (Conclusions)
  - "Moreover, no study explicitly estimated the correlations between physiological modifications and psychological/behavioral outcomes, with the notable exception of Park and Park (2012), which, however, focused on the correlation between changes of HRV- and EEG-related physiology (during slow breathing techniques) and stable personality traits" (Discussion, Summary of main findings)
- Quotes checked: 13 of 13 against the Europe PMC full text and the Europe PMC abstract, last on 2026-10-18

---

## What the numbers should be

15 sources.

#### `bae2021-exhalation-inhalation-ratio`

Bae, Dalbyeol; Matthews, Jacob J. L.; Chen, J. Jean; Mah, Linda. (2021). *Increased exhalation to inhalation ratio during breathing enhances high-frequency heart rate variability in healthy adults*. Psychophysiology 58(11): e13905

- DOI: [10.1111/psyp.13905](https://doi.org/10.1111/psyp.13905)
- Verification: crossref-verified | Access: paywalled | Read: abstract only | evidence tier **C**
- Backs:
  - a 2:1 exhale-to-inhale cue raised RMSSD and HF-HRV relative to a 1:1 cue at the participant's own breathing rate
    - "Twenty-eight individuals (16 young [6M, age = 21-28];12 older adults [6M, age = 66-80]) completed a task during which they paced breathing according to their intrinsic respiratory rate, but altered onset of exhalation and inhalation according to 1:1 sound cue (equal exhalation and inhalation duration) or 2:1 cue (exhalation twice as long as inhalation)." (Abstract)
    - "Analyses of HRV metrics by block showed that RMSSD and HF-HRV were higher in the 2:1 task condition compared to 1:1." (Abstract)
  - after the 2:1 block ended, HF-HRV rose during the paced 1:1 breathing that followed and stayed elevated for four minutes
    - "Paced 1:1 breathing followed these task conditions to examine residual effects." (Abstract)
    - "Time series analysis showed that HF-HRV increased after the end of the 2:1 task block and remained elevated for four minutes." (Abstract)
- Caveat: n = 28 (16 young, 12 older). In the manipulation check, the achieved ratios were 1.08 under the 1:1 cue and 1.33 under the 2:1 cue, so the longer-exhale condition fell well short of the instructed 2:1. Pacing was at each participant's spontaneous rate, which meehan2024-longer-exhalations reports as 14.16 and 14.47 breaths a minute in the two conditions, far above the resonance range. meehan2024-longer-exhalations rates the study at high risk of bias, citing incomplete randomization, more practice at the 1:1 ratio, participants not breathing at the instructed ratio and unreported baseline differences. One of six studies in this corpus that disagree about the ratio. See vandiest2014-ie-ratio-relaxation and laborde2021-ie-ratio-pauses (also positive), lin2014-equal-ratio-hrv (favors the equal ratio), and meehan2024-longer-exhalations and marchant2025-square-478-six (both null). Gap 4 in the ledger tabulates all six.
  - "Twenty-eight individuals (16 young [6M, age = 21-28];12 older adults [6M, age = 66-80])" (Abstract)
  - "Estimates of actual E:I ratio based on thoracic movement were 1.08(0.16) for 1:1 task and 1.33(0.20) for 2:1 task, which were significantly different from one another." (Abstract)
  - "We examined the impact of manipulating exhalation to inhalation ratio (E:I) on HRV, without altering the intrinsic breathing rate of healthy individuals." (Abstract)
- Quotes checked: 7 of 7 against the Europe PMC abstract, last on 2026-10-18

#### `balban2023-cyclic-sighing`

Balban, Melis Yilmaz; Neri, Eric; Kogon, Manuela M.; Weed, Lara; Nouriani, Bita; Jo, Booil; Holl, Gary; Zeitzer, Jamie M.; Spiegel, David; Huberman, Andrew D. (2023). *Brief structured respiration practices enhance mood and reduce physiological arousal*. Cell Reports Medicine 4(1): 100895

- DOI: [10.1016/j.xcrm.2022.100895](https://doi.org/10.1016/j.xcrm.2022.100895)
- Verification: crossref-verified | Access: open-access | Read: full text | evidence tier **C**
- Backs:
  - five minutes a day of exhale-emphasizing cyclic sighing gave a larger rise in positive affect from before to after each day's practice, and a larger fall in respiratory rate over one month, than an equal period of mindfulness meditation
    - "Here, we report a remote, randomized, controlled study (NCT05304000) of three different daily 5-min breathwork exercises compared with an equivalent period of mindfulness meditation over 1 month." (Summary)
    - "Using a mixed-effects model, we show that breathwork, especially the exhale-focused cyclic sighing, produces greater improvement in mood (p < 0.05) and reduction in respiratory rate (p < 0.05) compared with mindfulness meditation." (Summary)
    - "We found that the cyclic sighing group had a significantly higher increase in positive affect than those in the mindfulness meditation group (Figures 3, S2A, and S2B)." (Results)
    - "The breathwork group also showed significant physiological changes over time such that change in respiratory rate was significantly lower for the cyclic sighing group than mindfulness meditation group (Figure 4)." (Discussion)
    - "Changes in STAI state anxiety and PANAS positive and negative affect scores were calculated by subtracting the pre-condition score from the post-condition score daily for each participant." (Quantification and statistical analysis)
  - box breathing, equal inhale / hold / exhale, ran in the same trial and wasn't the best-performing arm
    - "The breathing conditions are (1) cyclic sighing, which emphasizes prolonged exhalations; (2) box breathing, which is equal duration of inhalations, breath retentions, and exhalations; and (3) cyclic hyperventilation with retention, with longer inhalations and shorter exhalations." (Summary)
    - "Cyclic sighing produced the highest daily improvement in positive affect as well as the highest reduction of respiratory rate, both significantly different from mindfulness meditation." (Discussion)
    - "then to hold their breath for the equivalent duration, then to exhale for the same duration and then to hold their breath for again, the same duration (e.g. inhale 4 s, hold 4 s, exhale 4 s, hold 4 s)" (Description of breathing protocols, Box Breathing)
- Caveat: Remote randomized controlled study with 108 participants: 24 in the mindfulness-meditation control, 30 cyclic sighing, 21 box breathing and 33 cyclic hyperventilation. Most participants were recruited from one undergraduate psychology class at Stanford, adherence was inferred from completion of the daily pre- and post-exercise surveys, and there was no follow-up after the four weeks. It was registered retrospectively as NCT05304000. The authors state it was intended as an exploratory study and 'was not pre-registered as a clinical trial'. The comparator is mindfulness meditation, an active control, so the arms differ in more than breath ratio. In the mixed-effects model, cyclic sighing separated from the control on positive affect. Box breathing and cyclic hyperventilation didn't. No arm-against-arm result is reported, and the authors say the small groups limited the statistical power to compare the breathwork arms with each other. Daily positive-affect gains were 1.89 points for cyclic sighing, 1.84 for box breathing and 1.97 for cyclic hyperventilation, so the inhale-weighted arm had the largest raw gain without separating from the control. This is the best evidence in the corpus for emphasizing the exhale, and it's weaker than the abstract suggests: the box arm is small and the difference between the two patterns isn't established. The cyclic-sighing protocol also adds a double inhale, so its effect can't be attributed to exhale length alone. Heart rate variability and resting heart rate slopes didn't differ between groups, while respiratory rate did. In raw daily means the mindfulness arm's reductions in negative affect and state anxiety were the larger ones (1.62 against 0.98 points, and 3.95 against 3.03), and the mixed-effects model found no group difference on either, so the breathwork advantage is specific to positive affect. The paper declares that one author became an advisor to WHOOP in June 2022. WHOOP donated the wrist straps that recorded the physiological outcomes, and the paper says WHOOP wasn't involved in the study's design or analysis.
  - "This study was originally intended as an exploratory study in preparation for a larger clinical trial and thus was not pre-registered as a clinical trial." (Limitations of the study)
  - "We registered the trial retrospectively (ClinicalTrials.gov: NCT05304000) and are now planning a larger confirmatory trial that will be pre-registered." (Limitations of the study)
  - "From the 108 subjects enrolled, 24 were randomized into the Mindfulness Meditation control condition and 84 were randomized to the treatment conditions (30 Cyclic Sighing, 21 Box Breathing, 33 Cyclic Hyperventilation with Retention)." (Method details)
  - "Most of the participants were recruited from an undergraduate psychology class at Stanford University, and a few by word of mouth." (Participants)
  - "In addition, we had to rely on the completion of daily surveys to assess adherence. [...] In addition, the findings of this study are limited to 4 weeks with no additional follow up." (Limitations of the study)
  - "Finally, sample size in each group was relatively small, limiting the statistical power to compare individual breathwork groups with each other." (Limitations of the study)
  - "The other two breathwork groups were also higher than mindfulness meditation; however, this difference was not significant (Figures 3, S2A, S2C, and S2D)." (Results)
  - "The average daily change per person in positive affect was 1.22 ± 2.34 for mindfulness meditation (p = 0.06) and 1.91 ± 3.38 for breathwork groups combined (p < 0.0001, 1.89 ± 3.76 for cyclic sighing [p = 0.025], 1.84 ± 3.24 for box breathing [p = 0.026], and 1.97 ± 3.21 for cyclic hyperventilation with retention [p = 0.003], where p values are based on a paired Wilcoxon test for before and after comparisons)." (Results)
  - "cyclic sighing, which differed from the other groups in two main ways: (1) extended exhalation and (2) the double inhale, which increases the depth of inhalation." (Discussion)
  - "The breathwork group had a significantly higher reduction in respiratory rate than the mindfulness meditation group (Figure 4A; p < 0.05)." (Results)
  - "As a secondary analysis, we compared each breathwork group with mindfulness meditation and found no differences in changes in resting heart rate or heart rate variability between any of the breathwork groups and mindfulness meditation (data not shown)." (Results)
  - "The average daily change per person in negative affect was −1.62 ± 1.91 for mindfulness meditation and −0.98 ± 1.39 for breathwork groups combined" (Results)
  - "The average daily change per person in state anxiety was −3.95 ± 4.16 for mindfulness meditation and −3.03 ± 3.83 for all breathwork groups combined" (Results)
  - "There were no differences between the two groups in state anxiety and negative affect changes (Figures 2B–2D)." (Results)
  - "We also thank WHOOP for generously donating the wrist straps used in the study. WHOOP was not involved in the design or analysis of this study." (Acknowledgments)
  - "A.D.H. became an advisor to WHOOP in June of 2022." (Declaration of interests)
- Quotes checked: 24 of 24 against the Europe PMC full text, last on 2026-10-18

#### `bernardi2001-slow-breathing-chemoreflex`

Bernardi, Luciano; Gabutti, Alessandra; Porta, Cesare; Spicuzza, Lucia. (2001). *Slow breathing reduces chemoreflex response to hypoxia and hypercapnia, and increases baroreflex sensitivity*. Journal of Hypertension 19(12): 2221-2229

- DOI: [10.1097/00004872-200112000-00016](https://doi.org/10.1097/00004872-200112000-00016)
- Verification: crossref-verified | Access: paywalled | Read: abstract only | evidence tier **C**
- Backs:
  - breathing at 6 per minute depressed both hypoxic and hypercapnic chemoreflex responses compared with spontaneous or 15 per minute breathing
    - "Breathing at 6 b.p.m. depressed (P < 0.01) both hypoxic and hypercapnic chemoreflex responses, compared with spontaneous or 15 b.p.m. controlled breathing." (Abstract)
  - baroreflex sensitivity was greater during slow breathing
    - "Baroreflex sensitivity was greater (P < 0.05) during slow breathing at baseline and remained greater at end rebreathing." (Abstract)
- Caveat: n = 15 healthy individuals, comparing 6 breaths per minute with spontaneous and 15-per-minute breathing. Carried to mark where the tested territory ends: 6 per minute is the slowest rate here, and nothing else in this corpus tests chemoreflex or baroreflex sensitivity at a slower pace. Gap 6 discusses what that leaves unsupported. See also bilo2012-slow-breathing-altitude.
  - "Fifteen healthy individuals." (Abstract)
  - "Progressive isocapnic hypoxia and progressive hyperoxic hypercapnia were measured during spontaneous breathing and during a breathing rate fixed at 6 and 15 breaths per minute (b.p.m.)." (Abstract)
- Quotes checked: 4 of 4 against the Europe PMC abstract, last on 2026-10-18

#### `bilo2012-slow-breathing-altitude`

Bilo, Grzegorz; Revera, Miriam; Bussotti, Maurizio; Bonacina, Daniele; Styczkiewicz, Katarzyna; Caldara, Gianluca; Giglio, Alessia; Faini, Andrea; Giuliano, Andrea; Lombardi, Carolina; Kawecka-Jaszcz, Kalina; Mancia, Giuseppe; Agostoni, Piergiuseppe; Parati, Gianfranco. (2012). *Effects of Slow Deep Breathing at High Altitude on Oxygen Saturation, Pulmonary and Systemic Hemodynamics*. PLoS ONE 7(11): e49074

- DOI: [10.1371/journal.pone.0049074](https://doi.org/10.1371/journal.pone.0049074)
- Verification: crossref-verified | Access: open-access | Read: full text | evidence tier **C**
- Backs:
  - fifteen minutes of paced breathing at 6 per minute raised oxygen saturation and lowered systolic pressure in both studies at high altitude, and pulmonary artery pressure fell by the end of the session in Study B and after 5 minutes of recovery in Study A
    - "We investigated the ventilatory and hemodynamic effects of slow deep breathing in normal subjects at high altitude." (Abstract)
    - "Study variables, including Sp(O2) and systemic and pulmonary arterial pressure, were assessed before, during and after 15 minutes of breathing at 6 breaths/min. At the end of slow breathing, an increase in Sp(O2) (Study A: from 80.2±7.7% to 89.5±8.2%; Study B: from 81.0±4.2% to 88.6±4.5; both p<0.001) and significant reductions in systemic and pulmonary arterial pressure occurred." (Abstract)
    - "In both studies, each subject underwent: 1) baseline assessment after at least 15 minutes of rest at a spontaneous breathing rate, 2) 15 minutes of slow deep breathing exercise at a rate of 0.1 Hz (6 breaths per minute) paced by a digital metronome (RESPeRATE, Intercure Ltd, Lod, Israel), and 3) recovery (5 minutes – Study A, 30 minutes – Study B) at a spontaneous breathing rate." (Methods, Design and Setting)
    - "A significant reduction in SBP was found in both studies at the end of the slow breathing session, with SBP values which remained lower than at baseline after 5 minutes (Study A) but not after 30 minutes (Study B) of recovery." (Results, Changes in Hemodynamic Variables)
    - "Pulmonary artery pressure was significantly reduced at the end of slow deep breathing exercise (Study B) and after 5 minutes of recovery (Study A)." (Results, Changes in Hemodynamic Variables)
  - a mechanism the authors propose and call the most likely, though they didn't measure it directly, is that slow deep breathing increased the alveolar volume in use and so lowered dead-space ventilation and the dead space to tidal volume ratio
    - "However, several other mechanisms may be hypothesized to participate in explaining our findings." (Discussion)
    - "We are likely to have increased the used alveolar volume with slow deep breathing and, consequently, we have reduced dead space minute ventilation and the dead space to tidal volume ratio to a percentage, as previously reported for heart failure patients during slow deep breathing exercise" (Discussion)
    - "A significant increase in PtO2 and a significant reduction in PtCO2 occurred during slow deep breathing (Study A, Table 1), together with a proportional increase in tidal volume and, consequently, with no change in minute ventilation (Study B)." (Results, Changes in Ventilatory Variables)
    - "Thirdly, due to logistic limitations, we were not able to directly assess alveolar ventilation and dead space ventilation, but such information can be extrapolated from the present study data and from a previous report in hypoxic patients with heart failure" (Discussion)
    - "This improvement was most likely due to a reduction of dead space ventilation and an increase in alveolar ventilation" (Discussion)
- Caveat: n = 39 (Study A, tested on the second and third full days at 4559 m) and 28 (Study B, tested on days 5 to 9 at 5400 m, after 12 to 16 days at altitude). The device paced the 6 breaths per minute at an inhale to exhale ratio of about 1:2, an exhale-weighted pattern. Conducted at high altitude in a hypoxic state, where a lower baseline saturation predicted a larger gain, so it doesn't transfer to a desk at sea level and must not be cited as if it did. It's carried for two narrow purposes: it adds another data point on the physiological effects of 6 breaths per minute outside the HRV literature, and its proposed dead-space mechanism is one reason slow breathing isn't the same as breathing less.
  - "We collected data in healthy lowlanders staying either at 4559 m for 2-3 days (Study A; N = 39) or at 5400 m for 12-16 days (Study B; N = 28)." (Abstract)
  - "Study tests were performed on the second and on the third full day of permanence at CRM. [...] Study tests were performed on 5th –9th day of permanence at 5400 m (i.e. after a total of 12–16 days of high altitude exposure)." (Methods, Design and Setting)
  - "Our main result is that in healthy subjects exposed to HA, i.e. to a low ambient-air PO2, the change in breathing pattern from a spontaneous rate to a paced frequency of 6 breaths per minute was associated with an improvement of ventilation efficiency, as shown by the significant increase in blood oxygen saturation." (Discussion)
  - "At sea level, all subjects were normotensive and their SpO2 was invariably >95%." (Results, Changes in Ventilatory Variables)
  - "In Study A and B, the only variable which significantly predicted the increase of SpO2 induced by slow deep breathing was the SpO2 baseline value itself. This relationship was inverse in nature (Study A: r = −0.38, p = 0.017, Study B: r = −0.48, p = 0.010), i.e. a lower baseline SpO2 predicted a larger increase in SpO2 with the slow breathing exercise." (Results, Predictors of Response to Slow Breathing Exercise)
  - "In particular, ventilatory variables assessed in Study B indicate that this increase is not due to changes in minute ventilation during slow deep breathing, and indeed, the reduced respiratory rate is compensated by a proportionally increased tidal volume, but total ventilation is the same." (Discussion)
  - "Breathing rate reduction is mainly obtained by prolonging the expiration phase so that the ratio between inspiration and expiration duration is about 1∶2." (Methods, Design and Setting)
- Quotes checked: 17 of 17 against the Europe PMC full text and the Europe PMC abstract, last on 2026-10-18

#### `laborde2021-ie-ratio-pauses`

Laborde, Sylvain; Iskra, Maša; Zammit, Nina; Borges, Uirassu; You, Min; Sevoz-Couche, Caroline; Dosseville, Fabrice. (2021). *Slow-Paced Breathing: Influence of Inhalation/Exhalation Ratio and of Respiratory Pauses on Cardiac Vagal Activity*. Sustainability 13(14): 7775

- DOI: [10.3390/su13147775](https://doi.org/10.3390/su13147775)
- Verification: crossref-verified | Access: open-access | Read: full text | evidence tier **C**
- Backs:
  - at six cycles per minute, RMSSD was higher when the exhalation was longer than the inhalation, across inhalation/exhalation ratios of 0.8, 1.0 and 1.2
    - "Each condition lasted 5 min, with 30 respiratory cycles, and each respiratory cycle lasted 10 s (six cycles per minute), with inhalation/exhalation ratios of 0.8, 1.0, 1.2; and with or without respiratory pauses (0.4 s) between respiratory phases." (Abstract)
    - "Results indicated that the root mean square of successive differences (RMSSD), a marker of cardiac vagal activity, was higher when exhalation was longer than inhalation." (Abstract)
    - "The six conditions were the following: 4.6 s/0.4 s/4.6 s/0.4 s (inhalation/exhalation ratio = 1.0); 4.1 s/0.4 s/5.1 s/0.4 s (inhalation/exhalation ratio = 0.8); 5.1 s/0.4 s/4.1 s/0.4 s (inhalation/exhalation ratio = 1.2); 5 s/5 s (inhalation/exhalation ratio = 1); 4.5 s/5.5 s (inhalation/exhalation ratio = 0.8); 5.5 s/4.5 s (inhalation/exhalation ratio = 1.2)." (2.2.2. Slow-Paced Breathing)
    - "Regarding the inhalation/exhalation ratio, findings showed that CVA is higher when the exhalation phase lasts longer than the inhalation phase, confirming our hypothesis." (4. Discussion)
  - brief 0.4 s pauses after inhalation and after exhalation didn't further change RMSSD
    - "The presence of a brief (0.4 s) post-inhalation and post-exhalation respiratory pause did not further influence RMSSD." (Abstract)
    - "Regarding the presence of a respiratory pause, contrary to our hypothesis, there was no evidence for CVA to be influenced by the presence of a respiratory pause after the inhalation or exhalation phase." (4. Discussion)
- Caveat: n = 64 athletes, within-subjects, six 5-minute conditions in one session. 66 were recruited and 2 excluded for technical faults. Effect sizes from the full text: ratio main effect partial eta-squared 0.12. The longer exhale beat the shorter exhale at d = 0.51 and the equal ratio at d = 0.37. Equal against shorter exhale was d = 0.14, not significant. The pause main effect was F(1, 63) = 2.495, p = 0.119. Only the breathing rate was measured, at 6.48 to 6.55 cycles a minute against the paced 6, so whether participants produced the 0.4 s pauses or the exact ratios is unknown. Every condition raised RMSSD over baseline at d = 1.05 to 1.36. The authors note that their ratio range, 0.8 to 1.2, is narrower than other studies'. This is one of six studies bearing on the ratio disagreement tabulated in gap 4, on the side of the longer exhale, and the only one of the six that crosses respiratory pauses with the ratio itself, which is why it bears on gap 11: its 0.4 s pauses didn't change RMSSD, and the authors say 0.4 s may have been too short to have an effect. Published in Sustainability, an MDPI journal outside the field, which matters when weighing it against the other five.
  - "A total of 64 athletes (27 female; Mage = 22, age range = 18–30 years old) participated in a within-subject experimental design. Participants performed six breathing conditions within one session, with a 5 min washout period between each condition." (Abstract)
  - "The video displayed a 5 min SPB exercise, with six conditions, varying in the inhalation/exhalation ratio (inhalation = exhalation; inhalation > exhalation; inhalation < exhalation) and the presence/absence of a respiratory pause (0.4 s) after both inhalation and exhalation phases (see Figure 1)." (2.2.2. Slow-Paced Breathing)
  - "Following recommendations for HRV research [10,69], we recruited 66 athletes to take part in this research project. [...] Due to technical issues, the data of 2 participants had to be excluded, and the final sample comprised 64 athletes (27 female; MAge = 22, age range = 18–30 years old; BMI: M = 23.10, SD = 2.16; waist-to-hips ratio: M = 0.80, SD = 0.08; number of sport hours per week: M = 7.5 h; SD = 3.2)." (2.1. Participants)
  - "The first manipulation check revealed that participants followed the 6 cpm breathing frequency, ranging from 6.48 (SD = 0.20) to 6.55 (SD = 0.26). A repeated-measures ANOVA with the Greenhouse–Geisser correction was conducted, and showed no significant effect of condition on breathing frequency, with F (4.401, 277.232) = 0.696, p = 0.608, and partial η2 = 0.01." (3. Results)
  - "A repeated-measures ANOVA with the Greenhouse–Geisser correction was conducted and showed a significant main effect of inhalation/exhalation ratio, with F (1.996, 125.753) = 8.778, p < 0.001, and partial η2 = 0.12; no main effect of respiratory pause, with F (1, 63) = 2.495, p = 0.119, and partial η2 = 0.04; and no interaction effect between inhalation/exhalation ratio and respiratory pause, with F (1.676, 105.567) = 0.141, p = 0.832, and partial η2 = 0." (3. Results)
  - "Log RMSSD was found to be significantly higher in the condition with inhalation < exhalation in comparison to inhalation > exhalation, with t (63) = 4.059, Cohen’s d = 0.51, and p < 0.001; and in comparison to inhalation = exhalation, with t (63) = 2.928, Cohen’s d = 0.37, and p = 0.012." (3. Results)
  - "No differences were found between the condition inhalation > exhalation and inhalation = exhalation, with t (63) = 1.155, Cohen’s d = 0.14, and p = 0.758." (3. Results)
  - "The first analysis showed that log RMSSD was significantly higher in all the breathing conditions than in the baseline, for inhalation > exhalation (respiratory pause) with t (63) = 8.693, p < 0.001, Cohen’s d = 1.09; for inhalation < exhalation (respiratory pause) with t (63) = 10.853, p < 0.001, Cohen’s d = 1.36; for inhalation = exhalation (respiratory pause) with t (63) = 9.925, p < 0.001, Cohen’s d = 1.24; for inhalation > exhalation (no respiratory pause) with t (63) = 8.393, p < 0.001, Cohen’s d = 1.05; for inhalation > exhalation (no respiratory pause) with t (63) = 9.428, p < 0.001, Cohen’s d = 1.05; and for inhalation > exhalation (no respiratory pause) with t (63) = 8.944, p < 0.001, Cohen’s d = 1.12." (3. Results)
  - "Third, our equipment did not allow us to control precisely the exact duration of the inhalation, exhalation, and respiratory pauses, so it is not possible for us to evaluate how accurately the participants performed the breathing techniques. Still, we were able to control for respiratory frequency, to assure that our participants followed the 6 cpm rhythm, by using the respiration algorithm of Kubios [72]." (4. Discussion)
  - "However, we have to note that the inhalation/exhalation ratio range was reduced in this study (0.8–1.2) in comparison to other studies, such as 0.5–1 [58]; 0.4–2.3 [64], or 1–3.4 [65]." (4. Discussion)
  - "© 2021 by the authors. Licensee MDPI, Basel, Switzerland." (Copyright notice)
  - "Given that no changes were observed in CVA, it may be that the respiratory pause we chose in this design (0.4 s) may have been too short to elicit these effects." (4. Discussion)
- Quotes checked: 18 of 18 against the publisher's page and the Crossref abstract, last on 2026-10-18

#### `laborde2021-spb-6cpm-biofeedback`

Laborde, Sylvain; Allen, Mark S.; Borges, Uirassu; Iskra, Maša; Zammit, Nina; You, Min; Hosang, Thomas; Mosley, Emma; Dosseville, Fabrice. (2021). *Psychophysiological effects of slow-paced breathing at six cycles per minute with or without heart rate variability biofeedback*. Psychophysiology 59(1): e13952

- DOI: [10.1111/psyp.13952](https://doi.org/10.1111/psyp.13952)
- Verification: crossref-verified | Access: open-access | Read: full text | evidence tier **C**
- Backs:
  - slow-paced breathing at six cycles per minute raised RMSSD from the pre-session baseline, with and without biofeedback
    - "Consequently, the current study will use SPB at 6 cpm for all participants." (1 INTRODUCTION)
    - "Log RMSSD was found to be significantly higher DURING compared with PRE, t(111) = 20.39, p < .001, d = 1.97, and POST, t(111) = 20.34, p < .001, d = 1.92." (3 RESULTS)
    - "Regarding the first hypothesis, results showed a similar increase in CVA from PRE to DURING, and a decrease in CVA from DURING to POST, in both SPB-HRVB and SPB-NoHRVB conditions." (4 DISCUSSION)
    - "CVA was indexed via RMSSD, calculated from HRV." (2.2.1 Cardiac vagal activity indexed via heart rate variability)
  - adding heart-rate-variability biofeedback on top of the six-per-minute pace didn't add a further RMSSD benefit, though that condition reported more positive emotional valence
    - "A main effect of condition was found for emotional valence only, valence being more positive overall in the SPB-HRVB condition." (Abstract)
    - "Regarding log RMSSD, a significant main effect of time was found, F(1.13, 125.08) = 398.18, p < .001, partial η2 = 0.78; no main effect of condition, F(1, 111) = 1.47, p = .228, partial η2 = 0.01; and no interaction effect between time and condition, F(1.89, 210.09) = 0.07, p = .937, partial η2 = 0.00." (3 RESULTS)
    - "Our first hypothesis regarding CVA was supported with no differences found between SPB-HRVB and SPB-NoHRVB during and after SPB." (4 DISCUSSION)
- Caveat: n = 112, single session, within-subjects, no no-breathing control: the comparison is PRE against DURING and POST. The pacer was a ball moving up and down at 6 cpm (EZ-Air), a different stimulus from the expanding shape exhale uses. Valence was lower during breathing than before it in both conditions (d = 0.37), which is the abstract's 'more negative emotional valence'. The condition main effect on valence was partial eta-squared 0.06. Crossref records this as issued 2021, appearing in the January 2022 issue (59:1). Both conditions raised RMSSD and lowered arousal. The biofeedback condition also reported more positive emotional valence, so the case that the sensor adds nothing rests on the cardiac measure alone. Together with tabor2022-guided-breathing-design this is why exhale ships without a sensor: for the physiological effect of a single session, the pace does the work. RMSSD in the five-minute recovery recording that followed the session didn't differ significantly from baseline (PRE against POST d = 0.22), and neither did arousal (d = 0.15). The paper prints p = .20 for that RMSSD contrast, but its t(111) = 2.362 gives about .02, just above the corrected threshold of .017. The authors call it a switch-on, switch-off effect: CVA returned to baseline immediately after SPB stopped. The authors list among their limitations that the design couldn't fully address a placebo effect, since participants knew they were receiving an intervention in both conditions, and that participants were untrained in slow paced breathing.
  - "In total, 112 participants took part in the study." (Abstract)
  - "The participants attended the lab once, in accordance with the within-subject design." (2.3 Procedure)
  - "Our design did not include a control group not performing a breathing task, which limits the interpretation of the results linked to the psychophysiological variables assessed." (4 DISCUSSION)
  - "A 5-min video showing a ball moving up and down at the rate of 6 cpm, based on the EZ-Air software (Thought Technology Ltd., Montreal, Canada), served as a breathing pacer." (2.2.2 Slow-paced breathing with and without biofeedback)
  - "Results showed that for PRE and POST comparisons (referring, respectively, to before and after SPB), both SPB-HRVB and SPB-NoHRVB conditions resulted in a more negative emotional valence, lower emotional arousal, higher emotional control, and higher RMSSD." (Abstract)
  - "Emotional valence was found to be significantly lower DURING in comparison to PRE, with t(111) = 3.90, p < .001, d = 0.37, and in comparison to POST, with t(111) = 3.20, p = .005, d = 0.30." (3 RESULTS)
  - "Regarding emotional valence, a significant main effect of time was found, F(1.94, 215.18) = 7.77, p < .001, partial η2 = 0.07; a significant main effect of condition, F(1, 111) = 6.433, p = .013, partial η2 = 0.06; and no interaction effect between time and condition, F(1.87, 207.76) = 0.84, p = .425, partial η2 = 0.01." (3 RESULTS)
  - "Log RMSSD was found to be significantly higher DURING compared with PRE [...] No significant difference was found between PRE and POST, t(111) = 2.362, p = .20, d = 0.22." (3 RESULTS)
  - "Emotional arousal was found to be significantly lower DURING compared with PRE, t(111) = 8.07, p < .001, d = 0.77, and POST, t(111) = 6.05, p < .001, d = 0.57. No significant difference was found between PRE and POST, t(111) = 1.53, p = .382, d = 0.15." (3 RESULTS)
  - "The increase of CVA observed with SPB and its subsequent decrease after completing the SPB task is in line with previous research (e.g., Hoffmann et al., 2019; Laborde, Iskra, et al., 2021; You, Laborde, Salvotti, et al., 2021), illustrating that the short-term effects of SPB on CVA are similar to the action of a “switch-on/switch-off” power switch, with CVA returning to baseline immediately after SPB stops." (4 DISCUSSION)
  - "Our finding indicates that the physiological effects of SPB—which are linked to the activation of the vagus nerve (Gerritsen & Band, 2018; Zaccaro et al., 2018) potentially via the stimulation of the baroreflex (Lehrer & Gevirtz, 2014; Shaffer & Meehan, 2020), the action on pulmonary afferents (Noble & Hochman, 2019), and the creation of brain oscillations (Mather & Thayer, 2018)—are not influenced by the presentation of the heart rate signal as biofeedback." (4 DISCUSSION)
  - "Our experimental design could not fully address the placebo effect, given it is obvious that participants are receiving an intervention in both SPB-HRVB and SPB-NoHRVB conditions." (4 DISCUSSION)
  - "Our participants were untrained to SPB." (4 DISCUSSION)
  - "Regarding the main effect of time, post-hoc t tests were conducted applying a Bonferroni correction with alpha adjusted to p = .017 (0.05/3)." (3 RESULTS)
- Quotes checked: 21 of 21 against the HAL repository copy and the Europe PMC abstract and the publisher's page, last on 2026-10-18

#### `lehrer2022-my-life-hrvb`

Lehrer, Paul. (2022). *My Life in HRV Biofeedback Research*. Applied Psychophysiology and Biofeedback 47(4): 289-298

- DOI: [10.1007/s10484-022-09535-5](https://doi.org/10.1007/s10484-022-09535-5)
- Open copy: <https://pmc.ncbi.nlm.nih.gov/articles/PMC8899772/> ([archived](https://web.archive.org/web/20260423032456/https://pmc.ncbi.nlm.nih.gov/articles/PMC8899772/))
- Verification: crossref-verified | Access: paywalled | Read: full text | evidence tier **D**
- Backs:
  - the resonance-frequency figure of about 6 breaths per minute came out of Vaschillo's 2002 biofeedback work, where each person's frequency fell between 4.5 and 6.5 cycles per minute, and Song and Lehrer's 2003 pacing study, which paced people from 3 to 14 breaths per minute and found RSA amplitude highest at 4 and 6 and lower at 3
    - "He found that the high-amplitude heart rate oscillations could only be replicated at one frequency for each person, and that this frequency varied between 4.5 and 6.5 cycles/min." (The dynamics of heart rate variability biofeedback)
    - "As Vaschillo and I discussed this work and its implications for the biofeedback discipline, we decided write a paper describing it in for publication in Applied Psychophysiology & Biofeedback. We had difficulty publishing it in several journals because reviewers could not believe that heart rate swings of close to 60 beats/min could reliably be produced in normal people, and even the biofeedback journal initially as reluctant to publish it because reviewers thought that the material was too abstruse; but it eventually it was published in this journal (Vaschillo et al., 2002), and has been widely cited since then." (The dynamics of heart rate variability biofeedback)
    - "In the meantime we had been doing a study of paced breathing effects on heart rate variability, with breathing paced at various rates from 3 per min to 14, We found the highest amplitude of RSA at four and six breaths/min, with a decrease at three per min (Song & Lehrer, 2003). This finding replicated Vaschillo et al.’s finding of resonance in the neighborhood of six per min" (The dynamics of heart rate variability biofeedback)
  - induced HRV amplitude is at a minimum around 2 to 3 cycles per minute, which the author attributes to negative resonance
    - "also replicated a finding from Vaschillo et al. (2002) that induced HRV amplitudes were at a minimum in the range of two to three cycles/min because of negative resonance at that frequency" (The dynamics of heart rate variability biofeedback)
  - each person's resonance frequency is related to height and gender, with slower frequencies for taller people and men, unrelated to age or weight, and appeared unchangeable even after several months of biofeedback practice, with an average of 5.5 breaths per minute
    - "In one of our later studies, at Rutgers University, we found that each person’s frequency of resonance was related to height and to gender, with slower frequencies for taller people and men than for shorter people and women (Vaschillo et al., 2006)—a relationship probably caused by blood volume." (The dynamics of heart rate variability biofeedback)
    - "There was no relationship between asthma condition and resonance frequency, and none between age or weight and resonance frequency, the latter probably because adipose tissue contains little blood flow." (The dynamics of heart rate variability biofeedback)
    - "We further found that resonance frequency appeared unchangeable, even after several months of biofeedback practice, where large increases were found in baroreflex gain and heart rate variability. We found that the average resonance frequency was 5.5 cycle times/min." (The dynamics of heart rate variability biofeedback)
  - paced breathing at a rate above resonance frequency, used as a control condition in several clinical studies, still seems to produce clinically significant effects, which the author lists as an open question, alongside some evidence that breathing slightly away from exact resonance frequency has a slightly smaller clinical effect on blood pressure and anxiety
    - "Studies of several conditions using a paced breathing control at a frequency higher than resonance frequency still seems to produce clinically significant effects." (Recommendations for Future Development of Our Work, Clinical Effects of HRVB)
    - "Is there something about paced breathing at a naturally relaxed rate that produces important psychophysiological effects, and what might the mechanisms be?" (Recommendations for Future Development of Our Work, Clinical Effects of HRVB)
    - "There is some evidence that breathing slightly away from exact resonance frequency has a slightly smaller clinical effect on blood pressure control (Lin et al., 2012) and anxiety (Steffen et al., 2017)." (Recommendations for Future Development of Our Work, Clinical Effects of HRVB)
- Caveat: A first-person retrospective by one of the technique's principal developers, so tier D and read only as history: the findings carried here are the author's summary of his own group's published papers, cited without re-analysis, and the primary sources (Vaschillo et al. 2002, Song and Lehrer 2003, Vaschillo et al. 2006) aren't in this corpus. Lehrer is also an author of lehrer2014-hrv-biofeedback, so the 5.5 figure and the individual-variation point come from one group twice. Carried for two reasons. It's the clearest account of where the 6-a-minute figure came from, and its report that RSA amplitude was highest at 4 and 6 a minute and fell at 3 is why gaps 2 and 6 say that no primary study in this corpus has measured 4 a minute or reported worse outcomes below 5. Those are HRV-amplitude findings, the same kind of peak shaffer2020-resonance-frequency-assessment describes, and they say nothing about relaxation or comfort at slower rates. Read from the published text Springer Nature deposited in PubMed Central under its COVID-19 terms, linked here as the open copy. The article itself is paywalled at Springer.
  - "This paper reviews the published work of me along with my students and close colleagues on the topic of heart rate variability biofeedback (HRVB)." (Abstract)
  - "Our protocol of HRVB is often used as a standard for HRVB research." (Impact on the Field of HRVB Research)
- Quotes checked: 12 of 12 against the Europe PMC full text and the Europe PMC abstract, last on 2026-10-18

#### `lin2014-equal-ratio-hrv`

Lin, I. M.; Tai, L. Y.; Fan, S. Y. (2014). *Breathing at a rate of 5.5 breaths per minute with equal inhalation-to-exhalation ratio increases heart rate variability*. International Journal of Psychophysiology 91(3): 206-211

- DOI: [10.1016/j.ijpsycho.2013.12.006](https://doi.org/10.1016/j.ijpsycho.2013.12.006)
- Verification: crossref-verified | Access: paywalled | Read: abstract only | evidence tier **C**
- Backs:
  - 5.5 breaths per minute with an equal 5:5 inhale-to-exhale ratio produced higher SDNN and LF power than 6 bpm or than a 4:6 ratio
    - "Participants were instructed to breathe at two different breathing rates (6 and 5.5 breaths) and two different I:E ratios (5:5 and 4:6)." (Abstract)
    - "The results revealed that a pattern of 5.5 bpm with an I:E ratio of 5:5 produced a higher NN interval standard deviation and higher low frequency power than the other breathing patterns." (Abstract)
    - "The study confirmed that a breathing pattern of 5.5 bpm with an I:E ratio of 5:5 achieved greater HRV than the other breathing patterns." (Abstract)
  - all four slow-breathing patterns increased self-reported relaxation relative to spontaneous breathing
    - "Moreover, the four different breathing patterns were associated with significantly increased feeling of relaxation compared with baseline." (Abstract)
    - "The HRV indices as well as anxiety and relaxation levels were measured at baseline (spontaneous breathing) and for the four different breathing patterns." (Abstract)
- Caveat: n = 47, Latin-square counterbalanced. The one study in this corpus that favors the equal ratio on HRV grounds, and one reason exhale makes no mechanism claim for a longer exhale. The longer exhale it tested was 4:6, so it bears on exhale's 1:2 ratio only by extrapolation. meehan2024-longer-exhalations rates it at high risk of bias because achieved rates and ratios went unreported and its SDNN comparison mixed rate with ratio. It also measured relaxation and anxiety across the four patterns and reports that all four increased relaxation over baseline. The abstract doesn't say whether the patterns differed from each other on relaxation, so that's the extent of the subjective finding this corpus can carry from it, one of three bearing on gap 4. As the second claim says, every pattern tested increased relaxation, so the ratio argument is about which slow pattern is best, a separate question from whether slow breathing works.
  - "Forty-seven healthy college students were recruited for the study, and a Latin square experimental design with a counterbalance in random sequences was applied." (Abstract)
  - "This study aimed to examine the differences in HRV indices and subjective feelings of anxiety and relaxation among four different breathing patterns." (Abstract)
- Quotes checked: 7 of 7 against the Europe PMC abstract, last on 2026-10-18

#### `marchant2025-square-478-six`

Marchant, Joshua; Khazan, Inna; Cressman, Mikel; Steffen, Patrick. (2025). *Comparing the Effects of Square, 4-7-8, and 6 Breaths-per-Minute Breathing Conditions on Heart Rate Variability, CO2 Levels, and Mood*. Applied Psychophysiology and Biofeedback 50(2): 261-276

- DOI: [10.1007/s10484-025-09688-z](https://doi.org/10.1007/s10484-025-09688-z)
- Verification: crossref-verified | Access: paywalled | Read: full text | evidence tier **B**
- Backs:
  - breathing at 6 breaths per minute raised LF-HRV more than either square (box) breathing or 4-7-8 breathing at both ratios tested, with small to medium effects, and on RMSSD only the 4:6 ratio's advantage over the hold patterns was significant
    - "Breathing at 6 bpm increased HRV measures more than square or 4–7-8 breathing with small to medium effects." (Abstract)
    - "However, LF-HRV was higher for participants breathing at 6 bpm than for square or 4–7-8 breathing with small to medium effects, indicating better effectiveness." (Discussion, p. 272)
    - "Post-hoc Scheffé comparisons indicated 5:5 breathing was significantly more effective than square breathing (M diff = 0.25 [0.04, 0.46], p < 0.01) and 4–7-8 breathing (M diff = 0.23 [0.02, 0.44], p = 0.02). Furthermore, the 4:6 breathing condition was significantly more effective than square breathing (M diff = 0.29 [0.08, 0.50], p < 0.01) and 4–7-8 breathing (M diff = 0.27 [0.06, 0.48], p < 0.01)" (Results, Hypothesis 1, p. 266)
    - "4:6 breathing was significantly better than square breathing (M-difference = 0.40 [0.18, 0.61], p < 0.001) and [...] breathing (M-diff = 0.38 [0.17, 0.60], p < 0.001)." (Results, Hypothesis 1, p. 266)
    - "5:5 breathing increased RMSSD more than square breathing (M-diff = 0.19 [0.02, 0.41], p = 0.1) and 4–7-8 breathing (M-diff = 0.18 [0.03, 0.39], p = 0.15), but these differences were not statistically significant after the log transformation." (Results, Hypothesis 1, p. 266)
    - "Pairwise comparisons for RMSSD with Scheffé corrections [...] 4:6 vs. 5:5 Square vs. 5:5 4–7-8 vs. 5:5 Square vs. 4:6 4–7-8 vs. 4:6 [...] 0.20 − 0.19 − 0.18 − 0.40 − 0.38 [...] 0.07 0.10 0.15 < 0.001 < 0.001" (Table 4, RMSSD pairwise comparisons, p. 267)
    - "One breathing strategy with substantial anecdotal support but limited empirical evidence is square breathing (also referred to as ‘box’ or ‘tactical’ breathing)." (Introduction, p. 262)
  - square and 4-7-8 breathing are popularly promoted but have little empirical support
    - "Square and 4–7-8 breathing are popularly promoted by psychotherapists but have little empirical support." (Abstract)
  - blood pressure didn't change significantly in any condition after Scheffe correction, neither 6 bpm ratio changed mood, and the two hold-based patterns showed small decreases in positive mood (0.19 and 0.24 SD) that didn't survive the same correction, after which the only significant mood difference was 4:6 against 4-7-8
    - "We saw no significant decreases in blood pressure in any of the breathing conditions" (Discussion, p. 273)
    - "None of the breathing conditions resulted in meaningful changes in blood pressure or mood." (Abstract)
    - "5:5 and square breathing lowered DBP with small effects in the initial model (respectively β = − 0.27 [− 0.46, − 0.08], p = 0.01; β = − 0.22 [− 0.41, -0.04], p = 0.02). However, these results were not statistically significant after Scheffé corrections." (Results, Hypothesis 2, p. 266)
    - "Square breathing lowered DBP with a small effect in the initial model (β = − 0.22 [− 0.40, − 0.04], p = 0.02), but post-hoc Scheffe comparisons indicated none of the breathing conditions significantly predicted SBP." (Results, Hypothesis 2, p. 271)
    - "5:5 and 4:6 breathing did not alter mood (respectively β = − 0.05 [− 0.22, − 0.12], p = 0.57; β = 0.03 [− 0.13, 0.20], p = 0.69)." (Results, Hypothesis 5, p. 272)
    - "Square breathing and 4–7-8 breathing decreased positive mood with small effects (respectively β = − 0.19 [− 0.36, − 0.02], p = 0.03; β = − 0.24 [− 0.41, − 0.08], p < 0.001). However, after Scheffe corrections, the only statistically significant difference was between 4:6 and 4–7-8 breathing (standardized M difference = − 0.28 [− 0.54, − 0.02], p = 0.03)." (Results, Hypothesis 5, p. 272)
    - "Results presented in the text were standardized for interpretability." (Results, p. 266)
  - breathing at 6 breaths per minute unexpectedly produced mild over-breathing
    - "Unexpectedly, breathing at 6 bpm led to mild over-breathing." (Abstract)
    - "Contrary to our hypotheses, both 5:5 and 4:6 breathing resulted in lower PETCO2 levels, on average placing participants in the mild over-breathing range (30–35 mmHg; see Fig. 4)." (Results, Hypothesis 3, p. 271)
    - "While most participants did not drop below more than a mild level of over-breathing (30–35 mmHg), this finding is still notable." (Discussion, p. 273)
- Caveat: n = 84 college students, within-subjects, four conditions: square, 4-7-8, 6 bpm at 4:6, and 6 bpm at 5:5. HRV analyses lost 5 participants and end-tidal CO2 analyses lost 6 to missing data. Pre-registered on the Open Science Framework. The sample was Brigham Young University undergraduates, 86% white with a mean age of about 20, which the authors call homogeneous in age and race. Single session, so it speaks to acute effect only, and it's the largest ratio comparison in this corpus. This is the head-to-head that ranks exhale's presets, and it ranks against box breathing and 4-7-8. RMSSD gains over baseline in standard-deviation units: 5:5 0.45, 4:6 0.65, square 0.25, 4-7-8 0.27, all significant. Scheffe-corrected pairwise: 4:6 beat square and 4-7-8 (p < 0.001). 5:5's edge over them, 0.19 and 0.18, didn't reach significance (p = 0.10 and 0.15). 4:6 against 5:5 was 0.20, p = 0.07. On LF-HRV both 6 bpm conditions beat both hold patterns. So the comparison supports 6 bpm as a rate. Within it the HRV point estimates favor 4:6, and only 4:6 separated from the hold patterns on RMSSD. The authors' own conclusion is that no outcome differed significantly by ratio, that the study wasn't powered below 0.17, and that if a longer exhale has an effect it's small. Adherence within a second of the target pace was 92.9 percent for 5:5, 86.9 percent for 4:6 and 73.8 percent for both hold patterns, and participants generally breathed faster than paced. End-tidal CO2 fell by 0.56 SD (5:5) and 0.65 SD (4:6), which on average put participants in the 30 to 35 mmHg range the authors call mild over-breathing. They also note the capnometer's baseline zeroing may have been off. Two of its findings cut against arguments made elsewhere in this corpus: mood didn't change at either 6 bpm ratio, which is the largest subjective null in gap 4, and 6 bpm produced mild over-breathing, which is gap 5.
  - "Eighty-four college students (60% female) had HRV measured at baseline and during 1) square breathing, 2) 4–7-8 breathing, 3) 6 bpm with a 4:6 inhale-to-exhale ratio, and 4) 6 bpm with a 5:5 ratio." (Abstract)
  - "Using a within-subjects design, we hypothesized that:" (Hypotheses, p. 263)
  - "We enrolled 84 undergraduate participants to account for the likelihood of missing physiological data and to increase power. Participants were enrolled via the SONA recruitment portal at Brigham Young University." (Methods, p. 263)
  - "Participants had an average age of 20 (range 17 to 25) and were 60% female and 86% white." (Methods, p. 263)
  - "Overall, we were missing 6 participants (7%) for PETCO2 analyses and 5 participants (6%) for HRV analyses." (Methods, p. 265)
  - "We pre-registered all hypotheses and procedures on the Open Science Foundation (OSF; [...] )." (Methods, p. 263)
  - "Firstly, we only measured the effects of each breathing condition throughout one session." (Limitations, p. 274)
  - "Our sample was also homogenous in terms of age and race" (Limitations, p. 274)
  - "5:5 breathing: Participants were instructed to breathe at a rate of 5 s per inhale and 5 s per exhale following a breathing pacer. 4:6 breathing: Participants were instructed to breathe at a rate of 4 s per inhale and 6 s per exhale following a breathing pacer." (Methods, Procedures, p. 264)
  - "Square breathing: Participants are instructed to breathe at a rate of 4 s per inhale, hold for 4 s, 4 s per exhale, and hold for 4 s following a breathing pacer." (Methods, Procedures, p. 264)
  - "4–7-8 breathing: Participants were instructed to breathe at a rate of 4 s per inhale, hold for 7 s, and exhale for 8 s following a breathing pacer." (Methods, Procedures, p. 264)
  - "Target breathing rate (BPM) [...] 5:5 4:6 Square 4–7-8 [...] 6.00 6.00 3.75 3.16" (Table 3, p. 266)
  - "(± 1 s) [...] 5:5 4:6 Square 4–7-8 [...] 92.9% 86.9% 73.8% 73.8%" (Table 3, p. 266)
  - "About 90% of participants breathed within ± 1 s of 6 breaths per minute, but this dropped to about 74% for square breathing for [...] breathing." (Results, p. 265)
  - "In general, participants breathed more quickly than the indicated pace." (Results, p. 266)
  - "We used a Scheffe test to account for multiple comparisons and run post-hoc pairwise comparisons." (Methods, Analyses, p. 265)
  - "Post-hoc comparisons using the Scheffe test indicated that all breathing conditions led to significant increases in RMSSD compared to baseline (see Table 4 and Fig. 2)." (Results, Hypothesis 1, p. 266)
  - "RMSSD results are reported in terms of standard deviations for interpretability." (Results, Hypothesis 1, p. 266)
  - "Pairwise comparisons for RMSSD with Scheffé corrections [...] 5:5 vs. baseline 4:6 vs. baseline Square vs. baseline 4–7-8 vs. baseline [...] 0.45 0.65 0.25 0.27 [...] < 0.001 < 0.001 0.01 0.01" (Table 4, RMSSD pairwise comparisons, p. 267)
  - "There was a small but significantly insignificant advantage in RMSSD for 4:6 compared to 5:5 breathing (standardized M diff = 0.20 [− 0.01, 0.42], p = 0.07)." (Results, Hypothesis 4, p. 272)
  - "There was no difference in LF-HRV between 5:5 and 4:6 breathing (standardized M diff = 0.04 [− 0.17, 0.25], p = 0.99)." (Results, Hypothesis 4, p. 272)
  - "Pairwise comparisons did not indicate any significant differences in PETCO2 between 5:5 and 4:6 breathing (standardized M diff = − 0.08 [− 0.36, 0.19], p = 0.93), supporting our hypothesis." (Results, Hypothesis 4, p. 272)
  - "Regarding breathing ratios, we found no significant differences in any outcome for the ratio of inhale to exhale. Whether participants breathed in for 4 s and out for 6, or if they breathed in for 5 s and out for 5, there were no statistically significant differences in HRV, blood pressure, PETCO2, or mood." (Discussion, p. 272)
  - "Importantly, our study was not [...] powered to detect effects smaller than 0.17, and there were small but statistically insignificant improvements in RMSSD while doing 4:6 breathing compared with 5:5 breathing. Thus, if there is an impact of longer exhalations, it is quite small." (Discussion, pp. 272-273)
  - "Our results suggest that there is no meaningful difference in the exhale ratio and that the choice depends on individual preferences." (Discussion, p. 273)
  - "Specifically, 5:5 breathing resulted in standardized M diff = − 0.56 [− 0.83, − 0.29], p < 0.001, and 4:6 breathing resulted in standardized M diff = − 0.65 [− 0.92, − 0.37], p < 0.001." (Results, Hypothesis 3, p. 271)
  - "This could be due to true baseline differences in [...] but was most likely due to inaccurate zeroing of the capnometer at baseline." (Results, Hypothesis 3, p. 272)
- Quotes checked: 45 of 45 against a copy of the published PDF and the Europe PMC abstract, last on 2026-10-18

#### `meehan2024-longer-exhalations`

Meehan, Zachary M.; Shaffer, Fred. (2024). *Do Longer Exhalations Increase HRV During Slow-Paced Breathing?* Applied Psychophysiology and Biofeedback 49(3): 407-417

- DOI: [10.1007/s10484-024-09637-2](https://doi.org/10.1007/s10484-024-09637-2)
- Verification: crossref-verified | Access: open-access | Read: full text | evidence tier **B**
- Backs:
  - at 6 breaths per minute, a 1:2 inhale-to-exhale ratio produced no HRV advantage over 1:1 in either an original experiment or its replication
    - "We conducted original (N = 26) and replication (N = 16) studies to determine whether a 1:2 IE ratio produces different HRV time-domain, frequency-domain, or nonlinear metrics than a 1:1 ratio when breathing at 6 bpm. Our original study found that IE ratio did not affect HRV time- and frequency-domain metrics. The replication study confirmed these results and found no effect on HRV nonlinear measurements." (Abstract)
    - "Thus, these findings suggest that breathing at a 1:2 IE ratio may not alter valued HRV metrics when breathing in the RF range." (Discussion)
  - the finding held across time-domain metrics, LF power and sample entropy, and the authors left out HF power as invalid at 6 breaths per minute
    - "Across an original experiment and a replication, participants experienced no difference in HRV time-domain, frequency-domain, or nonlinear metrics when breathing at a 1:1 or 1:2 ratio within the RF range." (Discussion)
    - "We measured HR, RR, the SDNN, RMSSD, and pNN50, and LF power." (Method, Study One, Dependent Variables)
    - "We measured RR, HR, the SDNN, RMSSD, and pNN50, LF power, and sample entropy (SampEn). We did not measure HF power since values obtained at 6 bpm would be invalid." (Method, Study Two, Dependent Variables)
- Caveat: Original n = 26, replication n = 16. The authors' power analysis assumed a large effect, Cohen's d of about 1, and put the sample needed at 10 participants. Both were undergraduate samples, both within-subjects. The manipulation checks compared respiration rates between the two ratios and confirmed the ratio by watching the respirometer waveform, and the authors list the lack of a software measure of ratio adherence as a limitation. Participants in both studies had one to two semesters of effortless breathing training, and the limitations add that they trained for 5 to 6 weeks at a 1:1 ratio before being tested at 1:1 and 1:2. Small, but it's the only entry in this corpus that ran its own replication, and its literature review tallies the older studies beyond this corpus as three further nulls, one further result favoring the longer exhale and one favoring the longer inhale. One of those three nulls, Cappo and Holmes 1984, didn't measure HRV. Its scope condition matters: it holds rate fixed inside the resonance range, so it doesn't rule out a ratio effect at a person's spontaneous rate, which is what bae2021-exhalation-inhalation-ratio measured. One of six disagreeing studies. See also bae2021-exhalation-inhalation-ratio, vandiest2014-ie-ratio-relaxation, laborde2021-ie-ratio-pauses, lin2014-equal-ratio-hrv and marchant2025-square-478-six. Its review rates bae2021-exhalation-inhalation-ratio, laborde2021-ie-ratio-pauses, lin2014-equal-ratio-hrv and vandiest2014-ie-ratio-relaxation at high risk of bias. All six measured HRV. vandiest2014-ie-ratio-relaxation, lin2014-equal-ratio-hrv and marchant2025-square-478-six also measured how participants felt, and they tested different contrasts: a longer exhale against a longer inhale, every pattern against baseline, and 4:6 against 5:5.
  - "We are reporting on two randomized control trials (RCTs) that used identical equipment, procedures, and manipulation checks. We performed the second RCT to ensure we could replicate the first study’s provocative findings with a different undergraduate sample." (Method)
  - "Twenty-six undergraduates (10 women and 16 men), 18 to 22, participated in this study. They had one to two semesters of effortless breathing training within the adult resonance frequency range (4.5–6.5 bpm; Lehrer & Gevirtz, 2014)." (Method, Study One, Participants)
  - "Sixteen undergraduates (8 women and 8 men), 18 to 22, participated in this study. They also completed one to two semesters of 6-bpm effortless breathing training." (Method, Study Two, Participants)
  - "We randomly assigned participants to one of two IE ratio orders, 1:1 ➔ 1:2 or 1:2 ➔ 1:1, to minimize order effects." (Method, Study One, Procedure)
  - "Researchers visually confirmed compliance with respiration rate and IE ratio instructions by monitoring the respirometer waveform in real-time. Additionally, significance tests showed no difference in respiration rates between 1:1 and 1:2 IE ratio conditions, t(25) = -0.21, p = .83." (Method, Study One, Manipulation Check)
  - "We estimated the necessary sample size for the mean difference between two dependent variables in a within-subjects design where alpha is 0.05, power is 0.80, and Cohen’s d is approximately 1 based on prior research investigating IE ratios (Van Diest et al., 2014). The results suggest that a sample size of 10 participants is necessary to detect large effect sizes at 80% power." (Results, Analytic Strategy)
  - "Finally, participants in both study one and study two were trained over the 5–6 weeks to breathe at a 1:1 IE ratio, then tested on the effects of breathing at a 1:1 or 1:2 IE ratio." (Limitations)
  - "Fourth, data acquisition software should measure the proportion of time participants breathing waveforms matched the pacing display, thereby confirming their adherence to the IE ratio." (Limitations)
  - "Three studies (Cappo & Holmes, 1984; Edmonds et al., 2009; Klintworth et al., 2012) found no IE ratio effect. One study reported an advantage for equal inhalations and exhalations (Lin et al., 2014), and another reported an advantage for longer inhalation periods (Paprika et al., 2014). Four studies observed an advantage for longer exhalations than inhalations (Bae et al., 2021; Laborde et al., 2021; Strauss-Blasche et al., 2000; Van Diest et al., 2014)." (Research Synthesis)
  - "Blood pressure and HR did not differ across the breathing ratio conditions. The authors did not measure HRV." (Literature Review, Cappo and Holmes)
  - "We conducted original and replication studies to determine whether a 1:2 IE ratio produces different HRV time-domain, frequency-domain, or nonlinear metrics than a 1:1 ratio when breathing at 6 bpm within the RF range." (Current Study)
  - "Bae and colleagues (2021) randomly assigned 28 participants to different orders of 1:1, 2:1, and 1:2 exhalation-to-inhalation (EI) ratios at spontaneous breathing rates." (Literature Review, Bae and Colleagues)
  - "The researchers reported that the actual EI ratios were 1.08 for 1:1 and 1.33 for 2:1, and their difference was significant. The RMSSD and HF power were greater in the 2:1 than 1:1 condition while breathing at 14.16 and 14.47 bpm, respectively." (Literature Review, Bae and Colleagues)
  - "In conclusion, this study yielded a high risk of bias because the authors reported incomplete randomization, participants who received 1:1 IE instructions received more practice, participants did not breathe at the instructed IE ratio, and the authors did not report baseline differences." (Literature Review, Bae and Colleagues)
  - "In conclusion, this study is at high risk of bias due to a lack of reporting on achieved RRs or IE ratios and confounding RR with IE ratio when analyzing SDNN." (Literature Review, Lin and Colleagues)
  - "In conclusion, this study yielded a high risk of bias because the participants did not breathe at the target rates and participants in the high-ratio condition did not follow the IE instructions." (Literature Review, Van Diest and Colleagues)
  - "In conclusion, this study yielded a high risk of bias because they did not measure baseline differences or report manipulation checks." (Literature Review, Laborde and Colleagues)
- Quotes checked: 22 of 22 against the Europe PMC full text and the Europe PMC abstract, last on 2026-10-18

#### `sevozcouche2022-coherence-resonance`

Sevoz-Couche, Caroline; Laborde, Sylvain. (2022). *Heart rate variability and slow-paced breathing: when coherence meets resonance*. Neuroscience & Biobehavioral Reviews 135: 104576

- DOI: [10.1016/j.neubiorev.2022.104576](https://doi.org/10.1016/j.neubiorev.2022.104576)
- Open copy: <https://hal.sorbonne-universite.fr/hal-03578368> ([archived](https://web.archive.org/web/20260218050014/https://hal.sorbonne-universite.fr/hal-03578368))
- Verification: crossref-verified | Access: paywalled | Read: full text | evidence tier **D**
- Backs:
  - temporal coherence of respiratory, blood-pressure and cardiac oscillations is reached at the baroreflex resonant frequency of about 0.1 Hz, where the delay between breathing and heart-rate oscillations disappears, especially in young participants
    - "Temporal coherence of respiratory, blood pressure, and cardiac oscillations is achieved only at the resonant frequency (~ 0.1 Hz)" (Highlights)
    - "Secondly, if there is a delay between respiratory and cardiac oscillations during spontaneous breathing (see p 8), this delay diminishes as the breathing rate decreases (Laude et al., 1993) and disappears when the breathing rate reaches approximately 0.1 Hz (Lehrer and Gevirtz, 2014)." (III- SPB & RSA, Effects of SPB on RSA and cardiac oscillation amplitude)
    - "Thus, the “baroreflex resonance” is generally composed of 0.1 Hz oscillations." (II-1- The low-frequency band (0.04-0.15 Hz))
    - "To note, this exact coherence at 0.1 Hz is especially obtained in young participants, suggesting that cardiovascular characteristics of older people may affect the phase relationship (Lehrer et al., 2020)." (III- SPB & RSA, Effects of SPB on RSA and cardiac oscillation amplitude)
  - the baroreflex loop's delay of roughly five seconds varies between about 4 and 6.5 seconds across people, putting the natural frequency between 0.075 and 0.12 Hz
    - "increase in blood pressure (due to an increase in the quantity of blood circulating during each cardiac pulse) is delayed by approximately five seconds" (II-1- The low-frequency band (0.04-0.15 Hz))
    - "It is important to note that the length of the delay (inertia) differs slightly across individuals and ranges between 4–6.5 seconds; i.e. the natural frequency of these oscillations is actually between 0.075–0.12 Hz (Vaschillo et al., 2006)." (II-1- The low-frequency band (0.04-0.15 Hz))
- Caveat: Narrative review, hence tier D. It defines its terms: 'resonance' is the roughly 0.1 Hz oscillation the baroreflex loop's delay produces, and 'coherence' the phase alignment of breathing, blood pressure and heart rate that breathing at that frequency produces. Carried as a terminology reference: 'cardiac coherence' also circulates as a product name in the market exhale sits in, and this is the paper to check before adopting that vocabulary in the app or the README. Its conclusion notes that some studies report no effects and asks for placebo comparisons such as pacing at a spontaneous rate. The delay figures, which the review takes from Vaschillo et al. 2006, tie resonance frequency to height the same way lehrer2014-hrv-biofeedback does, and the review notes that exact coherence at 0.1 Hz is especially obtained in young participants.
  - "This review will focus on the importance of reducing breathing rate at the resonant frequency (~ 0.1 Hz), which increases cardiac oscillations, thus reflecting improved vagally-mediated heart rate variability and baroreflex sensitivity." (Abstract)
  - "Keywords: deep breathing, thoracic breathing, slow-paced respiration, vagus nerve, parasympathetic nervous system, baroreflex, resonance, cardiac coherence" (Keywords)
  - "As a result, all cardiac oscillations are produced exactly in phase (coherence), reinforcing each other." (III- SPB & RSA, Effects of SPB on RSA and cardiac oscillation amplitude)
  - "These values seem dependent on height (lower in taller men) due to an effect on vascular inertia, but not age or weight (Vaschillo et al., 2006)." (II-1- The low-frequency band (0.04-0.15 Hz))
  - "It is still important to highlight that some studies also report no effects." (Conclusions)
  - "A more generalized use of placebo, including paced breathing at a spontaneous frequency or neutral interventions, would help to disentangle the effects of SPB from those solely linked to breathing awareness or distraction." (Conclusions)
- Quotes checked: 12 of 12 against the HAL repository copy and the Europe PMC abstract, last on 2026-10-18

#### `shaffer2020-resonance-frequency-assessment`

Shaffer, Fred; Meehan, Zachary M. (2020). *A Practical Guide to Resonance Frequency Assessment for Heart Rate Variability Biofeedback*. Frontiers in Neuroscience 14: 570400

- DOI: [10.3389/fnins.2020.570400](https://doi.org/10.3389/fnins.2020.570400)
- Updated by: corrigendum [10.3389/fnins.2020.627512](https://doi.org/10.3389/fnins.2020.627512), 2020-12-01
- Verification: crossref-verified | Access: open-access | Read: full text | evidence tier **D**
- Backs:
  - HRV amplitude peaks near each person's resonance frequency and is smaller at adjacent breathing rates
    - "External stimulation such as slow paced breathing or rhythmic skeletal muscle tension near an individual’s precise resonance frequency produces the greatest RSA and HRV and increases baroreflex gain (heart rate change per 1 mmHg change in blood pressure; Lehrer and Gevirtz, 2014)." (Resonance Frequency Model)
    - "While breathing at adjacent rates may still stimulate the baroreflex, it will produce smaller resonance effects (Lehrer et al., 2020a)." (A Resonance Frequency Assessment Protocol, Terms and Definitions)
    - "Higher absolute and percent total LF power are desirable because they increase as the respiration rate approaches the resonance frequency and more effectively stimulates the baroreflex (Vaschillo et al., 2002)." (Resonance Frequency Selection)
  - resonance frequency ranges from 4.5 to 6.5 breaths per minute in adults, and 6.5 to 9.5 in children
    - "The resonance frequency ranges from 4.5 to 6.5 bpm for adults and 6.5 to 9.5 bpm for children." (A Resonance Frequency Assessment Protocol)
  - breathing in a narrow band around the resonance frequency stimulates the baroreflex better than breathing across a wider range
    - "Breathing in a narrow frequency range around the resonance frequency better stimulates the baroreflex and increases RSA than breathing in a wider frequency range (Vaschillo et al., 2002)." (A Resonance Frequency Assessment Protocol, Terms and Definitions)
- Caveat: Methods guide with no experiment of its own, hence tier D. It bears on exhale's drift setting: if HRV amplitude peaks at the resonance frequency, breathing progressively slower moves away from that peak once past it, so a slower pace isn't always better for HRV amplitude. Gap 6 notes that the peak is in HRV amplitude and doesn't transfer to relaxation or comfort, so this bounds the HRV argument for drift without settling the question. The guide uses comfort to break ties between nearby rates and warns that 4.5 breaths a minute may lead to overbreathing and vagal withdrawal in clients who struggle with it. Shaffer is also an author of meehan2024-longer-exhalations, so this corpus leans on one group twice. The guide calls the evidence that breathing near the resonance frequency raises RSA and baroreflex gain compelling, and the authors say they lack conclusive evidence that resonance frequency breathing produces better clinical outcomes in most disorders.
  - "The authors explain the rationale for the resonance frequency model and provide detailed instructions on how to monitor and assess the resonance frequency." (Abstract)
  - "There is compelling evidence that breathing near an individual’s resonance frequency – but not at the exact rate – increases RSA amplitude and baroreflex gain compared with other frequencies (Vaschillo et al., 2002, 2004; Lehrer et al., 2020a). However, we lack conclusive evidence that resonance frequency breathing produces superior clinical outcomes in treatment of most disorders (Lehrer et al., 2020a)." (The Importance of Resonance Frequency Assessment)
  - "Consider your clients’ perspective when breaking ties between nearby breathing rates and then reconfirm that rate during the first training session. Which rate feels most comfortable?" (How to Break Ties and Confirm the Resonance Frequency)
  - "If your clients struggle with breathing at 4.5 bpm, this pace may result in overbreathing and vagal withdrawal" (How to Break Ties and Confirm the Resonance Frequency)
- Quotes checked: 9 of 9 against the Europe PMC full text and the Europe PMC abstract, last on 2026-10-18

#### `szulczewski2019-training-relaxation`

Szulczewski, Mikołaj Tytus. (2019). *Training of paced breathing at 0.1 Hz improves CO2 homeostasis and relaxation during a paced breathing task*. PLOS ONE 14(6): e0218550

- DOI: [10.1371/journal.pone.0218550](https://doi.org/10.1371/journal.pone.0218550)
- Verification: crossref-verified | Access: open-access | Read: full text | evidence tier **C**
- Backs:
  - over a week of daily ten-minute paced breathing, self-reported task pleasantness was significantly higher on day seven than on day one, and the drop in unpleasant arousal from before to after the task was significant on days four and seven only
    - "A total of 16 participants performed ten minutes of paced breathing every day for seven days." (Abstract)
    - "Affective state was shifted towards calmness and relaxation during the second and third laboratory measurements, but not during the first measurement. Additionally, the breathing task was perceived as more pleasant during subsequent laboratory measurements." (Abstract)
    - "The perceived pleasantness of the task increased with training, F(2, 30) = 5.22, p < .05, η2 = .26. [...] Post-hoc analysis showed that there was a significant difference between pleasantness during the first and third laboratory measurements, p < .01." (Results)
    - "Post-hoc comparison showed that the decrease in unpleasant arousal was non-significant during the first laboratory measurement, p = .63, but unpleasant arousal decreased during the second, p < .05, and third, p < .05, laboratory measurements." (Results)
    - "Partial pressure of end-tidal CO2 (PetCO2), symptoms of hyperventilation, affective state (before and after breathing), and pleasantness of the task were measured on the first, fourth, and seventh days of training." (Abstract)
  - the end-tidal CO2 drop during paced breathing shrank by day seven: 37.5% of participants fell below 30 mmHg on day one, and on day seven the only participant below it had already been below 30 mmHg at rest
    - "Results showed that the drop in PetCO2 significantly decreased with training and none of the participants experienced a drop in PetCO2 below 30 mmHg by day seven of training (except one participant who already had PetCO2 below 30 mmHg during baseline), in comparison to 37.5% of participants on the first day." (Abstract)
    - "Post-hoc analysis showed that the drop in PetCO2 between baseline and paced breathing was significant during the first, p < .001, and second laboratory measurements, p < .01, but not during the third laboratory measurement, p = .17." (Results)
    - "Paired sample t-tests showed that drop of PetCO2 during first and second laboratory measurements did not differ significantly, t(15) = 1.63, p = .13, but PetCO2 dropped less during the third measurement than during the first one, t(15) = 4.63, p < .001, and the second one, t(15) = 2.51, p < .05." (Results)
    - "PetCO2 below 30 mmHg Baseline 0% 6,3% 6,3% Paced breathing 37,5% 12,5% 6,3%" (Table 1)
- Caveat: n = 16, single group, no control, one week, and four participants skipped one day of training. The drop in end-tidal CO2 shrank with training, but the intensity of hyperventilation symptoms didn't decrease significantly. Small and uncontrolled, so treat the magnitudes as indicative. It's carried because it's the closest thing in this corpus to evidence for the pranayama proposition that the practice improves with practice: the drop in unpleasant arousal within each session grew across the week. In Table 2, mean unpleasant arousal before the task was -3.34, -2.84 and -2.72 on the three laboratory days and after it -3.50, -3.69 and -3.56, so the larger drop on later days came from higher values before the task. Training was at a fixed 0.1 Hz throughout, 4 s in and 6 s out, so it's evidence that repeated practice at one rate gets better. Whether progressively slowing within a session helps is a second proposition, and it remains untested. See also joshi1992-pranayam-training.
  - "A total of 16 participants were recruited via social media (8 men and 8 women)." (Method, Participants)
  - "The study lasted seven days. Participants performed a ten-minute-long paced breathing task every day." (Method, Procedure)
  - "Next, participants breathed for ten minutes at a frequency of 0.1 Hz." (Method, Procedure)
  - "Home practice was done using the same audiovisual pacer." (Method, Procedure)
  - "However, despite the decrease in the drop of PetCO2, the intensity of symptoms did not decrease significantly with training." (Discussion)
  - "Thus, the present results suggest that both CO2 homeostasis and the affective effects of paced breathing improve with training." (Discussion)
  - "Four participants skipped one day of training." (Method, Procedure)
  - "Unpleasant arousal Baseline -3.34(1.18) -2.84(1.39) -2.72(1.24) After paced breathing -3.50(1.69) -3.69(1.63) -3.56(2.05)" (Table 2)
  - "Inhalation lasted four seconds and exhalation lasted six seconds, a ratio that resembles the spontaneous ratio during breathing at 0.1 Hz when only the length of the whole breathing cycle is paced" (Method, Procedure)
- Quotes checked: 18 of 18 against the Europe PMC full text and the Europe PMC abstract, last on 2026-10-18

#### `vandiest2014-ie-ratio-relaxation`

Van Diest, Ilse; Verstappen, Karen; Aubert, André E.; Widjaja, Devy; Vansteenwegen, Debora; Vlemincx, Elke. (2014). *Inhalation/Exhalation Ratio Modulates the Effect of Slow Breathing on Heart Rate Variability and Relaxation*. Applied Psychophysiology and Biofeedback 39(3-4): 171-180

- DOI: [10.1007/s10484-014-9253-x](https://doi.org/10.1007/s10484-014-9253-x)
- Open copy: <https://lirias.kuleuven.be/retrieve/843e9b82-a9b0-42e6-897e-9177c10d71b1>
- Verification: crossref-verified | Access: paywalled | Read: full text | evidence tier **C**
- Backs:
  - participants reported more relaxation, more stress reduction, more mindfulness and more positive energy breathing with a low inhale/exhale ratio (longer exhale) than a high one
    - "Participants reported increased relaxation, stress reduction, mindfulness and positive energy when breathing with the low compared to the high i/e ratio." (Abstract)
    - "The participants scored higher on basic relaxation, mindfulness, positive energy and stress reduction after adjustment to the breathing patterns with a low as compared to a high i/e ratio" (Results, Self-reported relaxation and affect)
    - "The third (“3\_7”) consisted of a breathing rate of 6 breaths/min with a 3 s inhalation and a 7 s exhalation (i/e ratio of 0.42). A fourth and final video ('7\_3”) contained a slow breathing rate of 6 breaths/min with an i/e ratio of 2.33 (a 7 s inhalation to a 3 s exhalation)." (Method, Stimuli)
  - a low inhale/exhale ratio also produced more HF-HRV power, but only in the slow breathing condition
    - "A low i/e ratio was also associated with more power in the high frequency component of heart rate variability, but only for the slow breathing pattern." (Abstract)
    - "A low compared to a high i/e ratio resulted in a significantly higher HF-HRV when participants were breathing at 6, but not at 12 breaths/minute." (Discussion)
  - of the relaxation dimensions measured, slowing the rate on its own improved only positive energy, though it also raised pleasantness and lowered arousal on the SAM
    - "A lower compared to a higher respiration rate was associated only with an increased score on positive energy." (Abstract)
    - "Following the performance of each instructed breathing pattern (see Table 2), participants reported higher pleasantness for the low compared to the high i/e ratio, F(1, 22) = 7.41, p < .05, η2p = .25, and with a respiration rate of 6 compared to 12 breaths a minute, F(1, 22)= 4.43, p < .05, η2p = .17. Also, they reported lower arousal with a respiration rate of 6 compared to 12 breaths a minute, F(1, 22) = 8.17, p < .05" (Results, Self-reported relaxation and affect)
    - "Higher scores on arousal, control and valence indicate more arousal, more control and higher pleasantness, respectively; the Self-Assessment Manikin (SAM Scale) is from Bradley and Lang (1994)" (Table 2)
- Caveat: 30 recruited, 23 analyzed after five exclusions on criteria and two for not following the pacing, and every F-test carries df (1, 22). Of the 30 recruited, two were men. Four patterns crossing rate (6 or 12 breaths/min) with i/e ratio (0.42 or 2.33). At 6 a minute that's 3 s in and 7 s out against 7 s in and 3 s out, so the longer exhale was compared with a longer inhale, and no condition had an equal ratio. The achieved ratios at 6 bpm were 0.49 and 1.44. The achieved rates in those conditions were 7.32 and 7.69 breaths a minute, above the target of 6, so the slow-rate contrast tested was about 0.5 against 1.4 at a little over 7 a minute. The authors note that minute ventilation during instructed breathing was high compared with baseline and that participants may have been hyperventilating, which they offer as one reason the subjective effects were small. The tidal volumes come from a signal the authors didn't calibrate with spirometry, so they say the volumes shouldn't be read as absolute values. Heart rate was higher with the longer exhale, which the authors call contrary to expectation. The ratio had no effect on LF-HRV, which rose with the slower rate alone. This entry bears most on exhale's longer-exhale preference: it's the one study of the inhale/exhale ratio in this corpus in which the longer exhale won on how people felt. One explanation the authors give for that result is that the long-inhale pattern may have been artificial and harder to follow, resulting in more stress and lower relaxation. It isn't the only study that measured how people felt. lin2014-equal-ratio-hrv found every slow pattern raised relaxation over baseline, and its abstract doesn't say whether the patterns differed from each other. marchant2025-square-478-six, at more than three and a half times the sample, found no meaningful mood change in any condition. Gap 4 weighs the three.
  - "The sample consisted of 30 undergraduate students (age range: 18-22 years; 2 men)" (Method, Participants)
  - "Five participants were excluded from analysis because they were positive on at least one criterion: one participant with experience in relaxation and breathing techniques, two participants taking selective serotonin reuptake inhibitors and/or beta-blockers and two participants reporting asthma. Two other participants were excluded from analysis because they did not comply with the breathing instructions. As a result, N was 23 in our final dataset, which was subjected to statistical analysis." (Method, Participants)
  - "Thirty participants were instructed to breathe at 6 or 12 breaths/min, and with an i/e ratio of 0.42 or 2.33." (Abstract)
  - "Baseline 12 breaths/minute 6 breaths/minute Low i/e ratio High i/e ratio Low i/e ratio High i/e ratio RR 14.88 (4.22) 12.27(0.90) 12.48(0.64) 7.32(1.90) 7.69(2.08) Ti/Te 0.67 (0.11) 0.53(0.07) 1.42(0.19) 0.49(0.06) 1.44(0.26) Vt 532 (216) 854 (325) 874 (269) 1383 (424) 1262 (372)" (Table 1)
  - "We did not calibrate the RIP signal with spirometry, so the reported volumes should not be used as absolute quantitative values." (Method, Measures)
  - "Although we instructed persons not to breathe very deeply when following the paced breathing patterns, minute ventilation (respiration rate x tidal volume) during instructed breathing was high compared to spontaneous breathing during baseline. This is a common phenomenon, rather typical for untrained persons. Our participants may have been hyperventilating during instructed breathing, which may explain the rather small effects observed on self-reported relaxation and changes in affect for each of the breathing patterns." (Discussion)
  - "Contrary to what one would expect, heart rate was increased when breathing at a low i/e ratio as compared to a high i/e ratio." (Discussion)
  - "An alternative explanation may be that the high i/e ratio applied in the present study is a rather artificial breathing pattern that is unlikely to occur during spontaneous breathing in natural circumstances. Participants may have experienced more difficulties with complying with the breathing patterns with a high i/e ratio, resulting in more stress and lower relaxation." (Discussion)
  - "LF-HRV was higher when breathing at 6 compared to 12 breaths a minute, F(1, 22) = 862,48, p < .05, η2p = .98). No other effects were present for LF-HRV." (Results, Cardiovascular parameters)
- Quotes checked: 17 of 17 against the Lirias repository copy and the Europe PMC abstract, last on 2026-10-18

#### `you2023-respiratory-frequency`

You, Min; Laborde, Sylvain; Ackermann, Stefan; Borges, Uirassu; Dosseville, Fabrice; Mosley, Emma. (2023). *Influence of Respiratory Frequency of Slow-Paced Breathing on Vagally-Mediated Heart Rate Variability*. Applied Psychophysiology and Biofeedback 49(1): 133-143

- DOI: [10.1007/s10484-023-09605-2](https://doi.org/10.1007/s10484-023-09605-2)
- Verification: crossref-verified | Access: paywalled | Read: abstract only | evidence tier **C**
- Backs:
  - five minutes of slow-paced breathing at 5, 5.5, 6, 6.5 and 7 cycles per minute all raised cardiac vagal activity above spontaneous breathing
    - "Therefore, this study aimed to investigate the effects of a brief SPB intervention (i.e., 5 min) on CVA using heart rate variability (HRV) measurement as an index." (Abstract)
    - "including SPB at different frequencies (5 cycles per minute (cpm), 5.5 cpm, 6 cpm, 6.5 cpm, 7 cpm), and a control condition of spontaneous breathing" (Abstract)
    - "The study found that CVA was significantly higher in all SPB conditions compared to the control condition, as indexed by both root mean square of the successive differences (RMSSD) and low-frequency HRV (LF-HRVms2)." (Abstract)
  - LF-HRV discriminated between the tested frequencies more sensitively than RMSSD
    - "Interestingly, LF-HRVms2 was more sensitive in differentiating the respiratory frequencies than RMSSD." (Abstract)
  - the band tested and supported is 5 to 7 cycles per minute
    - "including SPB at different frequencies (5 cycles per minute (cpm), 5.5 cpm, 6 cpm, 6.5 cpm, 7 cpm)" (Abstract)
    - "These results suggest that SPB at a range of 5 cpm to 7 cpm can be an effective method to increase CVA and potentially improve stress management and emotion regulation in athletes." (Abstract)
- Caveat: Crossref records this as issued 2023 (online 8 December). It appears in the March 2024 issue, 49(1), which is how it's usually cited. The citekey follows the Crossref issued year, as everywhere else in this corpus. n = 75, all athletes aged 19-31, single lab session. Generalization to a desk worker is an assumption the study didn't test. This is the source that fixes the tested band the settings panel reports: 5 s in and 5 s out is 6 cycles per minute, inside it, and the earlier 5 s in and 10 s out is 4.0, below it.
  - "A total of 75 athletes (22 female; Mage = 22.32; age range = 19-31) participated in the study, attending one lab session" (Abstract)
- Quotes checked: 7 of 7 against the Europe PMC abstract, last on 2026-10-18

---

## Where the practice came from

3 sources.

#### `joshi1992-pranayam-training`

Joshi, L. N.; Joshi, V. D.; Gokhale, L. V. (1992). *Effect of short term 'Pranayam' practice on breathing rate and ventilatory functions of lung*. Indian Journal of Physiology and Pharmacology 36(2): 105-108

- PMID: [1506070](https://pubmed.ncbi.nlm.nih.gov/1506070/) (no DOI exists) ([archived](https://web.archive.org/web/20260114204518/https://pubmed.ncbi.nlm.nih.gov/1506070/))
- Open copy: <https://ijpp.com/IJPP%20archives/1992_36_2/105-108.pdf> ([archived](https://web.archive.org/web/20261018201939/https://ijpp.com/IJPP%20archives/1992_36_2/105-108.pdf))
- Verification: pubmed-verified | Access: paywalled | Read: full text | evidence tier **C**
- Backs:
  - six weeks of pranayama practice in 33 men and 42 women of mean age 18.5 prolonged breath-holding time, and the authors report a significant fall in respiratory rate, though the printed means fell only from 18.2 to 17.5 a minute in the men and from 19.2 to 19.1 in the women
    - "Thirty three normal male and forty two normal female subjects, of average age of 18.5 years, underwent six weeks course in 'Pranayam' and their ventilatory lung functions were studied before and after this practice." (Abstract)
    - "They had improved ventilatory functions in the form of lowered respiratory rate (RR), [...] and prolongation of breath holding time." (Abstract)
    - "There was a significant decrease in the rate of respiration, both in males and females." (Results and Discussion, p. 106)
    - "There is significant increase in BHT" (Results and Discussion, p. 107)
    - "R.R./min n = 28 18.2 17.5 2.07 ± 1.99 5.5 S" (Table I, p. 106)
    - "Females [...] n = 40 19.2 19.1 1.875± 1.71 6.9 S" (Table II, R.R./min row, p. 106)
  - the authors report that the same training significantly raised forced vital capacity, maximum voluntary ventilation and peak expiratory flow rate in both sexes. FEV1, which the paper reports as a percentage, rose in the women and fell in the men
    - "increases in the forced vital capacity (FVC), [...] maximum voluntary ventilation (MVV), peak expiratory flow rate (PEFR-lit/sec)" (Abstract)
    - "Forced expiratory vol. at the end of first second in %." (Methods, p. 106)
    - "There were significant increases in FVC, MVV and PEFR, both in males and females." (Results and Discussion, p. 107)
    - "The FEV [...] which was increased significantly in females but decreased in males, needs to be studied further." (Results and Discussion, p. 107)
- Caveat: Read in full from the journal's own archive PDF. No DOI exists, and the record was verified against the NCBI E-utilities API by PMID. 33 male and 42 female medical students, mean age 18.5, each their own control with no separate control group. The protocol was 20 minutes twice a day on weekdays and once on Saturdays for six weeks: two minutes of slow maximal breaths at 5 s in and 5 s out (6 a minute, exhale's default rate), sixteen minutes of 5 s maximal inhale, 17 s hold and 8 s maximal exhale (a 30-second cycle, 2 a minute), then two minutes at 5 s in and out again. Uncontrolled before-and-after design in a 1992 regional journal, so treat the effect sizes as unusable. Per-measure n in the tables runs from 23 to 42 per sex, so the respiratory-rate, FVC and FEV1% results rest on 68, 62 and 58 of the 75 people, and Table III's FVC and FEV1% differences don't match Tables I and II. The tables also don't reconcile with the reported significance. In none of the 14 before-and-after rows of Tables I and II does the mean difference match the gap between the before and after means: FVC moves from 3.169 to 3.20 L in the men and from 2.31 to 2.37 L in the women beside mean differences of 0.47 and 0.31, and the men's FEV1% falls from 95.2 to 94 beside a positive 4.46 marked significant. No Crossref record exists to carry a license and the journal's archive serves the PDF free without stating one, so the access level stays paywalled with the archive PDF as the open copy. It's carried for what it says about adaptation across sessions: breath-holding time grew over six weeks of one fixed pattern, so it bears on graded extension as a training progression without testing one. Extending the breath without limit inside a single sitting is a different proposition, and shaffer2020-resonance-frequency-assessment bears on it. The trained pattern was hold-heavy, which bears on gap 11: six weeks of a 17-second retention with no adverse effects reported, in an uncontrolled study that wasn't looking for them.
  - "33 normal male and 42 normal female medical students, of average age of 18.5 yrs, underwent a six weeks practice" (Methods, p. 105)
  - "Each subject acted as his or her own control." (Methods, p. 105)
  - "Pranayamic practice each time as indicated below, lasted for 20 minutes and was practised twice a day on week days and once on saturday, for a total period 6 weeks." (Methods, p. 105)
  - "slow deep maximum inhalation & exhalation each lasting for 5 secs. Thus RR was 6/min." (Methods, p. 105)
  - "17 secs - holding breath with efforts at same lung vol. 8 secs - slow maximum exhalation." (Methods, p. 105)
  - "(iii) Last 2 minutes [...] same as in (i)" (Methods, p. 105)
- Quotes checked: 16 of 16 against the journal's archive PDF and the Europe PMC abstract, last on 2026-10-18

#### `muktibodhananda1998-hatha-yoga-pradipika`

Muktibodhananda, Swami. (1998). *Hatha Yoga Pradipika*, 3rd ed. Munger, Bihar, India: Bihar School of Yoga 642 pp

- ISBN: 9788185787381 | [Open Library record](https://openlibrary.org/books/OL9083573M/Hatha_Yoga_Pradipika)
- Verification: openlibrary-verified | Access: paywalled | Read: catalog record only | evidence tier **E**
- Backs:
  - the classical hatha text and commentary in which breath retention is discussed is presented as the historical origin of the ratio instructions modern breathing apps repeat
- Caveat: Contents not consulted. The bibliographic record was checked against Open Library, which lists this ISBN as the third edition, Bihar School of Yoga, 1998, 642 pages. Not peer reviewed, and a commentary on a fifteenth-century text rather than a primary source in any modern sense. Carried because the longer-exhale instruction exhale ships came from this tradition, centuries before anyone measured HRV. Naming that is more accurate than retrofitting a citation to psychophysiology. It must never back a physiological claim.

#### `satyananda1999-apmb`

Satyananda Saraswati, Swami. (1999). *Asana Pranayama Mudra Bandha*, 3rd rev. ed. Munger, Bihar, India: Yoga Publications Trust 553 pp

- ISBN: 9788186336144 | [Open Library record](https://openlibrary.org/books/OL22138410M/Asana_pranayama_mudra_bandha) ([archived](https://web.archive.org/web/20260108002006/https://openlibrary.org/books/OL22138410M/Asana_pranayama_mudra_bandha))
- Verification: openlibrary-verified | Access: paywalled | Read: catalog record only | evidence tier **E**
- Backs:
  - the systematic pranayama tradition from which exhale's controllable inhale / retention / exhale / retention structure descends has a standard modern reference manual, whose contents haven't been read
- Caveat: Contents not consulted. The bibliographic record was checked against Open Library, which lists this ISBN as the third revised edition, Yoga Publications Trust, 1999, 553 pages. A 2008 fourth revised edition exists under the same imprint. This entry cites the edition the ISBN resolves to. Not peer reviewed. Cited only for lineage: it's where exhale's four-phase structure comes from, and presenting the design as derived from 2020s psychophysiology would be revisionist. It must never back a physiological claim.

---

## Whether an on-screen visual pacer works

3 sources.

#### `moraveji2011-peripheral-paced-respiration`

Moraveji, Neema; Olson, Ben; Nguyen, Truc; Saadat, Mahmoud; Khalighi, Yaser; Pea, Roy; Heer, Jeffrey. (2011). *Peripheral paced respiration: influencing user physiology during information work*. Proceedings of the 24th Annual ACM Symposium on User Interface Software and Technology (UIST '11) 423-428

- DOI: [10.1145/2047196.2047250](https://doi.org/10.1145/2047196.2047250)
- Open copy: <https://idl.cs.washington.edu/files/2011-PPR-UIST.pdf> ([archived](https://web.archive.org/web/20260816053026/https://idl.cs.washington.edu/files/2011-PPR-UIST.pdf))
- Verification: crossref-verified | Access: paywalled | Read: full text | evidence tier **C**
- Backs:
  - a translucent animated bar spanning the screen, running in the periphery of attention during normal information work, significantly lowered participants' breathing rate
    - "The peripheral paced respiration feedback used an animated, semi-translucent grey bar stretching across the screen." (Figure 2 caption, p. 425)
    - "We conducted an experiment to evaluate the efficacy and feasibility of using a PPR system to pace users’ respiration to a resting rate while authentically engaged in information work." (p. 425)
    - "We found a significant difference between no-feedback (M=17.58, SD=4.18) and PPR (M=15.7, SD=4.49) conditions; t(12)= 3.83, p<0.005." (RESULTS, p. 426)
  - the authors describe peripheral pacing as not requiring the user's full attention, and breathing slowed while participants did naturalistic work, with self-reports rating the pacer not too distracting
    - "Peripheral paced respiration differs from prior techniques in that it does not require the user’s full attention." (Abstract)
    - "Based on self-report data, PPR feedback was not too distracting and participants expressed motivation to use PPR for sustained periods in the future." (DISCUSSION AND FUTURE WORK, p. 427)
    - "Our hypothesis that PPR influences user respiration while engaged in naturalistic tasks is supported; the peripheral feedback reduced breath rate significantly." (DISCUSSION AND FUTURE WORK, p. 426)
- Caveat: n = 13 (9 men, 4 women, mean age 25.5), students from computer science and related fields doing their own work, two 20-minute conditions. The pacer was sensor-triggered and intermittent: it ran in 2-minute bursts when the wearer's breath rate rose 20% above their resting baseline or at least once every six minutes, and was on for about 60% of the pacer condition. It paced to 20% below that baseline, which was taken during three minutes of eyes-closed relaxation (a pre-survey for three of the thirteen) and averaged 9.3 a minute, so the target was near 7.5 a minute, close to exhale's 6. On average they didn't get there: the rate while a burst was on averaged 14.96 (SD 4.44), and the paper plots individual breath rates for only one of the thirteen participants (Figure 5). Mean rate was 15.7 a minute in the pacer condition against 17.6 without, a difference the paper gives as 1.8 bpm in the results and 1.9 in the discussion, and between bursts the rate returned to 17.1. This is the closest published analogue to exhale that exists, and its limitation matters most: the reduction occurred while the pacing feedback was active and didn't persist as a lasting change in respiratory pattern. An always-on overlay should be understood as an effect that lasts as long as it's on. The authors didn't collect quantitative measures of how distracting the pacer was. In their early tests, the pulse rate of a circle in the top-right screen corner wasn't noticeable when users were deeply engaged in their work, even when exaggerated, and the experiment used the screen-wide bar. An author copy is served from idl.cs.washington.edu. The ACM Digital Library labels its version Free Access, and the only license Crossref registers is ACM's own, so the entry is marked paywalled with the author copy as its open copy.
  - "We recruited thirteen university students (9 male, 4 female, mean age=25.5 yrs) from computer science and related disciplines to participate. We told them they could conduct their existing work during the experiment." (EXPERIMENT, p. 425)
  - "To guarantee that pacing would occur at least three times during the PPR condition (at least once every six minutes), the duration of each condition was 20 minutes." (Procedure, p. 426)
  - "If the user’s current breath rate is 20% above their resting rate, pacing is triggered. The software will also automatically trigger pacing at least once every six minutes. Similar to prior studies, we use a 2-minute duration" (Pacing Respiration Peripherally, p. 425)
  - "The mean difference between conditions was 1.8bpm. [...] We conducted a paired t-test to compare the means between when PPR was activated (M=14.96, SD=4.44) and when it was not (M=17.09, SD=5.25); t(12)=t3.5647, p<0.005. [...] The mean proportion of time that the feedback was activated was 0.60 (SD=0.14)." (RESULTS, p. 426)
  - "The target rate is set to 20% below their baseline to exaggerate the slow rhythm of resting breath." (Pacing Respiration Peripherally, p. 425)
  - "As in prior studies [8], participants were then asked to close their eyes for 3-minutes and relax. Unbeknownst to participants, baseline data was collected during this relaxation period. Three participants did not take part in the relaxation period; their baseline was recorded as they completed a pre-survey." (Procedure, p. 426)
  - "Consistent with prior studies, the mean breath rate across conditions was 16.67bpm (SD=4.28) and the breath rate of the relaxation period baseline was 9.33bpm (SD=5.31)." (RESULTS, p. 426)
  - "To illustrate how the breath rate is impacted by PPR, Figure 5 depicts the breath rate of one participant in each of the conditions." (RESULTS, p. 426)
  - "The 1.9bpm difference is close to the 2bpm interval changes shown by Song and Lehrer to correspond with significantly higher heart rate variability amplitude" (DISCUSSION AND FUTURE WORK, p. 426)
  - "Upon further analysis, we attribute this difference to a significant decrease in breath rate while the intermittent pacing feedback is active, rather than a persistent change in respiratory pattern." (Abstract)
  - "Breath rates were observed to return to working levels between pacing instances. Hence, there was no evidence of persistent rate change." (DISCUSSION AND FUTURE WORK, p. 427)
  - "Quantitative measures of the level of distraction caused by PPR (such as working while pacing) were not collected in this study." (DISCUSSION AND FUTURE WORK, p. 427)
  - "The first prototype used the common method of a pulsing circle atop other windows in the top-right screen corner as pacing stimulus. [...] Second, the rate of the pulsing circle was not noticeable when users were deeply engaged in their work, even when we exaggerated the pulse rate." (Wizard-of-Oz Prototype, p. 425)
- Quotes checked: 19 of 19 against the authors' PDF and the abstract in the authors' PDF, last on 2026-10-18

#### `tabor2022-guided-breathing-design`

Tabor, Aaron; Bateman, Scott; Scheme, Erik J.; schraefel, m.c. (2022). *Comparing heart rate variability biofeedback and simple paced breathing to inform the design of guided breathing technologies*. Frontiers in Computer Science 4: 926649

- DOI: [10.3389/fcomp.2022.926649](https://doi.org/10.3389/fcomp.2022.926649)
- Verification: crossref-verified | Access: open-access | Read: full text | evidence tier **C**
- Backs:
  - an expanding and contracting circle pacing 6 breaths per minute reduced breathing rate and raised LF power as much as sensor-driven HRV biofeedback, with no significant group difference, and its HRV amplitude gain was larger (4.2 to 8.0 against 5.0 to 7.0, phase by protocol interaction p = 0.03), which the authors still summarize as comparable
    - "Participants were instructed to inhale as the circle increased in size and became darker ( Figure 2 -left) and exhale as it got smaller and lighter ( Figure 2 -right). Participants were told that the circle represented the expansion and contraction of their stomach during each cycle of breath, which was calibrated to guide breathing a constant rate of six breaths per minute (i.e., 10-s cycles)." (Apparatus, Paced breathing guide)
    - "To address this gap in knowledge, we compare the relative effectiveness of the two most prominent protocols used in existing guided breathing technologies: heart rate variability biofeedback (HRV-b) – a comparatively more complex protocol that employs real-time sensing and feedback to guide breathing – and, paced breathing – a simpler protocol that guides breathing in a consistent and rhythmic pattern (that is not based upon sensed physiological data)." (Introduction)
    - "Participants who completed HRV-b achieved reductions in mean breathing rate from 11.0 bpm (SD = 4.3) during baseline down to 5.2 bpm (SD = 1.3) during treatment. Meanwhile, participants who completed paced breathing achieved reductions from a baseline rate of 12.5 bpm (SD = 5.8) down to 6.1 bpm (SD = 0.03) during treatment. ANOVA results were not significant for this comparison ( F 1,26 = 1.32, p = 0.26)." (Results, H2, Breathing rate)
    - "Participants who completed HRV-b achieved an increase in ln(LF) from 7.0 (SD = 1.5) during baseline to 8.5 (SD = 1.1) during the breathing exercise. Meanwhile, participants who completed paced breathing achieved a comparable increase from 6.4 (SD = 1.4) during baseline to 8.0 (SD = 1.6) during the breathing exercise. ANOVA results were not significant for this comparison ( F 1,26 = 1.18, p = 0.29)." (Results, H2, ln(LF))
    - "Participants who completed HRV-b achieved an increase in hrv amplitude from 5.0 (SD = 2.0) during baseline to 7.0 (SD = 2.1) during the breathing exercise. Meanwhile, participants who completed paced breathing achieved a comparable increase from 4.2 (SD = 1.7) during baseline to 8.0 (SD = 3.4) during the breathing exercise. ANOVA results were not significant for this comparison ( F 1,26 < 1.0, p = 0.90). [...] We interpret these observations as evidence that a comparable increase in hrv amplitude was achieved with both breathing exercise protocols. However, a significant experiment phase \* breathing protocol interaction was also detected ( F 1,26 = 5.5, p = 0.03)." (Results, H2, HRV amplitude)
  - in both conditions, HRV changes occurred largely within the first 2 minutes, judged by visual inspection
    - "However, in our study, we observed that the changes in HRV occurred largely within the first 2 min of both breathing exercises – well before completing the full 10 min of practice." (Discussion, How long is long enough?)
    - "To consider whether one approach may allow people to reach lower breathing rates or HRV increases more quickly, we visually inspected data over the 10-min exercise. [...] This analysis reveals that the onset of exercise effects was largely consistent between HRV-b and paced breathing." (Effects materialize consistently, regardless of treatment or baseline HRV)
  - paced breathing needs no sensor and no close attention to real-time feedback, and the authors argue it's simple enough to do as a secondary task during other activities
    - "However, to provide this adaptivity, HRV-b requires both a sensing technology to collect heartbeat data and close attention for users to observe and adapt their breathing in real time. Paced breathing does not have these requirements, and because it is static and predictable, it has been suggested that it can even be practiced while performing other tasks; for example, during information work (Moraveji et al., 2011 ; Ghandeharioun and Picard, 2017 ; Tabor et al., 2021 ) or while driving (Paredes et al., 2018 )." (Introduction)
    - "Paced breathing is a simpler protocol where a user is asked to match their breathing pattern with a cue paced at a predetermined rate and is simple enough to be completed as a secondary task during other activities." (Abstract)
    - "For example, research exploring the design of peripheral breathing guides (Moraveji et al., 2011 ; Ghandeharioun and Picard, 2017 ; Tabor et al., 2021 ) suggests that users may be able to adhere to a paced breathing stimulus without constant attention to their breath, partially mitigating the need to motivate the practice through measures of performance and progress." (Discussion, Providing performance feedback and measures of progress)
- Caveat: Between-subjects, n = 28 (14 per group), single 10-minute session. There was no control group, and participants were assigned to groups systematically to balance age and sex, without randomization. Paced breathing was run as a focused task in the study. The secondary-task suitability in the third claim is the paper's design argument, which the experiment didn't test. The abstract says paced breathing 'provides the same immediate benefits as HRV-b', and fourteen per group can fail to find a difference without establishing equivalence. In this corpus it's the most direct support for two of exhale's design choices: an expanding and contracting shape, and no sensor.
  - "In our N=28 between-subject design, participants were trained in either SPB or HRV-b and then completed a 10-minute session following their training protocol." (Abstract)
  - "This resulted in an analysis composed of 28 participants, 14 per group." (Participants)
  - "We did not include a control group in our study design because the effectiveness of both HRV-b and paced breathing compared to control is well-established in the literature (e.g., Sherlin et al., 2009 ; Prinsloo et al., 2014 ; Critchley et al., 2015 ; Steffen et al., 2017 )." (Discussion, Summary and interpretation of results)
  - "According to established best practice (e.g., Voss et al., 2015 ), participants were systematically assigned to treatment groups to balance age and sex as sessions occurred." (Participants)
  - "Following the distractor task, participants in each group then performed a timed 10-min session of their respective breathing exercises." (Procedure, 10-min breathing exercise)
  - "Our paper contributes new findings suggesting that simple paced breathing – a straightforward, intuitive, and easy-to-design breathing exercise – provides the same immediate benefits as HRV-b, but without its added design complexities." (Abstract)
- Quotes checked: 16 of 16 against the publisher's page and the Crossref abstract, last on 2026-10-18

#### `wongsuphasawat2012-cant-force-calm`

Wongsuphasawat, Kanit; Gamburg, Alex; Moraveji, Neema. (2012). *You can't force calm: designing and evaluating respiratory regulating interfaces for calming technology*. Adjunct Proceedings of the 25th Annual ACM Symposium on User Interface Software and Technology (UIST '12 Adjunct) 69-70

- DOI: [10.1145/2380296.2380326](https://doi.org/10.1145/2380296.2380326)
- Open copy: <https://hci.stanford.edu/publications/2012/CantForceCalmUIST2012.pdf> ([archived](https://web.archive.org/web/20260208135206/https://hci.stanford.edu/publications/2012/CantForceCalmUIST2012.pdf))
- Verification: crossref-verified | Access: paywalled | Read: full text | evidence tier **C**
- Backs:
  - auditory pacing was rated more calming than visual pacing (t = 1.92, df = 26, reported as p < 0.05), while the visual guide's larger reduction in breathing rate was a non-significant trend (t = 0.66, p = 0.26)
    - "A two-sample t-test revealed that auditory guidance is subjectively more calming than visual (t=1.92, df =26, p<0.05), despite a trend whereby auditory guidance led to a smaller change in breathing rate compared to visual (t=0.662, df =26, p=0.256)." (RESULTS AND DISCUSSION, p. 70)
- Caveat: n = 14, phone-based, 1-minute drills at 6.4 breaths per minute, each followed by 3 minutes of reading, both modalities intermittent. The visual guide showed the user's own stomach expansion against a target, so it was biofeedback, which exhale's visual pacer isn't. Both modes raised self-reported calm over reading. About half the participants closed their eyes while using the audio guide. Two-page adjunct paper, effectively a poster: the calm difference clears p < 0.05 only one-tailed, the breathing-rate p of 0.256 is one-tailed too (two-tailed about 0.51), and the between-modality tests were run as two-sample tests on paired data. Treat the finding only as a design signal. It's carried anyway because it's the one published comparison that puts exhale's visual-only design at a disadvantage on the outcome most users care about, which is whether they feel calmer. An author copy is served from hci.stanford.edu.
  - "We conducted a randomized-order within-subject study. 14 university students participated (3 females, ages ranged from 19-29)." (STUDY DESIGN, p. 70)
  - "The application consists of a 1-minute breathing drill [...] The drill involves matching one’s breathing to a guide (auditory or visual) that vacillates at a rate of 6.4 breath cycles per minute (a calm breath rate associated with high heart rate variability [2])." (APPARATUS DESIGN, p. 69)
  - "They were then asked to read a blog article about US scientific research for 3 minutes. They were then asked to do the other breathing drill then continue reading for another 3 minutes." (STUDY DESIGN, p. 70)
  - "We conducted an evaluation of visual and auditory methods of intermittent pacing of respiration via a mobile phone." (CONCLUSION, p. 70)
  - "The interface includes two circles: a solid blue line showing an ideal expansion level at any moment and a blurred orange circle, representing users’ current stomach expansion." (APPARATUS DESIGN, Visual Mode, p. 69)
  - "One-sample t-test revealed that each design also increased subjective calm compared to reading the article (A: t=5.17, df =13, p<0.05; V: t=2.40, df =13, p<0.05)." (RESULTS AND DISCUSSION, p. 70)
  - "About half of the participants closed their eyes while using the audio to ignore visual noise." (RESULTS AND DISCUSSION, Cognitive Load, p. 70)
- Quotes checked: 8 of 8 against the authors' PDF, last on 2026-10-18

---

## Physiology and neuroscience

7 sources.

#### `lehrer2014-hrv-biofeedback`

Lehrer, Paul M.; Gevirtz, Richard. (2014). *Heart rate variability biofeedback: how and why does it work?* Frontiers in Psychology 5: 756

- DOI: [10.3389/fpsyg.2014.00756](https://doi.org/10.3389/fpsyg.2014.00756)
- Verification: crossref-verified | Access: open-access | Read: full text | evidence tier **D**
- Backs:
  - maximum heart-rate oscillation is usually reached breathing at approximately 0.1 Hz, six breaths per minute
    - "However, most studies find that maximum effects usually are achieved when breathing at a rate of approximately 0.1 Hz (six breaths per minute)." (PHASE RELATIONSHIPS BETWEEN HEART RATE OSCILLATIONS AND BREATHING)
    - "The maximum heart rate oscillations at respiratory frequency occurred at approximately 0.1 Hz (six breaths per minute)" (PHASE RELATIONSHIPS BETWEEN HEART RATE OSCILLATIONS AND BREATHING)
  - refined measurement puts the average resonance frequency nearer 0.09 Hz, about 5.5 breaths per minute, a breath lasting roughly 11 seconds
    - "In later research we refined the average frequency for highest heart rate oscillations to be at about 0.09 Hz, or 5.5 breaths per minute, with breath duration of about 11 s" (PHASE RELATIONSHIPS BETWEEN HEART RATE AND BLOOD PRESSURE)
  - resonance frequency is individual: taller people and men tend to have lower resonance frequencies
    - "For each person, he found, there was a specific frequency where heart rate changes per unit change in blood pressure were greatest. This frequency varied from individual to individual" (PHASE RELATIONSHIPS BETWEEN HEART RATE AND BLOOD PRESSURE)
    - "We have found that taller people and men, who have a greater blood supply than, respectively, shorter people and women, have lower resonance frequencies" (RESONANCE)
  - baroreflex gain increases substantially during HRV biofeedback
    - "We have found large increases in baroreflex gain (number of beats per minute change in heart rate per 1 mm Hg change in blood pressure) during HRV biofeedback: i.e., the baroreflex operates more strongly" (THE BAROREFLEX)
- Caveat: Narrative mechanistic review by Lehrer, whose group began HRVB work in the 1990s, and Gevirtz. Read as the mechanism argument, which isn't independent evidence. The individual-variation point is the reason exhale's timing sliders are user-editable at all: there's no single correct number to hardcode. Crossref registers no license for this DOI, and the article itself carries a CC BY license.
  - "In this article, we review these and other possible mechanisms that might explain the positive effects of HRVB." (Abstract)
  - "In the 1990s Lehrer et al. (2000) began experimenting with a form of cardiorespiratory intervention that has subsequently been labeled HRVB, respiratory sinus arrhythmia (RSA) biofeedback, or resonance frequency feedback (RFF)." (INTRODUCTION)
  - "We have been working under the assumption that increases baroreflex represented a viable marker of improved autonomic homeostasis or increased complexity. We have reviewed the evidence that led us to this conclusion, which now are somewhat established by our labs and others." (CONCLUSION)
- Quotes checked: 9 of 9 against the Europe PMC full text and the Europe PMC abstract, last on 2026-10-18

#### `li2016-sigh-circuit`

Li, Peng; Janczewski, Wiktor A.; Yackle, Kevin; Kam, Kaiwen; Pagliardini, Silvia; Krasnow, Mark A.; Feldman, Jack L. (2016). *The peptidergic control circuit for sighing*. Nature 530(7590): 293-297

- DOI: [10.1038/nature16964](https://doi.org/10.1038/nature16964)
- Open copy: <https://pmc.ncbi.nlm.nih.gov/articles/PMC4852886/> ([archived](https://web.archive.org/web/20261018202156/https://pmc.ncbi.nlm.nih.gov/articles/PMC4852886/))
- Verification: crossref-verified | Access: paywalled | Read: full text | evidence tier **D**
- Backs:
  - in the murine brain, a peptidergic circuit projecting onto the preBotzinger complex controls sighing with little short-term effect on other breathing, and the authors propose it transforms normal breaths into sighs
    - "Here we use molecular, genetic, and pharmacologic approaches to identify a peptidergic sigh control circuit in murine brain. Small neural subpopulations in a key breathing control centre, the retrotrapezoid nucleus/parafacial respiratory group (RTN/pFRG), express bombesin-like neuropeptide genes neuromedin B (Nmb) or gastrin-releasing peptide (Grp). These project to the preBötzinger Complex (preBötC), the respiratory rhythm generator, which expresses NMB and GRP receptors in overlapping subsets of ~200 neurons." (Abstract)
    - "Ablating receptor-expressing neurons eliminated basal and hypoxia-induced sighing, but left breathing otherwise intact initially." (Abstract)
    - "We propose that these overlapping peptidergic pathways comprise the core of a sigh control circuit that integrates physiological and perhaps emotional input to transform normal breaths into sighs." (Abstract)
    - "Our results show sighing is controlled by two largely parallel bombesin-like neuropeptide pathways, NMB and GRP, which mediate signaling between key medullary breathing control centers." (Discussion)
    - "A striking aspect of our results is the selectivity of the circuit for sighing. Inhibition of the pathways, and even ablation of receptor-expressing neurons, had little effect on other aspects of breathing, at least in the short term." (Discussion)
- Caveat: Mouse and rat work. It shows that the preBotzinger neurons carrying the two peptide receptors have a critical and selective function in basal sighing, and the authors propose the pathways as the core of a sigh control circuit. That's the mechanistic backdrop for the cyclic-sighing result in balban2023-cyclic-sighing. Five days after those neurons were ablated, the animals had occasional brief apneas or disordered breathing, which the authors suggest may follow from the loss of sighing. It says nothing about humans deliberately performing sighs, and must not be cited as if it did.
  - "All mouse strains used were in the C57BL/6 genetic background. [...] Male Sprague Dawley rats were from Charles River." (Methods, Animals)
  - "We conclude that preBötC Nmbr and Grpr -expressing neurons have a critical and selective function in basal sighing." (Results, Effect of NMBR and GRPR neuron ablation)
  - "However, after five days we noted occasional brief episodes of apneas or disordered breathing, possibly a consequence of the loss of sighing (see Discussion)." (Results, Effect of NMBR and GRPR neuron ablation)
- Quotes checked: 8 of 8 against the PubMed Central author manuscript and the Europe PMC abstract, last on 2026-10-18

#### `vlemincx2013-sigh-reset-model`

Vlemincx, Elke; Abelson, James L.; Lehrer, Paul M.; Davenport, Paul W.; Van Diest, Ilse; Van den Bergh, Omer. (2013). *Respiratory variability and sighing: A psychophysiological reset model*. Biological Psychology 93(1): 24-32

- DOI: [10.1016/j.biopsycho.2012.12.001](https://doi.org/10.1016/j.biopsycho.2012.12.001)
- Verification: crossref-verified | Access: paywalled | Read: abstract only | evidence tier **D**
- Backs:
  - the model proposes that sighs act as resetters that restore the balance between components of respiratory variability and cause relief
    - "The causes and consequences of sighing are reviewed and integrated in the proposed model in which sighing is hypothesized to function as a resetter in the regulation of both breathing and emotions, because it restores a balance in respiratory variability fractions and causes relief." (Abstract)
- Caveat: Review and theoretical model paper. Carried because it's the nearest literature in this corpus to exhale's randomized-timing sliders: it models sighs as resetters that restore a balance in respiratory variability, and it doesn't address deliberately irregular breathing. It's not evidence that those sliders help. See the gaps ledger.
  - "The present paper reviews the literature on respiratory variability and elaborates on the importance of assessing various components of respiratory variability when studying the interrelationships between emotions and breathing. A model is proposed that predicts specific action tendencies related to emotions to disturb the balance between various respiratory variability components depending on valence by arousal and control dimensions." (Abstract)
- Quotes checked: 2 of 2 against the Europe PMC abstract, last on 2026-10-18

#### `vlemincx2016-sigh-relief`

Vlemincx, Elke; Van Diest, Ilse; Van den Bergh, Omer. (2016). *A sigh of relief or a sigh to relieve: The psychological and physiological relief effect of deep breaths*. Physiology & Behavior 165: 127-135

- DOI: [10.1016/j.physbeh.2016.07.004](https://doi.org/10.1016/j.physbeh.2016.07.004)
- Verification: crossref-verified | Access: paywalled | Read: abstract only | evidence tier **C**
- Backs:
  - self-reported relief was higher after an instructed deep breath than before it
    - "Results show that self-reported relief following an instructed deep breath was higher than before." (Abstract)
  - physiological tension fell after a spontaneous sigh in people high in anxiety sensitivity and after a spontaneous breath hold in people low in it, and frontalis EMG is the one physiological recording the abstract names
    - "Continuously, participants rated relief and Frontalis electromyography was recorded." (Abstract)
    - "Physiological tension decreased following a spontaneous sigh in high anxiety sensitive persons and following a spontaneous breath hold in low anxiety sensitive persons." (Abstract)
- Caveat: Findings taken from the abstract. The version of record is paywalled. The instructed-breath result is the closest thing in this corpus to evidence that being told to take one deliberate breath, which is what exhale's reminder does, changes how a person feels. Relief was compared 5 s before and after the instructed breath, which came during safety or danger cues, and the abstract reports no relief comparison with the instructed breath hold.
  - "These results are the first to show that a deep breath relieves and, in anxiety sensitive persons, reduces physiological tension." (Abstract)
  - "Self-reported relief and physiological tension were compared 5s before and after instructed deep breaths and breath holds, and before and after spontaneous deep breaths and breath holds in the respective blocks." (Abstract)
  - "During the presentation of the safety and danger cues, an instruction was given to either 'take a deep breath' or 'postpone the next inhalation for 2 s (breath hold)." (Abstract)
- Quotes checked: 6 of 6 against the Europe PMC abstract, last on 2026-10-18

#### `yackle2017-breathing-arousal-neurons`

Yackle, Kevin; Schwarz, Lindsay A.; Kam, Kaiwen; Sorokin, Jordan M.; Huguenard, John R.; Feldman, Jack L.; Luo, Liqun; Krasnow, Mark A. (2017). *Breathing control center neurons that promote arousal in mice*. Science 355(6332): 1411-1415

- DOI: [10.1126/science.aai7984](https://doi.org/10.1126/science.aai7984)
- Open copy: <https://pmc.ncbi.nlm.nih.gov/articles/PMC5505554/> ([archived](https://web.archive.org/web/20261018201658/https://pmc.ncbi.nlm.nih.gov/articles/PMC5505554/))
- Verification: crossref-verified | Access: paywalled | Read: full text | evidence tier **D**
- Backs:
  - in mice, a small preBotzinger subpopulation projects directly to and positively regulates noradrenergic locus coeruleus neurons, which the authors propose links the breathing center to arousal
    - "We found a neuronal subpopulation in the mouse preBötzinger complex (preBötC), the primary breathing rhythm generator, which regulates the balance between calm and arousal behaviors. [...] These neurons project to, synapse on, and positively regulate noradrenergic neurons in the locus coeruleus, a brain center implicated in attention, arousal, and panic that projects throughout the brain." (Abstract)
    - "We have identified and characterized a new neuronal subtype in the preBötC comprising ~175 of its ~3000 neurons." (Main text, paragraph 14)
    - "Monosynaptic tracing demonstrated that these neurons directly project to and synapse on noradrenergic neurons in the contralateral LC, which in turn project throughout the brain and control generalized arousal and sleep-wake transitions (15)." (Main text, paragraph 14)
    - "We propose that Cdh9/Dbx1 preBötC neurons function as gateway neurons directly linking the preBötC to the locus coeruleus, and through it to the rest of the brain (Fig. 4G). This ascending circuit allows the respiratory center to communicate directly with and control higher order brain structures associated with behavioral arousal." (Main text, paragraph 15)
  - ablating those neurons on both sides, about 175 per side, left breathing intact but increased calm behaviors
    - "Conditional, bilateral genetic ablation of the ~175 Cdh9/Dbx1 double-positive preBötC neurons in adult mice left breathing intact but increased calm behaviors and decreased time in aroused states." (Abstract)
    - "Ablation of these neurons left all major breathing patterns and regulation intact, but made the mice preternaturally calm: their activity and ECoG profiles shifted from active exploratory behavior and brain wave patterns toward calm behaviors such as still sitting and grooming." (Main text, paragraph 14)
    - "Ablation reduced exploration of a new environment (87% ± 9% vs. 62% ± 20% time spent in active exploration, pre- vs. post-ablation; p=0.02) and increased time engaged in grooming three-fold (10% ± 7% vs. 31% ± 16%; p = 0.02) and still sitting twofold (3% ± 4% vs. 7% ± 7%; p = 0.07), although the latter did not reach statistical significance (Fig. 2C)." (Main text, paragraph 8)
    - "We focused on the ~175 neurons in each preBötC (fig. S1F–J; ~350 neurons bilaterally) that co-express the Dbx1 lineage marker, the major subpopulation (56%, 165 of 292 scored Cdh9+ cells, were Dbx1-LacZ+) we call Cdh9/Dbx1 neurons (Fig. 1G to I)." (Main text, paragraph 3)
- Caveat: Mouse work. The authors propose that the circuit ties arousal to breathing pattern, keeping the animal calm when breathing is slow and regular. The study doesn't test voluntarily slowed breathing. It's the best answer in this corpus to why slow breathing would change how anyone feels at all, and it's still an inference from mice. Don't cite it for a human effect.
  - "If the Cdh9/Dbx1 circuit is conserved in humans, it could provide a therapeutic target for breathing-related anxiety disorders and perhaps prevention of sudden infant death syndrome (SIDS), widely hypothesized to result from inadequate arousal response to asphyxiation during sleep (24)." (Main text, paragraph 16)
  - "This respiratory corollary signal would thus serve to coordinate the animal’s state of arousal with the breathing pattern, leaving the animal calm and relaxed when breathing is slow and regular, but promoting (or maintaining) arousal when breathing is rapid or disturbed." (Main text, paragraph 15)
  - "Our results show, conversely, that the breathing center has a direct and powerful influence on higher order brain function." (Main text, paragraph 17)
- Quotes checked: 11 of 11 against the PubMed Central author manuscript and the Europe PMC abstract, last on 2026-10-18

#### `yasuma2004-rsa`

Yasuma, Fumihiko; Hayano, Jun-ichiro. (2004). *Respiratory Sinus Arrhythmia: Why Does the Heartbeat Synchronize With Respiratory Rhythm?* Chest 125(2): 683-690

- DOI: [10.1378/chest.125.2.683](https://doi.org/10.1378/chest.125.2.683)
- Verification: crossref-verified | Access: paywalled | Read: abstract only | evidence tier **D**
- Backs:
  - heart rate rises on inhalation and falls on exhalation, and this respiratory sinus arrhythmia has been used as an index of cardiac vagal function
    - "Respiratory sinus arrhythmia (RSA) is heart rate variability in synchrony with respiration, by which the R-R interval on an ECG is shortened during inspiration and prolonged during expiration." (Abstract)
    - "Although RSA has been used as an index of cardiac vagal function, it is also a physiologic phenomenon reflecting respiratory-circulatory interactions universally observed among vertebrates." (Abstract)
- Caveat: Opinion and hypothesis article, filed by Chest under Opinions/Hypotheses. The abstract cites studies showing that RSA improves the efficiency of pulmonary gas exchange and proposes that it may do so through efficient ventilation/perfusion matching. It also notes evidence of a possible dissociation between RSA and vagal control of heart rate, a caution against reading RSA as a pure index of vagal tone. This is the physiological fact underneath every claim in this corpus that a longer exhale calms you, which is why the claim is intuitive and why meehan2024-longer-exhalations failing to find a ratio effect should be taken seriously: a real beat-to-beat mechanism doesn't guarantee a measurable session-level outcome.
  - "Previous studies have shown that the efficiency of pulmonary gas exchange is improved by RSA, suggesting that RSA may play an active physiologic role." (Abstract)
  - "Furthermore, evidence has accumulated of a possible dissociation between RSA and vagal control of that heart rate, suggesting differential controls between the respiratory modulation of cardiac vagal outflow and cardiac vagal tone." (Abstract)
  - "RSA or heart rate variability in synchrony with respiration is a biological phenomenon, which may have a positive influence on gas exchange at the level of the lung via efficient ventilation/perfusion matching." (Abstract)
- Quotes checked: 5 of 5 against the Europe PMC abstract, last on 2026-10-18

#### `zelano2016-nasal-respiration-limbic`

Zelano, Christina; Jiang, Heidi; Zhou, Guangyu; Arora, Nikita; Schuele, Stephan; Rosenow, Joshua; Gottfried, Jay A. (2016). *Nasal Respiration Entrains Human Limbic Oscillations and Modulates Cognitive Function*. The Journal of Neuroscience 36(49): 12448-12467

- DOI: [10.1523/JNEUROSCI.2586-16.2016](https://doi.org/10.1523/JNEUROSCI.2586-16.2016)
- Verification: crossref-verified | Access: open-access | Read: full text | evidence tier **D**
- Backs:
  - in epilepsy patients, nasal breathing entrained oscillations in piriform cortex and raised inspiration-locked delta power in amygdala and hippocampus, and in three patients these effects diminished when breathing moved to the mouth
    - "In this study, we collected intracranial EEG data from rare patients with medically intractable epilepsy, and found evidence for respiratory entrainment of local field potential activity in human piriform cortex, amygdala, and hippocampus. These effects diminished when breathing was diverted to the mouth, highlighting the importance of nasal airflow for generating respiratory oscillations." (Significance Statement)
    - "These data indicate that the passage of air through the nose is critical for respiratory entrainment of delta and theta oscillations in PC, amygdala, and hippocampus." (Results, Human respiratory oscillations depend on nasal airflow)
    - "By using iEEG techniques to measure oscillatory activity directly from the human brain, we show that electrical fluctuations in human PC are in phase with the natural cycle of breathing. This respiratory synchrony extends across both temporal and spatial scales, with higher-frequency oscillatory entrainment not only in PC, but also in amygdala and hippocampus." (Discussion)
    - "The relevance of nasal airflow for respiratory cortical entrainment was established in separate experiments where three patients breathed through either the nose or the mouth. Diversion from nasal to oral breathing led to a disorganization of limbic oscillatory synchrony in all three brain regions." (Discussion)
    - "More generally, the demonstration that respiratory phase-locked oscillations—and their coupling to higher-frequency rhythms—are driven by nasal inhalation suggests that this breathing route serves as a common “clock” to organize spatiotemporal excitability broadly throughout the brain" (Discussion)
    - "In amygdala ( Fig. 4 B ), inspiration-locked oscillatory power changes were similar to those in PC, with consistent increases in the delta range in all seven patients. [...] In the hippocampus ( Fig. 4 C ), all seven patients exhibited delta power increases following inspiration, though these effects survived FDR correction in only four patients (P1, P2, P5, and P7)." (Results, Respiratory phase entrains higher-frequency limbic oscillations)
- Caveat: Human intracranial recordings in a small epilepsy-surgery cohort of seven patients, plus behavioral experiments. The nose-versus-mouth recordings covered three of the seven, and only one of those had piriform coverage. Carried because it's the reason nose-versus-mouth is a real variable. exhale gives no nasal-breathing guidance at all, which is a defensible omission for a wordless overlay but should be a conscious one.
  - "Participants in the iEEG experiment included seven patients (three women) with temporal lobe epilepsy whose seizures were poorly controlled by medication (Table 1, demographic and clinical details)." (Materials and Methods, Study participants)
  - "Behavioral study participants included a total of 107 healthy human subjects between the ages of 18 and 30 years." (Materials and Methods, Study participants)
  - "Limited availability of epilepsy research patients for these studies means that sample sizes tend to be low." (Discussion)
  - "We also found that the route of breathing was critical to these effects, such that cognitive performance significantly declined during oral breathing." (Discussion)
  - "We examined iEEG data from three patients (P5, P6, and P7) who were asked to breathe naturally through either the nose or mouth in separate recording sessions. [...] Electrode coverage for all three patients included amygdala and hippocampus (only P7 had PC coverage)." (Results, Human respiratory oscillations depend on nasal airflow)
- Quotes checked: 11 of 11 against the PubMed Central author manuscript, last on 2026-10-18

---

## Limits, harms and adherence

3 sources.

#### `fincham2024-high-ventilation-rct`

Fincham, Guy W.; Epel, Elissa; Colasanti, Alessandro; Strauss, Clara; Cavanagh, Kate. (2024). *Effects of brief remote high ventilation breathwork with retention on mental health and wellbeing: a randomised placebo-controlled trial*. Scientific Reports 14(1): 16893

- DOI: [10.1038/s41598-024-64254-7](https://doi.org/10.1038/s41598-024-64254-7)
- Verification: crossref-verified | Access: open-access | Read: full text | evidence tier **A**
- Backs:
  - high-ventilation breathwork with retention is a distinct practice from slow-paced breathing and warrants a placebo-controlled evaluation of its own
    - "Over 75% of RCTs were primarily focused on slow breathwork, however preliminary data also suggest benefits of fast breathwork." (Introduction)
    - "HVBR (fast breathing with long breath holds) or a placebo HVBR comparator (15 breaths/min with short breath holds)" (Abstract)
    - "For instance, there are no placebo-controlled studies exploring effects of HVBR, making it difficult to establish whether interventions had specific active effects on mood beyond attention/expectation." (Introduction)
    - "The evidence-based picture surrounding HVBR is ambiguous, warranting a well-designed study as a tool to add credibility to an uncertain field of findings currently in its infancy." (Introduction)
  - short-term effects reported by participants in the high-ventilation arm included light-headedness, dizziness and tetany, with no lasting adverse effects reported
    - "The very small number of reasons for agreeing with the statement of experiencing negative, unpleasant/unwanted short-term effects in the intervention group included: light-headedness, dizziness, sleepiness, initial negative feelings of sadness (but which gradually improved), and tingly sensations paired with inability to use fingers (i.e., tetany)." (Results, Short-term tolerability)
    - "In the intervention group, 72 (79%) participants strongly disagreed with experiencing any lasting adverse effects from the breathwork, 12 (13%) slightly disagreed, 7 (8%) were neutral, and no one slightly nor strongly agreed." (Results, Long-term tolerability)
- Caveat: Pre-registered as NCT06064474, 200 healthy young adults, blinded against an active comparator. It's the largest trial of this technique to date. The primary finding is a null: high-ventilation breathwork didn't outperform the comparator on stress. The authors say it appears the breathing pace didn't matter in this brief, remotely delivered form. It's carried for the distinction it draws and for its tolerability data. It matters to exhale because the sliders can be set to fast, hold-heavy patterns that leave the slow-breathing evidence base entirely, and this trial records the short-term effects of one such pattern: 9% of each arm reported unpleasant short-term effects, light-headedness and dizziness came up in both arms, including the 15-a-minute comparator, and tetany only in the fast arm. The first author declares being a qualified Breath Teacher with The Breath-Body-Mind Foundation, and the trial was part-funded by PsyDAO, which the paper describes as supporting research on psychedelics and mental health. The authors say the comparator's connected breathing rhythm may itself have been an active ingredient, and they call their Prolific sample, all of whom had taken part in earlier studies, "a very specific and thus biased sample".
  - "This study (NCT06064474) was the largest blinded randomised-controlled trial on HVBR to date in which 200 young, healthy adults balanced for gender were randomly allocated in blocks of 2 by remote software to 3 weeks of 20 min daily HVBR" (Abstract)
  - "preregistered with ClinicalTrials.gov: NCT06064474 (03/10/2023)" (Materials and methods, Ethics approval)
  - "At post-intervention (primary timepoint) for stress level (primary outcome), we found no significant group × time interaction, F(1,180) = 1.98, p = 0.16, ηp2 = 0.01, d = 0.21" (Abstract)
  - "Our findings parsimoniously indicate that brief remote HVBR, in the form of the current study, was not more efficacious in lowering stress levels compared to the active breath-focused comparator, with this finding replicated for the secondary outcomes." (Discussion, Findings summarised)
  - "Only 9% in each group agreed with the statement of experiencing any unpleasant/unwanted short-term effects from the breathwork. A very small number of acute side effects were reported and, interestingly, they were similar in both groups." (Discussion, Findings contextualised)
  - "G.W.F. is a qualified Breath Teacher with The Breath-Body-Mind Foundation, New York." (Competing interests)
  - "Trial funded by Sasakawa Young Leaders Fellowship Fund (Sylff Association), and PsyDAO." (Funding)
  - "PsyDAO, a decentralised autonomous organisation supporting research at the intersection of psychedelics and mental health, provided funding for participant payments for a sample of 200" (Materials and methods, Consent to participants)
  - "Rate aside, it might have been this conscious connected pattern (inhale:exhale) which formed an active ingredient which has inadvertently been present in the control condition" (Limitations, Comparator)
  - "It should also be noted that participants were recruited via Prolific all of whom had been participants previously in research projects, a very specific and thus biased sample." (Limitations, Population)
  - "Though it appears that the breathing pace in the brief, remotely delivered HVBR here did not matter" (Discussion, Recommendations)
  - "In the comparator, reasons, all of which were directly attributable to the HVBR comparator, included: light-headedness, dizziness, stress in relation to the music, tension in chest, and the breathing pace being slightly too fast thereby causing some mild anxiety." (Results, Short-term tolerability)
- Quotes checked: 18 of 18 against the Europe PMC full text and the Europe PMC abstract, last on 2026-10-18

#### `linardon2020-app-attrition`

Linardon, Jake; Fuller-Tyszkiewicz, Matthew. (2020). *Attrition and adherence in smartphone-delivered interventions for mental health problems: A systematic and meta-analytic review*. Journal of Consulting and Clinical Psychology 88(1): 1-13

- DOI: [10.1037/ccp0000459](https://doi.org/10.1037/ccp0000459)
- Verification: crossref-verified | Access: paywalled | Read: abstract only | evidence tier **A**
- Backs:
  - dropout and non-adherence are common in trials of smartphone-delivered mental health interventions and may undermine the validity of their findings, and usage declines over the course of a trial
    - "Study attrition and low adherence are common, problematic, and may undermine the validity of findings in RCTs of smartphone-delivered interventions for mental health problems." (Abstract)
    - "Adherence rates were also suboptimal based on our qualitative synthesis; several participants failed to download the intervention, and intervention usage consistently declined over the course of the trial." (Abstract)
- Caveat: Findings taken from the abstract of a review of 70 trials: mean attrition was 24.1% at short-term and 35.5% at longer-term follow-up. In trials reporting both per-protocol and intention-to-treat analyses the effect sizes differed on average by only d = 0.18. Carried as a check on every other entry in this corpus: an effect measured in a supervised session says little about a tool someone installs and forgets. exhale has no telemetry and therefore no measure of whether anyone keeps it running, as the gaps ledger states.
  - "We examined attrition and adherence in 70 RCTs of smartphone interventions." (Abstract)
  - "The mean meta-analytic study attrition rate was 24.1% (95% CI [19.3, 29.6]) at short-term follow up and 35.5% (95% CI [26.7, 45.3]) at longer-term follow up." (Abstract)
  - "However, the mean difference in the between-groups effect size on primary outcomes in trials that reported both per protocol an intention-to-treat analyses was only Δd = 0.18." (Abstract)
- Quotes checked: 5 of 5 against the Europe PMC abstract, last on 2026-10-18

#### `szulczewski2019-antihyperventilation-instruction`

Szulczewski, Mikołaj Tytus. (2019). *An Anti-hyperventilation Instruction Decreases the Drop in End-tidal CO2 and Symptoms of Hyperventilation During Breathing at 0.1 Hz*. Applied Psychophysiology and Biofeedback 44(3): 247-256

- DOI: [10.1007/s10484-019-09438-y](https://doi.org/10.1007/s10484-019-09438-y)
- Updated by: correction [10.1007/s10484-019-09441-3](https://doi.org/10.1007/s10484-019-09441-3), 2019-05-22
- Verification: crossref-verified | Access: open-access | Read: full text | evidence tier **C**
- Backs:
  - a two-sentence instruction cut the end-tidal CO2 drop during 6-per-minute paced breathing from 5.21 to 2.7 mmHg
    - "The results showed that paced breathing without instruction decreased PetCO2 by 5.21 mmHg and that the use of the anti-hyperventilation instruction reduced the drop in PetCO2 to 2.7 mmHg." (Abstract)
    - "The most widely studied and used paced breathing is at frequencies around 0.1 Hz (six breaths per minute)" (Introduction)
    - "Next, participants received either the anti-hyperventilation instruction or no instruction (in the control group) and performed a 10-minute-long paced breathing task at 0.1 Hz. Breathing was guided by a sound cue, with inhalation lasting four seconds and exhalation lasting six seconds." (Method, Procedure)
    - "In order to avoid hyperventilation during the paced breathing task, please avoid excessively deep breathing. Breathe shallowly and naturally." (Method, Anti-hyperventilation instruction)
  - hyperventilation symptoms rose 0.63 points on a 7-point scale without the instruction and didn't rise significantly with it
    - "In the control group, symptoms increased on average by 0.63 points (SD = 0.79) on a 7 point likert scale. In the group with the anti-hyperventilation instruction, there were no significant changes in symptoms of hyperventilation." (Discussion)
  - the instruction used was to avoid excessively deep breathing and to breathe shallowly and naturally
    - "The instruction asked participants to avoid excessively deep breathing and to breathe shallowly and naturally." (Abstract)
- Caveat: Randomized, two groups, n = 46 aged 19-26, single session, paced at 4 s in and 6 s out. This is the mitigation for gap 5 and it's unusually cheap: a two-sentence instruction roughly halves the drop, cutting it from 5.21 to 2.7 mmHg, while both groups breathed at about 7 a minute against the 6-a-minute pacer (6.88 and 7.04). The study measured end-tidal CO2 and respiratory rate and no breath depth, and the authors attribute the over-breathing to breathing deeper than necessary. 36.4% of the instructed group had end-tidal CO2 below 30 mmHg against 43.5% of controls, with no count given for baseline. Baseline end-tidal CO2 averaged about 36 mmHg in both groups. exhale paces rate and says nothing about depth, so this is the one instruction in the corpus with a safety rationale for appearing in the app.
  - "Forty-six participants were randomly assigned to one of two groups: a group given an anti-hyperventilation instruction and a control group without such an instruction." (Abstract)
  - "The age of the participants ranged from 19 to 26 (M = 21.88, SD =1.71)." (Method, Participants and design)
  - "After baseline measurement, participants performed a 10-minute-long paced breathing task." (Method, Participants and design)
  - "Indeed, slow breathing has to be deeper because it requires an increase in tidal volume to compensate for the decrease in respiratory rate. However, because overcompensation by breathing deeper than necessary often occurs, emphasis should not be put on deepening of breathing as a key element of the slow breathing exercise." (Discussion)
  - "Present and previous findings indicate that merely slowing down respiratory rate is not an effective method of decreasing hyperventilation and that, during paced breathing exercises, attention should be paid to achieving adequate ventilation." (Discussion)
  - "In the group with the instruction, 36.4% of participants had PetCO2 below 30 mmHg, in comparison to 43.5% in the control group." (Discussion)
  - "Instruction Control PetCO2 (mmHg) Baseline 35.66 (5.33) 36.17 (4.49)" (Table 1)
  - "Respiratory rate (breaths/minute) Baseline 16.76 (3.41) 15.44 (4.38) Paced breathing 6.88 (0.65) 7.04 (0.66)" (Table 1)
  - "PetCO2 and respiratory rate were measured with an infrared sidestream capnometer (Capnocheck Plus, model 9004-000, BCI International, Waukesha, WI, USA) by nasal cannula." (Method, Apparatus and materials)
- Quotes checked: 15 of 15 against the Europe PMC full text and the Europe PMC abstract, last on 2026-10-18

---

## Gaps and unsupported choices

This section is the point of the exercise: everything below is a place where exhale
ships something the literature doesn't settle, or where the evidence is thinner or more divided
than a bare citation would suggest. Nothing here is a reason not to use the app. It's a list of
things that are currently believed without being known.

### 1. What is actually measured about breathing at a screen

The relevant literature is old and small, and it's filed under ergonomics, outside breathwork.

- [`schleifer1994-vdt-petco2`](#schleifer1994-vdt-petco2): eleven data-entry operators, monitored
  continuously across three consecutive six-hour work days. During computer work, end-tidal CO2 was
  significantly **lower** and respiration frequency significantly **higher** than during either
  baseline relaxation or progressive muscle relaxation.
- [`schleifer2008-emg-gaps-computer-work`](#schleifer2008-emg-gaps-computer-work): the same group,
  fourteen years later, n = 23. Lower end-tidal CO2 under high mental workload during computer data
  entry, tracking reduced trapezius EMG gaps.
- [`schleifer2002-hyperventilation-job-stress`](#schleifer2002-hyperventilation-job-stress): the
  theory paper tying it together, which states that hyperventilation "is often characterized by a
  shift from a diaphragmatic to a thoracic breathing pattern," recruiting sternocleidomastoid,
  scalene and trapezius.

That diaphragmatic-to-thoracic shift is what people mean by *shallow*: chest breathing instead of
belly breathing. It's a theory of mechanism, and no study in this corpus measures chest against
diaphragm breathing in screen users.

Two things limit how much the keyboard can be blamed. The 1994 study compares work with
relaxation and the 2008 study compares high with low mental workload during data entry, and [`grassmann2016-cognitive-load-respiration`](#grassmann2016-cognitive-load-respiration)
finds the same faster, over-ventilated pattern across cognitive tasks in general, though it may vary
with the kind of demand, so the keyboard is where
the effect was measured and hasn't been shown to be its cause. And the two samples total 34 people
from one research group.

A second, independent line runs through posture.
[`jung2016-smartphone-posture-respiration`](#jung2016-smartphone-posture-respiration) found that
people using smartphones more than four hours a day had lower peak expiratory flow than lighter
users. It also reports a worse craniovertebral angle, but the means and standard deviations in its
own table don't reach significance (p of about 0.33 by its stated t-test). In
[`deniz2024-forward-head-lung-volumes`](#deniz2024-forward-head-lung-volumes), three of four
comparison studies found forward head posture lowered FVC, with reductions of 0.25 to 0.81 L across
115 participants.

**What the evidence doesn't support** is *shallow* meaning a reduced *volume of air moved*.
[`grassmann2016-cognitive-load-respiration`](#grassmann2016-cognitive-load-respiration), 54
experiments, finds that respiratory amplitude appears to stay roughly stable and minute ventilation
goes **up** under cognitive load, though its tidal-volume results are inconsistent and tasks with
performance feedback mostly lowered tidal volume. Both things can hold at once: more air per minute, on the theory above moved by
the wrong muscles, from a slumped posture associated with less capacity.

The defensible statement, and the one the README makes, is therefore that during demanding work at
a keyboard people breathe **faster and slightly over-ventilated**, from a posture associated with
reduced lung volumes, and that the shift to chest breathing, which no study measures, is the theory
of why.

It isn't the same claim as "screen apnea" or "email apnea," meaning outright breath-*holding* at a
screen. That framing traces to unpublished observations by Linda Stone from 2007, tested informally
on acquaintances without a protocol, published data or replication. Nothing found in this pass
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
corpus. "Inside the range that has been tested" is a statement about coverage and says nothing
about which rate is best.

**Resonance frequency is individual.** [`lehrer2014-hrv-biofeedback`](#lehrer2014-hrv-biofeedback)
puts the average at about 5.5 breaths per minute and is explicit that it varies from person to
person. [`lin2014-equal-ratio-hrv`](#lin2014-equal-ratio-hrv) found 5.5 bpm with an equal ratio
outperformed the 6 bpm variants tested. A single
shipped number can't be right for everyone, and finding a person's own resonance frequency takes an
assessment protocol and a sensor, neither of which exhale has. The default is a reasonable starting
point and isn't personalized.

Anyone who already has exhale installed keeps whatever they had: the timing fields carry no
`#[serde(default)]`, so an existing `settings.toml` is untouched and the change reaches only fresh
installs and Reset to Defaults. The previous default, `5` / `0` / `10` / `0` at 4 a minute, is still
offered as a one-click preset. No primary study in this corpus has measured 4 a minute, which is why
it's no longer what a new user gets without asking. The one report is secondhand:
[`lehrer2022-my-life-hrvb`](#lehrer2022-my-life-hrvb) recalls Song and Lehrer 2003 pacing people from 3
to 14 a minute and finding RSA amplitude highest at 4 and 6, with a decrease at 3. That paper isn't in
the corpus, and RSA amplitude, a heart-rate measure, doesn't record how anyone felt.

### 3. Box breathing is slower than it looks

`4` / `4` / `4` / `4` is a 16-second cycle, or **3.75 breaths per minute**: *slower* than exhale's
default and further below the tested band. The holds hide the rate, which is why
the settings panel computes it.

Box breathing has also been tested against alternatives twice, head-to-head in one of them, and
didn't win either time:

- [`marchant2025-square-478-six`](#marchant2025-square-478-six), n = 84, compared square breathing,
  4-7-8 breathing, and 6 breaths per minute at two ratios. Breathing at 6 raised HRV **more than
  either square or 4-7-8**, with small to medium effects. The paper opens by stating that square and
  4-7-8 "are popularly promoted by psychotherapists but have little empirical support."
- [`balban2023-cyclic-sighing`](#balban2023-cyclic-sighing), a randomized trial registered after
  the fact, tested box breathing, cyclic sighing and cyclic hyperventilation over a month against a
  mindfulness-meditation control. Cyclic sighing, the exhale-emphasizing arm, separated from the
  control on positive affect, and box breathing didn't. The box arm had 21 people, the arms weren't
  tested against each other, and their daily gains were 1.84 and 1.89 points, so this is a
  difference in reaching significance and doesn't demonstrate a gap.

The same evidence applies to 4-7-8, sometimes attributed to Andrew Weil: it lost in Marchant, and at
4+7+8 = 19 s it's 3.16 breaths per minute, slower still.

The pattern with the best direct support, and exhale's default, is `5` / `0` / `5` / `0`: 6 breaths
per minute in a 10-second cycle with no holds. It sits inside the tested band at the rate that won
in Marchant, and for a beginner, holding the breath is harder than breathing slowly. One
qualification: Marchant ran 6 a minute at two ratios, and on RMSSD the 4:6 ratio carried the
significant differences over the hold patterns while the 5:5 ratio's edge (0.19 and 0.18 SD)
wasn't significant after Scheffé correction (p = 0.10 and 0.15). The two ratios didn't differ
significantly from each other (p = 0.07), so 6 a minute is what won, and the choice of 5:5 over 4:6
within it rests on gap 4, where the ratio evidence splits. Box breathing remains a reasonable thing
to want, and exhale offers it as a preset, though the evidence points elsewhere.

### 4. Longer exhale: contested on the heart, thin on how people feel

On HRV, six results, and they don't line up:

| Source | n | Design | Result on ratio |
|---|---|---|---|
| [`bae2021-exhalation-inhalation-ratio`](#bae2021-exhalation-inhalation-ratio) | 28 | 2:1 vs 1:1 cue at spontaneous rate | Longer exhale raised RMSSD and HF-HRV |
| [`vandiest2014-ie-ratio-relaxation`](#vandiest2014-ie-ratio-relaxation) | 23 (30 recruited) | i/e 0.42 vs 2.33 targets (0.49 vs 1.44 achieved), at 6 and 12 bpm (7.3 to 7.7 achieved at the slow rate) | Longer exhale raised HF-HRV, but only at the slow rate |
| [`lin2014-equal-ratio-hrv`](#lin2014-equal-ratio-hrv) | 47 | 5:5 vs 4:6 at 5.5 and 6 bpm | **Equal** ratio at 5.5 bpm won on SDNN and LF |
| [`laborde2021-ie-ratio-pauses`](#laborde2021-ie-ratio-pauses) | 64 | i/e 0.8, 1.0, 1.2 at 6 bpm, with and without 0.4 s pauses | Longer exhale raised RMSSD, and pauses changed nothing |
| [`meehan2024-longer-exhalations`](#meehan2024-longer-exhalations) | 26 + 16 replication | 1:1 vs 1:2 at 6 bpm | No difference, in the original *and* the replication |
| [`marchant2025-square-478-six`](#marchant2025-square-478-six) | 84 | 5:5 vs 4:6 at 6 bpm | No significant ratio difference on RMSSD (4:6 ahead by 0.20 SD, p = 0.07), LF-HRV or end-tidal CO2 |

The count inside this corpus is three for, one against and two nulls.
[`meehan2024-longer-exhalations`](#meehan2024-longer-exhalations)'s introduction tallies the older
literature beyond this corpus as three further nulls, one of which didn't measure HRV, one further
result favoring the longer exhale and one favoring the longer inhale. No mechanism claim
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

This is a tension between two entries that nobody has looked at.

[`marchant2025-square-478-six`](#marchant2025-square-478-six) reports, as an unexpected finding, that
breathing at 6 breaths per minute produced **mild over-breathing**: HRV went up and end-tidal CO2
went down, into the 30 to 35 mmHg range.
[`vandiest2014-ie-ratio-relaxation`](#vandiest2014-ie-ratio-relaxation) reports the same from the other
direction: its uncalibrated tidal-volume signal more than doubled from baseline in its slow
conditions, and the authors suspect their participants were hyperventilating. Meanwhile [`schleifer1994-vdt-petco2`](#schleifer1994-vdt-petco2) and
[`schleifer2008-emg-gaps-computer-work`](#schleifer2008-emg-gaps-computer-work) show that a person at
a keyboard *already* runs end-tidal CO2 lower than at rest: one to two mmHg lower in the 1994
study, a change its authors call relatively small and a hyperventilatory stress effect, and lower
again under higher mental workload in the 2008 study, so over-breathing relative to rest, short of
hypocapnia.

exhale paces rate and says nothing about depth. A user who slows to 6 breaths per minute while taking
large breaths moves more air per minute. No study in this corpus tests a slow pacer on exhale's
user, a screen worker who is already over-breathing.

This isn't a safety warning: the effect Marchant reports is mild and was measured in a single
session. It's recorded because it's the most interesting unanswered question this corpus turned up,
and because an app that paces breathing should know that pacing rate isn't the same as pacing
volume. The one mitigation with evidence behind it is an instruction:
[`szulczewski2019-antihyperventilation-instruction`](#szulczewski2019-antihyperventilation-instruction)
shows a two-sentence anti-hyperventilation instruction cuts the end-tidal CO2 drop from 5.21 mmHg to
2.7, leaving roughly half of it.

### 6. `drift` is an invention of this app

`drift` lengthens every cycle by a fixed percentage, compounding, so the breath extends gradually
across a session. Graded extension of the breath fits the pranayama tradition broadly, though the
contents of [`satyananda1999-apmb`](#satyananda1999-apmb) haven't been read to confirm it, and no
study in this corpus examines a progressively lengthening pace at all.
[`szulczewski2019-training-relaxation`](#szulczewski2019-training-relaxation)
trained at a fixed rate and found the drop in unpleasant arousal during each session grew across a
week of practice, which supports continued practice and says nothing about slowing down further
within a session.

Nothing in this review contradicts the tradition either. The primary rate-comparison studies in this
corpus stop below about 5 breaths a minute and don't report worse outcomes there.
[`joshi1992-pranayam-training`](#joshi1992-pranayam-training)'s protocol spent 16 of its 20 minutes at 2
a minute, but it compared no rates, and its before-and-after results, which include a fall in FEV1%
among the men, cover the whole protocol. The one report from slower pacing is
secondhand and about HRV amplitude: [`lehrer2022-my-life-hrvb`](#lehrer2022-my-life-hrvb) recalls RSA
amplitude lower at 3 a minute than at 4 and 6, and induced HRV amplitude at a minimum around 2 to 3.
Both are the same kind of measure as the inverted-U below, and neither says anything about relaxation
or comfort. What subjective evidence exists points the tradition's way:
[`vandiest2014-ie-ratio-relaxation`](#vandiest2014-ie-ratio-relaxation) found the longer exhale
produced more relaxation, stress reduction, mindfulness and positive energy, and
[`lin2014-equal-ratio-hrv`](#lin2014-equal-ratio-hrv) found every slow pattern beat baseline on
relaxation. Objective measures point the same way:
[`joshi1992-pranayam-training`](#joshi1992-pranayam-training) reports that six weeks of practice
lowered respiratory rate, though its printed means barely moved, and lengthened breath-holding time.
The resonance peak in
[`shaffer2020-resonance-frequency-assessment`](#shaffer2020-resonance-frequency-assessment) is worth
reading alongside these, but it describes a peak in **HRV amplitude**, and doesn't transfer to a
peak in relaxation or comfort.

The one documented caution is about breathing **depth**, and it says nothing against a slower rate
(see gap 5).

`drift` is therefore unbounded and **defaults to 0, off**. Off by default is a coverage argument,
with no claim of harm behind it: on by default it would move every new user out of the region
anyone has measured, within minutes, without asking. It's unbounded because a 10-second inhale with
a 20-second exhale is unremarkable in pranayama, and absence of research isn't evidence of harm.

The stepper moves in 0.1 percentage points, and values below that can be typed. Display is capped at
three decimals, so 0.001 % is the finest value the field round-trips. Compounding is steep enough
that whole percents are unusable: from the 10 s default, 1 % doubles the breath in about 17
minutes, 0.1 % in about 2.8 hours, 0.01 % in about 28. The settings panel reports the doubling point in
**breaths**, because cycle `k` lasts `c · dᵏ` and so `dᵏ = 2` at
`k = ln2 / ln d`, with the starting cycle length canceling out: 1 % is 70 breaths from any starting
pace, 0.1 % is 693, 0.001 % is 69,315. A doubling time would depend on where the user started and
would disagree with the minute figures quoted above, which are anchored to the 10 s default.

### 7. Randomized timing has no literature behind it either

The four randomization sliders inject per-phase jitter. Every pacing study in this corpus uses a
fixed rate, which is what *paced* means. The nearest adjacent literature is
[`vlemincx2013-sigh-reset-model`](#vlemincx2013-sigh-reset-model), on natural respiratory
variability and sighs, which is about spontaneous breathing, while the sliders destabilize a
pacer. Defaults are 0, which is the right default. Treat the sliders as an
aesthetic option.

### 8. Nothing about exhale itself has ever been measured

No study in this corpus is about exhale. The closest published analogue is
[`moraveji2011-peripheral-paced-respiration`](#moraveji2011-peripheral-paced-respiration): a
translucent animated bar across the screen, running in the periphery during real information work,
which significantly lowered participants' breathing rate. Two things about it differ from exhale:
it was sensor-triggered and intermittent, running in two-minute bursts when the wearer's breathing
rose 20 percent above their resting rate or at least once every six minutes, and its target was set
per person at 20 percent below an eyes-closed resting rate that averaged 9.3 a minute, so near 7.5,
compared with exhale's 6. On average they didn't get there: breathing averaged 15 a minute while the
pacer was on. n = 13, and the drop was 1.9 breaths a minute. Its limitation is the one that matters
here: the reduction happened **while the pacing was active** and didn't persist as a lasting change
in respiratory pattern. An always-on overlay should be understood as an effect that lasts as long as
it's on.

The strongest design warrant is [`tabor2022-guided-breathing-design`](#tabor2022-guided-breathing-design):
an expanding and contracting circle at 6 breaths/min matched sensor-driven HRV biofeedback on
breathing rate and LF power, and its HRV amplitude gain was larger (interaction p = 0.03) though the
authors call the two comparable, with effects appearing in about two minutes and no hardware needed.
That's exhale's Circle mode, and it's why exhale needs no sensor, account or telemetry, though the
study is still n = 28 in one session.

### 9. Visual-only guidance is the weaker modality for the outcome users care about

[`wongsuphasawat2012-cant-force-calm`](#wongsuphasawat2012-cant-force-calm) found auditory pacing
was rated more calming than visual, while the visual guide's larger change in breathing rate was
only a non-significant trend (p = 0.26). Its visual arm was a biofeedback display of the user's own
breathing, which exhale isn't.
exhale is visual-only by design, because it's meant to sit silently in the corner of a working
screen, and that's a trade-off against felt calm. The source is a two-page adjunct paper with 14
participants, so it's a signal, short of a result.

### 10. exhale cites blink research and does nothing about blinking

The blink-rate literature is well supported, and exhale doesn't act on it. The overlay paces
breathing without prompting or detecting blinks, and it implements nothing from the digital eye
strain literature. The blink finding is context for why screens deserve a nudge.

Also relevant to exhale's whole genre: [`johnson2023-20-20-20`](#johnson2023-20-20-20) found that
scheduled 20-second breaks at any of three intervals produced no significant effect on symptoms,
reading speed or accuracy. A periodic on-screen nudge isn't effective merely because it's popular.

### 11. Both hold sliders default to 0: brief pauses are neutral and longer holds are untested

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
In [`linardon2020-app-attrition`](#linardon2020-app-attrition), across 70
trials of smartphone-delivered mental health interventions, attrition averaged 24.1% at short-term
and 35.5% at longer-term follow-up, usage consistently declined over the course of the trials, and
the authors say this may undermine the validity of the findings. Nearly every effect size in this
corpus was measured in a supervised session with a compliant participant.
[`balban2023-cyclic-sighing`](#balban2023-cyclic-sighing)'s month-long trial is the exception, run
remotely. Study participants, supervised or remote, are a different population from someone
who installed a menu-bar app in March.

### 13. Adverse-event reporting in this field is thin

[`fincham2023-breathwork-meta`](#fincham2023-breathwork-meta) found that only four of its twelve
primary-outcome trials reported on adverse events at all, none attributing lasting harm to
breathwork, and one fast-paced trial reported transient hot flushes, shortness of breath or sweating
in a small subgroup. [`laborde2022-vsb-meta`](#laborde2022-vsb-meta) remarks that "very few adverse
effects are reported" in the *slow* breathing studies it reviewed, though it didn't collect
adverse-event data, and slow breathing is the mode exhale is built around. exhale's sliders can also
be set to fast, hold-heavy patterns that leave that evidence base entirely. Those belong to the
high-ventilation literature ([`fincham2024-high-ventilation-rct`](#fincham2024-high-ventilation-rct)),
where transient tetany is documented in the fast arm, and light-headedness in both the fast arm and
its slower comparison arm. This is the basis for the
README's advice to take breaks if intense feelings arise.

### 14. The tradition sources are lineage rather than evidence, and are tiered accordingly

exhale's four-phase structure, inhale / retention / exhale / retention, is pranayama. It didn't come
from psychophysiology, and the corpus says so: [`satyananda1999-apmb`](#satyananda1999-apmb) and
[`muktibodhananda1998-hatha-yoga-pradipika`](#muktibodhananda1998-hatha-yoga-pradipika) are carried
at tier **E**, meaning they may be cited for where a practice came from and never for whether it
works.

They're included, without endorsement, for two reasons. Retrofitting a 2020s HRV
citation onto an instruction that's centuries older would be revisionist about the app's
design history. And the "longer exhale" idea in modern breathing apps predates any laboratory study
of it, since pranayama traditions taught longer-exhale ratios centuries earlier, which is worth
weighing when reading the studies that later tested it.

Both entries are catalog records only: checked against Open Library, contents not consulted, and
no claim about whether a practice works rests on them. The Satyananda entry cites the 1999 third
revised edition its ISBN resolves to. A 2008 fourth revised edition exists under the same imprint.

## What would close the biggest gaps

In rough order of value per unit effort:

1. Read in full the twelve entries cited from their abstract only:
   [`bae2021-exhalation-inhalation-ratio`](#bae2021-exhalation-inhalation-ratio),
   [`bernardi2001-slow-breathing-chemoreflex`](#bernardi2001-slow-breathing-chemoreflex),
   [`chaddha2019-slow-breathing-bp`](#chaddha2019-slow-breathing-bp),
   [`johnson2023-20-20-20`](#johnson2023-20-20-20),
   [`lin2014-equal-ratio-hrv`](#lin2014-equal-ratio-hrv),
   [`linardon2020-app-attrition`](#linardon2020-app-attrition),
   [`schleifer2002-hyperventilation-job-stress`](#schleifer2002-hyperventilation-job-stress),
   [`schleifer2008-emg-gaps-computer-work`](#schleifer2008-emg-gaps-computer-work),
   [`vlemincx2013-sigh-reset-model`](#vlemincx2013-sigh-reset-model),
   [`vlemincx2016-sigh-relief`](#vlemincx2016-sigh-relief), [`yasuma2004-rsa`](#yasuma2004-rsa) and
   [`you2023-respiratory-frequency`](#you2023-respiratory-frequency). Their full texts sit behind
   paywalls or captchas.
2. Nobody has tested a slow visual pacer on a screen worker who is already over-breathing (gap 5).
   The question is publishable, and exhale is unusually well placed to ask it.
3. Settle whether graded extension does anything, which would put a floor under gap 6. No study in
   this corpus varies the pace *within* a session, so the question is open in both directions.
4. Resonance frequency is individual (gap 2) and exhale ships one number for everyone. Whether a
   sensorless app can help someone find their own, by any method better than trying a few and
   noticing, is unresolved and would matter more than the default ever will.
