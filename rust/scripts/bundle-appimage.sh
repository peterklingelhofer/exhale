#!/usr/bin/env bash
#
# Package the Rust exhale binary as a portable Linux AppImage
#
# Produces (under `rust/target/appimage/`)
#   exhale-${VERSION}-x86_64.AppImage: single-file portable Linux app
#
# Requirements
#   - Linux host (appimagetool is Linux-only. macOS users should let CI run this)
#   - Rust toolchain with `x86_64-unknown-linux-gnu` target
#   - System libraries listed in snapcraft.yaml (libx11-dev, libxkbcommon-dev,
#     libwayland-dev, libglib2.0-dev, libgtk-3-dev, libayatana-appindicator3-dev,
#     libvulkan-dev, pkg-config), plus libxdo-dev and libxkbcommon-x11-0, which
#     provide libraries the AppImage bundles (see step 4)
#   - `patchelf`
#   - `appimagetool`, auto-downloaded into rust/target/appimage/bin if missing.
#     It's the current one from AppImage/appimagetool, whose runtime is static:
#     the AppImage runs without libfuse2, which Ubuntu 22.04 and later don't
#     install by default
#
# Usage
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
# Named after the release asset. Existing checkouts hold the old AppImageKit
# build under the plain `appimagetool` name, which must not be reused
APPIMAGETOOL="$TOOL_DIR/appimagetool-x86_64.AppImage"

VERSION="${VERSION:-2.0.23}"
OUT_APPIMAGE="$OUT_DIR/exhale-${VERSION}-x86_64.AppImage"

# ── Helpers ──────────────────────────────────────────────────────────────────
log() { printf '\033[1;34m[appimage]\033[0m %s\n' "$*" >&2; }
die() { printf '\033[1;31m[appimage] error:\033[0m %s\n' "$*" >&2; exit 1; }

case "$(uname -s)" in
    Linux*) ;;
    *) die "appimagetool requires Linux. Run this via GitHub Actions or a Linux VM." ;;
esac

for t in cargo rustup patchelf; do
    command -v "$t" >/dev/null 2>&1 || die "missing required tool: $t"
done

# ── 1. Build the Rust binary ─────────────────────────────────────────────────
log "cargo build --release --no-default-features --target x86_64-unknown-linux-gnu"
(cd "$RUST_ROOT" && rustup target add x86_64-unknown-linux-gnu >/dev/null)
(cd "$RUST_ROOT" && \
    cargo build --release --no-default-features -p exhale-app \
        --target x86_64-unknown-linux-gnu)

BIN_PATH="$RUST_ROOT/target/x86_64-unknown-linux-gnu/release/exhale"
[[ -x "$BIN_PATH" ]] || die "binary missing: $BIN_PATH"

# ── 2. Fetch appimagetool if needed ──────────────────────────────────────────
mkdir -p "$TOOL_DIR"
if [[ ! -x "$APPIMAGETOOL" ]]; then
    log "downloading appimagetool..."
    curl -fsSL -o "$APPIMAGETOOL" \
        "https://github.com/AppImage/appimagetool/releases/download/continuous/appimagetool-x86_64.AppImage"
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

# AppImage convention: exhale.desktop + exhale.png + AppRun all at the AppDir
# root, and .DirIcon, which AppImageHub's lint requires
install -m 644 "$RUST_ROOT/packaging/linux/exhale.desktop"      "$APPDIR/exhale.desktop"
install -m 644 "$PKG_DIR/exhale.png"                            "$APPDIR/exhale.png"
ln -s exhale.png "$APPDIR/.DirIcon"

cat > "$APPDIR/AppRun" <<'SH'
#!/usr/bin/env bash
HERE="$(dirname "$(readlink -f "$0")")"
export PATH="$HERE/usr/bin:$PATH"
export XDG_DATA_DIRS="$HERE/usr/share:${XDG_DATA_DIRS:-/usr/local/share:/usr/share}"
exec "$HERE/usr/bin/exhale" "$@"
SH
chmod +x "$APPDIR/AppRun"

# ── 4. Bundle the libraries a desktop may lack ───────────────────────────────
# The AppImage uses the host's glibc, GTK 3, X11 and GPU drivers, which every
# Linux desktop has.  These it can't count on: without libxdo or
# libxkbcommon-x11 exhale can't start, and without the tray library it runs
# with no tray icon:
#   - libxdo, and libXtst under it: linked by tray-icon's menu code
#   - libayatana-appindicator3 and the libraries it loads: dlopen'd by
#     tray-icon for the tray icon
#   - libxkbcommon-x11, and libxcb-xkb under it: dlopen'd by winit on X11.
#     It only works with the libxkbcommon of its own release (Debian pins
#     `libxkbcommon0 (= same version)`), so that ships too
BUNDLED_LIBS=(
    libxdo.so.3
    libXtst.so.6
    libayatana-appindicator3.so.1
    libayatana-indicator3.so.7
    libayatana-ido3-0.4.so.0
    libdbusmenu-glib.so.4
    libdbusmenu-gtk3.so.4
    libxkbcommon.so.0
    libxkbcommon-x11.so.0
    libxcb-xkb.so.1
)
LDCONFIG="$(command -v ldconfig || echo /sbin/ldconfig)"
mkdir -p "$APPDIR/usr/lib"
for lib in "${BUNDLED_LIBS[@]}"; do
    src="$("$LDCONFIG" -p | awk -v so="$lib" '$1 == so && /x86-64/ { print $NF; exit }')"
    [[ -n "$src" ]] || die "library not installed on this build machine: $lib"
    install -m 644 "$(readlink -f "$src")" "$APPDIR/usr/lib/$lib"
    # Each bundled library finds its bundled dependencies next to itself
    patchelf --set-rpath '$ORIGIN' "$APPDIR/usr/lib/$lib"
done

# The binary looks in usr/lib first, both for the libraries it links and for
# the ones it dlopens (dlopen honours the caller's RUNPATH), so nothing has to
# set LD_LIBRARY_PATH, which would leak into the browser that exhale opens.
# Linking the bundled libxkbcommon directly makes it the only copy in the
# process: host GTK and the bundled libxkbcommon-x11 both get it
patchelf --set-rpath '$ORIGIN/../lib' "$APPDIR/usr/bin/exhale"
patchelf --add-needed libxkbcommon.so.0 "$APPDIR/usr/bin/exhale"
for lib in libxdo.so.3 libxkbcommon.so.0; do
    resolved="$(ldd "$APPDIR/usr/bin/exhale" | awk -v so="$lib" '$1 == so { print $3 }')"
    [[ "$(readlink -f "$resolved")" == "$(readlink -f "$APPDIR/usr/lib/$lib")" ]] \
        || die "exhale resolves $lib to '$resolved' instead of the bundled copy"
done

# ── 5. Pack AppImage ─────────────────────────────────────────────────────────
# APPIMAGE_EXTRACT_AND_RUN: appimagetool is itself an AppImage, and this way
# it runs without FUSE (CI runners and containers often lack it)
log "appimagetool $APPDIR -> $OUT_APPIMAGE"
APPIMAGE_EXTRACT_AND_RUN=1 ARCH=x86_64 VERSION="$VERSION" \
    "$APPIMAGETOOL" --no-appstream "$APPDIR" "$OUT_APPIMAGE"

log "success"
printf '\n  %s\n\n' "$OUT_APPIMAGE"
echo "test locally:   \"$OUT_APPIMAGE\""
