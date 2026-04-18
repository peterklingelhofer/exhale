#!/usr/bin/env bash
#
# Package the Rust exhale binary as a portable Linux AppImage.
#
# Produces (under `rust/target/appimage/`):
#   exhale-${VERSION}-x86_64.AppImage   — single-file portable Linux app
#
# Requirements:
#   - Linux host (appimagetool is Linux-only; macOS users should let CI run this).
#   - Rust toolchain with `x86_64-unknown-linux-gnu` target.
#   - System libraries listed in snapcraft.yaml (libx11-dev, libxkbcommon-dev,
#     libwayland-dev, libglib2.0-dev, libgtk-3-dev, libayatana-appindicator3-dev,
#     libvulkan-dev, pkg-config).
#   - `appimagetool` — auto-downloaded into rust/target/appimage/bin if missing.
#
# Usage:
#   rust/scripts/bundle-appimage.sh                    # VERSION from crate
#   VERSION=2.0.8 rust/scripts/bundle-appimage.sh
#

set -euo pipefail

# ── Paths ────────────────────────────────────────────────────────────────────
REPO_ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
RUST_ROOT="$REPO_ROOT/rust"
PKG_DIR="$RUST_ROOT/packaging/linux/appimage"

OUT_DIR="$RUST_ROOT/target/appimage"
APPDIR="$OUT_DIR/exhale.AppDir"
TOOL_DIR="$OUT_DIR/bin"
APPIMAGETOOL="$TOOL_DIR/appimagetool"

VERSION="${VERSION:-2.0.8}"
OUT_APPIMAGE="$OUT_DIR/exhale-${VERSION}-x86_64.AppImage"

# ── Helpers ──────────────────────────────────────────────────────────────────
log() { printf '\033[1;34m[appimage]\033[0m %s\n' "$*" >&2; }
die() { printf '\033[1;31m[appimage] error:\033[0m %s\n' "$*" >&2; exit 1; }

case "$(uname -s)" in
    Linux*) ;;
    *) die "appimagetool requires Linux; run this via GitHub Actions or a Linux VM." ;;
esac

for t in cargo rustup; do
    command -v "$t" >/dev/null 2>&1 || die "missing required tool: $t"
done

# ── 1. Build the Rust binary ─────────────────────────────────────────────────
log "cargo build --release --no-default-features --target x86_64-unknown-linux-gnu"
rustup target add x86_64-unknown-linux-gnu >/dev/null
(cd "$RUST_ROOT" && \
    cargo build --release --no-default-features -p exhale-app \
        --target x86_64-unknown-linux-gnu)

BIN_PATH="$RUST_ROOT/target/x86_64-unknown-linux-gnu/release/exhale"
[[ -x "$BIN_PATH" ]] || die "binary missing: $BIN_PATH"

# ── 2. Fetch appimagetool if needed ──────────────────────────────────────────
mkdir -p "$TOOL_DIR"
if [[ ! -x "$APPIMAGETOOL" ]]; then
    log "downloading appimagetool…"
    curl -fsSL -o "$APPIMAGETOOL" \
        "https://github.com/AppImage/AppImageKit/releases/download/continuous/appimagetool-x86_64.AppImage"
    chmod +x "$APPIMAGETOOL"
fi

# ── 3. Assemble AppDir ───────────────────────────────────────────────────────
log "assembling $APPDIR"
rm -rf "$APPDIR"
mkdir -p "$APPDIR/usr/bin" \
         "$APPDIR/usr/share/applications" \
         "$APPDIR/usr/share/icons/hicolor/256x256/apps" \
         "$APPDIR/usr/share/icons/hicolor/512x512/apps"

install -m 755 "$BIN_PATH"                                      "$APPDIR/usr/bin/exhale"
install -m 644 "$RUST_ROOT/packaging/linux/exhale.desktop"      "$APPDIR/usr/share/applications/exhale.desktop"
install -m 644 "$RUST_ROOT/packaging/linux/icons/256x256/exhale.png" \
                                                                "$APPDIR/usr/share/icons/hicolor/256x256/apps/exhale.png"
install -m 644 "$RUST_ROOT/packaging/linux/icons/512x512/exhale.png" \
                                                                "$APPDIR/usr/share/icons/hicolor/512x512/apps/exhale.png"

# AppImage convention: exhale.desktop + exhale.png + AppRun all at the AppDir root.
install -m 644 "$RUST_ROOT/packaging/linux/exhale.desktop"      "$APPDIR/exhale.desktop"
install -m 644 "$PKG_DIR/exhale.png"                            "$APPDIR/exhale.png"

cat > "$APPDIR/AppRun" <<'SH'
#!/usr/bin/env bash
HERE="$(dirname "$(readlink -f "$0")")"
export PATH="$HERE/usr/bin:$PATH"
export XDG_DATA_DIRS="$HERE/usr/share:${XDG_DATA_DIRS:-/usr/local/share:/usr/share}"
exec "$HERE/usr/bin/exhale" "$@"
SH
chmod +x "$APPDIR/AppRun"

# ── 4. Pack AppImage ─────────────────────────────────────────────────────────
log "appimagetool $APPDIR → $OUT_APPIMAGE"
ARCH=x86_64 "$APPIMAGETOOL" --no-appstream "$APPDIR" "$OUT_APPIMAGE"

log "success"
printf '\n  %s\n\n' "$OUT_APPIMAGE"
echo "test locally:   \"$OUT_APPIMAGE\""
