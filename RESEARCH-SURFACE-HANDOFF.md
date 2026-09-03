# Handoff: the in-app research surface (phases 2-5)

Brief for an agent picking this up cold. Read this before touching the settings window.

**Status:** phases 0 through 4 are done and are on `feat/provenance-and-drift-enhancements`.
Phase 5 is deliberately deferred, and phase 4 shipping the way it did makes the case for deferring it
stronger rather than weaker: see the phase 5 note.

What phases 2 and 3 actually shipped is recorded under each phase heading below. Two things were
found while building them that are not in the design: the README's source counts had been wrong
since the corpus grew (42/40/2 against an actual 48/45/2/1), and the gaps ledger still described a
drift step of 0.01 percentage points when the code shipped 0.1. Both are fixed, and both now have
`--check` gates so they fail the build rather than rotting again.

The design below is the verdict of a four-specialist review (UX, full-stack, egui/frontend, and a
psychophysiology professor) that argued to convergence. Where they disagreed, the resolution and the
reason are recorded. **Do not re-litigate the settled parts without new evidence**; do challenge them
with new evidence.

---

## 1. What already exists

| Thing | Where |
|---|---|
| 48-source corpus | [`docs/CITATIONS.csl.json`](docs/CITATIONS.csl.json) |
| Hand-written prose + 14-item gaps ledger | [`docs/citations-notes.md`](docs/citations-notes.md) |
| Generated render | [`docs/CITATIONS.md`](docs/CITATIONS.md) |
| Generator, stdlib only, `--check` mode | [`scripts/generate-citations.py`](scripts/generate-citations.py) |
| CI: `citations` + `core-check` | [`.github/workflows/test.yml`](.github/workflows/test.yml) |
| Tray deep link + `platform::open_url` | [`tray.rs`](rust/crates/exhale-app/src/tray.rs), [`platform.rs`](rust/crates/exhale-app/src/platform.rs) |
| Pacing arithmetic and the copy derived from it | [`pacing.rs`](rust/crates/exhale-core/src/pacing.rs) |
| Preset patterns and derived selection | [`presets.rs`](rust/crates/exhale-core/src/presets.rs) |
| Store copy under denylist | [`snap/snapcraft.yaml`](snap/snapcraft.yaml), [`rust/packaging/windows/store-listing.md`](rust/packaging/windows/store-listing.md) |

~~**The binary has no research surface at all.**~~ It has two, as of phases 2 and 3. `platform::open_url`
now exists with three cfg'd implementations and an `https://`-only allowlist.

---

## 2. Verified facts. Do not re-derive these.

Each was checked against the code or a live API. Several contradict what you would reasonably assume.

**egui and rendering**
- egui **0.29.1** (+ `egui-winit`, `egui-wgpu` 0.29.1), wgpu 22, winit 0.30 via a **vendored fork** in
  `vendor/winit` that stubs `Window::set_blur`, because Apple rejected the private
  `_CGSSetWindowBackgroundBlurRadius` symbol. `lto = true` exists to strip it. App Review has already
  bitten this repo once.
- **`egui::Modal` does not exist in 0.29** (landed in 0.31). A modal means `egui::Window` plus
  hand-rolled dimming; there is precedent at `settings_window.rs:1436` (shortcut capture).
- **`ui.hyperlink_to` is doubly broken here.** `theme.rs:108` sets `override_text_color`, which bakes
  body-text colour into the galley so links render as ordinary text; and the macOS
  `handle_platform_output` bypass at `settings_window.rs:610-647` drops `open_url` on the floor. Use
  `ui.link()` plus an explicit `platform::open_url()`.
- **A second egui window would roughly double RSS.** `theme.rs:38` reads `/System/Library/Fonts/SFNS.ttf`
  (**8,330,740 bytes**, measured) into `FontData::from_owned`, resident for the life of the
  `egui::Context`. The release binary is ~7.7 MB. A second context is a second 8.3 MB blob plus a
  second atlas. **Option "separate research window" is dead.**
- **Tooltips cannot carry caveats.** `area.rs:429` clamps tooltip width to `ctx.screen_rect()`, which
  here is the 360 pt window, so a 350-character caveat truncates. `SETTINGS_WIDTH` is a
  `const 360` at `settings_window.rs:195`; only height is user-resizable, which is why long text is
  cheap (wrap width never changes).
- Card content width is `360 - 2*OUTER_PAD(14) - 2*CARD_PAD(12) = 308 pt`. `INITIAL_PREFERRED_H` is
  796 (`settings_window.rs:222`) and the comment at `:208` says Timers is deliberately just below the
  fold. **Adding a section may push existing content below the fold. Measure.**
- The full corpus rendered in-app would be **~11,000 pt, about eleven screens**, and would flip
  `set_max_inner_size` from ~900 to ~11,000 in one frame, fighting the `last_max_height` cache.

**Dependencies**
- `webbrowser` 1.2.1 is already in the graph via `egui-winit`'s default `links` feature. Linking out
  costs **zero new dependencies**. (It uses `LSCopyDefaultApplicationURLForURL` on macOS, not a
  subprocess; an earlier claim that it shells out to `open(1)` was wrong.)
- **`serde_json` is NOT in the graph.** Runtime-parsing the corpus would add a real dependency;
  having the Python generator emit a `.rs` file needs no Rust-side JSON parser at all.
- `windows-sys` already enables `Win32_UI_Shell`, so `ShellExecuteW` is reachable today.
  `objc2-app-kit` is already a direct dep; `NSWorkspace openURL:` needs only the `"NSWorkspace"`
  feature added, is public and unentitled, and **MAS entitlements need no change**
  (`bundle-mas.sh:208-217` ships only app-sandbox + user-selected read-only). Linux: `xdg-open`, and
  `snapcraft.yaml` already plugs `desktop`.
- `build.rs` already reaches three levels above the workspace into `swift/.../Assets.xcassets` and
  degrades to `cargo:warning` when the file is missing. All three store pipelines build from a full
  repo checkout.

**Accessibility. This is a hard constraint, not an aspiration.**
- `grep -c 'name = "accesskit' Cargo.lock` returns **0**. VoiceOver, Narrator and Orca see a blank
  window. **Every existing control is already invisible to screen readers.**
- Enabling AccessKit would not fix macOS on its own: tree updates are delivered through the very
  `handle_platform_output` call the vibrancy/RefCell workaround bypasses.
- Consequence: **tray menu items are native and ARE exposed to screen readers.** The tray link is
  therefore the only evidentiary string a blind user can perceive, which promotes phase 2 from
  nice-to-have to ship-blocker. Nothing rendered inside the egui panel may be caveat-load-bearing
  until AccessKit lands.

**Measurement**
- `rust/crates/exhale-render/examples/cpu_bench.rs` **cannot** measure any of this: it is an
  `exhale-render` example with no egui and no window. Use the README's own protocol instead
  (`ps -o %cpu`, 30 s, 15 samples, normalised to one core), in three conditions: settings closed,
  open+collapsed, open+expanded. The README's 3.19 % figure was taken with the settings window
  **closed**, so nothing here can regress it.

---

## 3. The design, and why

**The binary asserts no English sentences about the literature.** It ships numbers, one range, and
citekeys as identifiers. This is the load-bearing decision and it was reached the hard way.

The full-stack agent initially argued for quoting `custom.backsClaims` verbatim (never paraphrase,
always generate). That is right about drift-prevention and unusable in practice: the actual strings
read *"raised cardiac vagal activity"*, *"anxiety reduction"*, *"perceived stress"*. The professor
blocklists exactly that vocabulary from a store-reviewed binary, and the UX agent independently
banned it. Both were right; their conclusions were incompatible; so the app ships **derived arithmetic
instead of quoted prose**. Arithmetic cannot be retracted.

Rules that follow:
- **Rates are never stored.** Compute from live settings via `Settings::breaths_per_min()` and
  `breaths_per_min_after(minutes)`. True by construction, for slider users as well as preset users.
  A cached rate in a preset is false the moment `drift` is non-zero.
- **Chips are labelled by pattern, never by rate** (`5 s in, 5 s out`, not `6 a minute`).
- **No badges.** A badge slot is positionally a verdict whatever words are in it. "Most tested" was
  rejected on facts too: `lin2014` found 5.5 beat 6, and `you2023` singled out no rate. Use a
  sentence with an explicit referent: *"Rates from 5 to 7 a minute are the ones tested directly.
  This is one of them."*
- **No evidence-tier letters in the UI.** Tiers grade sources and license citation behaviour; a
  preset is not a sentence. Stamping `B` on a button drops the scope conditions that were the
  condition of the licence.
- **Brand names move out of labels into captions.** `5 a minute, short pause`, with "Sometimes called
  A52" in the caption. Removing a name is not removing an option.
- **Blocklisted from the binary entirely:** `chaddha2019` (blood pressure), `fincham2023` (anxiety /
  depression effect sizes), `balban2023` (NCT-registered mood trial), `little2025` (anxiety /
  perceived stress). Note `fincham2023` is the strongest warrant in the corpus and still cannot be
  used in-app. That cost is real; take it anyway.

---

## 4. Phases

### Phase 2: tray link. Ship-blocker. DONE, commit `cfbb180`.

**Shipped, and then revised twice on the user's call.**

`RESEARCH_URL` and its label are pinned together in `tray.rs` because the wording and the anchor are
one decision; the item sits directly under Preferences; the handler is inline in the `MenuEvent` loop
rather than routed through an `AppEvent`, since opening a URL touches no app state and
`about_to_wait` is already on the main thread, which is where `NSWorkspace` has to be called from.

**The label is now just "Research".** The design argued for wording that leaned toward the ledger
("Research, and what it doesn't support"); the user asked for the plain word. The epistemic intent is
intact because it never lived in the label alone: the anchor is `#gaps-and-unsupported-choices`, so
the item still opens on the fourteen things the literature does not support rather than on 48
references. Do not "simplify" the anchor to the top of the file.

**It is also in the macOS app menu**, directly under About, sharing `RESEARCH_LABEL` and
`platform::open_url` with the tray so the two can never disagree. Implemented as a second selector
(`exhaleShowResearch:`) on the existing leaked handler class, renamed `ExhaleAboutHandler` ->
`ExhaleMenuHandler` now that it serves two items. This matters more than menu parity: `NSMenu` items
are exposed to the accessibility tree and nothing inside the egui settings window is, so between
them the tray and the app menu are the only evidentiary strings a VoiceOver user can perceive.
Windows and Linux have no app-menu concept; the tray item is the whole surface there.

Three things worth knowing before changing it:

- The URL is pinned to `main`, not to a release tag. A binary outlives its tag, and a retraction has
  to reach people running old builds. A moved anchor is a CI failure; a stale claim is not.
- `scripts/generate-citations.py` now parses `RESEARCH_URL` out of `tray.rs` and fails if the
  constant disappears or its fragment matches no heading in the rendered document. Both failure
  modes were tested by injection.
- The allowlist is `is_openable` in `platform.rs`, with tests. It is load-bearing rather than
  decorative even though every current caller passes a constant: `ShellExecuteW` resolves a `file:`
  URL through the shell association table and would launch a local program.

macOS reaches `NSWorkspace` through `class!` + `msg_send!` rather than the typed bindings, so no new
cargo feature and no new dependency were needed on any platform. **The signed-sandboxed smoke test is
still outstanding** and is the one thing here that was verified by reading rather than by clicking.

Original design notes follow.


One `MenuItem` in [`tray.rs`](rust/crates/exhale-app/src/tray.rs) beside `preferences_item`, one arm
in the `MenuEvent` loop at [`main.rs:1066`](rust/crates/exhale-app/src/main.rs), plus
`platform::open_url` (three cfg'd impls, reject anything not `https://` at the boundary).

**The label and the anchor are the whole feature.** Not "Citations" pointing at the top of
`CITATIONS.md`, which lands the reader in 48 references and reads as a wall of authority. Point it at
`#gaps-and-unsupported-choices` and label it toward the ledger, e.g. **"Research, and what it doesn't
support"**. Same code, opposite epistemic effect.

Smoke-test `NSWorkspace openURL:` on a signed sandboxed build before submission. The API surface,
feature flags and entitlements were verified; an actual signed click was not.

### Phase 3: computed readout. DONE.

**Shipped, with three changes to the design, all deliberate.**

1. **The copy lives in `exhale-core`, not next to the widget.** CI runs `cargo test -p exhale-core`
   and does not compile `exhale-app`, which needs wgpu, winit and GTK. Copy that makes a coverage
   statement should not be the only text in the project with no test behind it. `pacing.rs` holds
   the range constants, the classification and the strings; the widget only paints.
2. **The drift line reports a doubling point and a projection, both revised after review.** The
   first version quoted a doubling *time* and an hour-out *rate*, and both were unreadable in
   different ways. The time depends on the starting cycle, so the same 1 % read as 17 minutes from a
   10 s cycle and 25 minutes from a 15 s one, which made the panel look like it could not do
   arithmetic even though both figures were right. The rate, "about 1.3 a minute after an hour", is
   correct and impossible to picture. What ships instead is *"the cycle doubles every 70 breaths.
   After an hour it is 46 s, not 10 s."* Breaths because `dᵏ = 2` has no `c` in it, so the count is
   a property of the drift setting alone and is a true repeat interval rather than a first
   milestone; seconds because that is the unit the four steppers above are set in. The second clause
   is suppressed when an hour moves the number by less than half a second, or 0.001 % would render
   "after an hour it is 15 s, not 15 s". `Settings::breaths_per_min_after` has an exact closed form
   (`D = c + 60·T·(d - 1)`, derived in the doc comment), so nothing here loops or calls `powi`, and
   a cycle-by-cycle simulation is kept as a test oracle.
3. **A fourth coverage state was added.** A pace that starts inside the tested range and drifts out
   of it within the hour says so, rather than reporting only its starting classification, which
   would read "one of them" and be misleading.

Two details that are constraints rather than polish: the classification runs on the **rounded**
displayed rate, so the panel can never print "5.0 breaths a minute" above "slower than any of them";
and `no_line_names_an_effect_a_benefit_or_a_condition` is a denylist test over the generated strings,
not a style rule. It is what enforces section 3's rule inside the binary, and it is the reason
`custom.backsClaims` still cannot be quoted verbatim however well-sourced those strings are.

`INITIAL_PREFERRED_H` went 796 -> 848 to keep the Randomization card above the fold on first run. The
figure is estimated, not measured, which is safe in both directions: it only picks the first-run
height, a saved height always wins, and `set_max_inner_size` clamps to laid-out content on frame one.
**Nobody has looked at this on a screen yet.** Everything above is compile-, test- and
arithmetic-verified; the visual result is not.

Original design notes follow.


One live line under the Timing sliders, computed from real settings:

```
Now: 4.0 breaths a minute (15 s cycle).
Drift is on: about 3.5 a minute after 4 minutes.
```

Plus a coverage line, one of three states, phrased about the literature and never about the user:
*"Rates from 5 to 7 a minute are the ones tested directly. Yours is below that range."*

It performs arithmetic the user genuinely cannot do (especially with `drift` compounding, and
especially for box breathing where the holds hide that 4/4/4/4 is 3.75 a minute), makes no health
claim, and volunteers bad news about the app's own default.

**Condition attached, from the professor:** if the shipped default stays out of band, this line must
render **unprompted on first open**, not behind a hover or a disclosure triangle. Disclosure reaches
users who open the panel; the default is imposed on everyone who never does.

### Phase 4: preset chips. DONE.

**Shipped, with two structural changes and one thing that turned out not to be true.**

1. **No per-chip evidentiary captions.** The design called for captions carrying each pattern's rate
   ("caption must say 3.75 a minute", "caption says it is below the tested band"). Phase 3's readout
   now sits three rows under the chips and says exactly that, computed, for whichever chip is lit.
   Five hand-written captions would restate it worse: they would be prose in the binary rather than
   arithmetic, they would need maintaining against the corpus, and they would only cover the
   patterns someone remembered to annotate. The captions that survive carry names and nothing else
   (`Sometimes called A52.`, `Sometimes called box breathing.`, `exhale's shipped default.`), and a
   denylist test rejects any label or note that names a rate, an effect or a recommendation.
2. **Presets set the four durations and nothing else**, not drift and not jitter. The design said
   presets should set those "and disclose that they do", while also saying not to silently zero a
   setting the user tuned. Not touching them satisfies both, and it is what makes derived selection
   sound: selection compares exactly the four fields a preset writes.
3. **The contrast worry did not hold.** Measured rather than estimated, the dark-mode selected pill
   is **4.94:1 at its worst across every possible backdrop**, not the ~4.0:1 the design feared,
   because alpha 230 leaves little of the card showing through. The real finding is next door:
   *unselected* text on the translucent card is fine on any backdrop the app controls and on any
   non-inverting vibrancy, and bottoms out near **3.0:1** against a near-white backdrop behind a
   dark-mode window. That is a property of every `ui.label` in this window, not of the chips, and it
   is now a pinned number in `widgets.rs` rather than an assumption.

The open question the design flagged is answered: **egui 0.29 does synthesise a click from Space and
Enter** on a focused `ui.interact` response (`Context::create_widget` sets `fake_primary_click`).
Both keys, and Tab-reachability of all five chips in display order, are covered by tests.

`selected_pill_fill`, `selected_pill_text` and `card_fill` were hoisted out of `segmented_row` and
`section` so the pickers and the chips share one definition and one contrast test.

**Two traps for whoever works on this file next.** `ui.set_width` on a `CentralPanel`'s own `Ui` is
silently undone (`Placer::set_max_width` unions the result back with `min_rect`, which for a panel is
already the full panel), so the first version of the chip test laid out at 384 pt and would have
passed while the shipped card is 308; go through `section` instead. And the Timing card is now
**250 pt**, measured, up from 160 before phase 3, which is why `INITIAL_PREFERRED_H` stayed at 796
and the fold moved up rather than the window growing.

**On the vertical budget.** The first working version measured 324 pt and looked it: a heading line,
three rows of chips and a caption line, five lines of chrome above four steppers. Three changes got
it to 250 without dropping anything. The labels became the four-number form; the caption merged into
the heading, which also stopped the card jumping as the selection changed; and `CHIP_PAD_X` and
`CHIP_GAP` went to 7 and 5, which is the difference between 297 pt of chips and 331 in a 308 pt card.
A test pins the result at no more than two rows, so a sixth preset makes that a deliberate decision
rather than a silent regression.

Original design notes follow.


Chips live **inside the existing Timing card** (`settings_window.rs:1355`), above the four
`duration_row` calls, so a click visibly moves all four steppers.

- **Selection is derived, not stored.** Compare the four `f64`s with the same `1e-9` epsilon
  `SettingsDiff::from` uses at `settings.rs:457-460`. No new persisted field, no desync, no
  migration. This is the single most important implementation decision.
- **No "Custom" chip.** Custom is the absence of a selected pill. A disabled chip is a Tab-traversal
  hazard; this file documents that at `:1245` and `:1298`.
- Build them the way `segmented_row` builds segments (`widgets.rs:520-745`): precomputed rects,
  `ui.interact`, `resp.gained_focus() -> scroll_to_me`, the same focus halo at `widgets.rs:672-696`.
- **Verify keyboard activation with a test.** It is unconfirmed whether egui 0.29 synthesises a click
  from Space/Enter on a focused `ui.interact` response. The harness exists at
  `settings_window.rs:1534` onward. Without this the feature is mouse-only.
- **Wrap, never truncate.** `paint_label_with_width` sets `max_rows: 1, overflow_character: Some('…')`
  (`widgets.rs:501-503`); do not use it for chip captions.
- **Contrast needs measuring.** The dark-mode selected pill is `rgb(110,110,110)` at alpha 230 with
  white text (`widgets.rs:603`), which lands near 4.0:1, under the 4.5:1 AA threshold. Measure the
  composite over the translucent card, not the nominal colour.
- Presets set the pattern **and** `drift`/jitter, and disclose that they do. Do **not** silently zero
  a setting the user tuned; the UX agent's `Make steady` inline action is the pattern: the app offers
  the fix rather than performing it.

Candidate set, deliberately including one that ships its own disconfirming evidence:

Shipped set. Labels are the four-number form, not the spelled-out one the design proposed: at 360 pt
the spelled-out labels took three rows, and `5/0/5/0` reads in the same order as the four steppers
directly beneath it.

| Chip | Pattern | Note shown when selected |
|---|---|---|
| `5/0/5/0` | 5/0/5/0 | none needed |
| `4/0/6/0` | 4/0/6/0 | none needed |
| `5/0/5/2` | 5/0/5/2 | `· sometimes called A52.` |
| `4/4/4/4` | box | `· sometimes called box breathing.` |
| `5/0/10/0` | 5/0/10/0 | `· exhale's shipped default.` |

The original design's rate captions are gone; see point 1 below.

### Phase 5: still deferred, and phase 4 strengthened the case.

The instruction was to ship 2-4 and see whether anything feels missing. What phase 4 found is that
the place prose would have gone is already occupied by something better: the live readout answers
"what is this pattern" for every preset, and a generated `custom.uiCaption` would be a second,
staler answer to the same question. Revisit only if a specific gap turns up that arithmetic cannot
fill.

Original notes on the disagreement follow.


Whether to add `custom.uiCaption` to the corpus (UX: yes, generated, under `--check`, with a
banned-vocabulary gate) or ship no prose at all (full-stack: any prose authored for the binary
inherits a two-week retraction latency through MAS review). **Ship 2-4 first and see whether anything
feels missing.** If it does, the UX position is the one to implement, because hand-written Rust
strings are ruled out by both.

---

## 5. Data pipeline, if phase 4 proceeds

- Hand-written input at `rust/crates/exhale-core/presets.json`: id, label, nine numbers (four
  durations, four jitters, drift), one citekey. **Nothing evidentiary.**
- Generated, checked-in output `rust/crates/exhale-core/src/presets.rs`, emitted by
  `scripts/generate-citations.py` as a third output beside `CITATIONS.md`, under the same `--check`
  diff gate. Checked in rather than `OUT_DIR` so the exact shipped strings appear in a PR diff.
- New `--check` rules: every `citekey` exists; is `group: timing` or `slow-breathing`; is not tier E;
  is not blocklisted. Encode the blocklist as `custom.inAppCitable: false` in the corpus, so the
  policy lives where the professor edits.
- `core-check` already runs `cargo check -p exhale-core --all-targets` and `cargo test`, so a
  generated `.rs` that does not compile fails CI. That job was added for exactly this reason.

---

## 6. Open decisions and loose ends

- **`feat/provenance-and-drift-enhancements` has no PR.** Five commits, from `60d567b`.
- **None of phases 2, 3 or 4 has been seen running.** The tray item needs a click on a signed
  sandboxed build; the readout and the chips need a look at the Timing card at the shipped window
  width. Layout, wrapping, click and keyboard behaviour are covered by headless egui tests, and
  contrast is computed from the shipped colour constants, so what is unverified is specifically the
  visual result rather than the behaviour.
- **Unselected text bottoms out near 3.0:1** against a near-white backdrop behind a dark-mode
  window. Pre-existing for every label in the settings window, pinned by a test, unfixed. The same
  accessibility debt as the missing AccessKit tree.
- **The Snap listing is still live with the retracted claims.** `snapcraft.yaml` is fixed in the repo,
  but the store shows the old copy until someone re-uploads. Memory says Snap upload is manual via
  Docker `snapcore/snapcraft` + interactive `snapcraft login`.
- **Mac App Store listing copy is nowhere in this repo.** Snap and Windows are now both tracked and
  scanned; MAS is not, and may still be asserting the retracted claims. Nobody has looked.
- **`MICROSOFT_STORE_HANDOFF.md` stays untracked** (session transcript, stale version pins). Its
  field values now live in `rust/packaging/windows/store-listing.md`.
- **Pre-existing test failure:** `platform::mac::tests::apply_app_visibility_roundtrip`, objc2 return
  type mismatch at `mac.rs:1710`, fails on a clean tree. `core-check` does not cover `exhale-app`.
- **Default cadence:** still 5/0/10/0 = 4.0 a minute, below the 4.5-6.5 resonance band. The professor
  recommended 4/0/6/0 for new installs only (timing fields carry no `#[serde(default)]`, so this
  touches fresh installs and Reset only, no cohort flag) then deprioritised it himself. Unresolved by
  choice, and gap 2 documents it.
- `drift` now defaults to 1.0 (off) with a 0.1 percentage-point step and no ceiling. See gaps ledger
  item 6, which carries two corrections worth reading before touching it.
