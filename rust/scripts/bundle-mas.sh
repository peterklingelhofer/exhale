#!/usr/bin/env bash
#
# Build + sign + package the Rust exhale binary into a Mac App Store
# submission (.pkg), ready for Transporter upload.
#
# Produces (under `rust/target/mas/`):
#   exhale.app          — signed .app bundle (for local TestFlight/install QA)
#   exhale.pkg          — signed installer package (upload via Transporter)
#
# Requirements:
#   - macOS with Xcode command-line tools (codesign, productbuild, iconutil,
#     sips, lipo, plutil).
#   - rustup targets: aarch64-apple-darwin, x86_64-apple-darwin.
#     Installed automatically on first run.
#   - Two signing identities in the login keychain (from Apple Developer
#     → "Certificates, IDs & Profiles"):
#         "3rd Party Mac Developer Application: …"   — signs the .app
#         "3rd Party Mac Developer Installer:  …"    — signs the .pkg
#     Check with: `security find-identity -v -p codesigning`.
#   - A Mac App Store provisioning profile for bundle ID
#     `peterklingelhofer.exhale`, download from developer.apple.com and
#     save as `rust/signing/exhale.provisionprofile`  (or point
#     `PROVISION_PROFILE` env var at any path).
#
# Usage:
#   rust/scripts/bundle-mas.sh                           # default version
#   VERSION=2.0.8 BUILD=208 rust/scripts/bundle-mas.sh   # override version
#   PROVISION_PROFILE=/path/to/exhale.provisionprofile \
#       rust/scripts/bundle-mas.sh                       # override profile
#
# Environment overrides:
#   APP_IDENT       default: "3rd Party Mac Developer Application: …VZCHHV7VNW…"
#   INSTALLER_IDENT default: "3rd Party Mac Developer Installer:  …VZCHHV7VNW…"
#   VERSION         default: cargo version from Cargo.toml (bumped manually)
#   BUILD           default: VERSION with dots stripped
#   PROVISION_PROFILE default: rust/signing/exhale.provisionprofile
#   DRY_RUN=1       skip identity checks, profile requirement, and signing;
#                   emit an unsigned exhale.app for local validation.  Useful
#                   before the MAS distribution certs are installed.

set -euo pipefail

# ── Constants ────────────────────────────────────────────────────────────────
REPO_ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
RUST_ROOT="$REPO_ROOT/rust"

BUNDLE_ID="peterklingelhofer.exhale"
TEAM_ID="VZCHHV7VNW"
EXECUTABLE="exhale"
APP_NAME="exhale"
APP_DISPLAY_NAME="exhale"
CATEGORY="public.app-category.healthcare-fitness"
MIN_MACOS="11.0"

MASTER_ICON="$REPO_ROOT/swift/exhale/Assets.xcassets/AppIcon.appiconset/exhaleColorGradient1024.png"

OUT_DIR="$RUST_ROOT/target/mas"
BUILD_DIR="$OUT_DIR/build"
APP_BUNDLE="$OUT_DIR/$APP_NAME.app"
OUT_PKG="$OUT_DIR/$APP_NAME.pkg"

# ── Overrides ────────────────────────────────────────────────────────────────
APP_IDENT="${APP_IDENT:-3rd Party Mac Developer Application: Peter Klingelhofer ($TEAM_ID)}"
INSTALLER_IDENT="${INSTALLER_IDENT:-3rd Party Mac Developer Installer: Peter Klingelhofer ($TEAM_ID)}"
PROVISION_PROFILE="${PROVISION_PROFILE:-$RUST_ROOT/signing/exhale.provisionprofile}"

# Read version from crate Cargo.toml unless overridden.  The crate currently
# reads 0.1.0, so users will almost always want to override — but we match the
# Swift 2.0.7 → 2.0.8 expectation by default so a fresh run produces a
# submission one higher than the current MAS listing.
VERSION="${VERSION:-2.0.8}"
BUILD="${BUILD:-${VERSION//./}}"

# ── Helpers ──────────────────────────────────────────────────────────────────
log()  { printf '\033[1;34m[mas]\033[0m %s\n' "$*" >&2; }
die()  { printf '\033[1;31m[mas] error:\033[0m %s\n' "$*" >&2; exit 1; }

need() { command -v "$1" >/dev/null 2>&1 || die "missing required tool: $1"; }
for t in cargo codesign productbuild iconutil sips lipo plutil rustup; do need "$t"; done

DRY_RUN="${DRY_RUN:-0}"

[[ -f "$MASTER_ICON" ]] || die "master icon missing: $MASTER_ICON"

if [[ "$DRY_RUN" != "1" ]]; then
    [[ -f "$PROVISION_PROFILE" ]] || die "provisioning profile missing: $PROVISION_PROFILE
  → download from developer.apple.com → Certificates → Profiles, save as that path
  → or export PROVISION_PROFILE=/path/to/file.provisionprofile before re-running"

    security find-identity -v -p codesigning | grep -q "$APP_IDENT"       || die "signing identity not found: $APP_IDENT"
    security find-identity -v -p basic       | grep -q "$INSTALLER_IDENT" || die "installer identity not found: $INSTALLER_IDENT"
else
    log "DRY_RUN=1: skipping provisioning-profile + signing-identity checks"
fi

# ── 1. Rust targets ──────────────────────────────────────────────────────────
log "ensuring rustup targets…"
rustup target add aarch64-apple-darwin x86_64-apple-darwin >/dev/null

# ── 2. Build universal binary (MAS = --no-default-features) ──────────────────
log "cargo build --release --no-default-features × (arm64, x86_64)"
(cd "$RUST_ROOT" && \
    cargo build --release --no-default-features -p exhale-app \
        --target aarch64-apple-darwin >/dev/null && \
    cargo build --release --no-default-features -p exhale-app \
        --target x86_64-apple-darwin  >/dev/null)

BIN_ARM="$RUST_ROOT/target/aarch64-apple-darwin/release/$EXECUTABLE"
BIN_X86="$RUST_ROOT/target/x86_64-apple-darwin/release/$EXECUTABLE"
[[ -x "$BIN_ARM" ]] || die "arm64 binary missing: $BIN_ARM"
[[ -x "$BIN_X86" ]] || die "x86_64 binary missing: $BIN_X86"

# ── 3. Build .icns from the 1024 master ──────────────────────────────────────
log "building AppIcon.icns from 1024 master…"
rm -rf "$BUILD_DIR"
mkdir -p "$BUILD_DIR"

ICONSET="$BUILD_DIR/AppIcon.iconset"
mkdir -p "$ICONSET"
# iconutil expects these exact names; each pair is (N×N, N×N@2x = 2N).
for pair in "16:32" "32:64" "128:256" "256:512" "512:1024"; do
    one="${pair%:*}"; two="${pair#*:}"
    sips -z "$one" "$one" "$MASTER_ICON" --out "$ICONSET/icon_${one}x${one}.png"     >/dev/null
    sips -z "$two" "$two" "$MASTER_ICON" --out "$ICONSET/icon_${one}x${one}@2x.png" >/dev/null
done
iconutil -c icns "$ICONSET" -o "$BUILD_DIR/AppIcon.icns"

# ── 4. Assemble .app bundle ──────────────────────────────────────────────────
log "assembling $APP_NAME.app…"
rm -rf "$APP_BUNDLE"
CONTENTS="$APP_BUNDLE/Contents"
mkdir -p "$CONTENTS/MacOS" "$CONTENTS/Resources"

# Universal binary.
lipo -create "$BIN_ARM" "$BIN_X86" -output "$CONTENTS/MacOS/$EXECUTABLE"
chmod +x "$CONTENTS/MacOS/$EXECUTABLE"

# Icon.
cp "$BUILD_DIR/AppIcon.icns" "$CONTENTS/Resources/AppIcon.icns"

# Info.plist.  Use plutil to emit a canonical binary plist; Apple accepts both
# XML and binary, but binary avoids whitespace diffs between runs.
cat > "$CONTENTS/Info.plist" <<PLIST
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>CFBundleIdentifier</key>            <string>$BUNDLE_ID</string>
    <key>CFBundleName</key>                  <string>$APP_NAME</string>
    <key>CFBundleDisplayName</key>           <string>$APP_DISPLAY_NAME</string>
    <key>CFBundleExecutable</key>            <string>$EXECUTABLE</string>
    <key>CFBundleIconFile</key>              <string>AppIcon</string>
    <key>CFBundlePackageType</key>           <string>APPL</string>
    <key>CFBundleShortVersionString</key>    <string>$VERSION</string>
    <key>CFBundleVersion</key>               <string>$BUILD</string>
    <key>CFBundleInfoDictionaryVersion</key> <string>6.0</string>
    <key>CFBundleSignature</key>             <string>????</string>
    <key>LSMinimumSystemVersion</key>        <string>$MIN_MACOS</string>
    <key>LSUIElement</key>                   <true/>
    <key>LSApplicationCategoryType</key>     <string>$CATEGORY</string>
    <key>NSHumanReadableCopyright</key>      <string>© $(date +%Y) Peter Klingelhofer. All rights reserved.</string>
    <key>NSHighResolutionCapable</key>       <true/>
    <key>CFBundleSupportedPlatforms</key>    <array><string>MacOSX</string></array>
    <key>DTPlatformName</key>                <string>macosx</string>
    <key>ITSAppUsesNonExemptEncryption</key> <false/>
</dict>
</plist>
PLIST
plutil -lint "$CONTENTS/Info.plist" >/dev/null || die "Info.plist failed plutil lint"

# Entitlements — mirror the Swift app exactly (sandbox + user-selected
# read-only).  No network, no camera, no hotkey entitlement (we ship the
# MAS build with `--no-default-features`, which drops the hotkey crate).
cat > "$BUILD_DIR/exhale.entitlements" <<ENT
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>com.apple.security.app-sandbox</key>                   <true/>
    <key>com.apple.security.files.user-selected.read-only</key> <true/>
</dict>
</plist>
ENT

# Embedded provisioning profile.  Apple's installer checks that the embedded
# profile's entitlements are a superset of the binary's entitlements.
if [[ "$DRY_RUN" != "1" ]]; then
    cp "$PROVISION_PROFILE" "$CONTENTS/embedded.provisionprofile"
fi

# ── 5. Sign the .app ─────────────────────────────────────────────────────────
if [[ "$DRY_RUN" != "1" ]]; then
    log "signing $APP_NAME.app…"
    # Single-binary bundle: no nested executables to sign individually.
    codesign --force --timestamp \
        --entitlements "$BUILD_DIR/exhale.entitlements" \
        --sign "$APP_IDENT" \
        "$APP_BUNDLE"

    codesign --verify --deep --strict --verbose=2 "$APP_BUNDLE" \
        || die "codesign verification failed"

    # ── 6. Build signed .pkg ─────────────────────────────────────────────────
    log "productbuild → $OUT_PKG"
    productbuild --component "$APP_BUNDLE" /Applications \
        --sign "$INSTALLER_IDENT" \
        "$OUT_PKG"
fi

# ── 7. Done ──────────────────────────────────────────────────────────────────
if [[ "$DRY_RUN" == "1" ]]; then
    log "DRY_RUN complete — unsigned bundle ready for local validation"
    printf '\n  %s\n\n' "$APP_BUNDLE"
    echo "to test the unsigned bundle:  open \"$APP_BUNDLE\""
    echo "(Gatekeeper will warn on first launch — right-click → Open to bypass)"
else
    log "success"
    printf '\n  %s\n  %s\n\n' "$APP_BUNDLE" "$OUT_PKG"
    echo "next steps:"
    echo "  1. Install locally to verify sandbox behaviour:"
    echo "       sudo installer -pkg \"$OUT_PKG\" -target /"
    echo "       open /Applications/$APP_NAME.app"
    echo "  2. Confirm settings persist under"
    echo "       ~/Library/Containers/$BUNDLE_ID/Data/…"
    echo "  3. Upload to App Store Connect:"
    echo "       xcrun altool --upload-app -f \"$OUT_PKG\" -t osx \\"
    echo "         -u <apple-id> -p <app-specific-pw>"
    echo "     or drag the .pkg into Transporter.app."
fi
