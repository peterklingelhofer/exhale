#!/usr/bin/env python3
"""Render docs/CITATIONS.md from the CSL-JSON corpus and the hand-written notes.

Single source of truth:
  - docs/CITATIONS.csl.json  bibliographic records + provenance metadata
  - docs/citations-notes.md  preamble, reading guide, gaps ledger

docs/CITATIONS.md is generated from both and must never be hand-edited.

Usage:
    uv run --no-project scripts/generate-citations.py            # write
    uv run --no-project scripts/generate-citations.py --check    # fail on drift

Stdlib only, so it runs anywhere the repo is checked out
"""

from __future__ import annotations

import json
import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
DOCS = ROOT / "docs"
CORPUS = DOCS / "CITATIONS.csl.json"
NOTES = DOCS / "citations-notes.md"
OUT = DOCS / "CITATIONS.md"
README = ROOT / "README.md"
# The one place the shipped binary names this document. See check_binary_deep_link
TRAY = ROOT / "rust/crates/exhale-app/src/tray.rs"

# Section order in the rendered document. Each group answers one question the
# app actually raises, rather than following the shape of the literature
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

VERIFICATIONS = {"crossref-verified", "openlibrary-verified", "pubmed-verified", "unverified"}
ACCESS_LEVELS = {"open-access", "paywalled"}
TIERS = {"A", "B", "C", "D", "E", None}

ID_RE = re.compile(r"^[a-z][a-z0-9]+[0-9]{4}-[a-z0-9-]+$")
# Same shape, unanchored, for spotting citekeys named inside prose
MENTION_RE = re.compile(r"\b[a-z][a-z0-9]+[0-9]{4}-[a-z0-9-]+\b")


class CorpusError(Exception):
    pass


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
                    f"{rid}: citekey year {keyed.group(1)} does not match issued year {issued}"
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

        claims = custom.get("backsClaims")
        if not isinstance(claims, list) or not claims:
            problems.append(f"{rid}: backsClaims must be a non-empty list")

        # A record with no DOI cannot honestly call itself Crossref-verified
        if custom.get("verification") == "crossref-verified" and not rec.get("DOI"):
            problems.append(f"{rid}: crossref-verified but carries no DOI")

    if problems:
        raise CorpusError("\n".join("  - " + p for p in problems))

    return sorted(records, key=lambda r: r["id"])


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
        raise CorpusError("\n".join("  - " + p for p in problems))


# Claims this project has retracted. The gaps ledger records WHY each was
# withdrawn; this list is what stops a withdrawn claim surviving somewhere the
# ledger has no jurisdiction.
#
# Scope is deliberate. Only surfaces that ASSERT are scanned: store listings and
# anything compiled into the binary. `docs/` and `README.md` are exempt because
# they are the retraction record itself and have to be able to quote what they
# withdraw. snapcraft.yaml went on shipping the parasympathetic claim to the
# Snap Store for two days after the README retracted it, which is the exact
# failure this catches
RETRACTED_PHRASES: list[tuple[str, str]] = [
    ("engage the parasympathetic nervous system", "gaps ledger 4: split 2-for / 1-against / 1-null"),
    ("engages the parasympathetic nervous system", "gaps ledger 4: split 2-for / 1-against / 1-null"),
    ("breathe more shallowly", "gaps ledger 1: unsupported as stated; the finding is faster and chest-high"),
    ("screen apnea", "gaps ledger 1: no peer-reviewed source"),
    ("email apnea", "gaps ledger 1: no peer-reviewed source"),
]

# Surfaces where one of the phrases above would be an assertion rather than a
# citation of one. Missing files are skipped: the Microsoft handoff is untracked,
# so it is present locally and absent in CI
ASSERTING_SURFACES: list[str] = [
    "snap/snapcraft.yaml",
    "rust/packaging/windows/AppxManifest.xml",
    "rust/packaging/windows/store-listing.md",
    # Untracked scratch file: present locally, absent in CI, skipped either way
    "MICROSOFT_STORE_HANDOFF.md",
]
ASSERTING_GLOBS: list[str] = ["rust/crates/**/*.rs"]


def check_retracted_phrases() -> None:
    """Fail if a withdrawn claim survives on a surface that asserts it."""
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
        for phrase, why in RETRACTED_PHRASES:
            if phrase in lowered:
                line = lowered[: lowered.index(phrase)].count("\n") + 1
                rel = path.relative_to(ROOT)
                problems.append(f"{rel}:{line} still asserts \"{phrase}\" ({why})")

    if problems:
        raise CorpusError("\n".join("  - " + p for p in problems))


def check_note_links(records: list[dict], text: str, where: str) -> None:
    """Prose links to entries by anchor. Catch the ones that rot.

    Both the gaps ledger and the README deep-link into the corpus by citekey.
    A rename silently breaks every one of them, and the README is the project's
    front door, so it is checked on exactly the same footing as the notes
    """
    known = {r["id"] for r in records}
    # Only anchors shaped like a citekey are entry references; the rest are
    # ordinary intra-document links to headings in the notes
    # The notes link as (#citekey); the README links as (docs/CITATIONS.md#citekey)
    anchors = re.findall(r"\]\((?:docs/CITATIONS\.md)?#([a-z0-9-]+)\)", text)
    referenced = {a for a in anchors if ID_RE.match(a)}
    dangling = sorted(referenced - known)
    if dangling:
        raise CorpusError(
            "\n".join(f"  - {where} links to unknown entry #{d}" for d in dangling)
        )


def slugify(heading: str) -> str:
    """GitHub's heading-anchor rule, reduced to what these headings use.

    Lowercase, drop everything that is not a word character, space or hyphen,
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
    so a renamed heading strands a user on the top of a 48-entry list at the
    exact moment they went looking for the limits. Renaming the heading is
    allowed; renaming it silently is not
    """
    if not TRAY.exists():
        return

    m = re.search(r'RESEARCH_URL: &str =\s*\n?\s*"([^"]+)"', TRAY.read_text(encoding="utf-8"))
    if not m:
        raise CorpusError(
            f"  - {TRAY.relative_to(ROOT)} no longer defines RESEARCH_URL; the shipped\n"
            "    binary has lost its only pointer at the evidence"
        )

    url = m.group(1)
    path, _, fragment = url.partition("#")
    if not path.endswith("/docs/CITATIONS.md"):
        raise CorpusError(f"  - RESEARCH_URL points outside the corpus: {url}")

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
    """Volume(issue): pages, omitting whatever the record does not have."""
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


def render_entry(rec: dict) -> list[str]:
    c = rec["custom"]
    lines = [f'#### `{rec["id"]}`', ""]

    # Titles that already end in their own punctuation should not collect a second period
    tail = "" if rec["title"].rstrip().endswith(("?", "!", ".")) else "."
    # Journal articles carry a container title; books carry a publisher and place
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

    if rec.get("DOI"):
        lines.append(f'- DOI: [{rec["DOI"]}](https://doi.org/{rec["DOI"]})')
    elif rec.get("PMID"):
        lines.append(f'- PMID: [{rec["PMID"]}]({rec["URL"]}) (no DOI exists)')
    elif rec.get("ISBN"):
        lines.append(f'- ISBN: {rec["ISBN"]} | [Open Library record]({rec["URL"]})')
    elif rec.get("URL"):
        lines.append(f'- URL: <{rec["URL"]}>')

    tier = c.get("evidenceTier")
    tier_text = f"evidence tier **{tier}**" if tier else "no evidence tier (not a study)"
    lines.append(
        f'- Verification: {c["verification"]} | Access: {c["accessLevel"]} | {tier_text}'
    )

    lines.append("- Backs:")
    lines.extend(f"  - {claim}" for claim in c["backsClaims"])

    if c.get("caveat"):
        lines.append(f'- Caveat: {c["caveat"]}')

    lines.append("")
    return lines


def tally(records: list[dict], key: str) -> list[str]:
    counts: dict[str, int] = {}
    for r in records:
        value = r["custom"].get(key)
        label = value if value is not None else "null (not a study)"
        counts[label] = counts.get(label, 0) + 1
    rows = [f"| {label} | {n} |" for label, n in sorted(counts.items())]
    rows.append(f"| **total** | **{len(records)}** |")
    return rows


def render_counts(records: list[dict]) -> str:
    out: list[str] = ["### Counts", ""]
    for key, heading in (
        ("verification", "Verification"),
        ("accessLevel", "Access level"),
        ("evidenceTier", "Evidence tier"),
    ):
        out += [f"| {heading} | n |", "|---|---|", *tally(records, key), ""]
    return "\n".join(out).rstrip()


def render_corpus(records: list[dict]) -> str:
    out: list[str] = []
    for group, heading in GROUPS:
        in_group = [r for r in records if r["custom"]["group"] == group]
        if not in_group:
            continue
        plural = "source" if len(in_group) == 1 else "sources"
        out += ["---", "", f"## {heading}", "", f"{len(in_group)} {plural}.", ""]
        for rec in in_group:
            out += render_entry(rec)
    return "\n".join(out).rstrip()


def render_summary(records: list[dict]) -> str:
    verified = sum(
        1 for r in records if r["custom"]["verification"] == "crossref-verified"
    )
    catalogued = sum(
        1 for r in records if r["custom"]["verification"] == "openlibrary-verified"
    )
    unread = sum(1 for r in records if "NUMBERS NOT READ" in (r["custom"].get("caveat") or ""))
    tiered_out = sum(1 for r in records if r["custom"].get("evidenceTier") == "E")
    return (
        f"{len(records)} sources: {verified} Crossref-verified, {catalogued} verified against "
        f"Open Library. {unread} are cited from their abstract only and say so, and "
        f"{tiered_out} are not peer-reviewed and are tiered E so they can back lineage but "
        f"never a claim."
    )


def main() -> int:
    check = "--check" in sys.argv[1:]

    try:
        records = load_corpus()
        check_cross_references(records)
        check_retracted_phrases()
        notes = NOTES.read_text(encoding="utf-8")
        check_note_links(records, notes, NOTES.name)
        if README.exists():
            check_note_links(records, README.read_text(encoding="utf-8"), README.name)
    except CorpusError as exc:
        print(f"{CORPUS.name}: invalid corpus\n{exc}", file=sys.stderr)
        return 1

    rendered = notes
    for marker, replacement in (
        ("<!-- SUMMARY -->", render_summary(records)),
        ("<!-- COUNTS -->", render_counts(records)),
        ("<!-- CORPUS -->", render_corpus(records)),
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
