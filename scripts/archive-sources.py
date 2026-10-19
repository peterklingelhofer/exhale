#!/usr/bin/env python3
"""Pin the ledger's links that have no DOI to Wayback Machine captures.

A source with no DOI has no durable identifier, and a repository copy can move or change after it
was read. Each target below gets a full https://web.archive.org/web/<timestamp>/<url> link in
docs/CITATIONS.csl.json, and scripts/generate-citations.py prints it under the entry:

  - custom.archived          the URL of every entry with no DOI
  - custom.openCopyArchived  every custom.openCopy

A target whose field is already set is left alone. Each remaining target goes through three steps:

  1. Look up the captures the Archive already holds, with the CDX API (status 200 only). The
     earliest capture on or after READ_DATE wins, which is the day the cached texts were fetched.
     With none on or after, the latest one before it is taken. A capture more than
     MAX_DISTANCE_DAYS from READ_DATE either way is out
  2. With none left, ask Save Page Now for one: GET web.archive.org/save/<url> with redirects
     followed, and read the capture time from the response. A fresh capture shows the page as it
     is today, which can be a later edition than the one read. --lookup-only skips this step
  3. Open the link once before recording it. It has to answer 2xx and land on a capture within
     MAX_DISTANCE_DAYS of READ_DATE. The Archive shows the capture nearest the timestamp, and
     Save Page Now saves only the page a URL redirects to, so a URL that redirects can open an
     older capture or none at all

Pace: at most two lookups a second and one capture every five seconds, counted from the start of
one request to the start of the next. A 429, a 5xx or a network error waits 60 seconds and tries
once more (scripts/fetching.py), and then the target is skipped and reported. A failed lookup
never leads to a capture, because the Archive might already hold the right one.

The ledger is saved as it is formatted: parsed, changed, and written back with
json.dumps(records, indent=2, ensure_ascii=False) and a trailing newline. Nothing is written
unless the unmodified file reads back byte for byte the same. Every save reads the file afresh and
adds only the new links, so edits made to the ledger during a long run survive. It saves after
every five targets and on Ctrl-C, so an interrupted run keeps its progress and a second run
carries on from there.

Usage:
    uv run --no-project scripts/archive-sources.py                  # every target
    uv run --no-project scripts/archive-sources.py <citekey> ...    # only these entries
    uv run --no-project scripts/archive-sources.py --dry-run        # list the targets, no network
    uv run --no-project scripts/archive-sources.py --lookup-only    # record existing captures only
    uv run --no-project scripts/archive-sources.py --self-check     # check the decisions offline

Exit status: 0 when the run finished, 1 when the ledger could not be read or saved, 2 for bad
arguments, 130 when interrupted

Stdlib only, so it runs anywhere the repo is checked out
"""

from __future__ import annotations

import argparse
import collections
import datetime
import json
import os
import re
import signal
import sys
import urllib.parse
from pathlib import Path
from typing import NamedTuple

import fetching
import registry

ROOT = Path(__file__).resolve().parent.parent
LEDGER = ROOT / "docs" / "CITATIONS.csl.json"

CDX = "https://web.archive.org/cdx/search/cdx"
SAVE = "https://web.archive.org/save/"

# The day the cached texts were fetched. A capture on or after it shows the page as it was read
READ_DATE = "20260920"
# How many days either side of READ_DATE a capture may sit and still stand in for the edition
# that was read: a year, plus a margin for a page revised on a yearly cycle
MAX_DISTANCE_DAYS = 400

# Seconds from the start of one request to the start of the next, per kind of request
LOOKUP = fetching.Pacer(gap=0.5)
CAPTURE = fetching.Pacer(gap=5.0)
RETRY_WAIT = 60.0
LOOKUP_TIMEOUT = 60.0
# Save Page Now answers only once the capture is done, and a large PDF takes a while
CAPTURE_TIMEOUT = 180.0
SAVE_EVERY = 5

TIMESTAMP_RE = re.compile(r"[0-9]{14}")
# /web/<14 digits>/ at the start of a path, with the host or without it, and with the replay mode
# some links add after the digits (id_, if_). Anchored so a captured page whose own path holds
# such a segment can't be mistaken for the capture
SAVE_PATH_RE = re.compile(r"(?:https?://web\.archive\.org)?/web/([0-9]{14})(?:[a-z]{2}_)?/")
ON_WAYBACK_RE = re.compile(r"https?://web\.archive\.org(?:[/:?#]|$)", re.IGNORECASE)


class ArchiveError(Exception):
    """A problem with the ledger file, in words ready to print"""


class Skip(Exception):
    """A target that gets no link this run, with the reason to report"""

    def __init__(self, reason: str) -> None:
        super().__init__(reason)
        self.reason = reason


class Target(NamedTuple):
    citekey: str
    field: str
    url: str


# The decisions. Pure, so --self-check can cover them with no network


def in_window(timestamp: str, read_date: str) -> bool:
    """Whether a capture time (YYYYMMDDhhmmss) is a real day within MAX_DISTANCE_DAYS of read_date (YYYYMMDD)"""
    try:
        day = datetime.date.fromisoformat(timestamp[:8])
    except ValueError:
        return False
    return abs((day - datetime.date.fromisoformat(read_date)).days) <= MAX_DISTANCE_DAYS


def closest_capture(rows: list[list[str]], read_date: str) -> tuple[str, str] | None:
    """The capture that best stands in for the edition read on read_date (YYYYMMDD).

    Rows are what the CDX API returns as JSON, each [timestamp, original, statuscode], and the
    first is a header. Only status 200 counts. The earliest capture on or after the read date
    wins over a nearer one before it. With none on or after, the latest one before it is taken.
    A capture more than MAX_DISTANCE_DAYS away either way is out. The result is (timestamp,
    original), where original is the URL the capture holds, which can differ from the URL asked
    for by its scheme or a leading www. A link built from it opens that exact capture
    """
    strings = (row[:3] for row in rows if len(row) >= 3 and all(isinstance(cell, str) for cell in row[:3]))
    captures = [
        (timestamp, original)
        for timestamp, original, status in strings
        # The header row fails the timestamp test, and so does anything else that isn't a capture
        if status == "200" and TIMESTAMP_RE.fullmatch(timestamp) and in_window(timestamp=timestamp, read_date=read_date)
    ]
    captures.sort(key=lambda capture: capture[0])
    # Digit strings of one width compare in date order
    after = [capture for capture in captures if capture[0][:8] >= read_date]
    return after[0] if after else (captures[-1] if captures else None)


def timestamp_from_save_response(
    location: str | None, content_location: str | None, final_url: str | None
) -> str | None:
    """The capture time Save Page Now reports, read from the first of the Location header, the
    Content-Location header and the final URL that carries /web/<14 digits>/.

    The save request follows redirects, so the final URL normally has it, and the headers cover
    a response that stopped at a redirect
    """
    for candidate in (location, content_location, final_url):
        found = SAVE_PATH_RE.match(candidate or "")
        if found is not None:
            return found.group(1)
    return None


def add_link(custom: dict, field: str, link: str) -> dict:
    """A copy of custom with the link added in its place in the file: openCopyArchived right
    after openCopy, and archived at the end"""
    if field != "openCopyArchived":
        return {**custom, field: link}
    placed: dict = {}
    for key, value in custom.items():
        if key != field:
            placed[key] = value
        if key == "openCopy":
            placed[field] = link
    placed.setdefault(field, link)
    return placed


def list_targets(records: list[dict], citekeys: list[str]) -> tuple[list[Target], int]:
    """The targets that still need a capture, in ledger order, and how many were left out.

    An entry with no DOI has one for its URL, since a DOI is the durable link already. An entry
    with an open copy has one for that. A target already on web.archive.org is a capture, and a
    recorded link means the work is done. With citekeys, only those entries are looked at
    """
    todo: list[Target] = []
    left_out = 0
    for entry in records:
        if citekeys and entry["id"] not in citekeys:
            continue
        for kind, url in registry.link_targets(entry=entry).items():
            field = registry.ARCHIVE_FIELDS[kind]
            if (entry.get("custom") or {}).get(field) or ON_WAYBACK_RE.match(url):
                left_out += 1
            else:
                todo.append(Target(citekey=entry["id"], field=field, url=url))
    return todo, left_out


# The ledger


def dump(records: list[dict]) -> str:
    return json.dumps(records, indent=2, ensure_ascii=False) + "\n"


def read_ledger() -> list[dict]:
    """The ledger records, read only when a save would reproduce the file byte for byte"""
    text = LEDGER.read_bytes().decode("utf-8")
    try:
        records = json.loads(text)
    except json.JSONDecodeError as exc:
        raise ArchiveError(f"{LEDGER.name} isn't valid JSON: {exc}") from exc
    if not isinstance(records, list) or not all(isinstance(entry, dict) for entry in records):
        raise ArchiveError(f"{LEDGER.name} must hold a JSON array of objects")
    if dump(records=records) != text:
        raise ArchiveError(
            f"{LEDGER.name} doesn't read back byte for byte after json.dumps(records, "
            "indent=2, ensure_ascii=False) and a trailing newline, so a save would reformat "
            "it. Nothing was written"
        )
    return records


def save(pending: dict[tuple[str, str], str]) -> int:
    """Add the pending links to a fresh read of the ledger and write it in one step, so an
    interrupt never leaves half a file. Returns how many were added. The pending links stay put
    when anything fails"""
    if not pending:
        return 0
    records = read_ledger()
    by_id = {entry["id"]: entry for entry in records}
    written = 0
    for (citekey, field), link in pending.items():
        entry = by_id.get(citekey)
        if entry is None:
            print(f"  {citekey} is no longer in the ledger, so its link was dropped")
        elif (entry.get("custom") or {}).get(field):
            print(f"  {citekey} gained {field} meanwhile, so it was left as it is")
        else:
            entry["custom"] = add_link(custom=entry.get("custom") or {}, field=field, link=link)
            written += 1
    temp = LEDGER.with_name(LEDGER.name + ".tmp")
    try:
        temp.write_bytes(dump(records=records).encode("utf-8"))
        os.replace(temp, LEDGER)
    finally:
        temp.unlink(missing_ok=True)
    pending.clear()
    print(f"saved to {LEDGER.relative_to(ROOT)}: {written} links")
    return written


def try_save(pending: dict[tuple[str, str], str], stats: collections.Counter) -> bool:
    """A save that reports its failure and leaves the links in pending, printed for a manual copy"""
    try:
        stats["written"] += save(pending=pending)
        return True
    except (ArchiveError, OSError) as exc:
        print(f"could not save:\n  - {exc}", file=sys.stderr)
        for (citekey, field), link in pending.items():
            print(f"  unsaved: {citekey} {field} {link}", file=sys.stderr)
        return False


# The network


def fetch(url: str, pacer: fetching.Pacer, what: str, timeout: float, read: object = None) -> fetching.Reply:
    """One GET, tried a second time after RETRY_WAIT seconds when it ends in a 429, a 5xx or a
    network error. Returns the reply for any other status, a 4xx included, which the caller
    reads. Raises Skip with a reason when both tries ended that way. what names the request in
    the reports"""
    reply = fetching.get(url=url, pacers=(pacer,), timeout=timeout, retry_wait=RETRY_WAIT, read=read)
    if reply.status is None or reply.status == 429 or reply.status >= 500:
        raise Skip(reason=f"{what} failed, {reply.error} after a retry")
    return reply


def lookup(url: str) -> list[list[str]]:
    """The CDX rows for a URL, status 200 only. An empty answer is the Archive's way of saying
    it holds nothing. Raises Skip for any other answer that isn't JSON rows, because the Archive
    might hold the right capture and a lookup that failed must not lead to a new one"""
    encoded = urllib.parse.quote(url, safe="")
    query = f"{CDX}?url={encoded}&output=json&filter=statuscode:200&fl=timestamp,original,statuscode"
    reply = fetch(url=query, pacer=LOOKUP, what="lookup", timeout=LOOKUP_TIMEOUT, read=fetching.read_text)
    if reply.status != 200:
        raise Skip(reason=f"lookup answered HTTP {reply.status}")
    text = reply.body.strip()
    if not text:
        return []
    try:
        rows = json.loads(text)
    except json.JSONDecodeError:
        rows = None
    if not isinstance(rows, list) or not all(isinstance(row, list) for row in rows):
        raise Skip(reason=f"lookup answered something other than JSON rows: {text[:60]}")
    return rows


def capture(url: str) -> tuple[str, str]:
    """Ask Save Page Now for a capture and return its 14-digit timestamp and the URL the
    response ended at. Raises Skip with the reason when there is no timestamp"""
    reply = fetch(url=SAVE + url, pacer=CAPTURE, what="capture", timeout=CAPTURE_TIMEOUT)
    if not 200 <= reply.status < 300:
        raise Skip(reason=f"capture answered HTTP {reply.status}")
    timestamp = timestamp_from_save_response(
        location=reply.headers.get("Location"), content_location=reply.headers.get("Content-Location"), final_url=reply.url
    )
    if timestamp is None:
        raise Skip(reason=f"capture answered HTTP {reply.status} with no timestamp to read (ended at {reply.url})")
    return timestamp, reply.url


def resolve(target: Target, lookup_only: bool) -> tuple[str, str]:
    """Find or make the capture for one target. Returns the link and how it was got (found or
    captured), and raises Skip when there is none"""
    rows = lookup(url=target.url)
    existing = closest_capture(rows=rows, read_date=READ_DATE)
    hint = ""
    if existing is not None:
        (timestamp, original), how = existing, "found"
    elif lookup_only:
        raise Skip(reason="no usable capture, and --lookup-only is set")
    else:
        # The first row is a header
        print(f"{target.citekey}: no usable capture ({max(len(rows) - 1, 0)} held with status 200), requesting one")
        timestamp, ended_at = capture(url=target.url)
        original, how, hint = target.url, "captured", f" (Save Page Now ended at {ended_at})"
    link = f"https://web.archive.org/web/{timestamp}/{original}"
    if problem := registry.wayback_problem(link=link, target=target.url):
        raise Skip(reason=f"the {how} link {problem}")
    # Open it once. It has to answer 2xx and land on a capture within MAX_DISTANCE_DAYS of READ_DATE
    reply = fetch(url=link, pacer=LOOKUP, what="link check", timeout=LOOKUP_TIMEOUT)
    if not 200 <= reply.status < 300:
        raise Skip(reason=f"the link opens HTTP {reply.status}{hint}")
    landed = SAVE_PATH_RE.match(reply.url)
    if landed is None or not in_window(timestamp=landed.group(1), read_date=READ_DATE):
        raise Skip(reason=f"the link opens {reply.url}, outside {MAX_DISTANCE_DAYS} days of {READ_DATE}{hint}")
    return link, how


# The run


def run(todo: list[Target], lookup_only: bool) -> int:
    pending: dict[tuple[str, str], str] = {}
    stats: collections.Counter = collections.Counter()
    skipped: dict[str, list[str]] = {}
    save_failed = interrupted = False

    def stop(signum: int, frame: object) -> None:
        raise KeyboardInterrupt

    signal.signal(signal.SIGTERM, stop)
    try:
        for index, target in enumerate(todo, start=1):
            tag = f"[{index}/{len(todo)}] {target.citekey} {target.field}"
            try:
                link, how = resolve(target=target, lookup_only=lookup_only)
            except Skip as skip:
                skipped.setdefault(skip.reason, []).append(f"{target.citekey} {target.field}")
                print(f"{tag}: skipped, {skip.reason}")
            else:
                pending[(target.citekey, target.field)] = link
                stats[how] += 1
                print(f"{tag}: {how} {link} (read {READ_DATE})")
            stats["looked"] += 1
            if index % SAVE_EVERY == 0 and not try_save(pending=pending, stats=stats):
                save_failed = True
                break
    except KeyboardInterrupt:
        interrupted = True
        print("\ninterrupted, saving what is done")

    if not save_failed:
        save_failed = not try_save(pending=pending, stats=stats)
    print()
    for label, key in (("looked up", "looked"), ("found existing", "found"), ("captured", "captured")):
        print(f"{label}: {stats[key]}")
    print(f"skipped: {sum(len(names) for names in skipped.values())}")
    for reason, names in skipped.items():
        print(f"  {reason} ({len(names)}): {', '.join(names)}")
    print("links written to the ledger:", stats["written"])
    if save_failed:
        print("The links listed as unsaved were not written, so there is nothing to render yet")
        return 1
    if stats["written"]:
        print("Run `uv run --no-project scripts/generate-citations.py` to render the links")
    return 130 if interrupted else 0


def self_check() -> None:
    """Plain asserts on worked examples of the pure decisions"""
    header = ["timestamp", "original", "statuscode"]
    x = "https://example.org/paper.pdf"
    stamp = "20261018123456"
    page = f"https://web.archive.org/web/{stamp}/{x}"
    read = "20260920"

    # Choosing a capture, with every row at status 200
    cases = [
        # the earliest on or after the read date wins over a nearer one before it
        (["20260919235959", "20261010000000", "20260925000000"], "20260925000000"),
        # a capture on the read date itself counts as on it
        (["20260920000001", "20260925000000"], "20260920000001"),
        # with none on or after, the latest one before it
        (["20260101000000", "20260915000000"], "20260915000000"),
        # 400 days either side is in and 401 is out
        (["20250816000000"], "20250816000000"),
        (["20250815235959"], None),
        (["20271025000000"], "20271025000000"),
        (["20271026000000"], None),
        # a day that doesn't exist and a short timestamp are skipped
        (["20261340000000", "2026092500"], None),
    ]
    for stamps, want in cases:
        rows = [header, *([s, x, "200"] for s in stamps)]
        got = closest_capture(rows=rows, read_date=read)
        assert got == ((want, x) if want else None), (stamps, got)
    # only status 200 counts, a short row is skipped, and the original comes back as held
    rows = [["20260925000000", x, "404"], ["20260925000000", x, "302"], ["20260925000000", x]]
    assert closest_capture(rows=rows, read_date=read) is None
    held = "http://www.example.org/paper.pdf"
    assert closest_capture(rows=[["20260925000000", held, "200"]], read_date=read) == ("20260925000000", held)
    assert closest_capture(rows=[], read_date=read) is None

    # Reading the capture time: Location first, then Content-Location, then the final URL
    older = f"https://web.archive.org/web/20200101000000/{x}"
    cases = [
        ((page, None, None), stamp),
        ((None, f"/web/{stamp}/{x}", None), stamp),
        ((None, None, page), stamp),
        ((None, f"/web/{stamp}if_/{x}", None), stamp),
        ((page, None, older), stamp),
        # a captured page whose own path holds /web/<14 digits>/ is no capture
        ((None, None, f"https://example.org/web/{stamp}/page"), None),
        ((None, None, None), None),
    ]
    for (location, content_location, final_url), want in cases:
        got = timestamp_from_save_response(location=location, content_location=content_location, final_url=final_url)
        assert got == want, (location, content_location, final_url, got)

    # Where a link goes in custom: openCopyArchived right after openCopy, archived at the end
    custom = {"group": "g", "openCopy": x, "caveat": "c"}
    placed = add_link(custom=custom, field="openCopyArchived", link=page)
    assert list(placed) == ["group", "openCopy", "openCopyArchived", "caveat"]
    assert list(add_link(custom=custom, field="archived", link=page)) == ["group", "openCopy", "caveat", "archived"]
    assert list(custom) == ["group", "openCopy", "caveat"]

    # Which targets a ledger has: a DOI makes the URL durable, and a recorded link settles one
    pubmed = "https://pubmed.ncbi.nlm.nih.gov/1/"
    records = [
        {"id": "a2020-x", "DOI": "10.1/x", "URL": "https://doi.org/1", "custom": {"openCopy": x}},
        {"id": "b2021-y", "URL": pubmed, "custom": {"openCopy": x, "openCopyArchived": page}},
        {"id": "c2022-z", "URL": pubmed, "custom": {}},
        {"id": "d2023-w", "URL": page, "custom": {}},
    ]
    todo, left_out = list_targets(records=records, citekeys=[])
    assert todo == [
        Target(citekey="a2020-x", field="openCopyArchived", url=x),
        Target(citekey="b2021-y", field="archived", url=pubmed),
        Target(citekey="c2022-z", field="archived", url=pubmed),
    ]
    assert left_out == 2
    assert list_targets(records=records, citekeys=["c2022-z"]) == (todo[2:], 0)


def main() -> int:
    parser = argparse.ArgumentParser(
        description="Pin the ledger's links that have no DOI to Wayback Machine captures"
    )
    parser.add_argument("citekeys", nargs="*", metavar="citekey", help="only these entries (default: every entry)")
    parser.add_argument("--dry-run", action="store_true", help="list the targets, with no network and no write")
    parser.add_argument(
        "--lookup-only", action="store_true", help="record captures the Archive already holds and request none"
    )
    parser.add_argument("--self-check", action="store_true", help="check the built-in decisions and exit")
    args = parser.parse_args()
    if args.self_check:
        self_check()
        print("archive-sources self-check passed")
        return 0
    sys.stdout.reconfigure(line_buffering=True)

    try:
        records = read_ledger()
    except ArchiveError as exc:
        print(f"{LEDGER.name}: invalid ledger\n  - {exc}", file=sys.stderr)
        return 1
    unknown = [key for key in args.citekeys if key not in {entry["id"] for entry in records}]
    if unknown:
        print(f"{LEDGER.name}: unknown citekeys\n" + "\n".join(f"  - {key}" for key in unknown), file=sys.stderr)
        return 2

    todo, left_out = list_targets(records=records, citekeys=args.citekeys)
    if args.dry_run:
        print("dry run: no network, and nothing is written")
        for target in todo:
            print(f"{target.citekey}  {target.field}  {target.url}")
    print(f"targets to look up: {len(todo)}, left out: {left_out} (already archived, or already a web.archive.org link)")
    if named := [key for key in args.citekeys if not any(target.citekey == key for target in todo)]:
        print(f"named but left out: {', '.join(named)}")
    if args.dry_run or not todo:
        return 0
    return run(todo=todo, lookup_only=args.lookup_only)


if __name__ == "__main__":
    raise SystemExit(main())
