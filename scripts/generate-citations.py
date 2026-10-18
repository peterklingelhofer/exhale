#!/usr/bin/env python3
"""Render docs/CITATIONS.md from the CSL-JSON corpus and the hand-written notes.

Single source of truth:
  - docs/CITATIONS.csl.json  bibliographic records + provenance metadata + the source's own
                             words for every claim (custom.quotes)
  - docs/citations-notes.md  preamble, reading guide, gaps ledger

docs/CITATIONS.md is generated from both and must never be hand-edited.

The ledger is held to three more tracked files with no network. Each one is written by the
tool named here and read by this script:
  - docs/CITATIONS.receipts.json  proof that each quote was found in its source's text
                                  (scripts/check-quotes.py --write-receipts, on a machine with
                                  the research cache)
  - docs/CITATIONS.registry.json  what Crossref, Open Library and PubMed said about each entry,
                                  and the state of each link (scripts/check-sources.py --write)
  - docs/CITATIONS.baseline.json  the gap counts that may fall and never rise (--write-baseline)

Beyond the shape of each record, both modes fail when
  - a receipt no longer stands for a quote the ledger carries, so a quote was edited after it
    was checked
  - a readDepth label claims more than the receipts show
  - a figure pinned in rust/crates/**/*.rs (a Pinned literal) differs from the quote it cites
  - a quote mark in a claim, a caveat, the README or the gaps ledger surrounds words that no
    quote of the entries it concerns has
  - the registry snapshot disagrees with the ledger, shows a retraction or has no check of a
    link the ledger carries
  - a gap count (quotes without a receipt, claims without a quote, README figures without a
    quote, links without an archive capture, unverified entries) differs from its baseline.
    Write mode only prints those differences. --check fails on them, and a count that fell says
    to lower the baseline so the gain holds. A README figure from 0 to 20 counts as quoted when
    a quote spells it as an English word

Usage:
    uv run --no-project scripts/generate-citations.py                    # write
    uv run --no-project scripts/generate-citations.py --check            # fail on drift
    uv run --no-project scripts/generate-citations.py --list-gaps        # list what the gap counts count
    uv run --no-project scripts/generate-citations.py --write-baseline   # record the gap counts
    uv run --no-project scripts/generate-citations.py --write-baseline --raise   # accept a higher count

--write-baseline runs every other check first and writes only the baseline. It refuses to raise
a count unless --raise is also given, and it works with no baseline file. --list-gaps prints the
items behind every nonzero count and exits 0. A tracked file that is missing is an error that
names the command which writes it

Stdlib only, so it runs anywhere the repo is checked out
"""

from __future__ import annotations

import argparse
import collections
import difflib
import json
import re
import sys
from collections.abc import Callable
from pathlib import Path

import registry
import verbatim

ROOT = Path(__file__).resolve().parent.parent
DOCS = ROOT / "docs"
CORPUS = DOCS / "CITATIONS.csl.json"
NOTES = DOCS / "citations-notes.md"
OUT = DOCS / "CITATIONS.md"
README = ROOT / "README.md"
# Written by check-quotes.py, check-sources.py and --write-baseline. See the module docstring
RECEIPTS = DOCS / "CITATIONS.receipts.json"
REGISTRY = DOCS / "CITATIONS.registry.json"
BASELINE = DOCS / "CITATIONS.baseline.json"
# Rust sources searched for Pinned literals, how many have to parse and where they live. See
# check_pinned_figures
PINNED_GLOB = "rust/crates/**/*.rs"
MIN_PINS = 2
PIN_HOME = "rust/crates/exhale-core/src/pacing.rs"
# The one place the shipped binary names this document. See check_binary_deep_link
TRAY = ROOT / "rust/crates/exhale-app/src/tray.rs"
# Patterns the binary offers as one click. See check_preset_citekeys
PRESETS = ROOT / "rust/crates/exhale-core/src/presets.rs"

# Section order in the rendered document. Each group answers one question the
# app raises
GROUPS = [
    ("premise", "Why a breathing reminder next to a screen"),
    ("slow-breathing", "Whether slow paced breathing does anything"),
    ("timing", "What the numbers should be"),
    ("tradition", "Where the practice came from"),
    ("interface", "Whether an on-screen visual pacer works"),
    ("mechanism", "Physiology and neuroscience"),
    ("safety", "Limits, harms and adherence"),
]
GROUP_IDS = {g for g, _ in GROUPS}

# How each verification status is named in the one-line summary. VERIFICATIONS
# below is these same keys, so a source verified some new way fails the
# validator instead of silently vanishing from the count later. The previous
# hand-written version added up to 47 of 48 for that reason
VERIFICATION_LABELS = {
    "crossref-verified":    "Crossref-verified",
    "openlibrary-verified": "verified against Open Library",
    "pubmed-verified":      "verified against PubMed",
    "unverified":           "unverified",
}
VERIFICATIONS = set(VERIFICATION_LABELS)

ACCESS_LEVELS = {"open-access", "paywalled"}

# How much of the source was read before its claims were written down.
# Verification confirms the citation. This field says what stands behind the claim
READ_DEPTH_LABELS = {
    "full-text": "full text",
    "abstract":  "abstract only",
    "record":    "catalog record only",
}
READ_DEPTHS = set(READ_DEPTH_LABELS)

TIERS = {"A", "B", "C", "D", "E", None}

ID_RE = re.compile(r"^[a-z][a-z0-9]+[0-9]{4}-[a-z0-9-]+$")
# Same shape, unanchored, for spotting citekeys named inside prose
MENTION_RE = re.compile(r"\b[a-z][a-z0-9]+[0-9]{4}-[a-z0-9-]+\b")

# The commands that write the tracked files the quote checks read. A problem names the one that
# fixes it
RECEIPTS_FIX = "uv run --no-project scripts/check-quotes.py --write-receipts"
SOURCES_FIX = "uv run --no-project scripts/check-sources.py --write"
BASELINE_FIX = "uv run --no-project scripts/generate-citations.py --write-baseline"
BASELINE_FORMAT = 1
UNAVAILABLE = "missing or unreadable input"

# Which registry each verification label names. `unverified` names none, so no check is due
VERIFIED_BY = {f"{service}-verified": service for service in registry.REGISTRY_NAMES}
# Crossref's update types that mean the work can't back a claim any more. Its other types
# (correction, corrigendum, erratum and the like) render under the entry and don't fail
DISQUALIFYING_UPDATES = {"retraction", "partial_retraction", "withdrawal", "removal", "expression_of_concern"}

# The README links entries as (docs/CITATIONS.md#citekey) and the gaps ledger as (#citekey)
README_LINK = re.compile(r"\[[^\]]*\]\(docs/CITATIONS\.md#([a-z0-9-]+)\)")
NOTES_LINK = re.compile(r"\[[^\]]*\]\(#([a-z0-9-]+)\)")
LIST_ITEM = re.compile(r"\s*(?:[-*]|\d+\.)\s")
# A link in the README becomes a marker that carries its key, so its text (an author and a year)
# counts as no figure of the sentence, and a year after "et al." can't open a sentence. The marker
# opens a sentence in the split below. So does a digit, as in "4-7-8 is slower still", and so does
# the product name, which is written lowercase. Without those two a sentence about the app's own
# numbers would join the sentence before it and charge them to the papers that one links
MARK_OPEN, MARK_CLOSE = "\u27e6", "\u27e7"
MARKER_RE = re.compile(MARK_OPEN + r"([a-z0-9-]+)" + MARK_CLOSE)
SENTENCE_BREAK = re.compile(r'(?<=[.!?])\s+(?=[A-Z0-9*_"(\[' + MARK_OPEN + r']|exhale\b)')
NUMBER_RE = re.compile(r"(?<![\w.])\d+(?:\.\d+)?(?![\w]|\.\d)")
# The whole numbers 0 to 20 as the English words a source may print them with
NUMBER_WORDS = (
    "zero one two three four five six seven eight nine ten eleven twelve thirteen fourteen "
    "fifteen sixteen seventeen eighteen nineteen twenty"
).split()

# A Pinned literal in a Rust source, whose fields come in this order. See check_pinned_figures
STRUCT_PINNED = re.compile(r"\bstruct\s+Pinned\s*\{")
PINNED_LITERAL = re.compile(r"\bPinned\s*\{")
PIN_START = re.compile(r"\bconst\s+([A-Z][A-Z0-9_]*)\s*:\s*Pinned\s*=\s*Pinned\s*\{")
RUST_STRING = r'"((?:[^"\\]|\\[\s\S])*)"'
RUST_NAME = r"[A-Z][A-Z0-9_]*"
PIN_FIELDS = re.compile(
    r"\s*value\s*:\s*[^,]+,"
    r"\s*spelled\s*:\s*" + RUST_STRING + r"\s*,"
    r"\s*citekey\s*:\s*" + RUST_STRING + r"\s*,"
    r"\s*locator\s*:\s*" + RUST_STRING + r"\s*,"
    r"\s*quote\s*:\s*(?:" + RUST_STRING + "|(" + RUST_NAME + r"))\s*,?\s*\}\s*;"
)

# The gap counts the baseline holds, in the order it lists them
COUNT_NAMES = (
    "unverifiedEntries", "quotesWithoutReceipt", "claimsWithoutQuote", "readmeFiguresWithoutQuote", "linksWithoutArchive",
)
# The chars of a claim or sentence shown in a list of gaps
SHOWN_CHARS = 100
# A ledger quote this close to a pin (a difflib ratio) is named as the quote the pin meant to copy
NEAR_PIN = 0.9


class CorpusError(Exception):
    """Raised with a ready-made message, or a list of problems that get
    formatted as one bullet per line, and optionally the command that fixes them"""

    def __init__(self, problems: str | list[str], fix: str | None = None) -> None:
        if isinstance(problems, str):
            message = problems
        else:
            message = "\n".join("  - " + p for p in problems)
        super().__init__(message + (f"\n  Fix: {fix}" if fix else ""))


class Failures(list):
    """The checks that failed, as (heading, CorpusError) in the order they ran"""

    def attempt(self, heading: str, step: Callable[..., object], **arguments: object) -> object | None:
        """Runs one load or check with the given keyword arguments.

        A CorpusError is kept under the heading and the run goes on to the next check, so one fix
        doesn't uncover the next failure a run later
        """
        try:
            return step(**arguments)
        except CorpusError as exc:
            self.append((heading, exc))
            return None

    def report(self) -> bool:
        """Prints each failed check under its heading and says whether any failed"""
        for heading, exc in self:
            print(f"{CORPUS.name}: {heading}\n{exc}", file=sys.stderr)
        return bool(self)


def load_corpus() -> list[dict]:
    records = json.loads(CORPUS.read_text(encoding="utf-8"))
    if not isinstance(records, list):
        raise CorpusError(f"{CORPUS.name} must hold a JSON array")

    problems: list[str] = []
    seen: set[str] = set()

    for index, rec in enumerate(records):
        rid = rec.get("id", f"<record {index}>")

        if not ID_RE.match(str(rec.get("id", ""))):
            problems.append(f"{rid}: id must match {ID_RE.pattern}")
        else:
            keyed = re.search(r"(\d{4})-", rid)
            issued = (rec.get("issued", {}).get("date-parts") or [[None]])[0][0]
            if keyed and issued and keyed.group(1) != str(issued):
                problems.append(
                    f"{rid}: citekey year {keyed.group(1)} doesn't match issued year {issued}"
                )
        if rid in seen:
            problems.append(f"{rid}: duplicate id")
        seen.add(rid)

        for field in ("type", "title", "issued"):
            if not rec.get(field):
                problems.append(f"{rid}: missing '{field}'")

        custom = rec.get("custom")
        if not isinstance(custom, dict):
            problems.append(f"{rid}: missing 'custom' block")
            continue

        if custom.get("group") not in GROUP_IDS:
            problems.append(f"{rid}: group must be one of {sorted(GROUP_IDS)}")
        if custom.get("verification") not in VERIFICATIONS:
            problems.append(f"{rid}: verification must be one of {sorted(VERIFICATIONS)}")
        if custom.get("accessLevel") not in ACCESS_LEVELS:
            problems.append(f"{rid}: accessLevel must be one of {sorted(ACCESS_LEVELS)}")
        if custom.get("evidenceTier", "missing") not in TIERS:
            problems.append(f"{rid}: evidenceTier must be A-E or null")
        if custom.get("readDepth") not in READ_DEPTHS:
            problems.append(f"{rid}: readDepth must be one of {sorted(READ_DEPTHS)}")
        # A legal open copy of a paywalled version of record, from a repository
        # or free at the publisher. Only ever an https URL, and never a DOI,
        # which would point back at the paywall
        open_copy = custom.get("openCopy")
        if open_copy is not None and not str(open_copy).startswith("https://"):
            problems.append(f"{rid}: openCopy must be an https URL")
        # Absent means citable. Only an explicit `false` blocks a record
        # from backing something the binary ships. Written as an opt-out so
        # the corpus doesn't need touching for the common case, and so the
        # blocklist is greppable as four lines and never has to be inferred
        # from forty-four omissions
        if custom.get("inAppCitable", False) not in (True, False):
            problems.append(f"{rid}: inAppCitable must be true or false")

        claims = custom.get("backsClaims")
        if not isinstance(claims, list) or not claims:
            problems.append(f"{rid}: backsClaims must be a non-empty list")

        # The source's own words behind each claim. The shape rules live in verbatim.py, which
        # check-quotes.py shares, so a quote is held to one definition
        problems.extend(
            f"{rid}: {flaw}"
            for flaw in verbatim.quote_flaws(
                quotes=custom.get("quotes"),
                claim_count=len(claims) if isinstance(claims, list) else 0,
                read_depth=custom.get("readDepth"),
            )
        )
        problems.extend(archive_link_problems(rec=rec))

        # A record with no DOI can't call itself Crossref-verified
        if custom.get("verification") == "crossref-verified" and not rec.get("DOI"):
            problems.append(f"{rid}: crossref-verified but carries no DOI")

    if problems:
        raise CorpusError(problems)

    return sorted(records, key=lambda r: r["id"])


def archive_link_problems(rec: dict) -> list[str]:
    """What is wrong with the Wayback links an entry carries, one message each.

    `archived` captures the entry's URL and belongs to an entry with no DOI, since a DOI is the
    durable link already. `openCopyArchived` captures the open copy
    """
    rid, custom = rec["id"], rec["custom"]
    problems: list[str] = []
    for field, source, target in (
        ("archived", "URL", rec.get("URL")),
        ("openCopyArchived", "custom.openCopy", custom.get("openCopy")),
    ):
        if field not in custom:
            continue
        if field == "archived" and rec.get("DOI"):
            problems.append(f"{rid}: archived is only for an entry with no DOI, since the DOI is the durable link")
        elif not target:
            problems.append(f"{rid}: {field} needs a {source} to capture")
        elif problem := registry.wayback_problem(link=custom[field], target=str(target)):
            problems.append(f"{rid}: {field} {problem}")
    return problems


def check_cross_references(records: list[dict]) -> None:
    """Caveats point at sibling entries by citekey. Catch the ones that rot.

    These cross-references are the most useful thing in the corpus and the
    easiest to break, because renaming an entry leaves every mention of it
    behind as plausible-looking prose
    """
    known = {r["id"] for r in records}
    problems: list[str] = []
    for rec in records:
        custom = rec["custom"]
        prose = " ".join([custom.get("caveat") or "", *custom.get("backsClaims", [])])
        for mention in MENTION_RE.findall(prose):
            if mention not in known:
                problems.append(f"{rec['id']}: names unknown entry '{mention}'")
    if problems:
        raise CorpusError(problems)


# Claims the corpus doesn't support, kept out of the surfaces that assert them
#
# Only surfaces that assert are scanned: store listings and anything compiled
# into the binary. `docs/` and `README.md` are exempt, because the gaps ledger
# has to be able to name a claim in order to explain what the evidence says
# about it
#
# `snapcraft.yaml` and the README have fallen out of sync on the parasympathetic
# claim before, because a store listing is edited in a different place from a
# README and nothing connected the two
UNSUPPORTED_PHRASES: list[tuple[str, str]] = [
    ("engage the parasympathetic nervous system", "gaps ledger 4: the cardiac evidence splits 3-for / 1-against / 2-null"),
    ("engages the parasympathetic nervous system", "gaps ledger 4: the cardiac evidence splits 3-for / 1-against / 2-null"),
    ("breathe more shallowly", "gaps ledger 1: the measured finding is faster and slightly over-ventilated, the chest-high shift is theorized, and no study measured shallower breathing"),
    ("screen apnea", "gaps ledger 1: no peer-reviewed source, and the measured effect points the other way"),
    ("email apnea", "gaps ledger 1: no peer-reviewed source, and the measured effect points the other way"),
    ("higher in the chest", "gaps ledger 1: the diaphragmatic-to-thoracic shift is theorized, and no study has measured it in screen users"),
    ("countermeasure with the most evidence", "no source compares countermeasures, and gaps ledger 5 says slow pacing can add to over-breathing"),
    ("hardware buys nothing", "laborde2021-spb-6cpm-biofeedback reports a valence advantage for biofeedback"),
]

# Surfaces where one of the phrases above would be an assertion. Missing files
# are skipped
ASSERTING_SURFACES: list[str] = [
    "snap/snapcraft.yaml",
    "rust/packaging/macos/store-listing.md",
    "rust/packaging/windows/AppxManifest.xml",
    "rust/packaging/windows/store-listing.md",
]
ASSERTING_GLOBS: list[str] = ["rust/crates/**/*.rs"]


def check_unsupported_phrases() -> None:
    """Fail if an unsupported claim survives on a surface that asserts it"""
    targets = [ROOT / rel for rel in ASSERTING_SURFACES]
    for pattern in ASSERTING_GLOBS:
        targets.extend(sorted(ROOT.glob(pattern)))

    problems: list[str] = []
    for path in targets:
        if not path.exists():
            continue
        try:
            text = path.read_text(encoding="utf-8")
        except (UnicodeDecodeError, OSError):
            continue
        lowered = text.lower()
        for phrase, why in UNSUPPORTED_PHRASES:
            if phrase in lowered:
                line = lowered[: lowered.index(phrase)].count("\n") + 1
                rel = path.relative_to(ROOT)
                problems.append(f"{rel}:{line} asserts \"{phrase}\" ({why})")

    if problems:
        raise CorpusError(problems)


def check_note_links(records: list[dict], text: str, where: str) -> None:
    """Prose links to entries by anchor. Catch the ones that rot.

    Both the gaps ledger and the README deep-link into the corpus by citekey.
    A rename silently breaks every one of them, and the README is the project's
    front door, so it's checked on the same footing as the notes
    """
    known = {r["id"] for r in records}
    # Only anchors shaped like a citekey are entry references. The rest are
    # ordinary intra-document links to headings in the notes
    # The notes link as (#citekey), the README as (docs/CITATIONS.md#citekey)
    anchors = re.findall(r"\]\((?:docs/CITATIONS\.md)?#([a-z0-9-]+)\)", text)
    referenced = {a for a in anchors if ID_RE.match(a)}
    dangling = sorted(referenced - known)
    if dangling:
        raise CorpusError([f"{where} links to unknown entry #{d}" for d in dangling])


def check_preset_citekeys(records: list[dict]) -> None:
    """Every pattern the app offers has to be traceable to a live record.

    The preset list is the one place the binary makes a *selection* from the
    literature, and a selection is an argument whether or not it's worded as
    one. Offering five patterns says these five are worth a click, so each
    carries a citekey that never reaches the screen and exists only to fail
    this check.

    Of the four conditions, the third and fourth do most of the work.
    A record downgraded to tier E is lineage-only and can't license a
    default. A record marked `inAppCitable: false` is one the design review
    blocklisted from a store-reviewed binary, which is a different judgment
    from whether it's good evidence: `fincham2023` is the strongest warrant
    in the corpus and is on that list
    """
    if not PRESETS.exists():
        return

    text = PRESETS.read_text(encoding="utf-8")
    # `citekey: "..."` inside the const table. Comments mention citekeys in
    # prose, so anchor on the field name
    found = re.findall(r'citekey:\s*"([^"]*)"', text)
    if not found:
        raise CorpusError(
            f"  - {PRESETS.relative_to(ROOT)} defines no preset citekeys. The shipped\n"
            "    patterns have lost their link to the corpus"
        )

    by_id = {r["id"]: r for r in records}
    problems: list[str] = []
    for key in sorted(set(found)):
        rec = by_id.get(key)
        if rec is None:
            problems.append(f"preset citekey '{key}' matches no corpus entry")
            continue
        custom = rec["custom"]
        if custom["group"] not in ("timing", "slow-breathing"):
            problems.append(
                f"preset citekey '{key}' is group '{custom['group']}'. A preset has to be "
                "backed by a timing or slow-breathing record"
            )
        if custom.get("evidenceTier") == "E":
            problems.append(
                f"preset citekey '{key}' is tier E, which is citable for lineage only"
            )
        if custom.get("inAppCitable", True) is False:
            problems.append(
                f"preset citekey '{key}' is marked inAppCitable: false and may not back "
                "anything the binary ships"
            )
    if problems:
        raise CorpusError(problems)


def check_readme_counts(records: list[dict], text: str) -> None:
    """The README states the size of the corpus. Check the figures.

    These numbers were wrong for the entire life of the previous commit: the
    README advertised 42 sources verified 40 / 2 while the corpus held 48
    verified 45 / 2 / 1. Nothing caught it, because a wrong number in prose
    looks like a right one. Undercounting is the harmless direction, and it's
    still the project's front door claiming a provenance figure it hadn't
    checked, which is the failure this apparatus exists to prevent
    """
    counts = collections.Counter(r["custom"]["verification"] for r in records)
    # Each phrase is matched wherever it appears, so the lede and the research
    # section can't drift apart from each other either
    expected = [
        (r"(\d+)\s+sources", len(records), "total sources"),
        (r"(\d+)\s+verified against the Crossref REST API", counts["crossref-verified"], "Crossref"),
        (r"(\d+)\s+against Open Library", counts["openlibrary-verified"], "Open Library"),
        (r"(\d+)\s+against PubMed", counts["pubmed-verified"], "PubMed"),
    ]
    problems = []
    for pattern, want, what in expected:
        found = re.findall(pattern, text)
        if not found:
            problems.append(f"README.md no longer states the {what} count")
            continue
        for got in found:
            if int(got) != want:
                problems.append(f"README.md says {got} for {what}. The corpus holds {want}")
    if problems:
        raise CorpusError(problems)


def load_tracked(path: Path, fix: str) -> dict:
    """A tracked JSON file the checks read, or a CorpusError that names the command which writes it"""
    if not path.is_file():
        raise CorpusError(problems=[f"{path.name} is missing"], fix=fix)
    try:
        data = json.loads(path.read_text(encoding="utf-8"))
    except (OSError, ValueError) as exc:
        raise CorpusError(problems=[f"{path.name} can't be read: {exc}"], fix=fix) from exc
    if not isinstance(data, dict):
        raise CorpusError(problems=[f"{path.name} must hold a JSON object"], fix=fix)
    return data


def receipt_index(records: list[dict], receipts: dict) -> dict[str, dict[str, dict]]:
    """citekey -> quote key -> the `found` record of its receipt, for the receipts of quotes the entry carries now"""
    keys = verbatim.ledger_quote_keys(records=records)
    index: dict[str, dict[str, dict]] = {citekey: {} for citekey in keys}
    by_entry = receipts.get("receipts")
    for citekey, items in (by_entry if isinstance(by_entry, dict) else {}).items():
        if citekey not in keys or not isinstance(items, list):
            continue
        for item in items:
            found = item.get("found") if isinstance(item, dict) else None
            quote = item.get("quote") if isinstance(item, dict) else None
            if isinstance(found, dict) and isinstance(quote, str) and quote in keys[citekey]:
                index[citekey].setdefault(quote, found)
    return index


def receipt_flaw(receipt: object) -> str | None:
    """Why a receipt isn't the {quote, preview, found} record that check-quotes.py writes, or None"""
    if not isinstance(receipt, dict) or not isinstance(receipt.get("quote"), str):
        return "isn't an object with a quote key"
    found = receipt.get("found")
    if not isinstance(found, dict):
        return "has no found record"
    if missing := [key for key in verbatim.FOUND_KEYS if key not in found]:
        return f"has a found record without {', '.join(missing)}"
    if found["scope"] not in verbatim.SCOPES:
        return f"has scope {found['scope']!r}, and the scopes are abstract and body"
    return None


def check_receipts(records: list[dict], receipts: dict) -> None:
    """Every receipt has to stand for a quote the ledger carries.

    A quote edited after it was checked leaves its receipt without a quote and the new words
    without a receipt, so a changed quote can't keep the proof of the old one
    """
    by_entry = receipts.get("receipts")
    if receipts.get("format") != verbatim.RECEIPTS_FORMAT or not isinstance(by_entry, dict):
        raise CorpusError(
            problems=[f"{RECEIPTS.name} must be a format {verbatim.RECEIPTS_FORMAT} receipts file"],
            fix=RECEIPTS_FIX,
        )
    problems: list[str] = []
    for citekey, items in by_entry.items():
        for number, item in enumerate(items if isinstance(items, list) else [None], start=1):
            flaw = receipt_flaw(receipt=item)
            if flaw:
                problems.append(f"{citekey}: receipt {number} {flaw}")
    if problems:
        raise CorpusError(problems=problems, fix=RECEIPTS_FIX)

    reasons = {
        "unknown-citekey": "belongs to a citekey the ledger doesn't have",
        "quote-not-in-ledger": (
            "stands for words that no quote of the entry has now, so the quote was edited or "
            "removed after it was checked"
        ),
        "duplicate": "repeats an earlier receipt on this entry",
    }
    for citekey, preview, reason in verbatim.stale_receipts(records=records, receipts=receipts):
        problems.append(f'{citekey}: the receipt for "{preview}" {reasons[reason]}')
    if problems:
        raise CorpusError(problems=problems, fix=f"{RECEIPTS_FIX} on a machine with the research cache")


def check_read_depth(records: list[dict], receipts: dict) -> None:
    """A readDepth label has to match what the receipts show.

    An abstract entry has only quotes whose locator starts with Abstract and none with a body
    receipt. A full-text entry has at least one quote with a body receipt. Every receipt's scope has
    to agree with where its quote's locator points (verbatim.locator_flaw). The rest of the quote
    shape is held in load_corpus
    """
    index = receipt_index(records=records, receipts=receipts)
    problems: list[str] = []
    for rec in records:
        rid, depth = rec["id"], rec["custom"]["readDepth"]
        has_body_receipt = False
        for number, quote in enumerate(verbatim.quotes_of(record=rec)):
            found = index[rid].get(verbatim.quote_key(text=quote["text"])) or {}
            scope, source, place = found.get("scope"), found.get("source"), quote["locator"]
            where = f'quotes[{number}] ("{verbatim.quote_preview(text=quote["text"])}")'
            has_body_receipt = has_body_receipt or scope == "body"
            if scope == "body" and depth == "abstract":
                problems.append(
                    f"{rid}: readDepth is abstract, but {where} was found in the body ({source}). "
                    "Set readDepth to full-text, or replace the quote with words from the abstract"
                )
            elif scope and (flaw := verbatim.locator_flaw(locator=place, scope=scope, source=source)):
                problems.append(f"{rid}: {where} {flaw}")
            elif scope is None and depth == "abstract" and verbatim.locator_scope(locator=place) != "abstract":
                problems.append(
                    f'{rid}: readDepth is abstract, so the locator "{place}" of {where} has to start with '
                    f'"{verbatim.ABSTRACT_LOCATOR}"'
                )
        if depth == "full-text" and not has_body_receipt:
            problems.append(
                f"{rid}: readDepth is full-text, but no quote has a body receipt, so nothing shows "
                "the text beyond the abstract was read. Add a quote from the body, or set readDepth "
                "to abstract"
            )
    if problems:
        raise CorpusError(
            problems=problems, fix=f"run {RECEIPTS_FIX}, then edit any entry above that this check still lists"
        )


def unrust(literal: str) -> str:
    """The text of a Rust string literal, with its two escapes undone"""
    return re.sub(r'\\(["\\])', r"\1", literal)


def parse_pins(text: str) -> tuple[list[dict], list[tuple[str, int]]]:
    """The Pinned literals in one Rust source, and the (name, line) of each that doesn't parse.

    A literal reads `const NAME: Pinned = Pinned { value: ..., spelled: "...", citekey: "...",
    locator: "...", quote: ... }` with any whitespace between tokens. The quote is a string literal
    or the name of a `const NAME: &str = "..."` in the same file
    """
    pins: list[dict] = []
    broken: list[tuple[str, int]] = []
    for start in PIN_START.finditer(text):
        line = text.count("\n", 0, start.start()) + 1
        fields = PIN_FIELDS.match(text, start.end())
        if fields is None:
            broken.append((start.group(1), line))
            continue
        _, citekey, locator, quote, constant = fields.groups()
        if quote is None:
            named = re.search(
                rf"\bconst\s+{re.escape(constant)}\s*:\s*&(?:'static\s+)?str\s*=\s*{RUST_STRING}\s*;", text
            )
            quote = named.group(1) if named else None
        pins.append({
            "name": start.group(1),
            "line": line,
            "citekey": unrust(literal=citekey),
            "locator": unrust(literal=locator),
            "quote": None if quote is None else unrust(literal=quote),
            "constant": constant,
        })
    return pins, broken


def nearest_quote_note(pinned: str, quotes: list[dict]) -> str:
    """Where a pin first differs from the quote nearest to it, or nothing when no quote is near"""
    ratio, text = max(
        ((difflib.SequenceMatcher(None, quote["text"], pinned).ratio(), quote["text"]) for quote in quotes),
        default=(0.0, ""),
    )
    if ratio < NEAR_PIN:
        return ""
    at = next((i for i, (a, b) in enumerate(zip(text, pinned)) if a != b), min(len(text), len(pinned)))
    lead = ("..." if at > 12 else "") + pinned[max(0, at - 12):at]
    return (
        f' The nearest quote differs at character {at + 1}. It reads "{lead}|{text[at:at + 20]}" '
        f'where the pin reads "{lead}|{pinned[at:at + 20]}".'
    )


def pin_problems(pin: dict, where: str, entry: dict | None, index: dict[str, dict[str, dict]]) -> list[str]:
    """Why a parsed pin doesn't match the corpus, one message each"""
    citekey, pinned = pin["citekey"], pin["quote"]
    if entry is None:
        return [f"{where} cites {citekey}, which is not in the ledger"]
    if pinned is None:
        return [
            f"{where} names the quote constant {pin['constant']}, and this file has no "
            f'const {pin["constant"]}: &str = "..."'
        ]
    quotes = verbatim.quotes_of(record=entry)
    exact = next((quote for quote in quotes if quote["text"] == pinned), None)
    if exact is None:
        return [
            f'{where} pins "{verbatim.quote_preview(text=pinned)}" for {citekey}, and no quote of '
            f"{citekey} has exactly those words (nothing is normalized, so case, quote marks and "
            f"dashes count).{nearest_quote_note(pinned=pinned, quotes=quotes)}"
        ]
    problems = []
    if exact["locator"] != pin["locator"]:
        problems.append(
            f'{where} says locator "{pin["locator"]}", and that quote of {citekey} is located '
            f'"{exact["locator"]}" in the ledger'
        )
    if verbatim.quote_key(text=exact["text"]) not in index[citekey]:
        problems.append(
            f"{where} pins a quote of {citekey} that has no receipt, so nothing shows its words are "
            "in the source"
        )
    return problems


def check_pinned_figures(records: list[dict], receipts: dict) -> None:
    """A number the binary states is held to the source sentence that prints it.

    Each `Pinned` literal in the Rust sources (see pacing.rs) has to match a quote of its entry in
    the ledger character for character, with the same locator, and that quote needs a receipt
    """
    by_id = {rec["id"]: rec for rec in records}
    index = receipt_index(records=records, receipts=receipts)
    problems: list[str] = []
    parsed = 0
    for path in sorted(ROOT.glob(PINNED_GLOB)):
        try:
            text = path.read_text(encoding="utf-8")
        except (UnicodeDecodeError, OSError):
            continue
        rel = path.relative_to(ROOT)
        pins, broken = parse_pins(text=text)
        parsed += len(pins)
        for name, line in broken:
            problems.append(
                f"{rel}:{line}: the Pinned literal {name} doesn't parse. It takes the fields value, "
                "spelled, citekey, locator and quote in that order, each with a trailing comma, and "
                "the quote is a string literal or the name of a const &str in the same file"
            )
        if not pins and not broken and PINNED_LITERAL.search(STRUCT_PINNED.sub("", text)):
            problems.append(
                f"{rel} builds a Pinned and no `const NAME: Pinned = Pinned {{ ... }}` literal in it "
                "parses, so none of its figures is held to the corpus"
            )
        for pin in pins:
            problems.extend(
                pin_problems(
                    pin=pin,
                    where=f"{rel}:{pin['line']}: {pin['name']}",
                    entry=by_id.get(pin["citekey"]),
                    index=index,
                )
            )
    if parsed < MIN_PINS:
        problems.append(
            f"fewer than {MIN_PINS} Pinned literals were found under rust/crates (parsed: {parsed}). TESTED_MIN and "
            f"TESTED_MAX belong in {PIN_HOME}, and without them nothing holds the tested rates to the corpus"
        )
    if problems:
        raise CorpusError(
            problems=problems,
            fix=f"make the Rust literal and the ledger quote the same text, then run {RECEIPTS_FIX}",
        )


def strip_markup(block: str) -> str:
    """A block of markdown without its HTML tags and inline code, which hold no words of the author's"""
    return re.sub(r"`[^`]*`", " ", re.sub(r"</?[A-Za-z!][^<>]*>", " ", block))


def prose_blocks(text: str) -> list[tuple[int, str]]:
    """The paragraphs and list items of a markdown file as (first line, text).

    A blank line or a heading ends a block and a list item starts one. Fenced code is left out
    """
    blocks: list[tuple[int, list[str]]] = []
    current: list[str] | None = None
    fenced = False
    for number, raw in enumerate(text.splitlines(), start=1):
        if raw.lstrip().startswith(("```", "~~~")):
            fenced, current = not fenced, None
            continue
        line = raw.strip()
        if fenced:
            continue
        if not line or line.startswith("#"):
            current = None
            continue
        if current is None or LIST_ITEM.match(raw):
            current = []
            blocks.append((number, current))
        current.append(line)
    return [(number, " ".join(lines)) for number, lines in blocks]


def spans_with_sentences(prose: str, single: bool) -> list[tuple[str, str]]:
    """Each quoted span of a text with the sentence it opens in, located by the quote mark that opens it"""
    cuts = [0, *(cut.end() for cut in SENTENCE_BREAK.finditer(prose)), len(prose)]
    found: list[tuple[str, str]] = []
    cursor = 0
    for span in verbatim.quoted_spans(prose=prose, single=single):
        opening = re.compile("[\"'\u2018\u201c]\\s*" + re.escape(span)).search(prose, cursor)
        start = opening.start() if opening else cursor
        cursor = start + len(span)
        number = sum(1 for cut in cuts[1:-1] if cut <= start)
        found.append((span, prose[cuts[number]:cuts[number + 1]]))
    return found


def normalized_quotes(records: list[dict]) -> dict[str, list[str]]:
    """citekey -> the normalized text of each quote the entry carries"""
    return {
        rec["id"]: [verbatim.normalize(text=quote["text"]) for quote in verbatim.quotes_of(record=rec)]
        for rec in records
    }


def words_in(span: str, texts: list[str]) -> bool:
    """True when the normalized span is a substring of any of the normalized quote texts"""
    wanted = verbatim.normalize(text=span)
    return any(wanted in text for text in texts)


def block_span_problems(name: str, text: str, link: re.Pattern, texts: dict[str, list[str]]) -> list[str]:
    """The double-quoted spans of a file's blocks that no quote of the entries a block links has"""
    problems = []
    for line, block in prose_blocks(text=text):
        cited = [key for key in dict.fromkeys(link.findall(block)) if ID_RE.match(key) and key in texts]
        if not cited:
            continue
        pool = [quote for key in cited for quote in texts[key]]
        for span in verbatim.quoted_spans(prose=strip_markup(block=block), single=False):
            if not words_in(span=span, texts=pool):
                problems.append(
                    f'{name} line {line}: "{span}" is in quote marks, and no quote of '
                    f"{', '.join(cited)} has those words"
                )
    return problems


def check_prose_quotes(records: list[dict], readme: str | None, notes: str) -> None:
    """A quote mark in prose has to surround words that a ledger quote has.

    In an entry's claims and caveat, a quoted span must sit inside a quote of that entry or of an
    entry the same sentence names. In the README and the gaps ledger, a block that links entries
    must put every double-quoted span inside a quote of one of them. Both sides are normalized
    """
    texts = normalized_quotes(records=records)
    problems: list[str] = []
    for rec in records:
        rid, custom = rec["id"], rec["custom"]
        prose = [(f"claim {number}", claim) for number, claim in enumerate(custom["backsClaims"])]
        prose.append(("caveat", custom.get("caveat") or ""))
        for where, text in prose:
            for span, sentence in spans_with_sentences(prose=text, single=True):
                named = sorted({key for key in MENTION_RE.findall(sentence) if key in texts and key != rid})
                pool = [quote for key in (rid, *named) for quote in texts[key]]
                if not words_in(span=span, texts=pool):
                    problems.append(
                        f'{rid}: {where} puts "{span}" in quote marks, and no quote of '
                        f"{' or '.join([rid, *named])} has those words"
                    )
    problems.extend(block_span_problems(name=NOTES.name, text=notes, link=NOTES_LINK, texts=texts))
    if readme is not None:
        problems.extend(block_span_problems(name=README.name, text=readme, link=README_LINK, texts=texts))
    if problems:
        raise CorpusError(
            problems=problems,
            fix="add the source's words as a quote to the ledger, or take the quote marks off the prose",
        )


def registry_problems(rec: dict, check: object) -> list[str]:
    """What the snapshot's registry check of one entry says against the entry's label"""
    rid, custom = rec["id"], rec["custom"]
    verified = custom["verification"]
    service = VERIFIED_BY.get(verified)
    if service is None:
        return []
    label, command = registry.REGISTRY_NAMES[service], f"uv run --no-project scripts/check-sources.py {rid} --write"
    if not isinstance(check, dict):
        return [f"{rid}: {verified} but {REGISTRY.name} holds no {label} check of it. Run: {command}"]
    problems: list[str] = []
    if check.get("registry") != service:
        problems.append(
            f"{rid}: {verified} means a {label} check, and the snapshot's check is of "
            f"{check.get('registry')!r}. Run: {command}"
        )
    wanted = registry.identifier(entry=rec, service=service)
    if wanted is None:
        problems.append(f"{rid}: {verified} but the entry carries no {registry.ID_FIELDS[service]}")
    elif not registry.same_doi(a=check.get("id"), b=wanted):
        problems.append(
            f"{rid}: the snapshot's {label} check is of {check.get('id')}, and the ledger carries "
            f"{wanted}. Run: {command}"
        )
    agrees = check.get("agrees")
    if check.get("status") != 200:
        problems.append(
            f"{rid}: {label} answered {check.get('status') or 'nothing'} for {check.get('id')} at the "
            f"last check, and {verified} needs a 200. Run: {command}"
        )
    elif not isinstance(agrees, dict) or not agrees:
        problems.append(f"{rid}: the snapshot records no agreement with {label}, field by field. Run: {command}")
    elif disagreeing := [field for field, agreed in agrees.items() if agreed is not True]:
        problems.append(
            f"{rid}: {label} disagrees with the ledger on {', '.join(disagreeing)}. Correct the "
            f"ledger or the registry record, then run: {command}"
        )
    at_check = check.get("ledger")
    if not isinstance(at_check, dict):
        problems.append(f"{rid}: the snapshot keeps no copy of the ledger as the check saw it. Run: {command}")
    elif drifted := registry.drift_between(current=registry.ledger_fields(entry=rec), at_check=at_check):
        problems.append(
            f"{rid}: {', '.join(drifted)} changed in the ledger after the {label} check on "
            f"{str(check.get('checked'))[:10]}. Check the new text against {label}, then run: {command}"
        )
    for notice in check.get("updates") or []:
        kind = notice.get("type") if isinstance(notice, dict) else None
        if kind in DISQUALIFYING_UPDATES:
            problems.append(
                f"{rid}: Crossref lists a {kind.replace('_', ' ')} notice for this work, "
                f"https://doi.org/{notice.get('doi')} dated {notice.get('date')}. Read it. A work with "
                "such a notice can't back a claim, so remove the entry and the claims that cite it"
            )
    record = check.get("record")
    licensed = isinstance(record, dict) and registry.has_cc_license(record=record)
    if service == "crossref" and custom["accessLevel"] == "paywalled" and licensed:
        problems.append(
            f"{rid}: Crossref registers a Creative Commons license for {check.get('id')}, and the "
            "ledger says paywalled. Set accessLevel to open-access"
        )
    return problems


def link_check_problems(records: list[dict], links: dict) -> list[str]:
    """The snapshot's link checks against the links the ledger carries now, both ways"""
    problems: list[str] = []
    by_id = {rec["id"]: rec for rec in records}
    for citekey, kinds in sorted(links.items()):
        targets = registry.link_targets(entry=by_id[citekey]) if citekey in by_id else {}
        for kind, check in sorted((kinds if isinstance(kinds, dict) else {}).items()):
            url = check.get("url") if isinstance(check, dict) else None
            if targets.get(kind) != url:
                now = f"now has {targets[kind]}" if kind in targets else "has no such link"
                problems.append(
                    f"{citekey}: the snapshot checked {kind} {url}, and the ledger {now}. "
                    f"Run: uv run --no-project scripts/check-sources.py {citekey} --write"
                )
    for rec in records:
        for kind, url in registry.link_targets(entry=rec).items():
            if kind not in (links.get(rec["id"]) or {}):
                problems.append(
                    f"{rec['id']}: {kind} {url} has no link check in {REGISTRY.name}. "
                    f"Run: uv run --no-project scripts/check-sources.py {rec['id']} --write"
                )
    return problems


def check_registry(records: list[dict], snapshot: dict) -> None:
    """The registry labels can't promise more than the last registry check showed.

    Each verified entry needs a check by the registry its label names, of its own DOI, ISBN or PMID,
    that answered 200 with every field in agreement and the ledger's fields unchanged since. A
    retraction, withdrawal, removal or expression of concern against a cited work, or a Creative
    Commons license on an entry marked paywalled, fails. A correction renders under the entry. The
    links in the snapshot have to be the ledger's links, and every link without a DOI has a check
    """
    records_checked, links = snapshot.get("records"), snapshot.get("links")
    if (
        snapshot.get("format") != registry.SNAPSHOT_FORMAT
        or not isinstance(records_checked, dict)
        or not isinstance(links, dict)
    ):
        raise CorpusError(
            problems=[f"{REGISTRY.name} must be a format {registry.SNAPSHOT_FORMAT} snapshot with records and links"],
            fix=SOURCES_FIX,
        )
    problems: list[str] = []
    for rec in records:
        problems.extend(registry_problems(rec=rec, check=records_checked.get(rec["id"])))
    problems.extend(link_check_problems(records=records, links=links))
    if problems:
        raise CorpusError(problems=problems)


def clipped(text: str) -> str:
    """One line of at most SHOWN_CHARS characters"""
    line = " ".join(text.split())
    return line if len(line) <= SHOWN_CHARS else line[: SHOWN_CHARS - 3].rstrip() + "..."


def figure_pattern(number: str) -> re.Pattern:
    """Matches a normalized quote text that prints the number.

    The digits have to stand alone (5 isn't 5.5 or 15), and a whole number from 0 to 20 also counts
    as an English word, in any case
    """
    pattern = r"(?<![\d.])" + re.escape(number) + r"(?!\d|\.\d)"
    if re.fullmatch(r"0|[1-9]\d?", number) and int(number) < len(NUMBER_WORDS):
        return re.compile(pattern + r"|\b" + NUMBER_WORDS[int(number)] + r"\b", re.IGNORECASE)
    return re.compile(pattern)


def readme_figure_gaps(records: list[dict], readme: str) -> list[str]:
    """The (sentence, number) pairs of the README that no quote of the sentence's sources prints.

    In each README block that links an entry, every link becomes a marker that carries its key and
    the block splits into sentences. In a sentence with a marker, each distinct number outside the
    markers has to appear in a normalized quote of one of the entries the sentence links. A number the
    source doesn't print, such as a sum or a rounding, counts until the sentence says where it comes from
    """
    texts = normalized_quotes(records=records)
    gaps: list[str] = []
    for line, block in prose_blocks(text=readme):
        if not any(ID_RE.match(key) for key in README_LINK.findall(block)):
            continue
        marked = README_LINK.sub(lambda link: f"{MARK_OPEN}{link.group(1)}{MARK_CLOSE}", block)
        for sentence in SENTENCE_BREAK.split(strip_markup(block=marked)):
            marked_keys = dict.fromkeys(MARKER_RE.findall(sentence))
            cited = [key for key in marked_keys if ID_RE.match(key) and key in texts]
            if not cited:
                continue
            bare = MARKER_RE.sub(" ", sentence)
            shown = clipped(text=MARKER_RE.sub(lambda mark: f"[{mark.group(1)}]", sentence))
            for number in sorted(set(NUMBER_RE.findall(bare)), key=lambda digits: (float(digits), digits)):
                printed = figure_pattern(number=number)
                if not any(printed.search(text) for key in cited for text in texts[key]):
                    gaps.append(f'{README.name} line {line}: {number} in "{shown}"')
    return gaps


def unbacked_claims(rec: dict) -> list[int]:
    """The indexes of the claims that no quote of the entry points at"""
    backed = {quote["claim"] for quote in verbatim.quotes_of(record=rec)}
    return [number for number in range(len(rec["custom"]["backsClaims"])) if number not in backed]


def gap_items(records: list[dict], receipts: dict, readme: str | None) -> dict[str, list[str]]:
    """The items behind each gap count, by the count's name.

    quotesWithoutReceipt counts each distinct quote once. claimsWithoutQuote includes the claims of a
    catalog-record entry, since such an entry carries no quotes. linksWithoutArchive is each URL with
    no DOI and each open copy that has no Wayback capture
    """
    index = receipt_index(records=records, receipts=receipts)
    items: dict[str, list[str]] = {name: [] for name in COUNT_NAMES}
    for rec in records:
        rid, custom = rec["id"], rec["custom"]
        if custom["verification"] == "unverified":
            items["unverifiedEntries"].append(rid)
        seen: set[str] = set()
        for quote in verbatim.quotes_of(record=rec):
            key = verbatim.quote_key(text=quote["text"])
            if key not in index[rid] and key not in seen:
                items["quotesWithoutReceipt"].append(f'{rid}: "{verbatim.quote_preview(text=quote["text"])}"')
            seen.add(key)
        for number in unbacked_claims(rec=rec):
            items["claimsWithoutQuote"].append(
                f"{rid} claim {number}: {clipped(text=custom['backsClaims'][number])}"
            )
        for kind, url in registry.link_targets(entry=rec).items():
            if not custom.get(registry.ARCHIVE_FIELDS[kind]):
                items["linksWithoutArchive"].append(f"{rid} {kind}: {url}")
    if readme is not None:
        items["readmeFiguresWithoutQuote"] = readme_figure_gaps(records=records, readme=readme)
    return items


def load_baseline(fix: str = BASELINE_FIX) -> dict[str, int]:
    """The counts in the baseline file, or a CorpusError when it is missing or isn't a baseline"""
    baseline = load_tracked(path=BASELINE, fix=fix)
    counts = baseline.get("counts")
    problems: list[str] = []
    if baseline.get("format") != BASELINE_FORMAT:
        problems.append(f"{BASELINE.name} must have format {BASELINE_FORMAT}")
    if not isinstance(counts, dict):
        problems.append(f"{BASELINE.name} must have a counts object")
    else:
        for name in COUNT_NAMES:
            value = counts.get(name)
            if isinstance(value, bool) or not isinstance(value, int) or value < 0:
                problems.append(f"{BASELINE.name}: counts.{name} must be a whole number")
        for name in sorted(set(counts) - set(COUNT_NAMES)):
            problems.append(f"{BASELINE.name}: counts.{name} is no gap count")
    if problems:
        raise CorpusError(problems=problems, fix=fix)
    return {name: counts[name] for name in COUNT_NAMES}


def count_differences(counts: dict[str, int], held: dict[str, int]) -> list[tuple[str, int, int]]:
    """(name, current, baseline) for each gap count that isn't its baseline"""
    return [(name, counts[name], held[name]) for name in COUNT_NAMES if counts[name] != held[name]]


def check_baseline(counts: dict[str, int], held: dict[str, int]) -> None:
    """The gap counts may fall and may not rise.

    A count below its baseline is progress that the baseline has to record, or the next regression
    would hide in the room it left
    """
    problems = []
    for name, current, baseline in count_differences(counts=counts, held=held):
        if current > baseline:
            problems.append(
                f"{name} is {current}, over the baseline of {baseline}. Close the gap. List the items "
                "with: uv run --no-project scripts/generate-citations.py --list-gaps"
            )
        else:
            problems.append(
                f"{name} is {current}, under the baseline of {baseline}. Lower the baseline so the "
                f"gain holds: {BASELINE_FIX}"
            )
    if problems:
        raise CorpusError(problems=problems)


def write_baseline(counts: dict[str, int], allow_raise: bool) -> int:
    """Records the current gap counts. A count that would rise is refused unless allow_raise is set"""
    held: dict[str, int] | None = None
    if BASELINE.exists():
        try:
            held = load_baseline(fix=f"delete {BASELINE.name} and run this again to start a new baseline")
        except CorpusError as exc:
            print(f"{BASELINE.name} is no baseline to compare with\n{exc}", file=sys.stderr)
            return 1
    raised = []
    if held is not None and not allow_raise:
        raised = [name for name in COUNT_NAMES if counts[name] > held[name]]
    if raised:
        print(f"{BASELINE.name}: refusing to raise a count without --raise", file=sys.stderr)
        for name in raised:
            print(f"  - {name} would go from {held[name]} to {counts[name]}", file=sys.stderr)
        return 1
    text = json.dumps({"format": BASELINE_FORMAT, "counts": counts}, indent=2) + "\n"
    BASELINE.write_text(text, encoding="utf-8")
    print(f"Wrote {BASELINE.name}:")
    for name in COUNT_NAMES:
        print(f"  {name}: {'new' if held is None else held[name]} -> {counts[name]}")
    return 0


def list_gaps(records: list[dict], readme: str | None) -> int:
    """Prints the items behind every nonzero gap count. With no receipts file every quote counts as unreceipted"""
    try:
        receipts = load_tracked(path=RECEIPTS, fix=RECEIPTS_FIX)
    except CorpusError:
        print(f"{RECEIPTS.name} is missing or unreadable, so every quote counts as unreceipted", file=sys.stderr)
        receipts = {}
    items = gap_items(records=records, receipts=receipts, readme=readme)
    for name in COUNT_NAMES:
        if items[name]:
            print(f"{name} ({len(items[name])})")
            for item in items[name]:
                print(f"  - {item}")
    if not any(items.values()):
        print("No gaps.")
    return 0


def slugify(heading: str) -> str:
    """GitHub's heading-anchor rule, reduced to what these headings use.

    Lowercase, drop everything that isn't a word character, space or hyphen,
    then spaces to hyphens. The corpus headings are plain prose, so the full
    GitHub algorithm (duplicate suffixes, emoji, HTML) buys nothing here
    """
    slug = heading.strip().lower()
    slug = re.sub(r"[^\w \-]", "", slug)
    return slug.replace(" ", "-")


def check_binary_deep_link(rendered: str) -> None:
    """The tray menu's URL is the only reference to this document that ships.

    Every other link into the corpus lives in a file the reader can see is
    stale. This one is compiled into a binary that stays installed for months,
    so it has to keep pointing into the corpus, and if it ever carries an
    anchor again that anchor has to name a heading that still exists.
    Renaming a heading is allowed as long as it doesn't happen silently

    The menu may point either at the file on GitHub or at docs/citations.html,
    which fetches that same file from `main` and renders it. Both are the
    corpus, so both are accepted. The URL currently opens the page at the top,
    with no anchor, so the reader meets the reading guide before the entries
    """
    if not TRAY.exists():
        return

    m = re.search(r'RESEARCH_URL: &str =\s*\n?\s*"([^"]+)"', TRAY.read_text(encoding="utf-8"))
    if not m:
        raise CorpusError(
            f"  - {TRAY.relative_to(ROOT)} no longer defines RESEARCH_URL. The shipped\n"
            "    binary has lost its only pointer at the evidence"
        )

    url = m.group(1)
    path, _, fragment = url.partition("#")
    if not path.endswith(("/docs/CITATIONS.md", "/exhale/citations.html")):
        raise CorpusError(f"  - RESEARCH_URL points outside the corpus: {url}")

    if not fragment:
        return
    anchors = {slugify(h) for h in re.findall(r"^#{1,6} +(.+)$", rendered, re.M)}
    if fragment not in anchors:
        raise CorpusError(
            f"  - RESEARCH_URL anchor #{fragment} matches no heading in {OUT.name}.\n"
            f"    The tray menu would drop the reader at the top of the file instead"
        )


def authors(rec: dict) -> str:
    people = rec.get("author") or []
    return "; ".join(
        f'{p.get("family", "?")}, {p.get("given", "?")}' for p in people
    )


def year(rec: dict) -> str:
    parts = (rec.get("issued", {}).get("date-parts") or [[None]])[0]
    return str(parts[0]) if parts and parts[0] else "n.d."


def locator(rec: dict) -> str:
    """Volume(issue): pages, omitting whatever the record doesn't have"""
    bits = ""
    if rec.get("volume"):
        bits += str(rec["volume"])
        if rec.get("issue"):
            bits += f'({rec["issue"]})'
    if rec.get("page"):
        bits += (": " if bits else "") + str(rec["page"])
    if not bits and rec.get("number-of-pages"):
        bits = f'{rec["number-of-pages"]} pp'
    return bits


def archive_suffix(link: str | None) -> str:
    """The end of a URL or open-copy line when the link has a Wayback capture, else nothing"""
    if not link:
        return ""
    return f" ([archived]({link.replace('(', '%28').replace(')', '%29').replace(' ', '%20')}))"


def quote_lines(quotes: list[dict], found: dict[str, dict], indent: str) -> list[str]:
    """One line per quote: its words, its locator and a flag when no receipt vouches for it.

    The words sit between straight double quotes, so a double quote inside them becomes a single
    quote and the characters markdown would format are escaped
    """
    lines = []
    for quote in quotes:
        words = re.sub(r"([*_`\\])", r"\\\1", quote["text"].replace('"', "'"))
        flag = "" if verbatim.quote_key(text=quote["text"]) in found else " (no receipt)"
        lines.append(f'{indent}- "{words}" ({quote["locator"]}){flag}')
    return lines


def checked_line(quotes: list[dict], found: dict[str, dict]) -> str:
    """How many of an entry's quotes have a receipt, against which sources and as of when"""
    receipted = [found[key] for key in (verbatim.quote_key(text=q["text"]) for q in quotes) if key in found]
    line = f"- Quotes checked: {len(receipted)} of {len(quotes)}"
    if receipted:
        sources = collections.Counter(receipt["source"] for receipt in receipted)
        line += f" against {' and '.join(source for source, _ in sources.most_common())}"
        line += f", last on {max(receipt['date'] for receipt in receipted)}"
    return line


def render_entry(rec: dict, found: dict[str, dict], updates: list[dict]) -> list[str]:
    c = rec["custom"]
    quotes = verbatim.quotes_of(record=rec)
    lines = [f'#### `{rec["id"]}`', ""]

    # Titles that already end in their own punctuation shouldn't collect a second period
    tail = "" if rec["title"].rstrip().endswith(("?", "!", ".")) else "."
    # Books name the edition the ISBN resolves to, so the reader can find the same one
    if rec.get("edition"):
        edition = rec["edition"]
        tail = f", {edition}" + ("" if edition.endswith(".") else tail)
    # Journal articles carry a container title, books a publisher and place
    source = rec.get("container-title") or ": ".join(
        p for p in (rec.get("publisher-place"), rec.get("publisher")) if p
    )
    who = authors(rec)
    # An author list ending in an initial already supplies its own period
    who_sep = "" if who.endswith(".") else "."
    head = f'{who}{who_sep} ({year(rec)}). *{rec["title"]}*{tail} {source}'
    loc = locator(rec)
    lines.append(head + (f" {loc}" if loc else ""))
    lines.append("")

    # An entry with no DOI links its URL, and the Wayback capture of that page follows the link
    archived = archive_suffix(link=c.get("archived"))
    if rec.get("DOI"):
        lines.append(f'- DOI: [{rec["DOI"]}](https://doi.org/{rec["DOI"]})')
    elif rec.get("PMID"):
        lines.append(f'- PMID: [{rec["PMID"]}]({rec["URL"]}) (no DOI exists){archived}')
    elif rec.get("ISBN"):
        lines.append(f'- ISBN: {rec["ISBN"]} | [Open Library record]({rec["URL"]}){archived}')
    elif rec.get("URL"):
        lines.append(f'- URL: <{rec["URL"]}>{archived}')
    if c.get("openCopy"):
        lines.append(f'- Open copy: <{c["openCopy"]}>{archive_suffix(link=c.get("openCopyArchived"))}')
    # Notices the registry lists against the paper. Corrections render here and a retraction fails the build
    for notice in updates:
        when = f", {notice['date']}" if notice.get("date") else ""
        lines.append(
            f"- Updated by: {notice['type'].replace('_', ' ')} [{notice['doi']}](https://doi.org/{notice['doi']}){when}"
        )

    tier = c.get("evidenceTier")
    tier_text = f"evidence tier **{tier}**" if tier else "no evidence tier (not a study)"
    lines.append(
        f'- Verification: {c["verification"]} | Access: {c["accessLevel"]} | '
        f'Read: {READ_DEPTH_LABELS[c["readDepth"]]} | {tier_text}'
    )

    lines.append("- Backs:")
    for number, claim in enumerate(c["backsClaims"]):
        lines.append(f"  - {claim}")
        backing = [q for q in quotes if q["claim"] == number]
        lines.extend(quote_lines(quotes=backing, found=found, indent="    "))

    # A quote tied to no claim backs the caveat, or a figure the README attributes to the entry
    loose = [q for q in quotes if q["claim"] is None]
    if c.get("caveat"):
        lines.append(f'- Caveat: {c["caveat"]}')
        lines.extend(quote_lines(quotes=loose, found=found, indent="  "))
    elif loose:
        lines.append("- Other quotes:")
        lines.extend(quote_lines(quotes=loose, found=found, indent="  "))
    if quotes:
        lines.append(checked_line(quotes=quotes, found=found))

    lines.append("")
    return lines


def tally(records: list[dict], key: str) -> list[str]:
    labels = []
    for r in records:
        value = r["custom"].get(key)
        labels.append(value if value is not None else "null (not a study)")
    counts = collections.Counter(labels)
    rows = [f"| {label} | {n} |" for label, n in sorted(counts.items())]
    rows.append(f"| **total** | **{len(records)}** |")
    return rows


def render_counts(records: list[dict]) -> str:
    out: list[str] = ["### Counts", ""]
    for key, heading in (
        ("verification", "Verification"),
        ("accessLevel", "Access level"),
        ("readDepth", "Read depth"),
        ("evidenceTier", "Evidence tier"),
    ):
        out += [f"| {heading} | n |", "|---|---|", *tally(records, key), ""]
    return "\n".join(out).rstrip()


def render_corpus(
    records: list[dict], receipts: dict[str, dict[str, dict]], updates: dict[str, list[dict]]
) -> str:
    out: list[str] = []
    for group, heading in GROUPS:
        in_group = [r for r in records if r["custom"]["group"] == group]
        if not in_group:
            continue
        plural = "source" if len(in_group) == 1 else "sources"
        out += ["---", "", f"## {heading}", "", f"{len(in_group)} {plural}.", ""]
        for rec in in_group:
            out += render_entry(rec=rec, found=receipts[rec["id"]], updates=updates.get(rec["id"], []))
    return "\n".join(out).rstrip()


def render_summary(records: list[dict], receipts: dict[str, dict[str, dict]]) -> str:
    counts = collections.Counter(r["custom"]["verification"] for r in records)
    parts = [
        f"{counts[key]} {VERIFICATION_LABELS[key]}"
        for key in VERIFICATION_LABELS
        if counts[key]
    ]
    depth = collections.Counter(r["custom"]["readDepth"] for r in records)
    tiered_out = sum(1 for r in records if r["custom"].get("evidenceTier") == "E")
    claims = sum(len(r["custom"]["backsClaims"]) for r in records)
    backed = claims - sum(len(unbacked_claims(rec=r)) for r in records)
    quotes = [(r["id"], q) for r in records for q in verbatim.quotes_of(record=r)]
    receipted = sum(1 for citekey, q in quotes if verbatim.quote_key(text=q["text"]) in receipts[citekey])
    found = f"all {len(quotes)}" if receipted == len(quotes) > 0 else f"{receipted} of them"
    return (
        f"{len(records)} sources: {', '.join(parts)}. "
        f"{depth['full-text']} were read in full, {depth['abstract']} from the abstract only, "
        f"and {depth['record']} are catalog records only. Each entry says which. "
        f"{tiered_out} aren't peer-reviewed and are tiered E so they can back lineage but "
        f"never a claim. "
        f"{len(quotes)} quotes back {backed} of the {claims} claims, and {found} were found in the "
        f"source's own text."
    )


def check_evidence(records: list[dict], readme: str | None, notes: str) -> tuple[Failures, dict, dict | None]:
    """Runs the checks that hold the ledger to its receipts, the Rust sources and the registry snapshot.

    Answers the failures, the receipts file and the registry snapshot. A missing file is a failure
    that names the command which writes it, and the checks that read it go on as if it held nothing
    """
    failures = Failures()
    receipts = failures.attempt(heading=UNAVAILABLE, step=load_tracked, path=RECEIPTS, fix=RECEIPTS_FIX)
    if receipts is None:
        receipts = {}
    else:
        failures.attempt(
            heading="receipts out of step with the quotes", step=check_receipts, records=records, receipts=receipts
        )
    failures.attempt(
        heading="readDepth not shown by the receipts", step=check_read_depth, records=records, receipts=receipts
    )
    failures.attempt(
        heading="pinned figures out of step with the corpus",
        step=check_pinned_figures, records=records, receipts=receipts,
    )
    failures.attempt(
        heading="quote marks around words no quote has",
        step=check_prose_quotes, records=records, readme=readme, notes=notes,
    )
    snapshot = failures.attempt(heading=UNAVAILABLE, step=load_tracked, path=REGISTRY, fix=SOURCES_FIX)
    if snapshot is not None:
        failures.attempt(
            heading="registry snapshot out of step with the ledger",
            step=check_registry, records=records, snapshot=snapshot,
        )
    return failures, receipts, snapshot


def parse_args(argv: list[str]) -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        prog="generate-citations.py",
        description="Render docs/CITATIONS.md from the CSL-JSON corpus and the hand-written notes",
    )
    mode = parser.add_mutually_exclusive_group()
    mode.add_argument("--check", action="store_true", help="fail when CITATIONS.md is stale, write nothing")
    mode.add_argument("--write-baseline", action="store_true", help="record the gap counts, write nothing else")
    mode.add_argument("--list-gaps", action="store_true", help="list the items behind every nonzero gap count")
    parser.add_argument("--raise", dest="allow_raise", action="store_true", help="let --write-baseline raise a count")
    args = parser.parse_args(argv)
    if args.allow_raise and not args.write_baseline:
        parser.error("--raise goes with --write-baseline")
    return args


def main() -> int:
    args = parse_args(argv=sys.argv[1:])
    check = args.check

    readme = None
    try:
        records = load_corpus()
        check_cross_references(records)
        check_preset_citekeys(records)
        check_unsupported_phrases()
        notes = NOTES.read_text(encoding="utf-8")
        check_note_links(records, notes, NOTES.name)
        if README.exists():
            readme = README.read_text(encoding="utf-8")
            check_note_links(records, readme, README.name)
            check_readme_counts(records, readme)
    except CorpusError as exc:
        print(f"{CORPUS.name}: invalid corpus\n{exc}", file=sys.stderr)
        return 1

    if args.list_gaps:
        return list_gaps(records=records, readme=readme)

    failures, receipts, snapshot = check_evidence(records=records, readme=readme, notes=notes)
    items = gap_items(records=records, receipts=receipts, readme=readme)
    counts = {name: len(listed) for name, listed in items.items()}
    if args.write_baseline:
        if failures.report():
            return 1
        return write_baseline(counts=counts, allow_raise=args.allow_raise)
    held = failures.attempt(heading=UNAVAILABLE, step=load_baseline)
    if held is not None and check:
        failures.attempt(heading="gap counts differ from the baseline", step=check_baseline, counts=counts, held=held)
    elif held is not None:
        # Write mode only says where the counts stand, since the baseline moves when someone decides it should
        differing = count_differences(counts=counts, held=held)
        if differing:
            print("Gap counts that differ from the baseline (current / baseline):")
            for name, now, baseline in differing:
                print(f"  {name}: {now} / {baseline}")
    if failures.report():
        return 1

    index = receipt_index(records=records, receipts=receipts)
    updates = {
        citekey: check["updates"]
        for citekey, check in snapshot["records"].items()
        if isinstance(check, dict) and check.get("updates")
    }
    rendered = notes
    for marker, replacement in (
        ("<!-- SUMMARY -->", render_summary(records=records, receipts=index)),
        ("<!-- COUNTS -->", render_counts(records)),
        ("<!-- CORPUS -->", render_corpus(records=records, receipts=index, updates=updates)),
    ):
        if marker not in rendered:
            print(f"{NOTES.name}: missing marker {marker}", file=sys.stderr)
            return 1
        rendered = rendered.replace(marker, replacement)

    if not rendered.endswith("\n"):
        rendered += "\n"

    try:
        check_binary_deep_link(rendered)
    except CorpusError as exc:
        print(f"{OUT.name}: broken deep link\n{exc}", file=sys.stderr)
        return 1

    if check:
        current = OUT.read_text(encoding="utf-8") if OUT.exists() else ""
        if current != rendered:
            print(
                f"{OUT.name} is stale. Run: uv run --no-project scripts/generate-citations.py",
                file=sys.stderr,
            )
            return 1
        print(f"{OUT.name} is up to date ({len(records)} sources).")
        return 0

    OUT.write_text(rendered, encoding="utf-8")
    print(f"Wrote {OUT.name}: {len(records)} sources.")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
