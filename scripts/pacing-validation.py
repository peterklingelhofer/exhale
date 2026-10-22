#!/usr/bin/env python3
"""Validate exhale's per-phase timing against the configured pace.

Reads a session log exported from a study build (JSON Lines, one event per
line), regroups its phase_end events into cycles using the cycle field each
line carries, and prints a Markdown table comparing measured timing against
the schedule the app set for each cycle

Only lines after the last app_start or log_cleared line are scored, so a
log spanning more than one launch is judged on its final session. That
session is split into runs, at each animation_start line and wherever
(cycle, phase order) stops increasing. The second case is a restart that
leaves no line: changing the color or the shape while it runs sends the
cycle count back to 0. Each run gets a table of its own, headed "Run 1 of
3" and so on when there's more than one. A log with a single run prints
one table with no heading

A Start while the animation runs restarts it, and the phase_end line of the
run it cuts short can land just after the new animation_start line. Between
an animation_start and the first line with cycle 0, a line with a cycle
above 0 belongs to the run before, so it doesn't make a run of its own

A phase configured as 0 is passed over by the app and logs nothing, so a
cycle counts as complete once every phase with a non-zero configured
duration has been logged for it: two lines per cycle at 5/0/5/0, four at
4/4/4/4

Each cycle is compared with the sum of the scheduled_ms its own lines
carry. Drift and randomized timing change what the app schedules for a
cycle, and the compared lengths follow it, so neither counts as timing
error. The configured-cycle row says when the header sets either

A line that isn't valid JSON is skipped with a warning on stderr and a
count. A write cut short (an app killed mid-line, a full disk) leaves one,
and it shouldn't cost the rest of the log. A phase_end line that is valid
JSON but lacks a whole-number cycle, a known phase, or a number in mono_ms,
scheduled_ms or actual_ms is skipped and counted the same way. A cycle that
lost a line to either counts as incomplete

Example:
    uv run --no-project scripts/pacing-validation.py exhale-log-P07-1758400420.jsonl
    uv run --no-project scripts/pacing-validation.py --self-check
"""

from __future__ import annotations

import argparse
import contextlib
import io
import json
import statistics
import sys
import tempfile
from pathlib import Path

PHASES = ("inhale", "hold_after_inhale", "exhale", "hold_after_exhale")
PHASE_ORDER = {name: index for index, name in enumerate(PHASES)}
HEADER_EVENTS = ("app_start", "log_cleared")
DURATION_FIELDS = (
    "inhale_duration", "post_inhale_hold_duration", "exhale_duration", "post_exhale_hold_duration",
)
JITTER_FIELDS = (
    "randomized_timing_inhale", "randomized_timing_post_inhale_hold",
    "randomized_timing_exhale", "randomized_timing_post_exhale_hold",
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
    parser.add_argument("log_path", type=Path, nargs="?", help="exported exhale-log-*.jsonl file")
    parser.add_argument("--self-check", action="store_true", help="check the script against built-in logs and exit")
    args = parser.parse_args(args=argv)
    if args.log_path is None and not args.self_check:
        parser.error(message="the following arguments are required: log_path")
    return args


def warn(message):
    print(f"pacing-validation: {message}", file=sys.stderr)


def phase_end_problem(event):
    """What keeps a phase_end line from being scored, as a phrase, or None when nothing does

    The scoring reads these five fields of every phase_end line, so a line that is valid JSON
    and lacks one of them would stop the script with a KeyError or a TypeError
    """
    cycle = event.get("cycle")
    if not isinstance(cycle, int) or isinstance(cycle, bool):
        return "with no whole-number cycle"
    phase = event.get("phase")
    if not isinstance(phase, str) or phase not in PHASE_ORDER:
        return "with no known phase"
    for field in ("mono_ms", "scheduled_ms", "actual_ms"):
        value = event.get(field)
        if not isinstance(value, (int, float)) or isinstance(value, bool):
            return f"with no number in {field}"
    return None


def read_events(log_path):
    """The log's lines as dicts, in order, without the lines that can't be read as log events"""
    try:
        # Bytes that aren't UTF-8 (a write torn inside a character) become U+FFFD, so
        # that line fails as JSON like any other cut line, and the rest still reads
        text = log_path.read_text(encoding="utf-8", errors="replace")
    except OSError as error:
        sys.exit(f"pacing-validation: can't read {log_path}: {error}")
    events, skipped = [], 0
    # Split on "\n" alone. str.splitlines() also breaks at U+0085, U+2028 and U+2029, and
    # the app writes those raw inside a JSON string, so it would cut a valid line in two
    for line_number, line in enumerate(text.split(sep="\n"), start=1):
        if not line.strip():
            continue
        try:
            event = json.loads(s=line)
        except json.JSONDecodeError as error:
            warn(message=f"line {line_number} isn't valid JSON ({error}), skipped")
            skipped += 1
            continue
        if not isinstance(event, dict):
            warn(message=f"line {line_number} isn't a JSON object, skipped")
            skipped += 1
            continue
        problem = phase_end_problem(event=event) if event.get("event") == "phase_end" else None
        if problem:
            warn(message=f"line {line_number} is a phase_end {problem}, skipped")
            skipped += 1
            continue
        events.append(event)
    if skipped:
        plural = "line" if skipped == 1 else "lines"
        warn(message=f"skipped {skipped} {plural} that couldn't be read as log events")
    return events


def find_header(events):
    header, header_index = None, -1
    for index, event in enumerate(events):
        if event.get("event") in HEADER_EVENTS:
            header, header_index = event, index
    if header is None:
        sys.exit("pacing-validation: no app_start or log_cleared line in this log")
    return header, header_index


def split_runs(events, header_index):
    """The phase_end lines after the header, one list per run, runs with no phase_end left out

    A run ends at the next animation_start line, and wherever (cycle, phase order) fails to
    increase from one phase_end to the next. Inside a run each phase_end of a cycle comes
    once, later than the one before it, so a line that doesn't follow on is the first of a
    restart that logged nothing

    A Start while the animation runs restarts it. The controller thread can log the cut-short
    run's last phase_end after the new animation_start line, because the two threads don't
    wait for each other. Every run starts at cycle 0, so from an animation_start to the first
    line with cycle 0, a line with a cycle above 0 is that straggler and joins the run the
    animation_start ended. A straggler with cycle 0, from a restart in a run's first cycle,
    looks like the new run's first line and starts the new run
    """
    runs, current, previous = [], [], None
    # The run an animation_start ended, and whether no line of the run it began has come yet.
    # A straggler can only join a run that ended with a line of its own
    ended, starting = None, False
    for event in events[header_index + 1 :]:
        kind = event.get("event")
        if kind == "animation_start":
            if current:
                runs.append(current)
                ended = current
            current, previous, starting = [], None, True
        elif kind == "phase_end":
            if starting and event["cycle"] > 0 and ended is not None:
                ended.append(event)
                continue
            starting = False
            key = (event["cycle"], PHASE_ORDER[event["phase"]])
            if previous is not None and key <= previous:
                runs.append(current)
                current = []
            current.append(event)
            previous = key
    if current:
        runs.append(current)
    return runs


def complete_cycles(run, expected):
    """A run's phase_end lines grouped by cycle, kept once every expected phase is in"""
    cycles = {}
    for event in run:
        cycles.setdefault(event["cycle"], {})[event["phase"]] = event
    return {n: p for n, p in cycles.items() if all(name in p for name in expected)}


def format_pct(actual, configured):
    if configured <= 0:
        return "n/a"
    return f"{(actual - configured) / configured * 100.0:+.1f}%"


def overshoot_by_phase(run):
    totals = {name: [] for name in PHASES}
    for event in run:
        totals[event["phase"]].append(event["actual_ms"] - event["scheduled_ms"])
    return {
        name: (statistics.mean(data=values) if values else None) for name, values in totals.items()
    }


def contiguous_blocks(numbers):
    """Sorted cycle numbers split into groups of consecutive numbers"""
    blocks = []
    for number in numbers:
        if blocks and number == blocks[-1][-1] + 1:
            blocks[-1].append(number)
        else:
            blocks.append([number])
    return blocks


def cumulative_drift(complete, scheduled, numbers):
    """Seconds the complete cycles took beyond what was scheduled for them

    Each stretch of consecutive cycles is measured from the start of its first phase to the
    end of its last, so a cycle missing from the middle (a skipped line) doesn't put its own
    length into the total
    """
    total = 0.0
    for block in contiguous_blocks(numbers=numbers):
        first = min(complete[block[0]].values(), key=lambda line: line["mono_ms"])
        last = max(complete[block[-1]].values(), key=lambda line: line["mono_ms"])
        span_secs = (last["mono_ms"] - (first["mono_ms"] - first["actual_ms"])) / 1000.0
        total += span_secs - sum(scheduled[n] for n in block)
    return total


def configured_text(header, configured):
    """The configured-cycle cell, with a note when drift or randomized timing is set"""
    causes = []
    if header.get("drift", 1.0) != 1.0:
        causes.append(f"drift {header['drift']}")
    if any(header.get(field, 0.0) > 0 for field in JITTER_FIELDS):
        causes.append("randomized timing")
    text = f"{configured:.2f} s"
    if causes:
        verb = "applies" if len(causes) == 1 else "apply"
        text += f" ({' and '.join(causes)} {verb}, so each cycle is compared with its scheduled length)"
    return text


def score_run(header, run):
    """The table rows below the configured cycle for one run, or None when it has no complete cycle"""
    configured = sum(header[field] for field in DURATION_FIELDS)
    expected = phases_that_run(header=header)
    complete = complete_cycles(run=run, expected=expected)
    if not complete:
        return None

    numbers = sorted(complete)
    # Every phase the cycle logged, so a log from a build that still ran
    # 0.1 s holds is measured as it ran
    actual = {n: sum(line["actual_ms"] for line in complete[n].values()) / 1000.0 for n in numbers}
    scheduled = {n: sum(line["scheduled_ms"] for line in complete[n].values()) / 1000.0 for n in numbers}
    mean_actual = statistics.mean(data=actual.values())
    mean_scheduled = statistics.mean(data=scheduled.values())
    worst_n = max(numbers, key=lambda n: abs(actual[n] - scheduled[n]))
    drift = cumulative_drift(complete=complete, scheduled=scheduled, numbers=numbers)

    overshoot = overshoot_by_phase(run=run)
    overshoot_text = ", ".join(
        f"{name} {overshoot[name]:+.1f} ms" if overshoot[name] is not None else f"{name} n/a"
        for name in PHASES
    )
    return [
        ("Cycles measured", f"{len(numbers)} (complete cycles only)"),
        ("Mean cycle", f"{mean_actual:.2f} s ({format_pct(actual=mean_actual, configured=mean_scheduled)})"),
        (
            "Worst single cycle",
            f"{actual[worst_n]:.2f} s ({format_pct(actual=actual[worst_n], configured=scheduled[worst_n])})",
        ),
        ("Cumulative drift", f"{drift:+.2f} s"),
        ("Per-phase mean overshoot", overshoot_text),
    ]


def print_table(rows):
    print("| Field | Value |")
    print("|---|---|")
    for field, value in rows:
        print(f"| {field} | {value} |")


def main(argv=None):
    args = parse_args(argv=argv)
    if args.self_check:
        self_check()
        print("pacing-validation self-check passed")
        return 0
    events = read_events(log_path=args.log_path)
    header, header_index = find_header(events=events)
    configured = sum(header[field] for field in DURATION_FIELDS)
    top_rows = [
        ("Platform, version", f"{header.get('os', '?')}, {header.get('version', '?')}"),
        ("Configured cycle", configured_text(header=header, configured=configured)),
    ]

    runs = split_runs(events=events, header_index=header_index)
    scored = [score_run(header=header, run=run) for run in runs]
    # No phase_end at all reads the same as one run with no complete cycle
    if not runs or (len(runs) == 1 and scored[0] is None):
        print_table(rows=top_rows)
        print()
        print("No complete cycles in this log")
        return 0

    for number, rows in enumerate(scored, start=1):
        if len(runs) > 1:
            if number > 1:
                print()
            print(f"### Run {number} of {len(runs)}")
            print()
        if rows is None:
            print_table(rows=top_rows)
            print()
            print("No complete cycles in this run")
        else:
            print_table(rows=top_rows + rows)
    return 0


# The built-in logs. Each is built in memory from a few numbers, so the expected output can be
# read off the fixture and the checks need no files beside the script


def phase_end(cycle, phase, mono_ms, scheduled_ms, actual_ms):
    return {
        "t": 1758400000000 + mono_ms, "mono_ms": mono_ms, "event": "phase_end",
        "phase": phase, "cycle": cycle, "scheduled_ms": scheduled_ms, "actual_ms": actual_ms,
    }


def header_line(durations=(2, 0, 2, 0), drift=1, jitter=0, event="app_start", participant_id="P07"):
    line = {"t": 1758400000123, "mono_ms": 3, "event": event, "version": "2.0.28", "os": "macos"}
    line["participant_id"] = participant_id
    line["condition"] = "B"
    for field, value in zip(DURATION_FIELDS, durations):
        line[field] = value
    line["drift"] = drift
    for field in JITTER_FIELDS:
        line[field] = jitter
    return line


def make_run(durations, cycles, mono_ms, first_cycle=0, scheduled=None, extra_ms=1.0):
    """phase_end lines for whole cycles, each phase running past its schedule

    scheduled(cycle, phase) gives a phase's scheduled milliseconds and defaults to the
    configured duration. extra_ms is the overshoot of every phase, or a function of
    (cycle, phase) for one that varies. Returns the lines and the mono_ms after the last one
    """
    lines = []
    for cycle in range(first_cycle, first_cycle + cycles):
        for phase, seconds in zip(PHASES, durations):
            if seconds <= 0:
                continue
            due = scheduled(cycle, phase) if scheduled else seconds * 1000.0
            over = extra_ms(cycle, phase) if callable(extra_ms) else extra_ms
            mono_ms += due + over
            lines.append(phase_end(
                cycle=cycle, phase=phase, mono_ms=int(mono_ms), scheduled_ms=due, actual_ms=due + over,
            ))
    return lines, mono_ms


def to_text(events):
    """JSON Lines the way the app writes them: compact, and non-ASCII left raw"""
    return "".join(
        json.dumps(obj=event, separators=(",", ":"), ensure_ascii=False) + "\n" for event in events
    )


def run_script(text, directory):
    """Run main on `text`, returning (exit status, stdout, stderr). A SystemExit's message is the status"""
    path = Path(directory) / "log.jsonl"
    path.write_bytes(data=text if isinstance(text, bytes) else text.encode("utf-8"))
    out, err = io.StringIO(), io.StringIO()
    with contextlib.redirect_stdout(new_target=out), contextlib.redirect_stderr(new_target=err):
        try:
            status = main(argv=[str(path)])
        except SystemExit as exit_:
            status = exit_.code
    return status, out.getvalue(), err.getvalue()


def self_check():
    """Plain asserts on worked logs: the single-run table, runs, schedules and bad lines"""
    with tempfile.TemporaryDirectory() as directory:
        check_single_run(directory=directory)
        check_runs(directory=directory)
        check_schedule(directory=directory)
        check_lines(directory=directory)


NO_HEADER = "pacing-validation: no app_start or log_cleared line in this log"


def check_single_run(directory):
    # One run prints one table with no heading, the table this script always printed
    durations = (2, 0, 2, 0)
    lines, _ = make_run(durations=durations, cycles=14, mono_ms=100, extra_ms=2.0)
    events = [header_line(durations=durations), {"event": "animation_start"}, *lines, {"event": "animation_stop"}]
    status, out, err = run_script(text=to_text(events=events), directory=directory)
    assert (status, err) == (0, ""), (status, err)
    assert out == (
        "| Field | Value |\n"
        "|---|---|\n"
        "| Platform, version | macos, 2.0.28 |\n"
        "| Configured cycle | 4.00 s |\n"
        "| Cycles measured | 14 (complete cycles only) |\n"
        "| Mean cycle | 4.00 s (+0.1%) |\n"
        "| Worst single cycle | 4.00 s (+0.1%) |\n"
        "| Cumulative drift | +0.06 s |\n"
        "| Per-phase mean overshoot | inhale +2.0 ms, hold_after_inhale n/a, exhale +2.0 ms, hold_after_exhale n/a |\n"
    ), out

    # Four phases, and a half-finished last cycle that isn't counted
    durations = (4, 4, 4, 4)
    lines, mono = make_run(durations=durations, cycles=2, mono_ms=50)
    tail, _ = make_run(durations=(4, 4, 0, 0), cycles=1, mono_ms=mono, first_cycle=2)
    events = [header_line(durations=durations), *lines, *tail]
    status, out, err = run_script(text=to_text(events=events), directory=directory)
    assert "| Cycles measured | 2 (complete cycles only) |" in out, out
    assert "| Configured cycle | 16.00 s |" in out, out
    assert "hold_after_inhale +1.0 ms" in out and "hold_after_exhale +1.0 ms" in out, out

    # No phase_end, or none complete: the header rows and the one sentence
    one_phase = make_run(durations=(5, 0, 5, 0), cycles=1, mono_ms=0)[0][:1]
    for body in ([], one_phase):
        events = [header_line(durations=(5, 0, 5, 0), event="log_cleared"), *body]
        status, out, err = run_script(text=to_text(events=events), directory=directory)
        assert out == (
            "| Field | Value |\n|---|---|\n| Platform, version | macos, 2.0.28 |\n"
            "| Configured cycle | 10.00 s |\n\nNo complete cycles in this log\n"
        ), out

    # Only what follows the last header is scored
    first, mono = make_run(durations=(2, 0, 2, 0), cycles=3, mono_ms=10)
    second, _ = make_run(durations=(2, 0, 2, 0), cycles=5, mono_ms=mono + 500)
    events = [header_line(), *first, header_line(event="log_cleared"), *second]
    out = run_script(text=to_text(events=events), directory=directory)[1]
    assert "| Cycles measured | 5 (complete cycles only) |" in out, out

    # A log with no header can't be scored
    assert run_script(text=to_text(events=first), directory=directory)[0] == NO_HEADER


def check_runs(directory):
    durations = (2, 0, 2, 0)
    start = {"event": "animation_start"}
    stop = {"event": "animation_stop"}

    # Stop, then Start: the cycle count goes back to 0 behind an animation_start line.
    # Grouped by cycle alone, the second run overwrote the first one's cycles
    one, mono = make_run(durations=durations, cycles=4, mono_ms=100, extra_ms=1.0)
    two, _ = make_run(durations=durations, cycles=6, mono_ms=mono + 9000, extra_ms=5.0)
    events = [header_line(), start, *one, stop, start, *two]
    status, out, err = run_script(text=to_text(events=events), directory=directory)
    assert (status, err) == (0, ""), (status, err)
    assert out.startswith("### Run 1 of 2\n\n| Field | Value |\n"), out
    assert out.count("| Cycles measured |") == 2, out
    run_1, run_2 = out.split("\n### Run 2 of 2\n")
    assert "| Cycles measured | 4 (complete cycles only) |" in run_1, out
    assert "| Cycles measured | 6 (complete cycles only) |" in run_2, out
    assert "inhale +1.0 ms" in run_1 and "inhale +5.0 ms" in run_2, out
    # The 9 s with the animation stopped is in neither run's drift
    assert "| Cumulative drift | +0.01 s |" in run_1, out
    assert "| Cumulative drift | +0.06 s |" in run_2, out

    # A restart that left no line: color or shape changed, so cycle 0 follows cycle 3
    one, mono = make_run(durations=durations, cycles=4, mono_ms=100)
    two, _ = make_run(durations=durations, cycles=3, mono_ms=mono + 50)
    runs = split_runs(events=[header_line(), start, *one, *two], header_index=0)
    assert [len(run) for run in runs] == [8, 6], runs
    assert [line["cycle"] for line in runs[1]] == [0, 0, 1, 1, 2, 2]
    out = run_script(text=to_text(events=[header_line(), start, *one, *two]), directory=directory)[1]
    assert out.count("### Run") == 2 and "### Run 2 of 2" in out, out

    # An animation_start begins a run even where the numbers alone wouldn't split it
    inhale = phase_end(cycle=0, phase="inhale", mono_ms=100, scheduled_ms=2000.0, actual_ms=2001.0)
    exhale = phase_end(cycle=0, phase="exhale", mono_ms=200, scheduled_ms=2000.0, actual_ms=2001.0)
    runs = split_runs(events=[header_line(), inhale, start, exhale], header_index=0)
    assert [len(run) for run in runs] == [1, 1], runs

    # A restart before the first phase ended repeats the first line, which doesn't increase
    lines, _ = make_run(durations=durations, cycles=1, mono_ms=0)
    runs = split_runs(events=[header_line(), lines[0], lines[0], lines[1]], header_index=0)
    assert [len(run) for run in runs] == [1, 2], runs

    # Phase order counts too: the same cycle with an earlier phase is a new run
    mixed = [
        phase_end(cycle=0, phase="exhale", mono_ms=100, scheduled_ms=2000.0, actual_ms=2001.0),
        phase_end(cycle=0, phase="inhale", mono_ms=200, scheduled_ms=2000.0, actual_ms=2001.0),
    ]
    assert [len(run) for run in split_runs(events=[header_line(), *mixed], header_index=0)] == [1, 1]

    # A start with nothing after it is no run. A run too short to hold a complete cycle is
    # listed, so the numbers match the starts
    short = make_run(durations=durations, cycles=1, mono_ms=0)[0][:1]
    full, _ = make_run(durations=durations, cycles=3, mono_ms=5000)
    events = [header_line(), start, *short, start, start, *full]
    out = run_script(text=to_text(events=events), directory=directory)[1]
    assert out.count("### Run") == 2, out
    assert "No complete cycles in this run" in out, out
    assert "| Cycles measured | 3 (complete cycles only) |" in out, out

    # After a Clear the first run has no animation_start before it
    cleared, mono = make_run(durations=durations, cycles=3, mono_ms=0, first_cycle=9)
    later, _ = make_run(durations=durations, cycles=2, mono_ms=mono + 1000)
    events = [header_line(event="log_cleared"), *cleared, start, *later]
    assert run_script(text=to_text(events=events), directory=directory)[1].count("### Run") == 2

    # A Start while the animation runs: the controller thread can log the cut-short run's last
    # phase_end after the new animation_start. That line stays with the run it ended, which it
    # completes, and makes no run of its own
    one, mono = make_run(durations=durations, cycles=4, mono_ms=100, extra_ms=1.0)
    late = one.pop()
    assert (late["cycle"], late["phase"]) == (3, "exhale")
    two, _ = make_run(durations=durations, cycles=6, mono_ms=mono + 5, extra_ms=5.0)
    events = [header_line(), start, *one, start, late, *two]
    runs = split_runs(events=events, header_index=0)
    assert [len(run) for run in runs] == [8, 12] and runs[0][-1] is late, runs
    status, out, err = run_script(text=to_text(events=events), directory=directory)
    assert (status, err) == (0, ""), (status, err)
    assert out.count("### Run") == 2 and "### Run 3" not in out, out
    run_1, run_2 = out.split("\n### Run 2 of 2\n")
    assert "| Cycles measured | 4 (complete cycles only) |" in run_1, out
    assert "| Cycles measured | 6 (complete cycles only) |" in run_2, out

    # Two Starts in a row change nothing: the line still goes to the last run that had lines
    events = [header_line(), start, *one, start, start, late, *two]
    assert [len(run) for run in split_runs(events=events, header_index=0)] == [8, 12]
    # Only up to the new run's first line: after it, a line with cycle above 0 is that run's own
    events = [header_line(), start, *one, start, *two[:2], late]
    runs = split_runs(events=events, header_index=0)
    assert [len(run) for run in runs] == [7, 3] and runs[1][-1] is late, runs
    # With no run before the start there is nothing for a late line to join
    events = [header_line(), start, late, *two]
    assert [len(run) for run in split_runs(events=events, header_index=0)] == [1, 12]


def check_schedule(directory):
    # Drift 1.01 lengthens every cycle by 1%, so the 12th is 4.5 s and the cycle that follows
    # the schedule exactly is nowhere near the 4 s the header configures. Every phase runs
    # 1 ms over its schedule, and the inhale of cycle 7 runs 30 ms more
    def drifted(cycle, phase):
        return 2000.0 * 1.01**cycle

    def spike(cycle, phase):
        return 31.0 if (cycle, phase) == (7, "inhale") else 1.0

    lines, _ = make_run(durations=(2, 0, 2, 0), cycles=12, mono_ms=100, scheduled=drifted, extra_ms=spike)
    out = run_script(text=to_text(events=[header_line(drift=1.01), *lines]), directory=directory)[1]
    assert (
        "| Configured cycle | 4.00 s (drift 1.01 applies, so each cycle is compared with its scheduled length) |"
    ) in out, out
    assert "| Mean cycle | 4.23 s (+0.1%) |" in out, out
    assert "| Worst single cycle | 4.32 s (+0.7%) |" in out, out
    assert "| Cumulative drift | +0.05 s |" in out, out
    # Against the configured 4 s the same cycles read as 5.8% slow, and drift as a timing error
    mean_secs = sum(line["actual_ms"] for line in lines) / 12 / 1000.0
    assert (mean_secs - 4.0) / 4.0 > 0.05

    # Randomized timing: every phase has its own scheduled length, and none of it is error
    lengths = {"inhale": [1800.0, 2100.0], "exhale": [2300.0, 1700.0]}
    lines, _ = make_run(
        durations=(2, 0, 2, 0), cycles=4, mono_ms=0, scheduled=lambda cycle, phase: lengths[phase][cycle % 2],
    )
    out = run_script(text=to_text(events=[header_line(jitter=0.25), *lines]), directory=directory)[1]
    assert (
        "| Configured cycle | 4.00 s (randomized timing applies, so each cycle is compared with its scheduled length) |"
    ) in out, out
    assert "| Mean cycle | 3.95 s (+0.1%) |" in out, out
    assert "-1.2%" not in out, out

    both = header_line(drift=1.5, jitter=0.1)
    assert configured_text(header=both, configured=4.0) == (
        "4.00 s (drift 1.5 and randomized timing apply, so each cycle is compared with its scheduled length)"
    )
    assert configured_text(header=header_line(), configured=4.0) == "4.00 s"

    # A cycle that lost a line is incomplete, and its length stays out of the drift
    lines, _ = make_run(durations=(2, 0, 2, 0), cycles=6, mono_ms=0, extra_ms=1.0)
    broken = [line for line in lines if (line["cycle"], line["phase"]) != (2, "exhale")]
    out = run_script(text=to_text(events=[header_line(), *broken]), directory=directory)[1]
    assert "| Cycles measured | 5 (complete cycles only) |" in out, out
    assert "| Cumulative drift | +0.01 s |" in out, out


def check_lines(directory):
    durations = (2, 0, 2, 0)
    lines, _ = make_run(durations=durations, cycles=4, mono_ms=100)
    clean = run_script(text=to_text(events=[header_line(durations=durations), *lines]), directory=directory)
    assert clean[0] == 0 and clean[2] == "" and "| Cycles measured | 4 " in clean[1], clean

    # A participant id holding the characters str.splitlines() breaks a line at, as the app
    # writes them: raw inside the JSON string
    for separator in ("\u0085", " ", " "):
        events = [header_line(durations=durations, participant_id=f"P{separator}07"), *lines]
        text = to_text(events=events)
        assert separator in text and text.count("\n") == len(lines) + 1
        assert run_script(text=text, directory=directory) == clean, repr(separator)

    # A line the app's write left cut, and two glued together: skipped, counted, the rest scored
    good = to_text(events=[header_line(durations=durations), *lines]).split("\n")
    cut = good[:3] + [good[3][:40]] + good[4:]
    status, out, err = run_script(text="\n".join(cut), directory=directory)
    assert status == 0, status
    assert err.count("isn't valid JSON") == 1 and "line 4 " in err, err
    assert "skipped 1 line that couldn't be read as log events" in err, err
    assert "| Cycles measured | 3 (complete cycles only) |" in out, out

    glued = good[:3] + [good[3] + good[4]] + good[5:]
    status, out, err = run_script(text="\n".join(glued), directory=directory)
    assert status == 0 and err.count("isn't valid JSON") == 1, err
    assert "| Cycles measured | 3 (complete cycles only) |" in out, out

    # A JSON value that isn't an object, a blank line, and CRLF line endings
    mixed = "\r\n".join(good[:2] + ["12345", "", "[1, 2]"] + good[2:])
    status, out, err = run_script(text=mixed, directory=directory)
    assert status == 0 and err.count("isn't a JSON object") == 2, err
    assert "skipped 2 lines that" in err and out == clean[1], (err, out)

    # A phase_end that is valid JSON and can't be scored is skipped and counted with the cut
    # lines, and the rest of the log still scores. Each of these used to stop the script in a
    # traceback
    whole = lines[1]
    unscorable = [
        ({key: value for key, value in whole.items() if key != "cycle"}, "with no whole-number cycle"),
        ({**whole, "cycle": "3"}, "with no whole-number cycle"),
        ({**whole, "cycle": True}, "with no whole-number cycle"),
        ({key: value for key, value in whole.items() if key != "phase"}, "with no known phase"),
        ({**whole, "phase": "sleep"}, "with no known phase"),
        ({**whole, "phase": ["exhale"]}, "with no known phase"),
        ({key: value for key, value in whole.items() if key != "actual_ms"}, "with no number in actual_ms"),
        ({**whole, "scheduled_ms": None}, "with no number in scheduled_ms"),
        ({**whole, "mono_ms": "7"}, "with no number in mono_ms"),
    ]
    events = [header_line(durations=durations), lines[0], *(bad for bad, _ in unscorable), *lines[1:]]
    status, out, err = run_script(text=to_text(events=events), directory=directory)
    assert status == 0 and out == clean[1], (status, out)
    for number, (_, problem) in enumerate(unscorable, start=3):
        assert f"line {number} is a phase_end {problem}, skipped" in err, err
    assert f"skipped {len(unscorable)} lines that couldn't be read as log events" in err, err
    assert "Traceback" not in err and "isn't valid JSON" not in err, err
    # Other events are none of its business, whatever fields they lack
    events = [header_line(durations=durations), {"event": "reminder"}, {"event": "auto_stop"}, *lines]
    assert run_script(text=to_text(events=events), directory=directory) == clean

    # Bytes that aren't UTF-8 cost their line and not the log
    torn = "\n".join(good[:3]).encode("utf-8") + b'\n{"t":1,"note":"\xe2\x82\n' + "\n".join(good[3:]).encode("utf-8")
    status, out, err = run_script(text=torn, directory=directory)
    assert status == 0 and "line 4 " in err and out == clean[1], (status, err, out)

    # A file with no header, and one that isn't there
    assert run_script(text=b"", directory=directory)[0] == NO_HEADER
    try:
        main(argv=[str(Path(directory) / "missing.jsonl")])
    except SystemExit as exit_:
        assert str(exit_.code).startswith("pacing-validation: can't read "), exit_.code
    else:
        raise AssertionError("a missing file should end the run")


if __name__ == "__main__":
    sys.exit(main())
