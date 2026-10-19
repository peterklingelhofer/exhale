#!/usr/bin/env python3
"""Check the quotes in docs/CITATIONS.csl.json against the text of their sources.

An entry's custom.quotes holds the source's own words. This script looks each quote up in
the cached text of its source and records the ones it finds as receipts in
docs/CITATIONS.receipts.json, which scripts/generate-citations.py holds to the ledger offline.
What counts as the same words (case, spacing, quote marks, dashes, wrapped lines, elisions)
is defined once, in scripts/verbatim.py

The cache
  scratch/research-cache/ is local and gitignored, since the texts are publisher and author
  copies. manifest.json maps a citekey to {"best", "variants": {name: {chars, source,
  sha256, label, strip}}}, and papers/<citekey>/<name>.txt holds each variant's text. A variant
  is usable when it isn't a stub (under 500 characters, a paywall or bot page) and, unless it
  is the abstract, when the entry's title occurs in it, which shows it is the right document.
  "strip": "line-numbers" on a variant that came from <name>.pdf makes --refresh-cache drop
  every line of the extracted text that holds only a number of one to four digits. An author
  manuscript prints a line number, and a page number, on a line of their own between lines of
  text, and a quote would need a [...] at each one where no word is missing

Receipts (docs/CITATIONS.receipts.json, tracked)
  {"format": 1, "receipts": {"<citekey>": [{"quote": "<quote key>", "preview": "<first 60
  characters>", "found": {"variant", "source", "scope", "chars", "textSha256", "date"}}]}}
  The quote key is verbatim.quote_key of the quote's text. scope is abstract when the quote is
  in the entry's abstract variant and body otherwise. variant is abstract for that scope, else
  the first usable variant (the manifest's best first) that holds the quote. source is the
  variant's label, chars the length of its text file, textSha256 the SHA-256 of its normalized
  text and date the day the quote was found. Receipts are sorted by citekey, preview, quote key.
  The locator decides where a quote has to be found. One that starts with the word Abstract
  ("Abstract", "Abstract, Significance Statement") needs the quote in the abstract variant, and
  any other needs it in a usable variant other than the abstract, whether or not the abstract has
  it too. A quote that only the other side has is found there, gets its receipt there and is flagged

Modes
  (default)         look every quote up and report found, partial, not found, no text cached
                    or stub text. Also lists the variants each entry skipped
  --strict          exit 1 when a quote with usable text is partial or not found, or a quote
                    is malformed or was found on the other side of the abstract from its locator
  --context         for a quote that isn't found, show where it breaks and the closest text,
                    and for a skipped variant, its first line
  --write-receipts  record the quotes found as receipts (docs/CITATIONS.receipts.json). A rerun
                    changes nothing, receipts of quotes this run didn't check stay and
                    receipts of quotes the ledger no longer carries are dropped
  --proposed DIR    overlay DIR/<citekey>.json ({"quotes": [...]}) on those entries for this run
                    and check the proposed quotes. Can't be combined with --write-receipts
  --live            check the receipts against Europe PMC's current text, with no cache. The
                    weekly job. Exit 1 when a receipted quote is missing from a text that was
                    fetched. A text that can't be fetched is reported and doesn't fail the run
  --summary FILE    with --live, append a markdown report to FILE
  --refresh-cache   rebuild the cache variants this script knows how to fetch (the Europe PMC
                    abstract and full text, else the Crossref abstract), re-extract every
                    cached PDF in reading order with pdftotext (minus the line numbers of a
                    variant marked strip) and recount chars and sha256 for every variant. It
                    never deletes a variant or changes another key
  --ledger PATH, --receipts PATH, --cache DIR   point at other copies, for testing
  citekey ...       limit the run to those entries

Usage:
    uv run --no-project scripts/check-quotes.py [options] [citekey ...]
    uv run --no-project scripts/check-quotes.py --proposed scratch/proposed-quotes \\
        --strict --context balban2023-cyclic-sighing

Requests go through scripts/fetching.py: User-Agent exhale-citations plus the address in
CROSSREF_MAILTO when it is set, at most two a second, one retry after 10 s. Stdlib only, apart
from pdftotext (poppler) for --refresh-cache
"""

from __future__ import annotations

import argparse
import collections
import dataclasses
import datetime
import functools
import hashlib
import json
import re
import shutil
import subprocess
import sys
import urllib.parse
from pathlib import Path
from typing import NamedTuple

import fetching
import verbatim

ROOT = Path(__file__).resolve().parent.parent
LEDGER = ROOT / "docs" / "CITATIONS.csl.json"
RECEIPTS = ROOT / "docs" / "CITATIONS.receipts.json"
CACHE = ROOT / "scratch" / "research-cache"

EPMC_API = "https://www.ebi.ac.uk/europepmc/webservices/rest"
CROSSREF_API = "https://api.crossref.org/works"
# Two requests a second, start to start, a minute to answer and one retry after 10 s
PACER = fetching.Pacer(gap=0.5)
get = functools.partial(fetching.get, pacers=(PACER,), timeout=60, retry_wait=10, read=fetching.read_text)

ABSTRACT, BODY = verbatim.SCOPES
LIVE_VARIANTS = {ABSTRACT: "abstract", "epmc": "full text"}     # the variants Europe PMC can vouch for today
# A line that holds only a number of one to four digits, and the form feed that starts its page
LINE_NUMBER = re.compile(rb"^(\f?)[ \t]*[0-9]{1,4}[ \t]*\n", re.MULTILINE)

# What a receipt and CITATIONS.md call a variant when the manifest gives it no label
VARIANT_LABELS = {
    "epmc": "the Europe PMC full text",
    "abstract": "the abstract",
    "pmc": "the PubMed Central author manuscript",
    "hal": "the HAL repository copy",
    "lirias": "the Lirias repository copy",
    "ijpp": "the journal's archive PDF",
    "cdc": "the CDC Stacks scan",
    "idl": "the authors' PDF",
    "stanford": "the authors' PDF",
    "hme": "a copy of the published PDF",
    "academia": "the Academia.edu copy",
    "rg": "the ResearchGate copy",
    "figshare": "the figshare copy",
    **dict.fromkeys(("cloak", "cloakh", "pub", "lww", "springer"), "the publisher's page"),
}


class CheckError(Exception):
    """A ready-made message, reported with exit status 1"""

    code = 1


class UsageError(CheckError):
    """A bad argument, reported with exit status 2"""

    code = 2


def read_json(path: Path, what: str) -> object:
    try:
        return json.loads(path.read_text(encoding="utf-8"))
    except OSError as exc:
        raise CheckError(f"can't read {what} {path}: {exc.strerror or exc}") from exc
    except ValueError as exc:
        raise CheckError(f"{what} {path} isn't valid JSON: {exc}") from exc


def load_manifest(cache: Path) -> dict:
    path = cache / "manifest.json"
    if not path.is_file():
        raise CheckError(
            f"no cache manifest at {path}. Name the cache folder with --cache DIR, "
            "or build it with --refresh-cache"
        )
    manifest = read_json(path=path, what="the cache manifest")
    if not isinstance(manifest, dict):
        raise CheckError(f"the cache manifest {path} must be a JSON object")
    return manifest


def load_receipts(path: Path) -> dict:
    """The receipts file, or an empty one when there is none"""
    if not path.is_file():
        return {"format": verbatim.RECEIPTS_FORMAT, "receipts": {}}
    parsed = read_json(path=path, what="the receipts file")
    if not (
        isinstance(parsed, dict)
        and parsed.get("format") == verbatim.RECEIPTS_FORMAT
        and isinstance(parsed.get("receipts"), dict)
    ):
        raise CheckError(
            f'{path} must be a format {verbatim.RECEIPTS_FORMAT} receipts file ({{"format": 1, "receipts": {{...}}}})'
        )
    return parsed


def chosen_records(records: list[dict], citekeys: list[str]) -> list[dict]:
    known = {r["id"] for r in records}
    unknown = [key for key in citekeys if key not in known]
    if unknown:
        raise UsageError(f"not in the ledger: {', '.join(unknown)}")
    return [r for r in records if not citekeys or r["id"] in citekeys]


def decode(data: bytes) -> str:
    return data.decode("utf-8", errors="replace")


@dataclasses.dataclass
class Variant:
    name: str
    label: str
    text: str
    status: str                     # ok, stub, title guard or missing

    @functools.cached_property
    def normalized(self) -> str:
        return verbatim.normalize(text=self.text)


def load_variants(cache: Path, manifest: dict, record: dict) -> list[Variant]:
    """The entry's cached variants, the manifest's best first and the rest in manifest order"""
    entry = manifest.get(record["id"]) or {}
    listed = entry.get("variants") or {}
    variants = []
    for name in sorted(listed, key=lambda candidate: candidate != entry.get("best")):
        path = cache / "papers" / record["id"] / f"{name}.txt"
        label = listed[name].get("label") or VARIANT_LABELS.get(name, "the cached copy")
        text = decode(data=path.read_bytes()) if path.is_file() else ""
        if not path.is_file():
            status = "missing"
        elif len(text.strip()) < verbatim.STUB_CHARS:
            status = "stub"
        elif name != ABSTRACT and not verbatim.title_in_text(title=record["title"], text=text):
            status = "title guard"
        else:
            status = "ok"
        variants.append(Variant(name=name, label=label, text=text, status=status))
    return variants


@dataclasses.dataclass
class Outcome:
    kind: str                       # found, partial, not found, no text cached or stub text
    scope: str = ""                 # abstract or body, when found
    variant: Variant | None = None  # where the quote was found
    detail: str = ""                # said after the kind
    notes: list[str] = dataclasses.field(default_factory=list)


def break_notes(reports: list[dict], context: bool) -> list[str]:
    """Lines that say where each missing segment of a quote goes wrong"""
    notes = []
    for number, report in enumerate(reports, start=1):
        if report["found"]:
            continue
        lead = f"segment {number} of {len(reports)}: " if len(reports) > 1 else ""
        if report["substring"]:
            words = report["segment"]
            if len(words) > 50:
                words = f"{words[:24]}...{words[-24:]}"
            notes.append(
                f"{lead}occurs only inside a longer word, so it starts or ends mid-word. Words joined "
                "by an unspaced hyphen or dash read as one word, so a quote can't start or end there. "
                f"The text reads: ...{report['text_before']}[{words}]{report['text_has']}..."
            )
            continue
        notes.append(
            f"{lead}matched {report['matched']}/{len(report['segment'])} chars, "
            f"breaks at: ...{report['before']}|{report['after']}..."
        )
        if context and report["text_has"]:
            notes.append(f"text has: ...{report['before']}|{report['text_has']}...")
        if context and report["window"]:
            notes.append(f"best window: ...{report['window']}...")
    return notes


def diagnose(quote: str, candidates: list[Variant], context: bool) -> Outcome:
    """Say why a quote isn't whole in any usable variant, using the variant it came closest to"""

    def score(variant: Variant) -> tuple[int, int]:
        reports = verbatim.segment_report(quote=quote, normalized_text=variant.normalized, window=False)
        return sum(r["found"] for r in reports), sum(r["matched"] for r in reports)

    best = max(candidates, key=score)
    reports = verbatim.segment_report(quote=quote, normalized_text=best.normalized, window=context)
    total, found = len(reports), sum(r["found"] for r in reports)
    notes = break_notes(reports=reports, context=context)
    detail = f" ({found} of {total} segments)" if found else ""
    if found == total > 0:
        where = ", ".join(str(r["position"]) for r in reports)
        detail = f" ({found} of {total} segments, out of order or too far apart)"
        notes.append(
            f"all {total} segments are in {best.name} but not in this order within "
            f"{verbatim.MAX_GAP} characters of each other (they start at {where})"
        )
    return Outcome(kind="partial" if found else "not found", detail=detail, notes=notes)


def look_up(quote: str, locator: str, variants: list[Variant], context: bool) -> Outcome:
    """Find a quote where its locator points: in the abstract for a locator that starts with Abstract,
    else in the first usable variant other than the abstract. One that only the other side has is found
    there, and the caller flags the disagreement"""
    abstract = [v for v in variants if v.name == ABSTRACT and v.status == "ok"]
    body = [v for v in variants if v.name != ABSTRACT and v.status == "ok"]
    pointed = abstract + body if verbatim.locator_scope(locator=locator) == ABSTRACT else body + abstract
    for variant in pointed:
        if verbatim.find_quote(quote=quote, normalized_text=variant.normalized) is not None:
            return Outcome(kind="found", scope=ABSTRACT if variant.name == ABSTRACT else BODY, variant=variant)
    if abstract or body:
        return diagnose(quote=quote, candidates=abstract + body, context=context)
    failed = [v.name for v in variants if v.status == "title guard"]
    stubs = [v for v in variants if v.status == "stub"]
    if stubs and not failed:
        largest = max(len(v.text.strip()) for v in stubs)
        return Outcome(kind="stub text", detail=f" ({largest} characters at most)")
    return Outcome(kind="no text cached", notes=[f"{', '.join(failed)} failed the title guard"] if failed else [])


def variant_lines(chosen: list[dict], loaded: dict[str, list[Variant]], context: bool) -> list[str]:
    """One line per entry with the title guard and stub result of each of its variants"""
    lines = []
    for record in chosen:
        variants = loaded[record["id"]]
        shown = [
            {
                "ok": f"{v.name} ok",
                "missing": f"{v.name} SKIPPED (file missing)",
                "stub": f"{v.name} SKIPPED (stub, {len(v.text.strip())} characters)",
                "title guard": f"{v.name} SKIPPED (title guard)",
            }[v.status]
            for v in variants
        ]
        lines.append(f"  {record['id']}: " + (", ".join(shown) or "no cached text"))
        for variant in variants if context else []:
            if variant.status in ("stub", "title guard"):
                first = next((ln.strip() for ln in variant.text.splitlines() if ln.strip()), "")
                lines.append(f'      {variant.name} starts: "{first[:100]}"')
    return lines


def found_record(variant: Variant, scope: str, today: str) -> dict:
    return {
        "variant": variant.name,
        "source": variant.label,
        "scope": scope,
        "chars": len(variant.text),
        "textSha256": verbatim.text_digest(normalized_text=variant.normalized),
        "date": today,
    }


def canonical(receipt: dict) -> dict:
    """A receipt with its fields in the order the file keeps them (verbatim.FOUND_KEYS inside found)"""
    found = receipt["found"]
    return {
        "quote": receipt["quote"],
        "preview": receipt["preview"],
        "found": {key: found[key] for key in verbatim.FOUND_KEYS if key in found},
    }


def merge_receipts(
    existing: dict, found_quotes: list[tuple[str, str, dict]], records: list[dict]
) -> tuple[dict, collections.Counter]:
    """The receipts file after this run, and the tally of what changed.

    Each quote this run found gets a receipt. One whose found record is unchanged apart from
    the date keeps its old record. Every other receipt stays unless the ledger no longer
    carries its quote
    """
    tally: collections.Counter = collections.Counter()
    by_entry: dict[str, dict[str, dict]] = {}
    for citekey, items in existing["receipts"].items():
        by_entry[citekey] = {}
        for receipt in items:
            if by_entry[citekey].setdefault(receipt["quote"], receipt) is not receipt:
                tally["pruned"] += 1                # a repeated receipt
    touched: set[tuple[str, str]] = set()
    for citekey, text, found in found_quotes:
        key = verbatim.quote_key(text=text)
        if (citekey, key) in touched:
            continue
        touched.add((citekey, key))
        before = by_entry.setdefault(citekey, {}).get(key)
        if before is None:
            tally["added"] += 1
        elif {**before["found"], "date": None} == {**found, "date": None}:
            tally["unchanged"] += 1
            continue
        else:
            tally["refreshed"] += 1
        by_entry[citekey][key] = {"quote": key, "preview": verbatim.quote_preview(text=text), "found": found}

    wanted = verbatim.ledger_quote_keys(records=records)
    for citekey, receipts in by_entry.items():
        for key in list(receipts):
            if key not in wanted.get(citekey, ()):
                del receipts[key]
                tally["pruned"] += 1
            elif (citekey, key) not in touched:
                tally["kept"] += 1
    merged = {
        citekey: sorted(
            (canonical(receipt=receipt) for receipt in by_entry[citekey].values()),
            key=lambda receipt: (receipt["preview"], receipt["quote"]),
        )
        for citekey in sorted(by_entry) if by_entry[citekey]
    }
    tally["total"], tally["entries"] = sum(len(items) for items in merged.values()), len(merged)
    return {"format": verbatim.RECEIPTS_FORMAT, "receipts": merged}, tally


def write_receipts(path: Path, records: list[dict], found_quotes: list[tuple[str, str, dict]]) -> None:
    merged, tally = merge_receipts(
        existing=load_receipts(path=path), found_quotes=found_quotes, records=records
    )
    text = json.dumps(merged, indent=2, ensure_ascii=False) + "\n"
    changed = not path.is_file() or text != path.read_text(encoding="utf-8")
    if changed:
        try:
            path.write_text(text, encoding="utf-8", newline="\n")
        except OSError as exc:
            raise CheckError(f"can't write the receipts file {path}: {exc.strerror or exc}") from exc
    print(f"\nReceipts ({path})")
    for name in ("added", "refreshed", "unchanged", "kept", "pruned"):
        note = " (receipts of quotes this run didn't find, left as they were)" if name == "kept" else ""
        print(f"  {name + ':':<11}{tally[name]}{note}")
    print(f"  total:     {tally['total']} receipts for {tally['entries']} entries")
    print(f"  file:      {'written' if changed else 'unchanged'}")


def read_fragment(path: Path) -> tuple[list | None, str | None]:
    """The quotes of a proposed fragment file, None when the file isn't there, and what is wrong with it"""
    if not path.is_file():
        return None, None
    try:
        data = json.loads(path.read_text(encoding="utf-8"))
    except (OSError, ValueError) as exc:
        return None, f"{path.name} can't be read: {exc}"
    if not isinstance(data, dict) or not isinstance(data.get("quotes"), list):
        return None, f'{path.name} must be an object with a "quotes" list'
    return data["quotes"], None


def run_local(args: argparse.Namespace, records: list[dict]) -> int:
    chosen = chosen_records(records=records, citekeys=args.citekeys)
    manifest = load_manifest(cache=args.cache)
    today = datetime.date.today().isoformat()

    # the quotes to check: the ledger's, or the proposed fragments laid over them
    flaws: list[str] = []
    quotes = {record["id"]: verbatim.quotes_of(record=record) for record in chosen}
    if args.proposed is not None:
        overlaid = 0
        for record in chosen:
            citekey = record["id"]
            fragment, problem = read_fragment(path=args.proposed / f"{citekey}.json")
            if fragment is not None:
                quotes[citekey] = fragment
                overlaid += 1
            elif problem or args.citekeys:
                flaws.append(f"[{citekey}] {problem or f'no fragment {citekey}.json in {args.proposed}'}")
        print(f"Proposed quotes from {args.proposed} laid over {overlaid} of {len(chosen)} entries.")
    for record in chosen:
        custom = record.get("custom") or {}
        flaws += [
            f"[{record['id']}] {flaw}"
            for flaw in verbatim.quote_flaws(
                quotes=quotes[record["id"]],
                claim_count=len(custom.get("backsClaims") or []),
                read_depth=custom.get("readDepth"),
            )
        ]

    loaded = {r["id"]: load_variants(cache=args.cache, manifest=manifest, record=r) for r in chosen}
    total = sum(len(items) for items in quotes.values())
    print(f"Checking {total} quotes from {sum(1 for q in quotes.values() if q)} of {len(chosen)} ledger entries.\n")

    counts: collections.Counter = collections.Counter()
    found_quotes: list[tuple[str, str, dict]] = []
    number = 0
    for record in chosen:
        citekey = record["id"]
        for index, quote in enumerate(quotes[citekey]):
            number += 1
            text = quote.get("text") if isinstance(quote, dict) else None
            if not isinstance(text, str) or not text.strip():
                print(f"{number}. [{citekey}] quotes[{index}] has no text to check")
                continue
            locator = quote.get("locator")
            outcome = look_up(
                quote=text, locator=locator if isinstance(locator, str) else "", variants=loaded[citekey], context=args.context
            )
            counts[outcome.kind] += 1
            where = ""
            if outcome.kind == "found":
                counts[f"found in the {outcome.scope}"] += 1
                where = f" ({outcome.scope}, {outcome.variant.name})"
                found_quotes.append((citekey, text, found_record(variant=outcome.variant, scope=outcome.scope, today=today)))
                if isinstance(locator, str) and (
                    flaw := verbatim.locator_flaw(locator=locator, scope=outcome.scope, source=outcome.variant.label)
                ):
                    flaws.append(f"[{citekey}] quotes[{index}] {flaw}")
            if not args.context and len(text) > verbatim.PREVIEW_CHARS:
                text = verbatim.quote_preview(text=text) + "..."
            print(f'{number}. [{citekey}] {outcome.kind}{where}{outcome.detail}: "{text}"')
            for note in outcome.notes:
                print(f"     {note}")

    if flaws:
        print(f"\nQuote flaws ({len(flaws)})")
        for flaw in flaws:
            print(f"  {flaw}")

    print("\nTitle guard and stub check (a variant that isn't ok is not searched)")
    for line in variant_lines(chosen=chosen, loaded=loaded, context=args.context):
        print(line)
    every = [v for variants in loaded.values() for v in variants]
    print(f"  {sum(v.status == 'ok' for v in every)} of {len(every)} variants usable")

    print("\nSummary")
    split = f"abstract {counts['found in the abstract']}, body {counts['found in the body']}"
    for label, value in (
        ("found", f"{counts['found']} ({split})"), ("partial", counts["partial"]), ("not found", counts["not found"]),
        ("no text cached", counts["no text cached"]), ("stub text", counts["stub text"]),
        ("total quotes", total), ("flaws", len(flaws)),
    ):
        print(f"  {label + ':':<16}{value}")

    if args.write_receipts:
        write_receipts(path=args.receipts, records=records, found_quotes=found_quotes)
    return 1 if args.strict and (counts["partial"] or counts["not found"] or flaws) else 0


class Fetched(NamedTuple):
    """A text as its live source serves it"""

    text: str       # verbatim.xml_to_text of raw
    raw: str
    suffix: str     # html or xml, what the raw markup is
    label: str
    source: str


def fetch_sources(record: dict, want_full: bool) -> tuple[dict[str, Fetched], list[str], list[str]]:
    """What Europe PMC and Crossref hold for an entry today, as {variant: Fetched}.

    The Europe PMC abstract, else the Crossref one, and with want_full the Europe PMC full text.
    Also why a text is absent (notes) and which fetches failed (errors)
    """
    texts: dict[str, Fetched] = {}
    notes: list[str] = []
    errors: list[str] = []
    doi, pmid = record.get("DOI"), record.get("PMID")
    if not (doi or pmid):
        return texts, ["the entry has no DOI or PMID"], errors
    query = f'DOI:"{doi}"' if doi else f"EXT_ID:{pmid} AND SRC:MED"
    search = f"{EPMC_API}/search?" + urllib.parse.urlencode({"query": query, "format": "json", "resultType": "core"})

    def parse_object(body: object) -> dict:
        try:
            parsed = json.loads(body)
        except ValueError:
            return {}
        return parsed if isinstance(parsed, dict) else {}

    hit: dict = {}
    reply = get(url=search)
    if reply.status == 200:
        hits = (parse_object(body=reply.body).get("resultList") or {}).get("result") or []
        first = hits[0] if hits else {}
        if not hits:
            notes.append("Europe PMC has no record of it")
        elif (doi and str(first.get("doi") or "").lower() == str(doi).lower()) or (
            pmid and str(first.get("pmid") or "") == str(pmid)
        ):
            hit = first
        else:
            notes.append(f"Europe PMC's first hit is another record (doi {first.get('doi')}, pmid {first.get('pmid')})")
    else:
        errors.append(f"the Europe PMC search failed ({reply.error})")

    if hit:
        raw = hit.get("abstractText") or ""
        if text := verbatim.xml_to_text(markup=raw):
            texts[ABSTRACT] = Fetched(
                text=text, raw=raw, suffix="html", label="the Europe PMC abstract", source=urllib.parse.unquote_plus(search)
            )
        else:
            notes.append("Europe PMC holds no abstract for it")
    if ABSTRACT not in texts and doi:
        reply = get(url=f"{CROSSREF_API}/{urllib.parse.quote(doi)}")
        if reply.status == 200:
            message = parse_object(body=reply.body).get("message")
            raw = str(message.get("abstract") or "") if isinstance(message, dict) else ""
            if text := verbatim.xml_to_text(markup=raw):
                texts[ABSTRACT] = Fetched(
                    text=text, raw=raw, suffix="xml", label="the Crossref abstract", source=f"{CROSSREF_API}/{doi}"
                )
            else:
                notes.append("Crossref holds no abstract for it")
        elif reply.status == 404:
            notes.append("Crossref has no record of the DOI")
        else:
            errors.append(f"the Crossref request failed ({reply.error})")

    pmcid = hit.get("pmcid")
    if want_full and not pmcid:
        notes.append("Europe PMC lists no PMCID, so no full text")
    elif want_full:
        url = f"{EPMC_API}/{pmcid}/fullTextXML"
        reply = get(url=url)
        if reply.status == 200:
            if text := verbatim.xml_to_text(markup=reply.body):
                texts["epmc"] = Fetched(
                    text=text, raw=reply.body, suffix="xml", label="the Europe PMC full text", source=url
                )
        elif reply.status == 404:
            notes.append(f"Europe PMC has no full text for {pmcid}")
        else:
            errors.append(f"the Europe PMC full text of {pmcid} failed ({reply.error})")
    return texts, notes, errors


def write_live_summary(
    path: Path, counts: collections.Counter, failures: list[list[str]], unavailable: list[list[str]]
) -> None:
    def table(headings: list[str], rows: list[list[str]]) -> list[str]:
        cells = [[cell.replace("|", "\\|").replace("\n", " ") for cell in row] for row in rows]
        return ["|" + "".join(f" {h} |" if h else " |" for h in headings), "|" + "---|" * len(headings)] + [
            "| " + " | ".join(row) + " |" for row in cells
        ]

    lines = ["## Quote receipts against the live Europe PMC text", ""]
    lines += table(headings=["", ""], rows=[
        [label, str(counts[key])] for label, key in (
            ("Receipts", "receipts"),
            ("Checked against live text", "checked"),
            ("Still found", "found"),
            ("Missing from the live text", "failed"),
            ("Live text unavailable", "unavailable"),
            ("Not live-checkable (other sources)", "not live-checkable"),
            ("Stale (quote not in the ledger)", "stale"),
        )
    ])
    lines.append("")
    if failures:
        lines += ["### Quotes missing from the live text", ""]
        lines += table(headings=["Entry", "Quote", "Checked against", "Why"], rows=failures) + [""]
    if unavailable:
        lines += ["### Live texts that couldn't be fetched", ""]
        lines += table(headings=["Entry", "Text", "Why"], rows=unavailable) + [""]
    try:
        with path.open("a", encoding="utf-8") as handle:
            handle.write("\n".join(lines) + "\n")
    except OSError as exc:
        raise CheckError(f"can't write the summary {path}: {exc.strerror or exc}") from exc


def run_live(args: argparse.Namespace, records: list[dict]) -> int:
    chosen = chosen_records(records=records, citekeys=args.citekeys)
    receipts_file = load_receipts(path=args.receipts)
    counts: collections.Counter = collections.Counter()
    failures: list[list[str]] = []
    unavailable: list[list[str]] = []

    for record in chosen:
        citekey = record["id"]
        items = receipts_file["receipts"].get(citekey) or []
        if not items:
            continue
        texts = verbatim.quote_texts(record=record)
        counts["receipts"] += len(items)
        checkable: dict[str, list[dict]] = {}
        for receipt in items:
            variant = receipt.get("found", {}).get("variant")
            if receipt["quote"] not in texts:
                counts["stale"] += 1
            elif variant in LIVE_VARIANTS:
                checkable.setdefault(variant, []).append(receipt)
            else:
                counts["not live-checkable"] += 1
        if not checkable:
            continue

        fetched, notes, errors = fetch_sources(record=record, want_full="epmc" in checkable)
        guard_failed = "epmc" in fetched and not verbatim.title_in_text(title=record["title"], text=fetched["epmc"].text)
        if guard_failed:
            del fetched["epmc"]
        for name, kind in LIVE_VARIANTS.items():
            group = checkable.get(name)
            if not group:
                continue
            if name not in fetched:
                why = ", ".join(errors + notes) or f"no {kind}"
                if guard_failed and name == "epmc":
                    why = "the live full text fails the title guard"
                counts["unavailable"] += len(group)
                unavailable.append([f"`{citekey}`", kind, why])
                print(f"[{citekey}] {name}: live text unavailable ({why})", flush=True)
                continue
            live = verbatim.normalize(text=fetched[name].text)
            missing = [r for r in group if verbatim.find_quote(quote=texts[r["quote"]], normalized_text=live) is None]
            counts["checked"] += len(group)
            counts["failed"] += len(missing)
            counts["found"] += len(group) - len(missing)
            for receipt in missing:
                failures.append([f"`{citekey}`", receipt["preview"], name, f"not in the live {kind}"])
                print(f'[{citekey}] MISSING not in the live {kind}: "{receipt["preview"]}"', flush=True)
            print(f"[{citekey}] {name}: {len(group) - len(missing)} of {len(group)} found", flush=True)

    print("\nSummary")
    print(f"  receipts:           {counts['receipts']}")
    print(f"  checked live:       {counts['checked']} ({counts['found']} found, {counts['failed']} missing)")
    print(f"  live text missing:  {counts['unavailable']}")
    print(f"  not live-checkable: {counts['not live-checkable']} (their sources aren't at Europe PMC)")
    if counts["stale"]:
        print(f"  stale:              {counts['stale']} (run scripts/generate-citations.py for the details)")
    print(f"  requests:           {PACER.requests}")
    if counts["receipts"] and not counts["checked"] and counts["unavailable"]:
        print("  WARNING: no live text could be fetched, so nothing was checked")
    if args.summary is not None:
        write_live_summary(path=args.summary, counts=counts, failures=failures, unavailable=unavailable)
    return 1 if failures else 0


def describe(data: bytes) -> dict:
    """The manifest's chars and sha256 for a variant's text file"""
    return {"chars": len(decode(data=data)), "sha256": hashlib.sha256(data).hexdigest()[:16]}


def store_text(
    folder: Path, entry: dict, name: str, data: bytes, source: str | None, label: str | None, keep_old: bool
) -> tuple[str, str]:
    """Write <name>.txt when its bytes changed and record its chars, sha256, source and label in the manifest entry.

    keep_old keeps the text it replaces as <name>.old.txt the first time. Returns "new",
    "changed" or "unchanged" and a sentence about it
    """
    path, old = folder / f"{name}.txt", folder / f"{name}.old.txt"
    before = path.read_bytes() if path.is_file() else None
    variants = entry.setdefault("variants", {})
    listed = dict(variants.get(name) or {})
    measured = describe(data=data)
    listed["chars"] = measured["chars"]
    listed["source"] = source or listed.get("source") or f"{name}.txt"
    listed["sha256"] = measured["sha256"]
    if label:
        listed["label"] = label
    unchanged_listing = variants.get(name) == listed
    variants[name] = listed
    entry.setdefault("best", name)
    if before == data:
        return "unchanged", "unchanged" if unchanged_listing else "unchanged text, manifest source, label or size updated"
    if keep_old and before is not None and not old.exists():
        shutil.copy2(path, old)
    scratch = folder / f".{name}.txt.new"
    scratch.write_bytes(data)
    scratch.replace(path)
    if before is None:
        return "new", f"new, {len(decode(data=data))} chars"
    return "changed", f"changed, {len(decode(data=before))} -> {len(decode(data=data))} chars"


def refresh_pdf(folder: Path, entry: dict, pdf: Path) -> tuple[str, str]:
    """Re-extract a PDF's text in reading order, minus the line numbers when the manifest says to.

    Returns "new", "changed", "unchanged" or "failed" and a sentence about it
    """
    tool = shutil.which("pdftotext")
    if tool is None:
        return "failed", "pdftotext isn't installed (poppler), so this PDF can't be re-extracted"
    scratch = folder / f".{pdf.stem}.txt.new"
    # reading order is pdftotext's default. -layout would put the columns of a page side by side
    run = subprocess.run(
        [tool, "-enc", "UTF-8", str(pdf), str(scratch)], capture_output=True, text=True, timeout=300
    )
    if run.returncode != 0 or not scratch.is_file():
        scratch.unlink(missing_ok=True)
        return "failed", f"pdftotext failed: {run.stderr.strip() or run.returncode}"
    data = scratch.read_bytes()
    scratch.unlink()
    listed = (entry.get("variants") or {}).get(pdf.stem)
    strip = (listed or {}).get("strip") == "line-numbers"
    if strip:
        data = LINE_NUMBER.sub(rb"\1", data)
    # a variant the manifest doesn't list yet keeps no URL to name, so it names the PDF
    kind, message = store_text(
        folder=folder, entry=entry, name=pdf.stem, data=data,
        source=None if pdf.stem in (entry.get("variants") or {}) else f"pdftotext of {pdf.name}", label=None, keep_old=True,
    )
    return kind, message + (", line numbers stripped" if strip else "")


def save_manifest(cache: Path, manifest: dict) -> None:
    text = json.dumps(manifest, indent=1, ensure_ascii=False) + "\n"
    (cache / "manifest.json").write_text(text, encoding="utf-8", newline="\n")


def run_refresh(args: argparse.Namespace, records: list[dict]) -> int:
    chosen = chosen_records(records=records, citekeys=args.citekeys)
    manifest = load_manifest(cache=args.cache)
    tally: collections.Counter = collections.Counter()
    problems: list[str] = []
    try:
        for record in chosen:
            citekey = record["id"]
            folder = args.cache / "papers" / citekey
            entry = manifest.setdefault(citekey, {})
            lines: list[str] = []

            if record.get("DOI") or record.get("PMID"):
                texts, notes, errors = fetch_sources(record=record, want_full=True)
                if ABSTRACT not in texts:
                    kept = (entry.get("variants") or {}).get(ABSTRACT)
                    lines.append(
                        "  abstract: none at Europe PMC or Crossref"
                        + (f", kept the cached one ({kept['source']})" if kept else "")
                    )
                    tally["abstract none"] += 1
                for name, item in texts.items():
                    folder.mkdir(parents=True, exist_ok=True)
                    (folder / f"{name}.{item.suffix}").write_text(item.raw, encoding="utf-8", newline="")
                    kind, message = store_text(
                        folder=folder, entry=entry, name=name, data=(item.text + "\n").encode("utf-8"),
                        source=item.source, label=item.label, keep_old=False,
                    )
                    lines.append(f"  {name} ({item.label}): {message}")
                    tally[f"{name} {kind}"] += 1
                if "epmc" in texts and not verbatim.title_in_text(title=record["title"], text=texts["epmc"].text):
                    lines.append("  WARNING: the Europe PMC text fails the title guard and won't be searched")
                    tally["epmc fails the title guard"] += 1
                lines += [f"  note: {note}" for note in notes] + [f"  ERROR: {error}" for error in errors]
                problems += [f"[{citekey}] {error}" for error in errors]

            for pdf in sorted(folder.glob("*.pdf")):
                kind, message = refresh_pdf(folder=folder, entry=entry, pdf=pdf)
                lines.append(f"  {pdf.name}: {message}")
                tally[f"pdf {kind}"] += 1
                if kind == "failed":
                    problems.append(f"[{citekey}] {pdf.name}: {message}")

            # variants this script doesn't fetch keep their source and label but get measured again,
            # so chars and sha256 mean the same thing for every variant
            recounted = []
            for name, listed in (entry.get("variants") or {}).items():
                if (folder / f"{name}.txt").is_file():
                    measured = describe(data=(folder / f"{name}.txt").read_bytes())
                    if any(listed.get(key) != value for key, value in measured.items()):
                        listed.update(measured)
                        recounted.append(name)
            if recounted:
                lines.append(f"  manifest chars or sha256 recounted for: {', '.join(recounted)}")
                tally["variants recounted"] += len(recounted)

            if not entry:
                del manifest[citekey]
            if lines:
                print(citekey, flush=True)
                for line in lines:
                    print(line, flush=True)
            save_manifest(cache=args.cache, manifest=manifest)
    except KeyboardInterrupt:
        problems.append("interrupted, the manifest holds what was done so far")
    finally:
        save_manifest(cache=args.cache, manifest=manifest)

    print("\nSummary")
    for key in sorted(tally):
        print(f"  {key}: {tally[key]}")
    print(f"  requests: {PACER.requests}")
    if problems:
        print(f"\nProblems ({len(problems)})")
        for problem in problems:
            print(f"  {problem}")
    return 1 if problems else 0


def parse_args(argv: list[str]) -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        prog="check-quotes.py", description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter
    )
    parser.add_argument("citekeys", nargs="*", metavar="citekey", help="limit the run to these entries")
    parser.add_argument("--cache", type=Path, default=CACHE, metavar="DIR")
    parser.add_argument("--ledger", type=Path, default=LEDGER, metavar="PATH")
    parser.add_argument("--receipts", type=Path, default=RECEIPTS, metavar="PATH")
    parser.add_argument("--strict", action="store_true")
    parser.add_argument("--context", action="store_true")
    parser.add_argument("--write-receipts", action="store_true")
    parser.add_argument("--proposed", type=Path, metavar="DIR")
    parser.add_argument("--live", action="store_true")
    parser.add_argument("--refresh-cache", action="store_true")
    parser.add_argument("--summary", type=Path, metavar="FILE")
    args = parser.parse_args(argv)

    if args.live and args.refresh_cache:
        parser.error("--live and --refresh-cache are separate runs")
    if args.proposed is not None and args.write_receipts:
        parser.error("--proposed can't be combined with --write-receipts, a proposed quote isn't in the ledger yet")
    if args.live and (args.write_receipts or args.proposed is not None):
        parser.error("--live checks the receipts as they stand, so it takes neither --write-receipts nor --proposed")
    if args.refresh_cache and (args.write_receipts or args.proposed is not None or args.strict or args.context):
        parser.error("--refresh-cache takes only --cache, --ledger and citekeys")
    if args.summary is not None and not args.live:
        parser.error("--summary goes with --live")
    if args.proposed is not None and not args.proposed.is_dir():
        parser.error(f"--proposed {args.proposed} isn't a folder")
    return args


def main() -> int:
    try:
        args = parse_args(argv=sys.argv[1:])
        records = read_json(path=args.ledger, what="the ledger")
        if not (isinstance(records, list) and all(isinstance(r, dict) and r.get("id") for r in records)):
            raise CheckError(f"the ledger {args.ledger} must be a JSON array of entries that each have an id")
        run = run_live if args.live else run_refresh if args.refresh_cache else run_local
        return run(args=args, records=records)
    except CheckError as exc:
        print(f"check-quotes.py: {exc}", file=sys.stderr)
        return exc.code
    except KeyboardInterrupt:
        print("check-quotes.py: interrupted", file=sys.stderr)
        return 130


if __name__ == "__main__":
    raise SystemExit(main())
