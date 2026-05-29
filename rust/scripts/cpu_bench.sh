#!/usr/bin/env bash
#
# Live-process CPU bench — samples the real Rust app while it runs.
#
# Methodology mirrors the Swift `measureCPU` helper:
#   - Phase 1 (baseline):  launch with `is_animating = false` in settings.toml.
#                          Sample process CPU% 5 × 1 s. Kill.
#   - Phase 2 (animating): launch with `is_animating = true`.  Sample 5 × 1 s.
#   - Report delta = max(0, animating − baseline_avg).
#
# Runs all six variants that the Swift PerformanceTests suite covers.
#
# The script backs up your existing settings.toml, writes a stripped one for
# each phase, and restores the original on exit (even on ^C).
#
# Usage:
#   rust/scripts/cpu_bench.sh             # run all 6 variants
#   rust/scripts/cpu_bench.sh rect_ripple # just one variant (tag match)
#
# Requires: macOS, cargo, bash ≥ 4, python3.

set -euo pipefail

# ── Config ────────────────────────────────────────────────────────────────────
REPO_ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
RUST_ROOT="$REPO_ROOT/rust"
CONFIG_DIR="$HOME/Library/Application Support/com.peterklingelhofer.exhale"
CONFIG_FILE="$CONFIG_DIR/settings.toml"
BACKUP_FILE="$CONFIG_DIR/settings.toml.bench-backup"

SAMPLE_COUNT=5
SAMPLE_SECONDS=1
WARMUP_SECONDS=3

FILTER="${1:-}"

# ── Variants (match Swift PerformanceTests) ───────────────────────────────────
# format: tag|shape|gradient|hold|ripple|label
VARIANTS=(
    "rect_grad|rectangle|on|0|off|rect    + gradient"
    "circle_grad|circle|on|0|off|circle  + gradient"
    "fullscreen|fullscreen|off|0|off|fullscr + solid"
    "rect_ripple|rectangle|on|4|gradient|rect    + hold ripple gradient"
    "rect_ripple_stark|rectangle|off|4|stark|rect    + hold ripple stark"
    "circle_ripple|circle|on|4|gradient|circle  + hold ripple gradient"
)

# ── Helpers ───────────────────────────────────────────────────────────────────
log() { printf '\033[2m[bench]\033[0m %s\n' "$*" >&2; }

# Convert ps's MM:SS.ss or HH:MM:SS.ss cpu time string → seconds.
parse_cputime() {
    python3 - <<PY
t = "$1".strip()
parts = t.split(":")
if len(parts) == 2:      h, m, s = "0", parts[0], parts[1]
elif len(parts) == 3:    h, m, s = parts
else:                    print(0); raise SystemExit
print(int(h)*3600 + int(m)*60 + float(s))
PY
}

get_cpu_seconds() {
    local raw
    raw=$(ps -p "$1" -o time= 2>/dev/null | awk '{print $1}') || return 1
    [[ -z "$raw" ]] && return 1
    parse_cputime "$raw"
}

sample_once() {
    local pid="$1"
    local t0 t1 w0 w1
    t0=$(get_cpu_seconds "$pid") || { echo "0.0"; return; }
    w0=$(python3 -c 'import time; print(time.time())')
    sleep "$SAMPLE_SECONDS"
    t1=$(get_cpu_seconds "$pid") || { echo "0.0"; return; }
    w1=$(python3 -c 'import time; print(time.time())')
    python3 -c "dt=$t1-$t0; dw=$w1-$w0; print(f'{(dt/dw)*100:.1f}')"
}

write_settings() {
    local animating="$1"
    local shape="$2"
    local gradient="$3"
    local hold="$4"
    local ripple="$5"
    cat > "$CONFIG_FILE" <<TOML
inhale_color              = [0.0, 0.5, 1.0, 1.0]
exhale_color              = [1.0, 0.2, 0.0, 1.0]
background_color          = [0.0, 0.0, 0.0, 0.0]
overlay_opacity           = 0.25
shape                     = "$shape"
color_fill_gradient       = "$gradient"
animation_mode            = "sinusoidal"
hold_ripple_mode          = "$ripple"
app_visibility            = "top_bar_only"
inhale_duration           = 2.0
post_inhale_hold_duration = $hold
exhale_duration           = 2.0
post_exhale_hold_duration = $hold
drift                     = 1.00
randomized_timing_inhale           = 0.0
randomized_timing_post_inhale_hold = 0.0
randomized_timing_exhale           = 0.0
randomized_timing_post_exhale_hold = 0.0
reminder_interval_minutes = 0.0
auto_stop_minutes         = 0.0
is_animating              = $animating
is_paused                 = false
TOML
}

restore_settings() {
    if [[ -f "$BACKUP_FILE" ]]; then
        mv "$BACKUP_FILE" "$CONFIG_FILE"
        log "restored original settings.toml"
    fi
}

cleanup() {
    if [[ -n "${APP_PID:-}" ]]; then
        kill "$APP_PID" 2>/dev/null || true
        wait "$APP_PID" 2>/dev/null || true
        APP_PID=""
    fi
    restore_settings
}
trap cleanup EXIT INT TERM

run_phase() {
    local animating="$1" shape="$2" gradient="$3" hold="$4" ripple="$5"

    write_settings "$animating" "$shape" "$gradient" "$hold" "$ripple"

    # `RUST_LOG=warn` keeps wgpu from firehosing INFO-level messages.
    RUST_LOG=warn "$BIN" >/dev/null 2>&1 &
    APP_PID=$!
    sleep "$WARMUP_SECONDS"

    if ! kill -0 "$APP_PID" 2>/dev/null; then
        echo "error: exhale exited before sampling began" >&2
        exit 1
    fi

    local samples=()
    for _ in $(seq 1 "$SAMPLE_COUNT"); do
        samples+=("$(sample_once "$APP_PID")")
    done

    kill "$APP_PID" 2>/dev/null || true
    wait "$APP_PID" 2>/dev/null || true
    APP_PID=""

    echo "${samples[*]}"
}

# ── Build once ────────────────────────────────────────────────────────────────
log "building exhale (release)…"
(cd "$RUST_ROOT" && cargo build --release -p exhale-app >/dev/null 2>&1)
BIN="$RUST_ROOT/target/release/exhale"
[[ -x "$BIN" ]] || { echo "error: binary missing at $BIN" >&2; exit 1; }

# ── Backup original settings ─────────────────────────────────────────────────
mkdir -p "$CONFIG_DIR"
if [[ -f "$CONFIG_FILE" ]]; then
    cp "$CONFIG_FILE" "$BACKUP_FILE"
    log "backed up settings.toml → $BACKUP_FILE"
fi

# ── Header ────────────────────────────────────────────────────────────────────
echo "exhale live-process CPU bench"
echo "$SAMPLE_COUNT × ${SAMPLE_SECONDS}s samples, warmup ${WARMUP_SECONDS}s"
echo "────────────────────────────────────────────────────────────────────────"

for entry in "${VARIANTS[@]}"; do
    IFS="|" read -r tag shape gradient hold ripple label <<< "$entry"

    if [[ -n "$FILTER" && "$tag" != *"$FILTER"* ]]; then
        continue
    fi

    log "→ [$tag] baseline"
    BASELINE=$(run_phase false "$shape" "$gradient" "$hold" "$ripple")
    log "→ [$tag] animating"
    ANIMATING=$(run_phase true "$shape" "$gradient" "$hold" "$ripple")

    python3 - "$tag" "$label" "$BASELINE" "$ANIMATING" <<'PY'
import sys
tag, label, baseline_str, animating_str = sys.argv[1:5]
base = [float(x) for x in baseline_str.split()]
anim = [float(x) for x in animating_str.split()]
base_avg = sum(base)/len(base)
anim_avg = sum(anim)/len(anim)
delta = [max(0, a - base_avg) for a in anim]
peak = max(delta)
avg = sum(delta)/len(delta)

def fmt(xs): return "[" + ", ".join(f"{x:>5.1f}%" for x in xs) + "]"

print(f"[{tag}] {label}")
print(f"  baseline : {fmt(base)}  avg {base_avg:>5.1f}%")
print(f"  animating: {fmt(anim)}  avg {anim_avg:>5.1f}%")
print(f"  delta    : {fmt(delta)}  peak {peak:>5.1f}%  avg {avg:>5.1f}%")
print()
PY
done

echo "Done.  Baseline includes full idle app (tray + event loop + static"
echo "overlay repainting at compositor cadence).  Delta isolates the"
echo "controller + state-update cost above that floor."
