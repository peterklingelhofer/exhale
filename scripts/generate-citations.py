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

DOCS = Path(__file__).resolve().parent.parent / "docs"
CORPUS = DOCS / "CITATIONS.csl.json"
NOTES = DOCS / "citations-notes.md"
OUT = DOCS / "CITATIONS.md"

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

VERIFICATIONS = {"crossref-verified", "openlibrary-verified", "unverified"}
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


def check_note_links(records: list[dict], notes: str) -> None:
    """The gaps ledger links to entries by anchor. Catch the ones that rot."""
    known = {r["id"] for r in records}
    # Only anchors shaped like a citekey are entry references; the rest are
    # ordinary intra-document links to headings in the notes
    referenced = {
        a for a in re.findall(r"\]\(#([a-z0-9-]+)\)", notes) if ID_RE.match(a)
    }
    dangling = sorted(referenced - known)
    if dangling:
        raise CorpusError(
            "\n".join(f"  - {NOTES.name} links to unknown entry #{d}" for d in dangling)
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
    head = f'{authors(rec)}. ({year(rec)}). *{rec["title"]}*{tail} {source}'
    loc = locator(rec)
    lines.append(head + (f" {loc}" if loc else ""))
    lines.append("")

    if rec.get("DOI"):
        lines.append(f'- DOI: [{rec["DOI"]}](https://doi.org/{rec["DOI"]})')
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
        notes = NOTES.read_text(encoding="utf-8")
        check_note_links(records, notes)
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
