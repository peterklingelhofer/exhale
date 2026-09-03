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

import collections
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
# Patterns the binary offers as one click. See check_preset_citekeys
PRESETS = ROOT / "rust/crates/exhale-core/src/presets.rs"

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
        # Absent means citable; only an explicit `false` blocks a record
        # from backing something the binary ships. Written as an opt-OUT so
        # the corpus does not need touching for the common case, and so the
        # blocklist is greppable as four lines rather than inferred from
        # forty-four omissions
        if custom.get("inAppCitable", False) not in (True, False):
            problems.append(f"{rid}: inAppCitable must be true or false")

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


# Claims the corpus does not support, kept out of the surfaces that assert them.
#
# Scope is deliberate. Only surfaces that ASSERT are scanned: store listings and
# anything compiled into the binary. `docs/` and `README.md` are exempt, because
# the gaps ledger has to be able to name a claim in order to explain what the
# evidence actually says about it.
#
# This is not hypothetical. `snapcraft.yaml` went on shipping the parasympathetic
# claim to the Snap Store for two days after the README had stopped making it,
# because a store listing is edited in a different place from a README and
# nothing connected the two
UNSUPPORTED_PHRASES: list[tuple[str, str]] = [
    ("engage the parasympathetic nervous system", "gaps ledger 4: the cardiac evidence splits 2-for / 1-against / 1-null"),
    ("engages the parasympathetic nervous system", "gaps ledger 4: the cardiac evidence splits 2-for / 1-against / 1-null"),
    ("breathe more shallowly", "gaps ledger 1: the measured finding is faster and chest-high, not shallower"),
    ("screen apnea", "gaps ledger 1: no peer-reviewed source; the measured effect points the other way"),
    ("email apnea", "gaps ledger 1: no peer-reviewed source; the measured effect points the other way"),
]

# Surfaces where one of the phrases above would be an assertion rather than a
# discussion of one. Missing files are skipped: the Microsoft handoff is
# untracked, so it is present locally and absent in CI
ASSERTING_SURFACES: list[str] = [
    "snap/snapcraft.yaml",
    "rust/packaging/windows/AppxManifest.xml",
    "rust/packaging/windows/store-listing.md",
    # Untracked scratch file: present locally, absent in CI, skipped either way
    "MICROSOFT_STORE_HANDOFF.md",
]
ASSERTING_GLOBS: list[str] = ["rust/crates/**/*.rs"]


def check_unsupported_phrases() -> None:
    """Fail if an unsupported claim survives on a surface that asserts it."""
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


def check_preset_citekeys(records: list[dict]) -> None:
    """Every pattern the app offers has to be traceable to a live record.

    The preset list is the one place the binary makes a *selection* from the
    literature rather than a statement about it, and a selection is an
    argument whether or not it is worded as one. Offering five patterns says
    these five are worth a click, so each carries a citekey that never reaches
    the screen and exists only to fail this check.

    Four conditions, and the third and fourth are the ones that earn their
    keep. A record downgraded to tier E is lineage-only and cannot license a
    default. A record marked `inAppCitable: false` is one the professor
    blocklisted from a store-reviewed binary, which is a different judgement
    from whether it is good evidence: `fincham2023` is the strongest warrant
    in the corpus and is on that list
    """
    if not PRESETS.exists():
        return

    text = PRESETS.read_text(encoding="utf-8")
    # `citekey: "..."` inside the const table. Comments mention citekeys in
    # prose, so anchor on the field name rather than scanning for the shape
    found = re.findall(r'citekey:\s*"([^"]*)"', text)
    if not found:
        raise CorpusError(
            f"  - {PRESETS.relative_to(ROOT)} defines no preset citekeys; the shipped\n"
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
                f"preset citekey '{key}' is group '{custom['group']}'; a preset has to be "
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
        raise CorpusError("\n".join("  - " + p for p in problems))


def check_readme_counts(records: list[dict], text: str) -> None:
    """The README states the size of the corpus. Make it prove it.

    These numbers were wrong for the entire life of the previous commit: the
    README advertised 42 sources verified 40 / 2 while the corpus held 48
    verified 45 / 2 / 1. Nothing caught it, because a number in prose looks
    exactly like a number in prose. Undercounting is the harmless direction and
    it is still the project's front door claiming a provenance figure it had
    not checked, which is the specific failure this whole apparatus exists to
    prevent
    """
    counts = collections.Counter(r["custom"]["verification"] for r in records)
    # Each phrase is matched wherever it appears, so the lede and the research
    # section cannot drift apart from each other either
    expected = [
        (r"(\d+)\s+sources", len(records), "total sources"),
        (r"(\d+)\s+verified references", len(records), "verified references"),
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
                problems.append(f"README.md says {got} for {what}; the corpus holds {want}")
    if problems:
        raise CorpusError("\n".join("  - " + p for p in problems))


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

    The menu may point either at the file on GitHub or at docs/citations.html,
    which fetches that same file from `main` and renders it. Both are the
    corpus and both break identically when an anchor moves, so both are
    accepted and the anchor is checked the same way either way
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
    if not path.endswith(("/docs/CITATIONS.md", "/exhale/citations.html")):
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


# How each verification status is named in the one-line summary. Driven off the
# same enum the validator uses, so adding a source verified some new way is a
# KeyError here rather than a source that silently vanishes from the count. The
# previous hand-written version added up to 47 of 48 for exactly that reason
VERIFICATION_LABELS = {
    "crossref-verified":    "Crossref-verified",
    "openlibrary-verified": "verified against Open Library",
    "pubmed-verified":      "verified against PubMed",
    "unverified":           "unverified",
}


def render_summary(records: list[dict]) -> str:
    counts = collections.Counter(r["custom"]["verification"] for r in records)
    parts = [
        f"{counts[key]} {VERIFICATION_LABELS[key]}"
        for key in VERIFICATION_LABELS
        if counts[key]
    ]
    unread = sum(1 for r in records if "NUMBERS NOT READ" in (r["custom"].get("caveat") or ""))
    tiered_out = sum(1 for r in records if r["custom"].get("evidenceTier") == "E")
    return (
        f"{len(records)} sources: {', '.join(parts)}. "
        f"{unread} are cited from their abstract only and say so, and "
        f"{tiered_out} are not peer-reviewed and are tiered E so they can back lineage but "
        f"never a claim."
    )


def main() -> int:
    check = "--check" in sys.argv[1:]

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
