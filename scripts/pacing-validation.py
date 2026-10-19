#!/usr/bin/env python3
"""Validate exhale's per-phase timing against the configured pace.

Reads a session log exported from a study build (JSON Lines, one event per
line), regroups its phase_end events into cycles using the cycle field each
line carries, and prints a Markdown table comparing measured timing against
the durations in the log's own header.

Only lines after the last app_start or log_cleared line are scored, so a
log spanning more than one launch is judged on its final session. A phase
configured as 0 is passed over by the app and logs nothing, so a cycle
counts as complete once every phase with a non-zero configured duration
has been logged for it: two lines per cycle at 5/0/5/0, four at 4/4/4/4

Example:
    uv run --no-project scripts/pacing-validation.py exhale-log-P07-1758400420.jsonl
"""

from __future__ import annotations

import argparse
import json
import statistics
import sys
from pathlib import Path

PHASES = ("inhale", "hold_after_inhale", "exhale", "hold_after_exhale")
HEADER_EVENTS = ("app_start", "log_cleared")
DURATION_FIELDS = (
    "inhale_duration", "post_inhale_hold_duration", "exhale_duration", "post_exhale_hold_duration",
)


def phases_that_run(header):
    """The phase names whose configured duration is above 0, in cycle order"""
    return tuple(
        name for name, field in zip(PHASES, DURATION_FIELDS) if header[field] > 0
    )


def parse_args(argv):
    parser = argparse.ArgumentParser(
        description="Print a Markdown timing-accuracy table from an exhale session log",
    )
    parser.add_argument("log_path", type=Path, help="exported exhale-log-*.jsonl file")
    return parser.parse_args(args=argv)


def read_events(log_path):
    try:
        text = log_path.read_text(encoding="utf-8")
    except OSError as error:
        sys.exit(f"pacing-validation: can't read {log_path}: {error}")
    events = []
    for line_number, line in enumerate(text.splitlines(), start=1):
        if not line.strip():
            continue
        try:
            events.append(json.loads(s=line))
        except json.JSONDecodeError as error:
            sys.exit(
                f"pacing-validation: {log_path}:{line_number} isn't valid JSON ({error}), "
                "so the file isn't a JSON Lines log"
            )
    return events


def find_header(events):
    header, header_index = None, -1
    for index, event in enumerate(events):
        if event.get("event") in HEADER_EVENTS:
            header, header_index = event, index
    if header is None:
        sys.exit("pacing-validation: no app_start or log_cleared line in this log")
    return header, header_index


def complete_cycles(events, header_index, expected):
    """Phase_end lines after the header, grouped by cycle, kept once every expected phase is in"""
    cycles = {}
    for event in events[header_index + 1 :]:
        if event.get("event") == "phase_end":
            cycles.setdefault(event["cycle"], {})[event["phase"]] = event
    return {n: p for n, p in cycles.items() if all(name in p for name in expected)}


def format_pct(actual, configured):
    if configured <= 0:
        return "n/a"
    return f"{(actual - configured) / configured * 100.0:+.1f}%"


def overshoot_by_phase(events, header_index):
    totals = {name: [] for name in PHASES}
    for event in events[header_index + 1 :]:
        if event.get("event") == "phase_end":
            totals[event["phase"]].append(event["actual_ms"] - event["scheduled_ms"])
    return {
        name: (statistics.mean(data=values) if values else None) for name, values in totals.items()
    }


def print_table(rows):
    print("| Field | Value |")
    print("|---|---|")
    for field, value in rows:
        print(f"| {field} | {value} |")


def main(argv=None):
    args = parse_args(argv=argv)
    events = read_events(log_path=args.log_path)
    header, header_index = find_header(events=events)
    configured = sum(header[field] for field in DURATION_FIELDS)
    platform_row = ("Platform, version", f"{header.get('os', '?')}, {header.get('version', '?')}")

    expected = phases_that_run(header=header)
    complete = complete_cycles(events=events, header_index=header_index, expected=expected)
    if not complete:
        print_table(rows=[platform_row, ("Configured cycle", f"{configured:.2f} s")])
        print()
        print("No complete cycles in this log")
        return 0

    numbers = sorted(complete)
    # Every phase the cycle logged, so a log from a build that still ran
    # 0.1 s holds is measured as it ran
    durations = {
        n: sum(line["actual_ms"] for line in complete[n].values()) / 1000.0 for n in numbers
    }
    mean_cycle = statistics.mean(data=durations.values())
    worst_n = max(numbers, key=lambda n: abs(durations[n] - configured))

    first = min(complete[numbers[0]].values(), key=lambda line: line["mono_ms"])
    last = max(complete[numbers[-1]].values(), key=lambda line: line["mono_ms"])
    span_secs = (last["mono_ms"] - (first["mono_ms"] - first["actual_ms"])) / 1000.0
    cumulative_drift = span_secs - len(numbers) * configured

    overshoot = overshoot_by_phase(events=events, header_index=header_index)
    overshoot_text = ", ".join(
        f"{name} {overshoot[name]:+.1f} ms" if overshoot[name] is not None else f"{name} n/a"
        for name in PHASES
    )

    print_table(rows=[
        platform_row,
        ("Configured cycle", f"{configured:.2f} s"),
        ("Cycles measured", f"{len(numbers)} (complete cycles only)"),
        ("Mean cycle", f"{mean_cycle:.2f} s ({format_pct(actual=mean_cycle, configured=configured)})"),
        (
            "Worst single cycle",
            f"{durations[worst_n]:.2f} s ({format_pct(actual=durations[worst_n], configured=configured)})",
        ),
        ("Cumulative drift", f"{cumulative_drift:+.2f} s"),
        ("Per-phase mean overshoot", overshoot_text),
    ])
    return 0


if __name__ == "__main__":
    sys.exit(main())
