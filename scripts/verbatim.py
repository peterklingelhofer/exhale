#!/usr/bin/env python3
"""The text rules behind the citation corpus's quote receipts.

How a source text becomes searchable, when a ledger quote is in it, which quotes an entry may
carry and which receipts the ledger still backs. scripts/check-quotes.py and
scripts/generate-citations.py both import this module, so a change to a rule reaches both.

Usage:
    uv run --no-project scripts/verbatim.py    # self-check on worked examples

Stdlib only, so it runs anywhere the repo is checked out
"""

from __future__ import annotations

import difflib
import hashlib
import html
import re
import unicodedata
from collections.abc import Iterator

RECEIPTS_FORMAT = 1
# The keys of a receipt's found record in the order the file keeps them, and the two scopes a
# quote is found in. A locator that starts with the word Abstract points at the abstract and any
# other locator at the body
FOUND_KEYS = ("variant", "source", "scope", "chars", "textSha256", "date")
SCOPES = ("abstract", "body")
ABSTRACT_LOCATOR = "Abstract"
ELISION = "[...]"
MAX_GAP = 2000          # characters allowed between two segments of an elided quote
STUB_CHARS = 500        # a cached text shorter than this is a paywall or bot page
PREVIEW_CHARS = 60      # how much of a quote a receipt keeps, enough to find it in the ledger
MAX_LOCATOR = 80        # longest locator a quote may carry
BREAK_CONTEXT = 40      # characters of a segment shown either side of a break
ANCHOR_CHARS = 12       # a matched prefix shorter than this is too short to say where the text goes on
WINDOW = 240            # characters of text shown as the closest match for a missing segment
WINDOW_STEP = 40
OCR_HEAD = 4000         # squeezed characters at the start of a text searched for a title a scan misread
OCR_RATIO = 0.9         # difflib ratio a window of that text needs to count as the title
OCR_MIN_TITLE = 20      # squeezed characters a title needs before OCR errors are forgiven

# Step 2 of normalize. U+2033 is folded before NFKC, which would split it into two primes
QUOTE_MARKS = str.maketrans({
    "\u2018": "'", "\u2019": "'", "\u201a": "'", "\u2032": "'",
    "\u201c": '"', "\u201d": '"', "\u201e": '"',
})
# Step 3: hyphens, dashes and minus signs all become one hyphen
DASHES = str.maketrans({
    chr(code): "-"
    for code in (*range(0x2010, 0x2016), 0x2212, 0xFE58, 0xFE63, 0xFF0D)
})
ELISION_SPLIT = re.compile(r"\s*\[\.\.\.\]\s*")

# Tags that end a line of text. Anything else is dropped without a trace, so CO<sub>2</sub> reads CO2
BLOCK_TAGS = (
    "title p sec abstract caption label td th tr list-item fig table-wrap article-title "
    "ref li div h1 h2 h3 h4 h5 h6 br"
).split()

# A quoted span in single or double quotes (the agrivoltaic QUOTED_SPAN rule). A single-quoted
# span may hold an apostrophe that a letter follows, as in don't. It doesn't open after a letter
# or digit and it ends at a quote that no letter or digit follows
DOUBLE_SPAN = r'"([^"]*)"|\u201c([^\u201d]*)\u201d'
SINGLE_SPAN = r"(?<!\w)'((?:[^']|'(?=\w))*)'(?!\w)|\u2018((?:[^\u2019]|\u2019(?=\w))*)\u2019(?!\w)"
DOUBLE_SPANS = re.compile(DOUBLE_SPAN)
ALL_SPANS = re.compile(SINGLE_SPAN + "|" + DOUBLE_SPAN)


def text_digest(normalized_text: str) -> str:
    """SHA-256 of an already normalized text as 64 lowercase hex digits"""
    return hashlib.sha256(normalized_text.encode("utf-8")).hexdigest()


def normalize(text: str) -> str:
    """The text a quote is looked up in, with typography and line wrapping evened out.

    Case stays. Curly quotes, dashes and ligatures fold, whitespace runs become one space and
    spacing around punctuation and operators folds. A hyphen or dash with a letter on both sides
    is ignored, because PDFs break and join words there (rest-and-digest, rest-anddigest and
    inter-national all compare equal), so a quote can't start or end at one. A hyphen next to a
    digit stays (26-30), and a dash with a space on either side keeps its spaces
    """
    text = text.replace("\u2033", '"')
    text = unicodedata.normalize("NFKC", text).translate(QUOTE_MARKS).translate(DASHES)
    text = re.sub(r"\u00ad\s*\n\s*", "", text).replace("\u00ad", "")
    text = re.sub(r"-\s*\n\s*", "-", text)                  # a hyphen at a line end, whatever comes before it
    text = re.sub(r"\s+", " ", text)
    text = re.sub(r"(?<=[^\W\d_])-(?=[^\W\d_])", "", text)  # a hyphen between two letters
    text = re.sub(r" ([.,;:!?)\]%])", r"\1", text)
    text = re.sub(r"([(\[]) ", r"\1", text)
    text = re.sub(r" ?([=<>\u2264\u2265\u00b1]) ?", r"\1", text)
    return text.strip()


def quote_key(text: str) -> str:
    """The identity of a quote: SHA-256 of its normalized text as 64 lowercase hex digits"""
    return text_digest(normalized_text=normalize(text=text))


def quote_preview(text: str) -> str:
    """The first characters of a quote, for a person looking for it in the ledger"""
    return text[:PREVIEW_CHARS]


def quotes_of(record: dict) -> list:
    """The quotes of a ledger entry, empty when it has none"""
    return (record.get("custom") or {}).get("quotes") or []


def segments(quote: str) -> list[str]:
    """The normalized segments of a quote cut at its elisions, in order"""
    pieces = (normalize(text=piece) for piece in ELISION_SPLIT.split(quote))
    return [piece for piece in pieces if piece]


def _occurrences(segment: str, text: str, low: int, high: int) -> Iterator[int]:
    """Start positions in low..high where segment sits in text on word boundaries"""
    position = text.find(segment, low)
    while position != -1 and position <= high:
        end = position + len(segment)
        before_ok = not (segment[0].isalnum() and position > 0 and text[position - 1].isalnum())
        after_ok = not (segment[-1].isalnum() and end < len(text) and text[end].isalnum())
        if before_ok and after_ok:
            yield position
        position = text.find(segment, position + 1)


def find_quote(quote: str, normalized_text: str) -> tuple[int, int] | None:
    """Where a quote sits in a normalized text, as (start, end), or None.

    The segments of the quote have to come in order, each starting on word boundaries and no
    more than MAX_GAP characters after the one before. Case counts
    """
    pieces = segments(quote=quote)
    if not pieces:
        return None
    dead: set[tuple[int, int]] = set()      # (segment index, end) known to lead nowhere

    def match_from(index: int, low: int, high: int) -> tuple[int, int] | None:
        for start in _occurrences(segment=pieces[index], text=normalized_text, low=low, high=high):
            end = start + len(pieces[index])
            if index == len(pieces) - 1:
                return start, end
            if (index, end) in dead:
                continue
            rest = match_from(index=index + 1, low=end, high=end + MAX_GAP)
            if rest is not None:
                return start, rest[1]
            dead.add((index, end))
        return None

    return match_from(index=0, low=0, high=len(normalized_text))


def _matched_prefix(segment: str, text: str) -> int:
    """Length of the longest prefix of segment that occurs in text, by bisection"""
    low, high = 0, len(segment)
    while low < high:
        middle = (low + high + 1) // 2
        if segment[:middle] in text:
            low = middle
        else:
            high = middle - 1
    return low


def _best_window(segment: str, text: str) -> str:
    """The WINDOW characters of text that share the most character trigrams with segment"""
    grams = {segment[i:i + 3] for i in range(len(segment) - 2)}
    best, best_shared = "", 0
    for start in range(0, len(text), WINDOW_STEP):
        chunk = text[start:start + WINDOW]
        shared = len({chunk[i:i + 3] for i in range(len(chunk) - 2)} & grams)
        if shared > best_shared:
            best, best_shared = chunk, shared
    return best


def segment_report(quote: str, normalized_text: str, window: bool = True) -> list[dict]:
    """One dict per segment of a quote, to show where a quote that isn't found goes wrong.

    found: on word boundaries, in any order and at any distance. position: where, or -1.
    substring: occurs, perhaps inside a longer word. matched: length of the longest prefix that
    occurs. before, after: BREAK_CONTEXT characters of the segment either side of the break.
    text_has: what the text says after the matched prefix. text_before: what it says before an
    occurrence inside a word. window: the WINDOW characters of text that share the most trigrams
    with the segment, left empty when window is False because the search is slow
    """
    reports = []
    for segment in segments(quote=quote):
        position = next(
            _occurrences(segment=segment, text=normalized_text, low=0, high=len(normalized_text)), -1
        )
        matched = _matched_prefix(segment=segment, text=normalized_text)
        report = {
            "segment": segment, "found": position != -1, "position": position,
            "substring": matched == len(segment), "matched": matched, "before": "", "after": "",
            "text_has": "", "text_before": "", "window": "",
        }
        if position == -1:
            at = normalized_text.find(segment[:matched])
            report["before"] = segment[max(0, matched - BREAK_CONTEXT):matched]
            report["after"] = segment[matched:matched + BREAK_CONTEXT]
            if matched >= ANCHOR_CHARS or report["substring"]:
                report["text_has"] = normalized_text[at + matched:at + matched + BREAK_CONTEXT]
            if report["substring"]:
                report["text_before"] = normalized_text[max(0, at - BREAK_CONTEXT):at]
            if window:
                report["window"] = _best_window(segment=segment, text=normalized_text)
        reports.append(report)
    return reports


def xml_to_text(markup: str) -> str:
    """Text from the XML or HTML this tooling reads (Europe PMC JATS and abstract HTML, Crossref JATS).

    Comments and namespace prefixes go, block tags become line breaks, a citation tag becomes a
    space so its number never glues to the word before it, every other tag vanishes and
    character references are decoded
    """
    text = re.sub(r"<!--.*?-->", "", markup, flags=re.S)
    text = re.sub(r"<(/?)[A-Za-z_][\w.-]*:(?=[A-Za-z_])", r"<\1", text)
    text = re.sub(r"</?(?:" + "|".join(BLOCK_TAGS) + r")(?=[\s/>])[^<>]*>", "\n", text, flags=re.I)
    text = re.sub(r"<xref(?=[\s/>])[^<>]*>", " ", text, flags=re.I)
    # A tag opens with a letter, a slash, ? or !, so the < in "p < 0.05" in a plain text abstract survives
    text = re.sub(r"<(?:/?[A-Za-z][^<>]*|[?!][^<>]*)>", "", text)
    text = re.sub(r"[ \t\r]+\n", "\n", html.unescape(text))
    return re.sub(r"\n{3,}", "\n\n", text).strip()


def squeeze(text: str) -> str:
    """Lowercase letters and digits only, so two spellings of one title compare equal"""
    return "".join(filter(str.isalnum, unicodedata.normalize("NFKC", text).lower()))


def _title_misread(title: str, squeezed_text: str) -> bool:
    """Some window of the first OCR_HEAD characters is the squeezed title give or take a few letters"""
    size = len(title)
    if size < OCR_MIN_TITLE:
        return False
    head = squeezed_text[:OCR_HEAD]
    # quick_ratio is an upper bound on ratio, so a window it rules out can't pass. It is cheap
    # because the title is counted once and a window only has to be counted against it
    screen = difflib.SequenceMatcher(None, autojunk=False)
    screen.set_seq2(title)
    for start in range(len(head) - size + 1):
        window = head[start:start + size]
        screen.set_seq1(window)
        if screen.quick_ratio() < OCR_RATIO:
            continue
        if difflib.SequenceMatcher(None, title, window, autojunk=False).ratio() >= OCR_RATIO:
            return True
    return False


def title_in_text(title: str, text: str) -> bool:
    """The right-document guard: does the text carry the entry's title.

    Letters and digits only, case ignored. It passes when the title occurs in the text, when the
    part before its first colon or question mark occurs and is at least 20 characters, or, for
    a scan whose OCR misreads a few letters (EFFECf for Effect), when some title-sized window of
    the first OCR_HEAD characters matches it with a difflib ratio of at least OCR_RATIO
    """
    squeezed = squeeze(text=text)
    whole = squeeze(text=title)
    if whole and whole in squeezed:
        return True
    mark = re.search(r"[:?]", title)
    if mark is not None:
        head = squeeze(text=title[:mark.start()])
        if len(head) >= 20 and head in squeezed:
            return True
    return _title_misread(title=whole, squeezed_text=squeezed)


def quoted_spans(prose: str, single: bool = True) -> list[str]:
    """The spans inside quote marks, in order, stripped, with a trailing period or comma removed.

    Straight and curly double quotes count, and with single=True so do single quotes, where an
    apostrophe (don't) and a possessive (the authors' view) open no span
    """
    spans = []
    for match in (ALL_SPANS if single else DOUBLE_SPANS).finditer(prose):
        span = next(group for group in match.groups() if group is not None).strip()
        span = span[:-1].rstrip() if span.endswith((".", ",")) else span
        if span:
            spans.append(span)
    return spans


def has_bare_ellipsis(quote: str) -> bool:
    """A three-period ellipsis or the one character remains once every [...] is removed"""
    rest = quote.replace(ELISION, "")
    return "..." in rest or "\u2026" in rest


def locator_scope(locator: str) -> str:
    """Where a locator points: abstract when it starts with the word Abstract ("Abstract" or
    "Abstract, Significance Statement"), else body"""
    return SCOPES[0] if re.match(rf"{ABSTRACT_LOCATOR}\b", locator) else SCOPES[1]


def locator_flaw(locator: str, scope: str, source: str) -> str | None:
    """Why a quote's locator and the place it was found disagree, or None.

    The locator decides where a quote has to be found. One that starts with the word Abstract
    points at the abstract variant. Any other points at the body, a usable variant other than the
    abstract, whether or not the abstract has the words too. scope is where the quote was found
    (abstract or body) and source names that text
    """
    points_at = locator_scope(locator=locator)
    if scope not in SCOPES or scope == points_at:
        return None
    if points_at == SCOPES[0]:
        return (
            f'has locator "{locator}", which points at the abstract, and was found in the body ({source}). '
            "Locate it by the section or page it sits on"
        )
    return (
        f'has locator "{locator}", which points at the body, and was found in the abstract ({source}). '
        f'If the words are only in the abstract, start the locator with {ABSTRACT_LOCATOR}, '
        f'for example "{ABSTRACT_LOCATOR}, {locator}"'
    )


def quote_flaws(quotes: object, claim_count: int, read_depth: str | None = None) -> list[str]:
    """Flaws in an entry's custom.quotes, one message each, empty when the quotes are sound.

    Each item is {text, locator, claim}. The text is the source's own words on one line with no
    outer whitespace and the only elision is " [...] " between two segments. The locator has 1 to
    MAX_LOCATOR characters, the claim indexes backsClaims or is null, no two quotes share a key
    and a catalog-record entry has none. Whether the words are in the source is for check-quotes.py
    """
    if quotes is None:
        return []
    if not isinstance(quotes, list):
        return ["quotes must be a list"]
    flaws: list[str] = []
    if read_depth == "record" and quotes:
        flaws.append("an entry with readDepth record has no text to quote and must carry no quotes")
    seen: dict[str, int] = {}
    for index, quote in enumerate(quotes):
        where = f"quotes[{index}]"
        if not isinstance(quote, dict):
            flaws.append(f"{where}: must be an object with text, locator and claim")
            continue
        flaws += [f"{where}: unexpected key '{extra}'" for extra in sorted(set(quote) - {"text", "locator", "claim"})]

        text = quote.get("text")
        if not isinstance(text, str) or not text.strip():
            flaws.append(f"{where}: text must be a non-empty string")
        else:
            if "\n" in text or "\r" in text:
                flaws.append(f"{where}: text has a line break, join the lines with a space")
            if text != text.strip():
                flaws.append(f"{where}: text has whitespace at its start or end")
            if has_bare_ellipsis(quote=text):
                flaws.append(f"{where}: text has a bare ... or \u2026, write the gap as ' {ELISION} '")
            if any(not piece.strip() for piece in ELISION_SPLIT.split(text)):
                flaws.append(
                    f"{where}: {ELISION} has to sit between two segments of words, "
                    "not at either end and not twice in a row"
                )
            spaced = re.escape(ELISION)
            if re.search(rf"(?<=[^ ]){spaced}|{spaced}(?=[^ ])", text):
                flaws.append(f"{where}: write the elision as ' {ELISION} ' with a space either side")
            key = quote_key(text=text)
            if key in seen:
                flaws.append(f"{where}: repeats quotes[{seen[key]}]")
            seen.setdefault(key, index)

        locator = quote.get("locator")
        if not isinstance(locator, str) or not locator.strip():
            flaws.append(f"{where}: locator must be a non-empty string")
        elif len(locator) > MAX_LOCATOR:
            flaws.append(f"{where}: locator is {len(locator)} characters, the limit is {MAX_LOCATOR}")
        elif locator != locator.strip() or "\n" in locator:
            flaws.append(f"{where}: locator has a line break or whitespace at its start or end")

        claim = quote.get("claim")
        if "claim" not in quote:
            flaws.append(f"{where}: claim is missing, use null for a quote that backs the caveat")
        elif claim is not None and (isinstance(claim, bool) or not isinstance(claim, int)):
            flaws.append(f"{where}: claim must be an index into backsClaims or null")
        elif claim is not None and not 0 <= claim < claim_count:
            flaws.append(f"{where}: claim {claim} is out of range, the entry has {claim_count} claims")
    return flaws


def quote_texts(record: dict) -> dict[str, str]:
    """The text of each quote a ledger entry carries now, by quote key"""
    return {
        quote_key(text=quote["text"]): quote["text"]
        for quote in quotes_of(record=record)
        if isinstance(quote, dict) and isinstance(quote.get("text"), str)
    }


def ledger_quote_keys(records: list[dict]) -> dict[str, set[str]]:
    """The quote keys each ledger entry carries now, by citekey (an empty set for an entry with none)"""
    return {record["id"]: set(quote_texts(record=record)) for record in records}


def stale_receipts(records: list[dict], receipts: dict) -> list[tuple[str, str, str]]:
    """The receipts of the parsed receipts file that the ledger no longer backs, as (citekey, preview, reason).

    The reason is unknown-citekey, quote-not-in-ledger (the quote was edited or removed after it
    was checked) or duplicate (an earlier receipt on the entry has the same key). A quote with no
    receipt is no concern of this function
    """
    keys = ledger_quote_keys(records=records)
    stale = []
    for citekey, items in receipts["receipts"].items():
        seen: set[str] = set()
        for receipt in items:
            if citekey not in keys:
                reason = "unknown-citekey"
            elif receipt["quote"] not in keys[citekey]:
                reason = "quote-not-in-ledger"
            else:
                reason = "duplicate" if receipt["quote"] in seen else ""
            seen.add(receipt["quote"])
            if reason:
                stale.append((citekey, receipt.get("preview", ""), reason))
    return stale


def self_check() -> None:
    # typography folds, case stays
    assert normalize(text="it\u2019s \u201cslow\u201d \u2013 and 5\u22126\u2033") == 'it\'s "slow" - and 5-6"'
    assert normalize(text="Wait \u2026") == "Wait..."
    assert normalize(text="A\u00a0B\u2009C \ufb01x CO\u2082") == "A B C fix CO2"
    # a hyphen or dash between two letters is ignored, however the source wrapped or joined the word
    assert normalize(text="rest-and-\ndigest") == normalize(text="rest-anddigest") == "restanddigest"
    assert normalize(text="rest-and-digest") == "restanddigest"
    assert normalize(text="result\u2014\nwhich") == normalize(text="result\u2014which") == "resultwhich"
    assert normalize(text="inter-\n  national") == normalize(text="international") == "international"
    assert normalize(text="respiration-pacing") == normalize(text="respirationpacing")
    assert normalize(text="treat-\n  ment of a -\nb") == "treatment of a -b"
    # a dash with a space on either side keeps its spaces and glues nothing, an unspaced one folds
    assert normalize(text="alpha \u2013 beta") == normalize(text="alpha - beta") == "alpha - beta"
    assert normalize(text="alpha\u2013beta") == "alphabeta"
    assert normalize(text="breathing\n- Respiratory rate") == "breathing - Respiratory rate"
    # next to a digit a hyphen stays, wrapped or not
    assert normalize(text="26-\n30") == normalize(text="26\u201330") == "26-30"
    assert normalize(text="COVID-\n19") == "COVID-19"
    assert normalize(text="5-\nHT and 1.5-fold") == "5-HT and 1.5-fold"
    assert normalize(text="x \u2212 0.5 and -3") == "x - 0.5 and -3"
    # soft hyphens go, wrapped or not
    assert normalize(text="co\u00ad\noperate and co\u00adoperate") == "cooperate and cooperate"
    # spacing around punctuation and operators folds
    assert normalize(text="( p < 0.05 ) , 5 % of x = 3 \u00b1 1 ;") == "(p<0.05), 5% of x=3\u00b11;"
    assert normalize(text="  a\n\t b  ") == "a b"
    # a key is stable under typography and changes with a word
    key = quote_key(text="Slow \u201cbreathing\u201d  works")
    assert key == quote_key(text='Slow "breathing" works') and len(key) == 64
    assert key != quote_key(text='Slow "breathing" work')
    assert quote_preview(text="x" * 90) == "x" * PREVIEW_CHARS
    assert segments(quote="a b [...] c d") == ["a b", "c d"]
    assert segments(quote="a [...]  [...] b") == ["a", "b"]

    # finding quotes
    text = normalize(text="Slow breathing at six breaths per minute raised HRV.\nIt is not new.")
    sentence = "Slow breathing at six breaths per minute raised HRV."
    assert find_quote(quote=sentence, normalized_text=text) == (0, len(sentence))
    assert find_quote(quote="slow breathing", normalized_text=text) is None
    assert find_quote(quote="Slow breathing [...] raised HRV.", normalized_text=text) is not None
    assert find_quote(quote="Slow breathing at six breaths per minu", normalized_text=text) is None
    assert find_quote(quote="low breathing", normalized_text=text) is None
    assert find_quote(quote="breaths per minute raised HRV. I", normalized_text=text) is None
    assert find_quote(quote="breaths per minute raised HRV. It", normalized_text=text) is not None
    assert find_quote(quote="raised HRV. [...] Slow breathing", normalized_text=text) is None
    padded = "start " + "filler " * 400 + "end"
    assert find_quote(quote="start [...] end", normalized_text=padded) is None
    near = "start " + "filler " * 100 + "end"
    assert find_quote(quote="start [...] end", normalized_text=near) is not None
    retried = "foo " + "x " * 1500 + "foo bar qux"
    assert find_quote(quote="foo [...] qux", normalized_text=retried) == (3004, 3015)
    report = segment_report(quote="Slow breathing [...] per minuet", normalized_text=text)
    assert [r["found"] for r in report] == [True, False]
    assert report[1]["matched"] == len("per minu") and report[1]["window"]
    assert segment_report(quote="breathing at six breaths per minu", normalized_text=text)[0]["substring"]

    # markup
    assert xml_to_text(markup="<p>CO<sub>2</sub> rose</p>") == "CO2 rose"
    jats = "<jats:p>Slow <jats:italic>breathing</jats:italic></jats:p><jats:p>Next</jats:p>"
    assert xml_to_text(markup=jats) == "Slow breathing\n\nNext"
    assert xml_to_text(markup="<p>a<!-- hidden -->b &amp; c &lt; d</p>") == "ab & c < d"
    assert xml_to_text(markup='<p>rates<xref ref-type="bibr" rid="b1">12</xref>, then</p>') == "rates 12, then"
    assert xml_to_text(markup="<sec><title>Results</title><p>x</p><p>y</p></sec>") == "Results\n\nx\n\ny"
    assert xml_to_text(markup="<title-group>kept</title-group>") == "kept"

    # the right-document guard
    paper = "PERIPHERAL PACED RESPIRATION: influencing user physiology"
    assert title_in_text(title="Peripheral paced respiration: influencing user physiology", text=paper)
    assert title_in_text(title="Peripheral paced respiration: a different subtitle", text=paper)
    assert not title_in_text(title="Cyclic sighing: a different subtitle", text=paper)
    assert not title_in_text(title="Short: head", text="Short is the head of another title")
    assert not title_in_text(title="", text=paper)
    # a scan whose OCR misreads a few letters still passes, and a different title doesn't
    scan = (
        "Indian ] Physiol Phannacol 1992; 36(2) : 105-108\n\n"
        "EFFECf OF SHORT TERM 'PRANAYAM' PRACTICE ON BREATHING\n"
        "RATE AND VENTILATORY FUNCfIONS OF LUNG\n"
        "L. N. JOSHI\u00b7, V. D. JOSHI AND L. V. GOKHALE\nDepartment of Physiology\n"
    )
    joshi = "Effect of short term 'Pranayam' practice on breathing rate and ventilatory functions of lung"
    other = "Influence of short term 'Kapalbhati' drills on resting heart rhythm and cardiac output in men"
    assert len(squeeze(text=joshi)) == len(squeeze(text=other)) == 77
    assert title_in_text(title=joshi, text=scan)
    assert not title_in_text(title=other, text=scan)
    assert not title_in_text(title=joshi, text="filler " * 800 + scan)
    assert not title_in_text(title="Sighs", text="Sihs and more")

    # quoted spans and ellipses
    prose = "it's \"fine.\" and 'don't panic,' and \u201ccurly,\u201d and \u2018solo\u2019"
    assert quoted_spans(prose=prose) == ["fine", "don't panic", "curly", "solo"]
    assert quoted_spans(prose="the authors' view and the others' view") == []
    assert quoted_spans(prose="he said 'a' and \"b\"", single=False) == ["b"]
    assert has_bare_ellipsis(quote="a ... b") and has_bare_ellipsis(quote="a \u2026 b")
    assert has_bare_ellipsis(quote="a [...] b ...")
    assert not has_bare_ellipsis(quote="a [...] b")

    # the shape of quotes and where they sit
    good = {"text": "Slow breathing [...] raised HRV.", "locator": "Abstract", "claim": 0}
    assert quote_flaws(quotes=[good, {**good, "text": "Other words", "claim": None}], claim_count=1) == []
    assert quote_flaws(quotes=None, claim_count=1) == []
    assert len(quote_flaws(quotes=[good, good], claim_count=1)) == 1
    assert len(quote_flaws(quotes=[{**good, "text": "a\nb"}], claim_count=1)) == 1
    assert len(quote_flaws(quotes=[{**good, "text": "a... b"}], claim_count=1)) == 1
    assert len(quote_flaws(quotes=[{**good, "text": "[...] b"}], claim_count=1)) >= 1
    assert len(quote_flaws(quotes=[{**good, "text": "a[...]b"}], claim_count=1)) == 1
    assert len(quote_flaws(quotes=[{**good, "locator": "x" * 81}], claim_count=1)) == 1
    assert len(quote_flaws(quotes=[{**good, "claim": 3}], claim_count=2)) == 1
    assert len(quote_flaws(quotes=[{**good, "claim": True}], claim_count=2)) == 1
    assert len(quote_flaws(quotes=[good], claim_count=1, read_depth="record")) == 1
    # the locator decides where a quote has to be found: Abstract and its extensions point at the abstract
    places = ("Abstract", "Abstract, Significance Statement", "Abstract: Key points", "Abstracts", "abstract", "Results", "p. 175")
    assert [locator_scope(locator=place) for place in places] == ["abstract"] * 3 + ["body"] * 4
    for place, scope in (
        ("Abstract", "abstract"), ("Abstract, Significance Statement", "abstract"), ("Results", "body"),
        ("Significance Statement", "body"), ("p. 175", "body"), ("Results", "elsewhere"),
    ):
        assert locator_flaw(locator=place, scope=scope, source="s") is None, (place, scope)
    for place, scope, points in (
        ("Abstract", "body", "abstract"), ("Abstract, Key points", "body", "abstract"),
        ("Summary", "abstract", "body"), ("Results, p. 175", "abstract", "body"),
    ):
        message = locator_flaw(locator=place, scope=scope, source="the text") or ""
        assert f'"{place}", which points at the {points}, and was found in the {scope} (the text)' in message, message

    # receipts the ledger no longer backs
    ledger = [{"id": "a2020-x", "custom": {"quotes": [good]}}, {"id": "b2021-y", "custom": {}}]
    assert ledger_quote_keys(records=ledger) == {"a2020-x": {quote_key(text=good["text"])}, "b2021-y": set()}
    kept = {"quote": quote_key(text=good["text"]), "preview": "kept"}
    file = {"format": RECEIPTS_FORMAT, "receipts": {
        "a2020-x": [kept, {"quote": "0" * 64, "preview": "edited"}, kept],
        "b2021-y": [{"quote": "1" * 64, "preview": "orphan"}],
        "c2022-z": [{"quote": "2" * 64, "preview": "gone"}],
    }}
    assert stale_receipts(records=ledger, receipts=file) == [
        ("a2020-x", "edited", "quote-not-in-ledger"), ("a2020-x", "kept", "duplicate"),
        ("b2021-y", "orphan", "quote-not-in-ledger"), ("c2022-z", "gone", "unknown-citekey"),
    ]
    assert stale_receipts(records=ledger, receipts={"format": 1, "receipts": {"a2020-x": [kept]}}) == []

    print("verbatim self-check passed")


if __name__ == "__main__":
    self_check()
