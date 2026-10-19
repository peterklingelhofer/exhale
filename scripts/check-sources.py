#!/usr/bin/env python3
"""Ask the registries and the open web again about every source in the citation ledger.

docs/CITATIONS.csl.json records how each entry was checked (crossref-verified,
openlibrary-verified, pubmed-verified). Each of those checks happened once, by hand. A DOI that
stopped resolving, a registry that corrected a title, a page that moved and a field edited by
accident would all sit under their old label. This is the second look. The Sources workflow
(.github/workflows/sources.yml) runs it weekly and never on a push, because a publisher's outage
must not block a merge.

What it asks
  - Every entry with a DOI: Crossref works/<doi>, and works?filter=updates:<doi> for the notices
    (retractions, corrections, expressions of concern) that update it. An entry with an ISBN and
    no DOI: the Open Library edition and each of its authors. An entry with a PMID and no DOI:
    the PubMed summary. Each answer is read into the fields the ledger carries (scripts/registry.py)
    and set beside the ledger's own
  - Every entry with no DOI: GET its URL. Every custom.openCopy: GET it. Redirects are followed,
    and the status, the final URL and the SHA-256 of the streamed body are kept, never the body
  Requests go through scripts/fetching.py and start at most two a second. Crossref's public pool
  answers one request at a time and one search a second, so Crossref requests run one after
  another and the update searches start 1.25 s apart

What it reports (to stdout, and as markdown appended to the file given by --summary, which is
where the workflow points GITHUB_STEP_SUMMARY): registry fields that disagree with the ledger,
registry records that did not answer 200, links that did not answer 200, ledger fields that
changed since the snapshot, update notices, a Creative Commons license on an entry marked
paywalled, and (report only) links whose body changed and open-access entries with no Creative
Commons license in Crossref. The registry lists split into what is new since the snapshot and
what is standing.

What fails the run
  A job that failed on a standing 403 or a registry quirk would stay red and stop being read, so
  the exit status follows what changed since the snapshot. It is 1 when
  - a registry record answers something other than 200 and answered 200 in the snapshot, or has
    no check there
  - a link answered 200 in the snapshot and answers 404 or 410 now. The snapshot is written from
    another network than GitHub's runners and bot walls answer by IP, so a 403, a 203 challenge
    page, a 429, a 5xx, a timeout and a link with no check in the snapshot are listed and never fail
  - a field disagrees with its registry and agreed in the snapshot, or has no check there
  - a ledger field differs from the snapshot's copy of the ledger
  - an update notice appears that the snapshot did not have
  - a Creative Commons license newly appears on an entry marked paywalled
  and 0 otherwise. A problem the snapshot already had is listed as standing and does not fail
  the run. A changed body hash never does. A check made of another DOI, ISBN, PMID or URL than
  the ledger holds now counts as no check. With no snapshot every problem is new. Bad arguments
  exit 2

The snapshot
  docs/CITATIONS.registry.json holds what the registries said at the last --write, beside the
  ledger's fields at that moment. scripts/generate-citations.py holds the ledger to it with no
  network, so an edit to one of those fields fails CI until this script has looked at the entry
  again. --write replaces the snapshot after the report and leaves the exit status alone. A run
  limited to some citekeys replaces only their parts.

Usage:
    uv run --no-project scripts/check-sources.py                  # check, write nothing
    uv run --no-project scripts/check-sources.py --write          # check, then write the snapshot
    uv run --no-project scripts/check-sources.py --dry-run        # count the requests, make none
    uv run --no-project scripts/check-sources.py --summary FILE   # also append the report to FILE
    uv run --no-project scripts/check-sources.py KEY [KEY ...]    # check only those entries

Stdlib only, so it runs anywhere the repo is checked out
"""

from __future__ import annotations

import argparse
import collections
import json
import sys
import urllib.parse
from concurrent.futures import ThreadPoolExecutor, as_completed
from dataclasses import dataclass
from datetime import datetime, timezone
from pathlib import Path

import fetching
import registry

ROOT = Path(__file__).resolve().parent.parent
LEDGER = ROOT / "docs" / "CITATIONS.csl.json"
SNAPSHOT = ROOT / "docs" / "CITATIONS.registry.json"
USAGE = "check-sources.py [--write] [--dry-run] [--summary FILE] [citekey ...]"

# Requests start 0.5 s apart however many are in flight, which lets a few slow hosts overlap and
# never queues every request behind the slowest one. Crossref searches (works?filter=...) are
# limited harder than lookups, so they wait for a pacer of their own as well
PACER = fetching.Pacer(gap=0.5)
LIST_PACER = fetching.Pacer(gap=1.25)
LIST_QUERY = "https://api.crossref.org/works?"
CROSSREF_WORKERS, OTHER_WORKERS = 1, 3
HASH_SHOWN = 12
CELL_MAX = 90

# What a job asks, and how a table names it
LABELS = {**registry.REGISTRY_NAMES, "URL": "URL", "openCopy": "open copy"}
LINK_KINDS = ("URL", "openCopy")

# The lists find_problems fills. Only the first group fails the run
FAILING = ("disagree_new", "unanswered_new", "links_gone", "drifted", "updates_new", "cc_new")
STANDING = ("disagree_standing", "unanswered_standing", "updates_standing", "cc_standing")

# A link that answered 200 in the snapshot fails the run only when it answers one of these
# now. The snapshot is written from another network than GitHub's runners, bot walls answer by
# IP, and every open copy is meant to carry a Wayback capture, so 403, 429, 5xx and timeouts
# say little about the link and are listed without failing
GONE = (404, 410)

DISAGREE_COLUMNS = ["Citekey", "Registry", "Id", "Field", "Ledger", "Registry says"]
UNANSWERED_COLUMNS = ["Citekey", "Registry", "Id", "Status", "Note"]
LINK_COLUMNS = ["Citekey", "Kind", "URL", "Status", "In the snapshot", "Ended at"]
UPDATE_COLUMNS = ["Citekey", "Type", "Notice DOI", "Date"]
LICENSE_COLUMNS = ["Citekey", "DOI", "Crossref licenses"]
NEW_NOTE = "Agreed or answered 200 in the snapshot, or had no check there. Fails the run."
STANDING_NOTE = "The snapshot had the same problem. Does not fail the run."

# The report's sections in order: the list to print, its heading, its columns and a note
SECTIONS = [
    ("disagree_new", "New: registry fields that disagree with the ledger", DISAGREE_COLUMNS, NEW_NOTE),
    ("unanswered_new", "New: registry records that did not answer 200", UNANSWERED_COLUMNS, NEW_NOTE),
    (
        "links_gone",
        "New: links that went from 200 to 404 or 410",
        LINK_COLUMNS,
        "Answered 200 in the snapshot and answers 404 or 410 now. Fails the run.",
    ),
    (
        "drifted",
        "Ledger fields that changed since the snapshot",
        ["Citekey", "Field", "Was", "Now"],
        "Fails the run. The ledger holds the Now value and the snapshot holds the Was value. "
        "`uv run --no-project scripts/check-sources.py <citekey> --write` records the Now value "
        "after checking it against the registry.",
    ),
    ("updates_new", "New: update notices", UPDATE_COLUMNS, "The snapshot did not have them. Fails the run."),
    (
        "cc_new",
        "New: Creative Commons license on an entry marked paywalled",
        LICENSE_COLUMNS,
        "Crossref registers a Creative Commons license for a work the ledger calls paywalled. "
        "Fails the run. Mark the entry open-access, or say in its caveat why not.",
    ),
    (
        "links_other",
        "Links that did not answer 200, other than those above",
        LINK_COLUMNS,
        "Report only. The snapshot is written from another network than GitHub's runners, "
        "publishers' bot walls answer by IP, and every open copy is meant to carry a Wayback "
        "capture. So a 403, a challenge page, a 429, a 5xx, a timeout and a link with no check "
        "in the snapshot are listed here and never fail the run.",
    ),
    (
        "hash_changed",
        "Links whose body changed since the snapshot",
        ["Citekey", "Kind", "URL", "Was (SHA-256)", "Now (SHA-256)"],
        "Report only. A page that prints the date or a session token changes on every fetch.",
    ),
    (
        "open_without_cc",
        "Open-access entries with no Creative Commons license in Crossref",
        LICENSE_COLUMNS,
        "Report only. An entry is open-access when its license is registered or its title is "
        "fully open access, so this is a prompt to check the caveat.",
    ),
    ("disagree_standing", "Standing: registry fields that disagree with the ledger", DISAGREE_COLUMNS, STANDING_NOTE),
    ("unanswered_standing", "Standing: registry records that did not answer 200", UNANSWERED_COLUMNS, STANDING_NOTE),
    ("updates_standing", "Standing: update notices", UPDATE_COLUMNS, STANDING_NOTE),
    ("cc_standing", "Standing: Creative Commons license on an entry marked paywalled", LICENSE_COLUMNS, STANDING_NOTE),
]


@dataclass
class Job:
    """One thing to ask for: a registry record of an entry, or one of its links"""

    citekey: str
    kind: str
    ident: str
    entry: dict


def now() -> str:
    return datetime.now(timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ")


def describe_status(status: int | None) -> str:
    return "no answer" if status is None else str(status)


def read_json(response: object, deadline: float) -> object:
    """The parsed JSON body, so a body that isn't JSON is no answer. The deadline is unused"""
    return json.loads(response.read())


def get(url: str, read: object) -> fetching.Reply:
    pacers = (LIST_PACER, PACER) if url.startswith(LIST_QUERY) else (PACER,)
    return fetching.get(url=url, pacers=pacers, timeout=60, retry_wait=10, read=read)


def crossref_url(doi: str) -> str:
    return f"https://api.crossref.org/works/{urllib.parse.quote(doi, safe='')}"


def updates_url(doi: str) -> str:
    return f"{LIST_QUERY}filter=updates:{urllib.parse.quote(doi, safe='')}"


def edition_url(isbn: str) -> str:
    return f"https://openlibrary.org/isbn/{isbn}.json"


def pubmed_url(pmid: str) -> str:
    query = urllib.parse.urlencode({"db": "pubmed", "id": pmid, "retmode": "json"})
    return f"https://eutils.ncbi.nlm.nih.gov/entrez/eutils/esummary.fcgi?{query}"


def identify(entry: dict) -> tuple[str, str] | None:
    """The first registry whose id field the entry fills in, and the identifier to ask it for"""
    for service, field in registry.ID_FIELDS.items():
        if entry.get(field):
            return service, registry.identifier(entry=entry, service=service) or ""
    return None


def make_jobs(entries: list[dict]) -> list[Job]:
    jobs: list[Job] = []
    for entry in entries:
        if found := identify(entry=entry):
            jobs.append(Job(citekey=entry["id"], kind=found[0], ident=found[1], entry=entry))
    for entry in entries:
        for kind, url in registry.link_targets(entry=entry).items():
            jobs.append(Job(citekey=entry["id"], kind=kind, ident=url, entry=entry))
    return jobs


def message_of(body: object) -> dict | None:
    message = body.get("message") if isinstance(body, dict) else None
    return message if isinstance(message, dict) else None


def snapshot_record(job: Job, status: int | None, record: dict | None, updates: list[dict]) -> dict:
    ledger = registry.ledger_fields(entry=job.entry)
    return {
        "registry": job.kind,
        "id": job.ident,
        "status": status,
        "record": record,
        "ledger": ledger,
        "agrees": None if record is None else registry.agreement(ledger=ledger, record=record, registry=job.kind),
        "updates": updates,
        "checked": now(),
    }


def unanswered(job: Job, reply: fetching.Reply, note: str) -> tuple[dict, str]:
    """A check that found no record: the registry's own status, or None when its answer was unusable"""
    status = reply.status if reply.status != 200 else None
    return snapshot_record(job=job, status=status, record=None, updates=[]), note


def check_crossref(job: Job) -> tuple[dict, str]:
    lookup = get(url=crossref_url(doi=job.ident), read=read_json)
    message = message_of(body=lookup.body)
    if lookup.status != 200 or message is None:
        return unanswered(job=job, reply=lookup, note="" if lookup.status != 200 else "no message in the answer")
    # The notices come from a second query, and a check without them is not a check
    asked = get(url=updates_url(doi=job.ident), read=read_json)
    found = message_of(body=asked.body)
    items = found.get("items") if found else None
    if asked.status != 200 or not isinstance(items, list):
        reply = describe_status(status=asked.status) if asked.status != 200 else "no items"
        return unanswered(job=job, reply=asked, note=f"the updates query answered {reply}")
    record = registry.record_from_crossref(message=message)
    updates = registry.updates_from_crossref(message=message, notices=items, doi=job.ident)
    return snapshot_record(job=job, status=200, record=record, updates=updates), ""


def check_openlibrary(job: Job) -> tuple[dict, str]:
    lookup = get(url=edition_url(isbn=job.ident), read=read_json)
    edition = lookup.body if isinstance(lookup.body, dict) else None
    if lookup.status != 200 or edition is None:
        return unanswered(job=job, reply=lookup, note="" if lookup.status != 200 else "no edition in the answer")
    names: list[str] = []
    for author in edition.get("authors") or []:
        key = author.get("key") if isinstance(author, dict) else None
        if not isinstance(key, str) or not key.startswith("/authors/"):
            continue
        asked = get(url=f"https://openlibrary.org{key}.json", read=read_json)
        name = asked.body.get("name") if isinstance(asked.body, dict) else None
        if asked.status != 200 or not isinstance(name, str):
            note = f"the author record {key} answered {describe_status(status=asked.status)}"
            return unanswered(job=job, reply=asked, note=note)
        names.append(name)
    record = registry.record_from_openlibrary(edition=edition, author_names=names)
    return snapshot_record(job=job, status=200, record=record, updates=[]), ""


def check_pubmed(job: Job) -> tuple[dict, str]:
    lookup = get(url=pubmed_url(pmid=job.ident), read=read_json)
    result = lookup.body.get("result") if isinstance(lookup.body, dict) else None
    summary = result.get(job.ident) if isinstance(result, dict) else None
    if lookup.status == 200 and not (isinstance(summary, dict) and "error" not in summary):
        # PubMed answers 200 with an error object for a PMID it doesn't hold, which reads as a 404
        return unanswered(job=job, reply=lookup._replace(status=404), note="PubMed holds no summary for this PMID")
    if lookup.status != 200:
        return unanswered(job=job, reply=lookup, note="")
    return snapshot_record(job=job, status=200, record=registry.record_from_pubmed(summary=summary), updates=[]), ""


def check_link(job: Job) -> tuple[dict, str]:
    reply = get(url=job.ident, read=fetching.read_sha256)
    return {
        "url": job.ident,
        "status": reply.status,
        "finalUrl": reply.url,
        "bodySha256": reply.body,
        "checked": now(),
    }, ""


def run_jobs(jobs: list[Job]) -> tuple[dict[str, dict], dict[str, dict[str, dict]], dict[tuple[str, str], str]]:
    """Runs every job and answers (records, links, notes) keyed by citekey.

    Crossref jobs share one worker. The rest run OTHER_WORKERS at a time
    """
    records: dict[str, dict] = {}
    links: dict[str, dict[str, dict]] = {}
    notes: dict[tuple[str, str], str] = {}
    checks = {"crossref": check_crossref, "openlibrary": check_openlibrary, "pubmed": check_pubmed}
    crossref_lane = ThreadPoolExecutor(max_workers=CROSSREF_WORKERS)
    other_lane = ThreadPoolExecutor(max_workers=OTHER_WORKERS)
    try:
        futures = {}
        for job in jobs:
            lane = crossref_lane if job.kind == "crossref" else other_lane
            futures[lane.submit(checks.get(job.kind, check_link), job=job)] = job
        width = len(str(len(jobs)))
        for done, future in enumerate(as_completed(futures), start=1):
            job = futures[future]
            check, note = future.result()
            if job.kind in LINK_KINDS:
                links.setdefault(job.citekey, {})[job.kind] = check
            else:
                records[job.citekey] = check
            if note:
                notes[(job.citekey, job.kind)] = note
            status = describe_status(status=check["status"])
            print(f"[{done:{width}}/{len(jobs)}] {status} {job.citekey} ({LABELS[job.kind]})", file=sys.stderr)
    except BaseException:
        for lane in (crossref_lane, other_lane):
            lane.shutdown(wait=False, cancel_futures=True)
        raise
    crossref_lane.shutdown()
    other_lane.shutdown()
    return records, links, notes


def read_snapshot() -> dict:
    """The snapshot on disk, or an empty one. Raises ValueError for a file this script can't read"""
    empty = {"format": registry.SNAPSHOT_FORMAT, "checked": None, "records": {}, "links": {}}
    if not SNAPSHOT.exists():
        return empty
    snapshot = json.loads(SNAPSHOT.read_text(encoding="utf-8"))
    found = snapshot.get("format") if isinstance(snapshot, dict) else None
    if found != registry.SNAPSHOT_FORMAT:
        raise ValueError(f"{SNAPSHOT.name} has format {found!r}, and this script reads format {registry.SNAPSHOT_FORMAT}")
    return {**empty, **snapshot}


def differing_part(old: object, new: object) -> tuple[object, object]:
    """Two long strings cut to start 20 characters before they first differ.

    A table cell is clipped, and an edit near the end of a long title would otherwise be clipped away
    """
    if not (isinstance(old, str) and isinstance(new, str)) or max(len(old), len(new)) <= CELL_MAX:
        return old, new
    same = 0
    while same < min(len(old), len(new)) and old[same] == new[same]:
        same += 1
    start = max(0, same - 20)
    lead = "..." if start else ""
    return lead + old[start:], lead + new[start:]


def find_problems(
    entries: dict[str, dict],
    records: dict[str, dict],
    links: dict[str, dict[str, dict]],
    notes: dict[tuple[str, str], str],
    previous: dict,
) -> dict[str, list[list[object]]]:
    """Sorts what the checks found into the lists the report prints.

    A problem is new when the snapshot had no check of the same identifier or URL, or a clean one,
    and standing when the snapshot had the same problem. Fields are judged one at a time, and a
    snapshot check that found no record to compare cannot have had a disagreement. Only a link
    that answered 200 in the snapshot and answers one of GONE now fails the run. Every other link
    that did not answer 200 is listed with the status the snapshot had
    """
    found: dict[str, list[list[object]]] = collections.defaultdict(list)
    for citekey, check in sorted(records.items()):
        label = LABELS[check["registry"]]
        # The snapshot's check of the same registry and identifier. Another one counts as no check
        before = previous["records"].get(citekey)
        if not (
            isinstance(before, dict)
            and before.get("registry") == check["registry"]
            and registry.same_doi(a=before.get("id"), b=check["id"])
        ):
            before = None
        if check["status"] != 200:
            standing = before is not None and before.get("status") != 200
            note = notes.get((citekey, check["registry"]), "")
            row = [citekey, label, check["id"], describe_status(status=check["status"]), note]
            found["unanswered_standing" if standing else "unanswered_new"].append(row)
        for field, agrees in (check["agrees"] or {}).items():
            if agrees:
                continue
            standing = ((before or {}).get("agrees") or {}).get(field) is False
            offered = check["record"]["years"] if field == "year" else check["record"][field]
            ours, theirs = differing_part(old=check["ledger"][field], new=offered)
            found["disagree_standing" if standing else "disagree_new"].append(
                [citekey, label, check["id"], field, ours, theirs]
            )
        earlier = (previous["records"].get(citekey) or {}).get("ledger")
        if earlier is not None:
            for field in registry.drift_between(current=check["ledger"], at_check=earlier):
                was, now_value = differing_part(old=earlier.get(field), new=check["ledger"][field])
                found["drifted"].append([citekey, field, was, now_value])
        known = {str(notice.get("doi")).casefold() for notice in (before or {}).get("updates") or []}
        for notice in check["updates"]:
            row = [citekey, notice["type"], notice["doi"], notice["date"]]
            found["updates_standing" if notice["doi"].casefold() in known else "updates_new"].append(row)
        record = check["record"]
        if record is None or check["registry"] != "crossref":
            continue
        access = (entries[citekey].get("custom") or {}).get("accessLevel")
        has_cc = registry.has_cc_license(record=record)
        row = [citekey, check["id"], record["license"]]
        if has_cc and access == "paywalled":
            earlier_record = (before or {}).get("record")
            was_cc = earlier_record is not None and registry.has_cc_license(record=earlier_record)
            found["cc_standing" if was_cc else "cc_new"].append(row)
        elif not has_cc and access == "open-access":
            found["open_without_cc"].append(row)

    for citekey, kinds in sorted(links.items()):
        for kind, check in sorted(kinds.items()):
            before = (previous["links"].get(citekey) or {}).get(kind)
            if not (isinstance(before, dict) and before.get("url") == check["url"]):
                before = None
            if check["status"] != 200:
                was = "no check" if before is None else describe_status(status=before.get("status"))
                landed = check["finalUrl"] if check["finalUrl"] != check["url"] else ""
                status = describe_status(status=check["status"])
                row = [citekey, LABELS[kind], check["url"], status, was, landed]
                gone = before is not None and before.get("status") == 200 and check["status"] in GONE
                found["links_gone" if gone else "links_other"].append(row)
            # A hash only means something when both fetches answered 200
            was_hash, now_hash = (before or {}).get("bodySha256"), check["bodySha256"]
            if was_hash and now_hash and was_hash != now_hash:
                row = [citekey, LABELS[kind], check["url"], was_hash[:HASH_SHOWN], now_hash[:HASH_SHOWN]]
                found["hash_changed"].append(row)
    return found


def failures(found: dict[str, list[list[object]]]) -> int:
    """The number of problems that fail the run"""
    return sum(len(found[name]) for name in FAILING)


def cell(value: object) -> str:
    if isinstance(value, list):
        text = ", ".join(str(item) for item in value) or "(none)"
    else:
        text = "(none)" if value is None or value == "" else str(value)
    text = " ".join(text.split())
    clipped = text if len(text) <= CELL_MAX else f"{text[: CELL_MAX - 3]}..."
    # A DOI can hold angle brackets, which a markdown table would swallow as a tag
    return clipped.replace("|", "\\|").replace("<", "&lt;").replace(">", "&gt;")


def plural(n: int, noun: str) -> str:
    return f"{n} {noun}" + ("" if n == 1 else "s")


def render_report(found: dict[str, list[list[object]]], jobs: list[Job], limit: str) -> str:
    n_records = sum(1 for job in jobs if job.kind not in LINK_KINDS)
    standing = sum(len(found[name]) for name in STANDING)
    lines = [
        "## Sources",
        "",
        f"Checked {plural(n=n_records, noun='registry record')} and {plural(n=len(jobs) - n_records, noun='link')}{limit}. "
        f"New since the snapshot: {failures(found=found)}. Standing: {standing}. "
        f"Link problems that don't fail the run: {len(found['links_other'])}.",
        "",
        "The run fails only on what is new since the snapshot. What the snapshot already had is "
        "listed as standing. With no snapshot everything is new, so the first run fails on all of "
        "it and `--write` records the baseline.",
        "",
    ]
    for name, heading, columns, note in SECTIONS:
        rows = found[name]
        lines += [f"### {heading} ({len(rows)})", "", note, ""]
        if rows:
            lines += [f"| {' | '.join(columns)} |", "|" + " --- |" * len(columns)]
            lines += [f"| {' | '.join(cell(value=value) for value in row)} |" for row in rows]
        else:
            lines.append("None.")
        lines.append("")
    return "\n".join(lines)


def next_snapshot(
    previous: dict,
    entries: dict[str, dict],
    checked: list[dict],
    records: dict[str, dict],
    links: dict[str, dict[str, dict]],
    started: str,
) -> dict:
    """This run's checks laid over the earlier ones that the ledger still backs.

    The earlier ones only survive a run limited to some citekeys, since a full run redoes every
    check. A check survives when its entry is outside this run and still has the same
    identifier or URL
    """
    redone = {entry["id"] for entry in checked}
    kept_records = {}
    for citekey, before in previous["records"].items():
        found = identify(entry=entries[citekey]) if citekey in entries else None
        same = found and found[0] == before.get("registry") and registry.same_doi(a=found[1], b=before.get("id"))
        if citekey not in redone and same:
            kept_records[citekey] = before
    kept_links: dict[str, dict[str, dict]] = {}
    for citekey, kinds in previous["links"].items():
        targets = registry.link_targets(entry=entries[citekey]) if citekey in entries else {}
        still = {kind: old for kind, old in kinds.items() if targets.get(kind) == old.get("url")}
        if citekey not in redone and still:
            kept_links[citekey] = still
    return {
        "format": registry.SNAPSHOT_FORMAT,
        "checked": started,
        "records": {**kept_records, **records},
        "links": {**kept_links, **links},
    }


def dry_run(jobs: list[Job]) -> None:
    """Counts the requests and shows the first three of each kind, making none"""
    by_kind = {kind: [job for job in jobs if job.kind == kind] for kind in LABELS}
    groups = [
        ("Crossref lookups", by_kind["crossref"], lambda job: crossref_url(doi=job.ident)),
        ("Crossref update queries", by_kind["crossref"], lambda job: updates_url(doi=job.ident)),
        (
            "Open Library editions (each author record adds a request)",
            by_kind["openlibrary"],
            lambda job: edition_url(isbn=job.ident),
        ),
        ("PubMed summaries", by_kind["pubmed"], lambda job: pubmed_url(pmid=job.ident)),
        (
            f"Link fetches ({len(by_kind['URL'])} URL, {len(by_kind['openCopy'])} open copy)",
            [job for job in jobs if job.kind in LINK_KINDS],
            lambda job: job.ident,
        ),
    ]
    print(f"User-Agent: {fetching.USER_AGENT}")
    for label, group, url_for in groups:
        print(f"{label}: {len(group)}")
        for job in group[:3]:
            print(f"  {job.citekey}  GET {url_for(job=job)}")
    total = sum(len(group) for _, group, _ in groups)
    print(f"{total} requests before the author records, at two a second. Nothing was fetched or written")


def parse_args(argv: list[str]) -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        prog="check-sources.py",
        usage=USAGE,
        description="Ask the registries and the open web again about every source in the citation ledger.",
    )
    parser.add_argument("--write", action="store_true", help="replace docs/CITATIONS.registry.json after the report")
    parser.add_argument(
        "--dry-run", action="store_true", help="count the requests and show the first three of each kind, making none"
    )
    parser.add_argument("--summary", metavar="FILE", help="also append the report to FILE")
    parser.add_argument("citekeys", nargs="*", metavar="citekey", help="check only these entries")
    args = parser.parse_args(argv)
    if args.write and args.dry_run:
        parser.error("--dry-run makes no requests, so there is nothing to --write")
    return args


def main() -> int:
    args = parse_args(argv=sys.argv[1:])

    try:
        ledger = json.loads(LEDGER.read_text(encoding="utf-8"))
        entries = {entry["id"]: entry for entry in ledger}
    except (OSError, ValueError, KeyError, TypeError) as exc:
        print(f"cannot read {LEDGER.name}: {exc}", file=sys.stderr)
        return 2

    unknown = [key for key in args.citekeys if key not in entries]
    if unknown:
        print(f"not in the ledger: {', '.join(unknown)}\n{USAGE}", file=sys.stderr)
        return 2
    chosen = [entries[key] for key in dict.fromkeys(args.citekeys)] if args.citekeys else ledger
    jobs = make_jobs(entries=chosen)

    if args.dry_run:
        dry_run(jobs=jobs)
        return 0

    try:
        previous = read_snapshot()
    except (OSError, ValueError) as exc:
        print(f"cannot read {SNAPSHOT.name}: {exc}", file=sys.stderr)
        return 2

    started = now()
    records, links, notes = run_jobs(jobs=jobs)
    found = find_problems(entries=entries, records=records, links=links, notes=notes, previous=previous)
    limit = f" (limited to {plural(n=len(chosen), noun='citekey')})" if args.citekeys else ""
    report = render_report(found=found, jobs=jobs, limit=limit)
    print(report)
    if args.summary:
        with open(args.summary, "a", encoding="utf-8") as handle:
            handle.write(report + "\n")

    if args.write:
        snapshot = next_snapshot(
            previous=previous, entries=entries, checked=chosen, records=records, links=links, started=started
        )
        SNAPSHOT.write_text(json.dumps(snapshot, indent=2, ensure_ascii=False, sort_keys=True) + "\n", encoding="utf-8")
        n_links = sum(len(kinds) for kinds in snapshot["links"].values())
        where = SNAPSHOT.relative_to(ROOT)
        print(f"wrote {len(snapshot['records'])} records and {n_links} links to {where}", file=sys.stderr)

    return 1 if failures(found=found) > 0 else 0


if __name__ == "__main__":
    raise SystemExit(main())
