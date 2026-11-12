#!/usr/bin/env python3
"""Draw the README charts as static SVGs, each in a light and a dark theme.

Single source of truth:
  - rust/crates/exhale-core/src/presets.rs  the timings of the patterns the app offers
  - SOURCED below                           every figure a paper prints, as the paper prints it
  - docs/CITATIONS.csl.json                 the quotes that have to print each of those figures

Both modes fail when a sourced figure stands in no quote of the entry it cites, so a chart
can't carry a number the ledger doesn't back. The app's own timings are not paper figures and
aren't checked. Each chart is one layout function that takes a theme, so the light and dark files
share their data and their layout. The SVGs hold no timestamps, so a rerun writes the same bytes

Usage:
    uv run --no-project scripts/generate-charts.py            # write docs/charts/*.svg
    uv run --no-project scripts/generate-charts.py --check    # fail when an SVG on disk is stale

Stdlib only, so it runs anywhere the repo is checked out
"""

from __future__ import annotations

import argparse
import json
import math
import re
import sys
from collections.abc import Iterator
from pathlib import Path

import verbatim

ROOT = Path(__file__).resolve().parent.parent
CORPUS = ROOT / "docs" / "CITATIONS.csl.json"
PRESETS = ROOT / "rust/crates/exhale-core/src/presets.rs"
OUT = ROOT / "docs" / "charts"
# presets.rs lists at least this many labels, so finding fewer means the regex in preset_codes went blind
MIN_PRESETS = 5

MARCHANT = "marchant2025-square-478-six"
YOU = "you2023-respiratory-frequency"
TABOR = "tabor2022-guided-breathing-design"
# The 4-7-8 protocol in seconds (inhale, hold, exhale) and the pattern code it makes
PROTOCOL_478 = ("4", "7", "8")
CODE_478 = "/".join((*PROTOCOL_478, "0"))
# Figures as the papers print them, by citekey, keyed by pattern code where a pattern is the subject
# Each string has to stand in a quote of its entry, see unprinted
SOURCED = {
    MARCHANT: {
        "protocol": PROTOCOL_478,
        # Table 4: RMSSD rise over baseline in standard deviations, for the paper's 4:6, 5:5, 4-7-8 and square
        "rise": {"4/0/6/0": "0.65", "5/0/5/0": "0.45", CODE_478: "0.27", "4/4/4/4": "0.25"},
        # Table 3: target breaths a minute
        "rate": {"4/4/4/4": "3.75", CODE_478: "3.16"},
        "students": "84",
    },
    # The tested frequencies in breaths a minute, lowest and highest
    YOU: {"tested": ("5", "7")},
    # Group means as (before, during) in breaths a minute and in HRV amplitude
    TABOR: {
        "rate": {"Plain pacer": ("12.5", "6.1"), "Biofeedback": ("11.0", "5.2")},
        "amplitude": {"Plain pacer": ("4.2", "8.0"), "Biofeedback": ("5.0", "7.0")},
        "per_group": "14",
        "minutes": "10",
    },
}
NAMES = {"5/0/5/0": "default", "5/0/10/0": "earlier default", "4/4/4/4": "box", CODE_478: "4-7-8"}

WIDTH = 720
FONT = "-apple-system, BlinkMacSystemFont, 'Segoe UI', 'Noto Sans', Helvetica, Arial, sans-serif"
THEMES = {
    "light": {
        "ink": "#1f2328", "ink2": "#59636e", "grid": "#e6eaef", "axis": "#c8d1da", "inhale": "#e34948",
        "exhale": "#2a78d6", "accent": "#2a78d6", "neutral": "#8c959f", "band": "#1baf7a", "band_opacity": "0.12",
    },
    "dark": {
        "ink": "#e6edf3", "ink2": "#9198a1", "grid": "#21262d", "axis": "#3d444d", "inhale": "#e66767",
        "exhale": "#3987e5", "accent": "#3987e5", "neutral": "#6e7681", "band": "#199e70", "band_opacity": "0.22",
    },
}


def leaves(tree: object) -> Iterator[str]:
    """Every string in a nest of dicts, tuples and strings, in order"""
    if isinstance(tree, str):
        yield tree
    else:
        for child in (tree.values() if isinstance(tree, dict) else tree):
            yield from leaves(tree=child)


def unprinted(records: list[dict]) -> list[str]:
    """The sourced figures that no quote of their entry prints, one message each"""
    by_id = {record["id"]: record for record in records}
    missing = []
    for citekey, figures in SOURCED.items():
        if citekey not in by_id:
            missing.append(f"{citekey}: no such entry in {CORPUS.name}")
            continue
        quotes = [verbatim.normalize(text=quote["text"]) for quote in verbatim.quotes_of(record=by_id[citekey])]
        for figure in dict.fromkeys(leaves(tree=figures)):
            standalone = r"(?<![\d.])" + re.escape(figure) + r"(?!\d|\.\d)"
            if not any(re.search(standalone, quote) for quote in quotes):
                missing.append(f"{citekey}: {figure} is printed in none of its quotes")
    return missing


def preset_codes() -> list[str]:
    """The a/b/c/d labels of the presets in presets.rs, in the order it lists them"""
    codes = re.findall(r'label:\s*"([\d.]+(?:/[\d.]+){3})"', PRESETS.read_text(encoding="utf-8"))
    if len(codes) < MIN_PRESETS:
        raise SystemExit(f"{PRESETS.name}: found {len(codes)} preset labels, expected at least {MIN_PRESETS}")
    return codes


def total(code: str) -> float:
    """Seconds in one cycle of a pattern code"""
    return sum(float(part) for part in code.split("/"))


def fmt(value: float) -> str:
    """At most two decimals with trailing zeros stripped: 10, 3.75, 3.16"""
    return f"{value:.2f}".rstrip("0").rstrip(".")


def el(tag: str, body: str | list[str] = "", **attrs: float | str | None) -> str:
    """One element, attributes in the order given with a hyphen for each underscore and None left out.

    A float attribute is written with one decimal. A string body is text and gets escaped, and a list
    body holds elements, one on each line
    """
    attrs = {k: f"{v:.1f}" if isinstance(v, float) else v for k, v in attrs.items() if v is not None}
    shown = "".join(f' {k.replace("_", "-")}="{v}"' for k, v in attrs.items())
    if isinstance(body, list):
        body = "\n" + "\n".join(body) + "\n"
    else:
        body = body.replace("&", "&amp;").replace("<", "&lt;").replace(">", "&gt;")
    return f"<{tag}{shown}>{body}</{tag}>" if body else f"<{tag}{shown}/>"


def text(x: float, y: float, body: str, fill: str, size: int = 13, anchor: str | None = None, weight: int | None = None) -> str:
    """A line of text with its baseline at y"""
    return el(tag="text", body=body, x=x, y=y, fill=fill, font_size=size, text_anchor=anchor, font_weight=weight)


def bar(x0: float, x1: float, y: float, height: float, fill: str, rounded: bool) -> str:
    """A horizontal bar, square at x0 and, when rounded, with a 4px radius on both corners at x1"""
    end = (
        f"H{x1 - 4:.1f}A4 4 0 0 1 {x1:.1f} {y + 4:.1f}V{y + height - 4:.1f}A4 4 0 0 1 {x1 - 4:.1f} {y + height:.1f}"
        if rounded else f"H{x1:.1f}V{y + height:.1f}"
    )
    return el(tag="path", d=f"M{x0:.1f} {y:.1f}{end}H{x0:.1f}Z", fill=fill)


def axis(marks: list[tuple[float, str]], top: float, bottom: float, theme: dict) -> list[str]:
    """A solid 1px hairline at each (x, label) mark, the first in the axis color, and the labels under the plot"""
    lines = []
    for index, (x, _) in enumerate(marks):
        edge = math.floor(x) + 0.5
        lines.append(el(tag="line", x1=edge, y1=top, x2=edge, y2=bottom, stroke=theme["grid" if index else "axis"]))
    labels = el(
        tag="g", fill=theme["ink2"], font_size=12, text_anchor="middle", style="font-variant-numeric: tabular-nums",
        body=[el(tag="text", body=label, x=x, y=bottom + 17) for x, label in marks],
    )
    return [*lines, labels]


def frame(
    name: str, theme: dict, title: str, subtitle: str, table: list[str], height: int, legend: list, parts: list[str]
) -> str:
    """The whole file: root, accessible title and desc, heading, legend row, parts.

    The desc is the subtitle and then the table, a sentence for each plotted row. A legend item is
    (shape, attributes, label), the shape rect or circle, drawn as a 12px swatch
    """
    key, x = [], 0.0
    for shape, attrs, label in legend:
        if shape == "rect":
            key.append(el(tag="rect", x=x, y=56, width=12, height=12, rx=2, **attrs))
        else:
            key.append(el(tag="circle", cx=x + 6, cy=62, r=5, **attrs))
        key.append(text(x=x + 18, y=66, body=label, fill=theme["ink2"], size=12))
        x += 18 + 6 * len(label) + 22
    return el(
        tag="svg",
        body=[
            el(tag="title", body=title, id=f"{name}-title"),
            el(tag="desc", body=" ".join([subtitle, *table]), id=f"{name}-desc"),
            text(x=0, y=20, body=title, fill=theme["ink"], size=15, weight=600),
            text(x=0, y=40, body=subtitle, fill=theme["ink2"]),
            *key,
            *parts,
        ],
        xmlns="http://www.w3.org/2000/svg", width=WIDTH, height=height, viewBox=f"0 0 {WIDTH} {height}",
        role="img", aria_labelledby=f"{name}-title {name}-desc", font_family=FONT,
    )


def cycles(theme: dict) -> dict:
    """One bar per pattern split into its four phases, over the band of rates the papers tested"""
    tested = SOURCED[YOU]["tested"]
    low, high = (float(figure) for figure in tested)
    rows = sorted(dict.fromkeys([*preset_codes(), CODE_478]), key=lambda code: total(code=code))
    span = math.ceil(max(total(code=code) for code in rows) / 5) * 5
    left, right, thick, pitch, top, pad = 190.0, 590.0, 18.0, 30.0, 106.0, 16.0
    bottom = top + 2 * pad + pitch * (len(rows) - 1)
    phase_colors = [theme["inhale"], theme["neutral"], theme["exhale"], theme["neutral"]]

    def at(seconds: float) -> float:
        return left + seconds * (right - left) / span

    band_left, band_right = at(seconds=60 / high), at(seconds=60 / low)
    parts = [
        el(tag="rect", x=band_left, y=top, width=band_right - band_left, height=bottom - top,
           fill=theme["band"], fill_opacity=theme["band_opacity"]),
        text(x=(band_left + band_right) / 2, y=top - 6, body="tested range", fill=theme["ink2"], size=12, anchor="middle"),
        *axis(marks=[(at(seconds=t), f"{t} s" if t == span else str(t)) for t in range(0, span + 1, 5)],
              top=top, bottom=bottom, theme=theme),
    ]
    table = []
    for index, code in enumerate(rows):
        cy, cycle = top + pad + pitch * index, total(code=code)
        cycle_s, rate_s = fmt(value=cycle), fmt(value=60 / cycle)
        parts.append(text(x=0, y=cy + 4.5, body=code, fill=theme["ink"]))
        if code in NAMES:
            parts.append(text(x=70, y=cy + 4.5, body=NAMES[code], fill=theme["ink2"]))
        phases = [(float(part), color) for part, color in zip(code.split("/"), phase_colors) if float(part) > 0]
        start = 0.0
        for number, (length, color) in enumerate(phases):
            last = number == len(phases) - 1
            end = at(seconds=start + length) - (0 if last else 2)
            parts.append(bar(x0=at(seconds=start), x1=end, y=cy - thick / 2, height=thick, fill=color, rounded=last))
            start += length
        note, note_x = f"{cycle_s} s, {rate_s} a minute", at(seconds=cycle) + 8
        # An annotation that would sit on the band starts past the band's right edge instead
        if note_x < band_right and note_x + 6 * len(note) > band_left:
            note_x = band_right + 8
        parts.append(text(x=note_x, y=cy + 4, body=note, fill=theme["ink2"], size=12))
        one, two, three, four = code.split("/")
        name = f" ({NAMES[code]})" if code in NAMES else ""
        table.append(f"{code}{name}: inhale {one}, hold {two}, exhale {three}, hold {four}, "
                     f"{cycle_s} seconds a cycle, {rate_s} breaths a minute.")
    return {
        "title": "One breath cycle, pattern by pattern",
        "subtitle": f"Seconds per cycle. The shaded band covers {tested[0]} to {tested[1]} breaths a minute, "
                    "the range tested directly.",
        "table": table,
        "height": round(bottom + 26),
        "legend": [("rect", {"fill": theme["inhale"]}, "Inhale"), ("rect", {"fill": theme["neutral"]}, "Hold"),
                   ("rect", {"fill": theme["exhale"]}, "Exhale")],
        "parts": parts,
    }


def patterns(theme: dict) -> dict:
    """The RMSSD rise by pattern, accent for the two with no holds and neutral for the two with holds"""
    paper = SOURCED[MARCHANT]
    rows = sorted(paper["rise"].items(), key=lambda item: -float(item[1]))
    left, right, span, thick, pitch, top, pad = 220.0, 690.0, 0.7, 20.0, 36.0, 98.0, 14.0
    bottom = top + 2 * pad + pitch * (len(rows) - 1)

    def at(rise: float) -> float:
        return left + rise / span * (right - left)

    parts = axis(marks=[(at(rise=t), fmt(value=t)) for t in (0, 0.2, 0.4, 0.6)], top=top, bottom=bottom, theme=theme)
    table = []
    for index, (code, rise) in enumerate(rows):
        cy, end = top + pad + pitch * index, at(rise=float(rise))
        rate = paper["rate"].get(code) or fmt(value=60 / total(code=code))
        name = f"{rate} a minute" + (f", {NAMES[code]}" if code in NAMES else "")
        held = any(float(part) for part in code.split("/")[1::2])
        parts += [
            text(x=0, y=cy + 4.5, body=code, fill=theme["ink"]),
            text(x=70, y=cy + 4.5, body=name, fill=theme["ink2"]),
            bar(x0=left, x1=end, y=cy - thick / 2, height=thick, fill=theme["neutral" if held else "accent"], rounded=True),
            text(x=end + 6, y=cy + 4.5, body=rise, fill=theme["ink"]),
        ]
        table.append(f"{code} ({name}): {rise}.")
    return {
        "title": "Heart rate variability rise by breathing pattern",
        "subtitle": f"RMSSD above each person's baseline, in standard deviations. {paper['students']} college students, "
                    "one session each.",
        "table": table,
        "height": round(bottom + 26),
        "legend": [("rect", {"fill": theme["accent"]}, "6 a minute, no holds"),
                   ("rect", {"fill": theme["neutral"]}, "With holds, under 4 a minute")],
        "parts": parts,
    }


def pacer(theme: dict) -> dict:
    """Before and during values for two protocols, a dumbbell per protocol in each of two panels"""
    paper = SOURCED[TABOR]
    colors = {"Plain pacer": theme["accent"], "Biofeedback": theme["neutral"]}
    panels = [
        ("Breathing rate, breaths a minute", "rate", 130.0, 14, (0, 4, 8, 12)),
        ("HRV amplitude", "amplitude", 440.0, 9, (0, 3, 6, 9)),
    ]
    width, top, pitch, pad, r = 270.0, 110.0, 40.0, 22.0, 5.0
    centers = [top + pad + pitch * index for index in range(len(colors))]
    bottom = top + 2 * pad + pitch * (len(colors) - 1)
    parts, table = [], []
    for (name, color), cy in zip(colors.items(), centers):
        parts += [el(tag="circle", cx=5.0, cy=cy, r=4, fill=color), text(x=16, y=cy + 4.5, body=name, fill=theme["ink"])]
    for title, key, left, ceiling, marks in panels:
        parts += [
            text(x=left, y=100, body=title, fill=theme["ink"], weight=600),
            *axis(marks=[(left + t / ceiling * width, str(t)) for t in marks], top=top, bottom=bottom, theme=theme),
        ]
        table.append(f"{title}:")
        for (name, color), cy in zip(colors.items(), centers):
            before, during = paper[key][name]
            x_before, x_during = (left + float(value) / ceiling * width for value in (before, during))
            side = 1 if x_during > x_before else -1
            parts += [
                el(tag="line", x1=x_before + side * r, y1=cy, x2=x_during - side * r, y2=cy, stroke=color, stroke_width=2),
                el(tag="circle", cx=x_before, cy=cy, r=r, fill="none", stroke=color, stroke_width=2),
                el(tag="circle", cx=x_during, cy=cy, r=r, fill=color),
                text(x=x_before - side * 10, y=cy + 4, body=before, fill=theme["ink2"], size=12,
                     anchor="end" if side > 0 else "start"),
                text(x=x_during + side * 10, y=cy + 4.5, body=during, fill=theme["ink"], anchor="start" if side > 0 else "end"),
            ]
            table.append(f"{name} {before} before, {during} during.")
    return {
        "title": "Plain pacer vs sensor biofeedback",
        "subtitle": f"Group means before and during a {paper['minutes']}-minute session, {paper['per_group']} people per group.",
        "table": table,
        "height": round(bottom + 26),
        "legend": [("circle", {"fill": "none", "stroke": theme["ink2"], "stroke_width": 2}, "Before"),
                   ("circle", {"fill": theme["ink2"]}, "During the session")],
        "parts": parts,
    }


def render() -> dict[str, str]:
    """Every SVG the charts make, by file name"""
    return {
        f"{layout.__name__}-{mode}.svg": frame(name=layout.__name__, theme=theme, **layout(theme=theme)) + "\n"
        for layout in (cycles, patterns, pacer)
        for mode, theme in THEMES.items()
    }


def parse_args(argv: list[str]) -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        prog="generate-charts.py",
        description="Draw the README charts into docs/charts from the presets and the cited papers' figures",
    )
    parser.add_argument("--check", action="store_true", help="fail when an SVG on disk is stale, write nothing")
    return parser.parse_args(argv)


def main() -> int:
    args = parse_args(argv=sys.argv[1:])
    missing = unprinted(records=json.loads(CORPUS.read_text(encoding="utf-8")))
    if missing:
        raise SystemExit("\n  ".join([f"A sourced figure is in no quote of its entry in {CORPUS.name}:", *missing]))

    files = render()
    stale = [
        name for name, svg in files.items()
        if not (OUT / name).is_file() or (OUT / name).read_bytes() != svg.encode("utf-8")
    ]
    if args.check:
        for name in stale:
            print(f"{(OUT / name).relative_to(ROOT)} is stale", file=sys.stderr)
        if stale:
            print("Run: uv run --no-project scripts/generate-charts.py", file=sys.stderr)
            return 1
        print(f"{OUT.relative_to(ROOT)} is up to date ({len(files)} charts).")
        return 0

    OUT.mkdir(exist_ok=True)
    for name, svg in files.items():
        (OUT / name).write_bytes(svg.encode("utf-8"))
    print(f"Wrote {len(files)} charts to {OUT.relative_to(ROOT)}.")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
