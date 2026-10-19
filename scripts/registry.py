#!/usr/bin/env python3
"""What the citation registries say about a work, and the rules for setting that beside the ledger.

docs/CITATIONS.csl.json marks each entry crossref-verified, openlibrary-verified or
pubmed-verified. Each label promises that the bibliographic fields printed in docs/CITATIONS.md
are the ones the registry returned. This module reads a registry's answer into the fields the
ledger carries and compares the two, so the promise can be checked on a schedule instead of once.
It also holds the facts about an entry's identifiers and links that check-sources.py,
archive-sources.py and generate-citations.py share. Everything here is a pure function over
parsed JSON.

A record is one work as a registry or the ledger describes it, cut down to the fields the two
share. A missing value is None, or [] for a list:
  title           the title, with markup and character references removed
  year            the earliest year the source offers
  years           every year the source offers, ascending and without repeats
  containerTitle  the journal
  volume, issue   as printed
  page            the first page of a range ("105-108" gives "105"), else the article number
  authors         family names in the order given, and whole names for Open Library
  publisher, numberOfPages, edition   what a book carries
  articleNumber, alternativeIds, license
                  Crossref only: its article-number, its alternative-id list and the license
                  URLs, the lists sorted and without repeats

Usage:
    uv run --no-project scripts/registry.py     # run the self-check
"""

from __future__ import annotations

import html
import itertools
import re
import unicodedata

SNAPSHOT_FORMAT = 1
# The scalar fields a record carries on both sides, in the order drift_between lists them
SCALAR_FIELDS = (
    "title", "year", "containerTitle", "volume", "issue", "page", "publisher", "numberOfPages", "edition",
)
# The fields each registry answers for, which are the ones agreement judges. A journal
# registry has no edition or page count, and Open Library has no journal
AGREEMENT_FIELDS = {
    "crossref": ("title", "year", "containerTitle", "volume", "issue", "page", "authors"),
    "openlibrary": ("title", "year", "publisher", "numberOfPages", "edition", "authors"),
    "pubmed": ("title", "year", "containerTitle", "volume", "issue", "page", "authors"),
}
# What each registry is called, and the ledger field that holds the id it is asked for. An entry
# goes to the first registry in this order whose id it carries
REGISTRY_NAMES = {"crossref": "Crossref", "openlibrary": "Open Library", "pubmed": "PubMed"}
ID_FIELDS = {"crossref": "DOI", "openlibrary": "ISBN", "pubmed": "PMID"}
# The two links an entry can carry, and the ledger field that holds each one's Wayback capture
ARCHIVE_FIELDS = {"URL": "archived", "openCopy": "openCopyArchived"}
WAYBACK_FORM = "https://web.archive.org/web/<14 digits>/<url>"
# A license URL on this host names a Creative Commons license
CC_HOST = "creativecommons.org"


def normalize_title(text: str) -> str:
    """A title with compatibility forms, case, punctuation and runs of spaces folded away and the words kept apart"""
    return re.sub(r"[\W_]+", " ", unicodedata.normalize("NFKC", text).lower()).strip()


def squeeze_title(text: str) -> str:
    """normalize_title with the spaces gone too, since Crossref damages titles by moving spaces or gluing on a digit"""
    return normalize_title(text=text).replace(" ", "")


def trimmed(value: object) -> str | None:
    """A non-blank string trimmed, a whole number as its digits, and None for anything else"""
    if isinstance(value, float) and value.is_integer():
        value = int(value)
    if isinstance(value, int) and not isinstance(value, bool):
        value = str(value)
    return (value.strip() or None) if isinstance(value, str) else None


def registry_text(value: object) -> str | None:
    """trimmed for a string from a registry, with markup removed and character references decoded"""
    text = trimmed(value=value)
    return None if text is None else trimmed(value=html.unescape(re.sub(r"</?[A-Za-z][^<>]*>", "", text)))


def year_in(value: object) -> int | None:
    """The first four-digit year inside a date string such as 'September 1, 1998' or '1992 Apr'"""
    found = re.search(r"(?<!\d)\d{4}(?!\d)", trimmed(value=value) or "")
    return int(found.group()) if found else None


def first_item(value: object) -> object:
    """The first element of a list, and anything else as it is"""
    return (value[0] if value else None) if isinstance(value, list) else value


def first_page(page: str) -> str:
    """The part of a page range before its first hyphen or dash"""
    return re.split("[-\u2010-\u2015\u2212]", page, maxsplit=1)[0].strip()


def date_parts(date: object) -> list[int]:
    """The leading integers of a Crossref date's first date-parts list: year, month, day.

    Empty when there is no date or its year isn't positive
    """
    try:
        parts = list(itertools.takewhile(lambda part: type(part) is int, date["date-parts"][0]))
    except (KeyError, IndexError, TypeError):
        return []
    return parts if parts and parts[0] > 0 else []


def date_text(date: object) -> str | None:
    """YYYY-MM-DD from a Crossref date object, with a missing month or day read as 01.

    Crossref fills a partial date the same way in the date-time that sits beside date-parts
    """
    stamp = date.get("date-time") if isinstance(date, dict) else None
    if isinstance(stamp, str) and re.match(r"\d{4}-\d{2}-\d{2}", stamp):
        return stamp[:10]
    parts = date_parts(date=date)
    if not parts:
        return None
    year, month, day = (parts + [1, 1])[:3]
    return f"{year:04d}-{month:02d}-{day:02d}"


def blank_record(**fields: object) -> dict:
    """A record with every field missing, then the given ones set. years is [year] unless given"""
    record = {**dict.fromkeys(SCALAR_FIELDS), "title": "", "authors": [], **fields}
    record.setdefault("years", [] if record["year"] is None else [record["year"]])
    return record


def ledger_fields(entry: dict) -> dict:
    """A ledger entry read into the shape of a registry record.

    The year is issued.date-parts[0][0], the page is the first page of its range, and an author
    with a literal name and no family name is left out, because there is nothing to compare
    """
    page = trimmed(value=entry.get("page"))
    parts = date_parts(date=entry.get("issued"))
    authors = [trimmed(value=a.get("family")) for a in entry.get("author") or [] if isinstance(a, dict)]
    return blank_record(
        title=trimmed(value=entry.get("title")) or "",
        year=parts[0] if parts else None,
        containerTitle=trimmed(value=entry.get("container-title")),
        volume=trimmed(value=entry.get("volume")),
        issue=trimmed(value=entry.get("issue")),
        page=None if page is None else first_page(page=page),
        authors=list(filter(None, authors)),
        publisher=trimmed(value=entry.get("publisher")),
        numberOfPages=trimmed(value=entry.get("number-of-pages")),
        edition=trimmed(value=entry.get("edition")),
    )


def record_from_crossref(message: dict) -> dict:
    """A Crossref works/<doi> message read into a record.

    years holds the year of each of issued, published-print and published-online that has one and
    year is the earliest, since the ledger may carry either. An author without a family name is an
    organization, and its name stands in. page is the first page of the range, else the article
    number, and alternativeIds is where a publisher can keep the article ID the ledger holds as page
    """
    years = sorted({
        parts[0] for key in ("issued", "published-print", "published-online")
        if (parts := date_parts(date=message.get(key)))
    })
    authors = [
        registry_text(value=a.get("family")) or registry_text(value=a.get("name"))
        for a in message.get("author") or [] if isinstance(a, dict)
    ]
    licenses = [trimmed(value=i.get("URL")) for i in message.get("license") or [] if isinstance(i, dict)]
    ids = message.get("alternative-id")
    alternative = [registry_text(value=i) for i in ids] if isinstance(ids, list) else []
    pages, number = registry_text(value=message.get("page")), registry_text(value=message.get("article-number"))
    return blank_record(
        title=registry_text(value=first_item(value=message.get("title"))) or "",
        year=years[0] if years else None,
        years=years,
        containerTitle=registry_text(value=first_item(value=message.get("container-title"))),
        volume=registry_text(value=message.get("volume")),
        issue=registry_text(value=message.get("issue")),
        # A journal that numbers its articles gives no page range, and the ledger keeps that
        # article number in page
        page=first_page(page=pages) if pages is not None else number,
        authors=list(filter(None, authors)),
        publisher=registry_text(value=message.get("publisher")),
        articleNumber=number,
        alternativeIds=sorted(set(filter(None, alternative))),
        license=sorted(set(filter(None, licenses))),
    )


def record_from_openlibrary(edition: dict, author_names: list[str]) -> dict:
    """An Open Library edition (openlibrary.org/isbn/<isbn>.json) read into a record.

    The title carries ': subtitle' when there is one. author_names stay whole, because a name can
    end in a title ("Satyananda Saraswati Swami"), and agreement looks for the family name inside
    """
    title = registry_text(value=edition.get("title")) or ""
    subtitle = registry_text(value=edition.get("subtitle"))
    year = year_in(value=edition.get("publish_date"))
    return blank_record(
        title=f"{title}: {subtitle}" if title and subtitle else title,
        year=year,
        authors=list(filter(None, [registry_text(value=name) for name in author_names])),
        publisher=registry_text(value=first_item(value=edition.get("publishers"))),
        numberOfPages=trimmed(value=edition.get("number_of_pages")),
        edition=registry_text(value=edition.get("edition_name")),
    )


def family_from_pubmed(name: object) -> str | None:
    """A PubMed author name without its trailing initials: 'Joshi LN' gives 'Joshi'"""
    words = (registry_text(value=name) or "").split()
    while len(words) > 1 and re.fullmatch(r"[A-Z]{1,4}", words[-1]):
        words.pop()
    return " ".join(words) or None


def record_from_pubmed(summary: dict) -> dict:
    """A PubMed esummary result (the result[<pmid>] object) read into a record, minus the period PubMed adds to a title"""
    title = registry_text(value=summary.get("title")) or ""
    year = year_in(value=summary.get("pubdate"))
    pages = registry_text(value=summary.get("pages"))
    authors = [family_from_pubmed(name=a.get("name")) for a in summary.get("authors") or [] if isinstance(a, dict)]
    return blank_record(
        title=title.removesuffix(".").rstrip(),
        year=year,
        containerTitle=registry_text(value=summary.get("fulljournalname")),
        volume=registry_text(value=summary.get("volume")),
        issue=registry_text(value=summary.get("issue")),
        page=None if pages is None else first_page(page=pages),
        authors=list(filter(None, authors)),
    )


def contains(a: str, b: str) -> bool:
    """True when one non-empty string sits inside the other"""
    return a != "" and b != "" and (a in b or b in a)


def within_words(part: str, whole: str) -> bool:
    """True when a non-empty normalized string sits inside another one on word boundaries"""
    return part != "" and f" {part} " in f" {whole} "


def page_agrees(ledger: dict, record: dict) -> bool:
    """The rule for page, which journals fill in as a range, an article number or nothing.

    Nothing on either side agrees. Otherwise the ledger's first page has to equal the registry's
    first page, its article number or one of its alternative ids
    """
    ours = ledger.get("page")
    page, number = record.get("page"), record.get("articleNumber")
    if ours is None or (page is None and number is None):
        return True
    theirs = [page, number, *(record.get("alternativeIds") or [])]
    return any(value is not None and first_page(page=ours) == first_page(page=value) for value in theirs)


def agreement(ledger: dict, record: dict, registry: str) -> dict[str, bool]:
    """Whether the ledger's record agrees with a registry's, field by field (AGREEMENT_FIELDS).

    - title: one squeezed title contains the other, which absorbs a subtitle that only one side
      has. A blank title never agrees
    - year: the registry offers no year, or the ledger's year is one of its years
    - containerTitle, publisher: the normalized strings contain one another on word boundaries.
      edition: they contain one another, so "3rd ed." agrees with "3rd edition". volume, issue,
      numberOfPages: the trimmed strings are equal. page: see page_agrees
    - authors: the ledger's first family name is among the registry's (under normalize_title).
      Open Library keeps whole names, so there it has to occur as whole words inside one. A
      registry with no authors never agrees

    A ledger without a value agrees, except for title and year. A registry without one agrees for
    year, containerTitle, volume, issue and page, and disagrees for the rest
    """
    if registry not in AGREEMENT_FIELDS:
        raise ValueError(f"unknown registry '{registry}', expected one of {sorted(AGREEMENT_FIELDS)}")

    def judge(field: str, how: str, absent_agrees: bool) -> bool:
        """how: equal trimmed strings, strings that contain one another on word boundaries or at all"""
        ours, theirs = ledger.get(field), record.get(field)
        if ours is None:
            return True
        if theirs is None:
            return absent_agrees
        if how == "equal":
            return ours.strip() == theirs.strip()
        a, b = normalize_title(text=ours), normalize_title(text=theirs)
        return within_words(part=a, whole=b) or within_words(part=b, whole=a) if how == "words" else contains(a=a, b=b)

    years = record.get("years") or []
    lead = (ledger.get("authors") or [None])[0]
    names = [normalize_title(text=name) for name in record.get("authors") or []]
    if lead is None:
        authors_agree = True
    elif registry == "openlibrary":
        authors_agree = any(within_words(part=normalize_title(text=lead), whole=name) for name in names)
    else:
        authors_agree = normalize_title(text=lead) in names
    judged = {
        "title": contains(
            a=squeeze_title(text=ledger.get("title") or ""), b=squeeze_title(text=record.get("title") or "")
        ),
        "year": not years or (ledger.get("year") is not None and ledger["year"] in years),
        "containerTitle": judge(field="containerTitle", how="words", absent_agrees=True),
        "volume": judge(field="volume", how="equal", absent_agrees=True),
        "issue": judge(field="issue", how="equal", absent_agrees=True),
        "page": page_agrees(ledger=ledger, record=record),
        "publisher": judge(field="publisher", how="words", absent_agrees=False),
        "numberOfPages": judge(field="numberOfPages", how="equal", absent_agrees=False),
        "edition": judge(field="edition", how="contains", absent_agrees=False),
        "authors": authors_agree,
    }
    return {field: judged[field] for field in AGREEMENT_FIELDS[registry]}


def drift_between(current: dict, at_check: dict) -> list[str]:
    """The fields that differ between the ledger now and as the last registry check saw it.

    The scalar fields come first, in SCALAR_FIELDS order, then authors. years is left out,
    since on the ledger side it only restates year
    """
    drifted = [field for field in SCALAR_FIELDS if current.get(field) != at_check.get(field)]
    if list(current.get("authors") or []) != list(at_check.get("authors") or []):
        drifted.append("authors")
    return drifted


def same_doi(a: object, b: object) -> bool:
    """DOIs are case-insensitive, and Crossref returns them lowercase"""
    ours, theirs = trimmed(value=a), trimmed(value=b)
    return ours is not None and theirs is not None and ours.casefold() == theirs.casefold()


def updates_from_crossref(message: dict, notices: list[dict], doi: str) -> list[dict]:
    """The notices that update a DOI: retractions, corrections, expressions of concern and the like.

    They come from message["updated-by"] and from the items of works?filter=updates:<doi>, whose
    update-to entries name the DOI they update. A notice found both ways counts once. Each is
    {"type": "retraction", "doi": "<notice doi>", "date": "YYYY-MM-DD"}, sorted by date, the type
    in Crossref's own words in lowercase with underscores and the date None when it gives none
    """
    updates = [(u, u.get("DOI"), None) for u in message.get("updated-by") or [] if isinstance(u, dict)]
    updates += [
        (u, n.get("DOI"), n)
        for n in notices if isinstance(n, dict)
        for u in n.get("update-to") or [] if isinstance(u, dict) and same_doi(a=u.get("DOI"), b=doi)
    ]
    found: dict[str, dict] = {}
    for update, notice_doi, notice in updates:
        named = trimmed(value=notice_doi)
        if named is None or named.casefold() in found:
            continue
        date = date_text(date=update.get("updated"))
        if date is None and notice is not None:
            date = date_text(date=notice.get("issued")) or date_text(date=notice.get("created"))
        kind = trimmed(value=update.get("type")) or "update"
        found[named.casefold()] = {"type": re.sub(r"[\s-]+", "_", kind.lower()), "doi": named, "date": date}
    return sorted(found.values(), key=lambda item: (item["date"] or "", item["doi"].casefold()))


def has_cc_license(record: dict) -> bool:
    """True when any license URL of a Crossref record names a Creative Commons license"""
    return any(CC_HOST in str(url).lower() for url in record.get("license") or [])


def identifier(entry: dict, service: str) -> str | None:
    """The DOI, ISBN or PMID that a check by this registry asks for, else None. An ISBN loses its hyphens and spaces"""
    value = str(entry.get(ID_FIELDS[service]) or "").strip()
    return ("".join(value.replace("-", "").split()) if service == "openlibrary" else value) or None


def link_targets(entry: dict) -> dict[str, str]:
    """The links of an entry that get a link check and a Wayback capture: its URL when it has no DOI, and its open copy"""
    open_copy = (entry.get("custom") or {}).get("openCopy")
    candidates = {"URL": None if entry.get("DOI") else entry.get("URL"), "openCopy": open_copy}
    return {kind: str(url).strip() for kind, url in candidates.items() if url}


def wayback_problem(link: object, target: str) -> str | None:
    """Why a Wayback link isn't a capture of the target page, or None when it is.

    It has to read https://web.archive.org/web/<14 digits>/<url>, with the url equal to the target
    once the scheme and a leading www. are dropped from both
    """
    found = re.fullmatch(r"https://web\.archive\.org/web/[0-9]{14}/(.+)", link) if isinstance(link, str) else None
    if found is None:
        return f"must look like {WAYBACK_FORM}"

    def bare(url: str) -> str:
        return re.sub(r"^https?://(?:www\.)?", "", url.strip(), flags=re.IGNORECASE)

    if bare(url=found[1]) != bare(url=target):
        return f"holds a capture of {found[1]}, and the page to capture is {target}"
    return None


def self_check() -> None:
    def failing(ledger: dict, record: dict, registry: str) -> list[str]:
        """The fields agreement finds in disagreement"""
        verdict = agreement(ledger=ledger, record=record, registry=registry)
        return [field for field, agrees in verdict.items() if not agrees]

    # folding and the small readers
    assert normalize_title(text="  CO\u2082: the \u201cPacer\u201d, 2nd ed.! \ufb01x ") == "co2 the pacer 2nd ed fix"
    assert squeeze_title(text="Peach Trees\u00a01 (Del.)") == "peachtrees1del"
    assert squeeze_title(text="") == "" and normalize_title(text="--") == ""
    assert first_page(page="165\u2013174") == "165" and first_page(page="e88695") == "e88695"
    assert registry_text(value="p<0.05 in <i>Homo</i> &amp; <mml:mi>CO</mml:mi>2") == "p<0.05 in Homo & CO2"
    assert trimmed(value=642) == "642" and trimmed(value=" ") is None and trimmed(value=True) is None
    assert year_in(value="September 1, 1998") == 1998 and year_in(value="n.d.") is None
    assert date_parts(date={"date-parts": [[2017, None, 3]]}) == [2017] and date_parts(date="2017") == []

    # a Crossref message with markup, a character reference, three dates and two kinds of license
    message = {
        "title": ["Slow breathing and <i>heart rate</i> variability: CO<sub>2</sub> &amp; pacing"],
        "author": [{"family": "Russo", "given": "M. A."}, {"name": "The Breathing Group"}],
        "issued": {"date-parts": [[2017]]},
        "published-print": {"date-parts": [[2017, 3]]},
        "published-online": {"date-parts": [[2016, 12, 5]]},
        "container-title": ["Journal of Breathing &amp; Health"],
        "volume": "13",
        "issue": "4",
        "page": "298\u2013309",
        "license": [
            {"URL": "https://www.example.org/tdm/userlicense/1.0/", "content-version": "tdm"},
            {"URL": "http://creativecommons.org/licenses/by/4.0/", "content-version": "vor"},
            {"URL": "http://creativecommons.org/licenses/by/4.0/", "content-version": "am"},
        ],
    }
    record = record_from_crossref(message=message)
    assert record["title"] == "Slow breathing and heart rate variability: CO2 & pacing"
    assert record["containerTitle"] == "Journal of Breathing & Health"
    assert (record["year"], record["years"]) == (2016, [2016, 2017])
    assert (record["page"], record["volume"], record["issue"]) == ("298", "13", "4")
    assert record["authors"] == ["Russo", "The Breathing Group"]
    assert record["license"] == [
        "http://creativecommons.org/licenses/by/4.0/",
        "https://www.example.org/tdm/userlicense/1.0/",
    ]
    assert has_cc_license(record=record) and not has_cc_license(record={"license": []})
    assert record_from_crossref(message={})["title"] == ""
    assert (record["articleNumber"], record["alternativeIds"]) == (None, [])
    numbered = record_from_crossref(message={"article-number": "e13905", "alternative-id": ["x1", "x1"]})
    assert (numbered["page"], numbered["articleNumber"], numbered["alternativeIds"]) == ("e13905", "e13905", ["x1"])
    both = record_from_crossref(message={"page": "1-16", "article-number": "8146809"})
    assert (both["page"], both["articleNumber"]) == ("1", "8146809")

    entry = {
        "title": "Slow breathing and heart rate variability: CO2 & pacing",
        "author": [{"family": "Russo", "given": "Mary"}, {"literal": "A consortium"}],
        "issued": {"date-parts": [[2017]]},
        "container-title": "Journal of Breathing & Health",
        "volume": "13",
        "page": "298-309",
    }
    ledger = ledger_fields(entry=entry)
    assert (ledger["year"], ledger["years"], ledger["page"]) == (2017, [2017], "298")
    assert ledger["authors"] == ["Russo"] and ledger["issue"] is None
    verdict = agreement(ledger=ledger, record=record, registry="crossref")
    assert list(verdict) == ["title", "year", "containerTitle", "volume", "issue", "page", "authors"]
    assert all(verdict.values())

    def edited(**changes: object) -> list[str]:
        return failing(ledger={**ledger, **changes}, record=record, registry="crossref")

    # the year may be any of the registry's dates, and a ledger with no year disagrees
    assert edited(year=2016) == [] and edited(year=2018) == ["year"] and edited(year=None) == ["year"]
    assert agreement(ledger=ledger, record={**record, "years": []}, registry="crossref")["year"]
    # a title agrees by squeezed containment, and a changed word or a blank title doesn't
    assert edited(title="Slow breathing and heart rate variability") == []
    assert edited(title="Slow breathing and heart-rate variability: CO2 & pacing!") == []
    assert edited(title="Slow breathing and blood pressure") == ["title"] and edited(title="") == ["title"]
    blank = agreement(ledger={**ledger, "title": ""}, record={**record, "title": ""}, registry="crossref")
    assert blank["title"] is False
    # a container agrees on word boundaries, and a partial word doesn't
    assert edited(containerTitle="Breathing & Health") == [] and edited(containerTitle=None) == []
    assert edited(containerTitle="Journal of Breath") == ["containerTitle"]
    # volume, issue and page compare as trimmed strings, the page by its first page
    assert edited(volume=" 13 ") == [] and edited(volume="14") == ["volume"]
    assert edited(issue="5") == ["issue"] and edited(issue=None) == []
    assert edited(page="298") == [] and edited(page="300-309") == ["page"] and edited(page=None) == []
    # the first family name decides, and case and an apostrophe's shape don't matter
    assert edited(authors=["russo", "Other"]) == [] and edited(authors=[]) == []
    assert edited(authors=["Smith", "Russo"]) == ["authors"]
    quoted = agreement(
        ledger={**ledger, "authors": ["O\u2019Brien"]}, record={**record, "authors": ["O'Brien"]}, registry="crossref"
    )
    assert quoted["authors"]
    try:
        agreement(ledger=ledger, record=record, registry="datacite")
        raise AssertionError("an unknown registry must raise")
    except ValueError:
        pass

    # drift: a one-letter title typo is caught, and so is a changed author list
    assert drift_between(current=ledger, at_check=ledger) == []
    assert drift_between(current={**ledger, "title": ledger["title"] + "x"}, at_check=ledger) == ["title"]
    assert drift_between(current={**ledger, "authors": ["Russo", "Lee"]}, at_check=ledger) == ["authors"]
    assert drift_between(current={**ledger, "years": [1], "issue": "9", "edition": "2nd"}, at_check=ledger) == [
        "issue", "edition",
    ]

    # page: a range needs the same first page, an article number may sit beside a range, an ID
    # that only the alternative ids carry counts (Hindawi), and a ledger or registry with none agrees
    hindawi = {"page": "1-16", "alternative-id": ["8146809", "8146809"]}
    for ledger_page, message, agrees in (
        ("298-309", {"page": "298-309"}, True), ("300", {"page": "298-309"}, False),
        ("e13905", {"article-number": "e13905"}, True), ("e13906", {"article-number": "e13905"}, False),
        ("15", {"page": "1-3", "article-number": "15"}, True),
        ("8146809", hindawi, True), ("8146810", hindawi, False), ("8146809", {"page": "1-16"}, False),
        (None, {"page": "1-16"}, True), ("756", {}, True), ("756", {"alternative-id": ["x1"]}, True),
    ):
        ours = ledger_fields(entry={"title": "T", "page": ledger_page})
        theirs = record_from_crossref(message={"title": ["T"], **message})
        assert agreement(ledger=ours, record=theirs, registry="crossref")["page"] is agrees, (ledger_page, message)

    # fields a registry lacks agree, except the title and the authors
    sparse = record_from_crossref(message={"title": ["Slow breathing"], "author": [{"family": "Russo"}]})
    full = ledger_fields(entry={**entry, "title": "Slow breathing", "issue": "4", "issued": {"date-parts": [[2017]]}})
    assert failing(ledger=full, record=sparse, registry="crossref") == []
    assert failing(ledger=full, record={**sparse, "volume": "14"}, registry="crossref") == ["volume"]
    assert failing(ledger=full, record={**sparse, "containerTitle": "Journal of Medicine"}, registry="crossref") == [
        "containerTitle"
    ]
    assert failing(ledger=full, record={**sparse, "page": "300"}, registry="crossref") == ["page"]
    assert failing(ledger=full, record={**sparse, "title": ""}, registry="crossref") == ["title"]
    assert failing(ledger=full, record={**sparse, "authors": []}, registry="crossref") == ["authors"]

    # Open Library: the book fields keep the strict rule, and a family name only has to occur in a whole name
    edition = {
        "title": "Hatha Yoga Pradipika",
        "publishers": ["Bihar School of Yoga"],
        "publish_date": "September 1, 1998",
        "number_of_pages": 642,
        "edition_name": "3rd edition",
    }
    book = record_from_openlibrary(edition=edition, author_names=["Swami Muktibodhananda"])
    assert book["authors"] == ["Swami Muktibodhananda"] and (book["year"], book["years"]) == (1998, [1998])
    assert (book["publisher"], book["numberOfPages"], book["edition"]) == ("Bihar School of Yoga", "642", "3rd edition")
    assert record_from_openlibrary(edition={**edition, "subtitle": "with notes"}, author_names=[])["title"] == (
        "Hatha Yoga Pradipika: with notes"
    )
    undated = record_from_openlibrary(edition={"title": "X", "publish_date": "n.d."}, author_names=[])
    assert undated["year"] is None and undated["years"] == []
    shelf = ledger_fields(entry={
        "title": "Hatha Yoga Pradipika",
        "author": [{"family": "Muktibodhananda", "given": "Swami"}],
        "issued": {"date-parts": [[1998]]},
        "publisher": "Bihar School of Yoga",
        "number-of-pages": "642",
        "edition": "3rd ed.",
    })
    verdict = agreement(ledger=shelf, record=book, registry="openlibrary")
    assert list(verdict) == ["title", "year", "publisher", "numberOfPages", "edition", "authors"] and all(verdict.values())

    def shelved(record: dict = book, **changes: object) -> list[str]:
        return failing(ledger={**shelf, **changes}, record=record, registry="openlibrary")

    assert shelved(numberOfPages="553") == ["numberOfPages"] and shelved(edition="4th ed.") == ["edition"]
    assert shelved(publisher="Bihar School") == [] and shelved(publisher="Bihar Sch") == ["publisher"]
    bare = {**book, "numberOfPages": None, "edition": None, "publisher": None}
    assert shelved(record=bare) == ["publisher", "numberOfPages", "edition"]
    saraswati = record_from_openlibrary(edition=edition, author_names=["Satyananda Saraswati Swami"])
    assert saraswati["authors"] == ["Satyananda Saraswati Swami"]
    assert shelved(authors=["Muktibodhananda"]) == [] and shelved(authors=["Swami"]) == []
    assert shelved(record=saraswati, authors=["Satyananda Saraswati"]) == []
    assert shelved(record=saraswati, authors=["Saraswat"]) == ["authors"]
    assert shelved(record=saraswati, authors=["Saraswati Satyananda"]) == ["authors"]
    assert shelved(record={**book, "authors": []}) == ["authors"]
    names = {"authors": ["Satyananda Saraswati"]}
    for service, agrees in (("crossref", False), ("pubmed", False), ("openlibrary", True)):
        assert agreement(ledger={"authors": ["Saraswati"]}, record=names, registry=service)["authors"] is agrees

    # PubMed: the period after a title and the initials after a surname go
    summary = {
        "pubdate": "1992 Apr",
        "authors": [{"name": "Joshi LN"}, {"name": "Van Dijk AB"}, {"name": "Gokhale LV"}],
        "title": "Effect of short term 'Pranayam' practice on breathing rate.",
        "volume": "36",
        "issue": "2",
        "pages": "105-8",
        "fulljournalname": "Indian journal of physiology and pharmacology",
    }
    pubmed = record_from_pubmed(summary=summary)
    assert pubmed["title"] == "Effect of short term 'Pranayam' practice on breathing rate"
    assert pubmed["authors"] == ["Joshi", "Van Dijk", "Gokhale"]
    assert (pubmed["year"], pubmed["page"], pubmed["volume"], pubmed["issue"]) == (1992, "105", "36", "2")
    joshi = ledger_fields(entry={
        "title": "Effect of short term 'Pranayam' practice on breathing rate.",
        "author": [{"family": "Joshi", "given": "L. N."}],
        "issued": {"date-parts": [[1992]]},
        "container-title": "Indian Journal of Physiology and Pharmacology",
        "volume": "36",
        "issue": "2",
        "page": "105-108",
    })
    assert all(agreement(ledger=joshi, record=pubmed, registry="pubmed").values())
    assert failing(ledger={**joshi, "volume": "37"}, record=pubmed, registry="pubmed") == ["volume"]
    assert record_from_pubmed(summary={})["authors"] == []

    # update notices: a retraction named by the work and by the search, a correction only the search knows,
    # a notice for another DOI, and a notice that carries only a year and a month
    retraction = {
        "DOI": "10.1234/notice-r",
        "type": "retraction",
        "updated": {"date-parts": [[2010, 2, 6]], "date-time": "2010-02-06T00:00:00Z"},
    }
    concern = {"DOI": "10.1234/notice-e", "type": "Expression of Concern", "updated": {"date-parts": [[2009, 5]]}}
    notices = [
        {"DOI": "10.1234/NOTICE-R", "update-to": [{"DOI": "10.1234/Work", "type": "retraction"}]},
        {
            "DOI": "10.1234/notice-c",
            "issued": {"date-parts": [[2004, 3]]},
            "update-to": [{"DOI": "10.1234/work", "type": "correction"}],
        },
        {"DOI": "10.1234/elsewhere", "update-to": [{"DOI": "10.1234/other", "type": "retraction"}]},
    ]
    found = updates_from_crossref(message={"updated-by": [retraction, concern]}, notices=notices, doi="10.1234/work")
    assert found == [
        {"type": "correction", "doi": "10.1234/notice-c", "date": "2004-03-01"},
        {"type": "expression_of_concern", "doi": "10.1234/notice-e", "date": "2009-05-01"},
        {"type": "retraction", "doi": "10.1234/notice-r", "date": "2010-02-06"},
    ]
    assert updates_from_crossref(message={}, notices=[], doi="10.1234/work") == []
    assert updates_from_crossref(message={}, notices=notices[2:], doi="10.1234/work") == []

    # identifiers and links
    assert identifier(entry={"DOI": " 10.1/x "}, service="crossref") == "10.1/x"
    assert identifier(entry={"ISBN": "978-81 85787-38 1"}, service="openlibrary") == "9788185787381"
    assert identifier(entry={"DOI": "10.1/x"}, service="pubmed") is None
    page = "https://web.archive.org/web/20261018123456/https://example.org/paper.pdf"
    doi_entry = {"DOI": "10.1/x", "URL": "https://doi.org/x", "custom": {"openCopy": " https://e.org/a "}}
    assert link_targets(entry=doi_entry) == {"openCopy": "https://e.org/a"}
    assert link_targets(entry={"URL": "https://e.org/b", "custom": {}}) == {"URL": "https://e.org/b"}
    assert wayback_problem(link=page, target="https://example.org/paper.pdf") is None
    assert wayback_problem(link=page, target="http://www.example.org/paper.pdf") is None
    assert wayback_problem(link=page, target="https://example.org/other.pdf")
    assert wayback_problem(link=page.replace("2026101812", "20261018"), target="https://example.org/paper.pdf")
    assert wayback_problem(link=page.replace("https", "http", 1), target="https://example.org/paper.pdf")

    print("registry self-check passed")


if __name__ == "__main__":
    self_check()
