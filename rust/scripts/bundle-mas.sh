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
#         "Apple Distribution: …"                    — signs the .app
#         "3rd Party Mac Developer Installer: …"     — signs the .pkg
#     Check with: `security find-identity -v -p basic`.
#     ("Apple Distribution" is the modern unified iOS+macOS App Store cert;
#      the legacy "3rd Party Mac Developer Application" still works but
#      Apple no longer issues it — select "Apple Distribution" in the
#      portal when creating new certs.)
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
#   APP_IDENT       default: "Apple Distribution: …VZCHHV7VNW…"
#   INSTALLER_IDENT default: "3rd Party Mac Developer Installer: …VZCHHV7VNW…"
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
APP_IDENT="${APP_IDENT:-Apple Distribution: Peter Klingelhofer ($TEAM_ID)}"
INSTALLER_IDENT="${INSTALLER_IDENT:-3rd Party Mac Developer Installer: Peter Klingelhofer ($TEAM_ID)}"
PROVISION_PROFILE="${PROVISION_PROFILE:-$RUST_ROOT/signing/exhale.provisionprofile}"

# Read version from crate Cargo.toml unless overridden.  The crate currently
# reads 0.1.0, so users will almost always want to override — but we match the
# Swift 2.0.7 → 2.0.8 expectation by default so a fresh run produces a
# submission one higher than the current MAS listing.
VERSION="${VERSION:-2.0.21}"
BUILD="${BUILD:-${VERSION//./}}"

# ── Helpers ──────────────────────────────────────────────────────────────────
log()  { printf '\033[1;34m[mas]\033[0m %s\n' "$*" >&2; }
die()  { printf '\033[1;31m[mas] error:\033[0m %s\n' "$*" >&2; exit 1; }

need() { command -v "$1" >/dev/null 2>&1 || die "missing required tool: $1"; }
for t in cargo codesign productbuild iconutil sips lipo plutil rustup; do need "$t"; done

DRY_RUN="${DRY_RUN:-0}"

# SKIP_PKG=1 — skip productbuild, emit a signed `.app.zip` instead. Set in CI
# because productbuild deterministically hangs on macos-latest runners (likely
# OCSP / CRL revocation check on an unreachable endpoint that the codesign
# code path doesn't trigger). Local devs leave it off so they keep getting a
# `.pkg` for Transporter. App Store submissions go through Transporter from a
# local build anyway; the CI artifact is just for the GitHub Release page
SKIP_PKG="${SKIP_PKG:-0}"

[[ -f "$MASTER_ICON" ]] || die "master icon missing: $MASTER_ICON"

if [[ "$DRY_RUN" != "1" ]]; then
    [[ -f "$PROVISION_PROFILE" ]] || die "provisioning profile missing: $PROVISION_PROFILE
  → download from developer.apple.com → Certificates → Profiles, save as that path
  → or export PROVISION_PROFILE=/path/to/file.provisionprofile before re-running"

    security find-identity -v -p basic | grep -q "$APP_IDENT"       || die "signing identity not found: $APP_IDENT"
    security find-identity -v -p basic | grep -q "$INSTALLER_IDENT" || die "installer identity not found: $INSTALLER_IDENT"
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
#
# `application-identifier` + `team-identifier` MUST be present in the
# binary's code signature and must match the embedded provisioning profile
# exactly, or App Store Connect rejects with error 90886.  Derive both
# from the profile itself so renames/team-changes propagate automatically.
# Use PlistBuddy instead of `plutil -extract`: plutil's key-path syntax
# uses `.` as the separator, which collides with the dotted key names
# (`com.apple.application-identifier` etc.) that the entitlements plist
# actually uses.  PlistBuddy uses `:` and handles the real key names.
PROF_PLIST=$(mktemp)
trap 'rm -f "$PROF_PLIST"' EXIT
security cms -D -i "$PROVISION_PROFILE" > "$PROF_PLIST" \
    || die "could not decode $PROVISION_PROFILE"
APP_IDENTIFIER=$(/usr/libexec/PlistBuddy -c "Print :Entitlements:com.apple.application-identifier" "$PROF_PLIST") \
    || die "could not read application-identifier from $PROVISION_PROFILE"
TEAM_IDENTIFIER=$(/usr/libexec/PlistBuddy -c "Print :Entitlements:com.apple.developer.team-identifier" "$PROF_PLIST") \
    || die "could not read team-identifier from $PROVISION_PROFILE"

cat > "$BUILD_DIR/exhale.entitlements" <<ENT
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>com.apple.application-identifier</key>                 <string>$APP_IDENTIFIER</string>
    <key>com.apple.developer.team-identifier</key>              <string>$TEAM_IDENTIFIER</string>
    <key>com.apple.security.app-sandbox</key>                   <true/>
    <key>com.apple.security.files.user-selected.read-only</key> <true/>
</dict>
</plist>
ENT

# Embedded provisioning profile.  Apple's installer checks that the embedded
# profile's entitlements are a superset of the binary's entitlements.
if [[ "$DRY_RUN" != "1" ]]; then
    cp "$PROVISION_PROFILE" "$CONTENTS/embedded.provisionprofile"
    # App Store Connect rejects bundles containing `com.apple.quarantine`
    # (set automatically by browsers on downloaded files like the
    # provisioning profile) with error 91109.  Wipe every xattr from the
    # whole bundle before signing — any attribute Apple doesn't explicitly
    # strip is treated as a package-contents violation.
    xattr -cr "$APP_BUNDLE"
fi

# ── 5. Sign the .app ─────────────────────────────────────────────────────────
#
# `codesign --timestamp` reaches out to Apple's RFC 3161 TSA
# (timestamp.apple.com); `productbuild --sign` walks the installer cert
# chain and on a fresh keychain without WWDR can hang fetching the
# intermediate. Neither tool has a default client-side timeout, so
# when either back-end flakes the step blocks indefinitely. We wrap
# both in `gtimeout` (GNU coreutils) on macOS to bound the wait and
# retry on the timeout-specific exit code 124. CI installs coreutils
# explicitly; locally, install via `brew install coreutils` or the
# script will run without timeout protection (and warn once at top)
if   command -v gtimeout >/dev/null 2>&1; then TIMEOUT_BIN="gtimeout"
elif command -v timeout  >/dev/null 2>&1; then TIMEOUT_BIN="timeout"
else
    TIMEOUT_BIN=""
    log "warning: gtimeout/timeout not found; signing steps will run without"
    log "         timeout protection. Install via 'brew install coreutils'"
    log "         if you want bounded retries on TSA / cert-chain hangs"
fi

# Retry $@ up to 3 times, each attempt bounded by $secs.  Distinguishes
# "command timed out" (exit 124 from gtimeout) from "command failed"
# (any other non-zero exit). Captures the wrapped command's exit code
# directly into rc instead of via `if … ; then return 0; fi; rc=$?`
# (which is a notorious bash trap: $? after `if` with no else returns
# 0 when the test failed, masking the real exit code)
retry_signing() {
    local name="$1" secs="$2"; shift 2
    local attempt rc
    for attempt in 1 2 3; do
        log "$name (attempt $attempt, ${secs}s timeout)…"
        # Append `|| rc=$?` so errexit (set -e) doesn't bail before we can
        # inspect the return code. Without this, a 124 from gtimeout exits
        # the whole script instead of letting us retry
        rc=0
        if [[ -n "$TIMEOUT_BIN" ]]; then
            "$TIMEOUT_BIN" --kill-after=10s "$secs" "$@" || rc=$?
        else
            "$@" || rc=$?
        fi
        if [[ $rc -eq 0 ]]; then return 0; fi
        if [[ $rc -eq 124 || $rc -eq 137 ]]; then
            log "$name timed out after ${secs}s, retrying"
            sleep $((attempt * 10))
            continue
        fi
        die "$name failed with exit $rc (not a timeout)"
    done
    die "$name still timing out after 3 attempts; check Apple TSA / keychain"
}

if [[ "$DRY_RUN" != "1" ]]; then
    # Single-binary bundle: no nested executables to sign individually.
    retry_signing "codesign --sign" 300 \
        codesign --force --timestamp \
            --entitlements "$BUILD_DIR/exhale.entitlements" \
            --sign "$APP_IDENT" \
            "$APP_BUNDLE"

    codesign --verify --deep --strict --verbose=2 "$APP_BUNDLE" \
        || die "codesign verification failed"

    if [[ "$SKIP_PKG" != "1" ]]; then
        # ── 6. Signed .pkg for Transporter / App Store Connect ───────────────
        retry_signing "productbuild → $OUT_PKG" 300 \
            productbuild --component "$APP_BUNDLE" /Applications \
                --sign "$INSTALLER_IDENT" \
                "$OUT_PKG"
    else
        # SKIP_PKG=1 (CI only): stop after codesign. No .pkg, no .zip.
        # An Apple-Distribution-signed .app zipped and downloaded directly
        # cannot launch on a user's Mac — the embedded provisioning profile
        # + sandbox entitlements only validate when the package is delivered
        # via the App Store. So shipping a CI-produced .zip just creates a
        # broken download. Mac users go through the App Store badge in the
        # README / GitHub Release notes; this job exists for build smoke-
        # testing only
        log "SKIP_PKG=1: codesign passed, skipping productbuild and zip"
    fi
fi

# ── 7. Done ──────────────────────────────────────────────────────────────────
if [[ "$DRY_RUN" == "1" ]]; then
    log "DRY_RUN complete — unsigned bundle ready for local validation"
    printf '\n  %s\n\n' "$APP_BUNDLE"
    echo "to test the unsigned bundle:  open \"$APP_BUNDLE\""
    echo "(Gatekeeper will warn on first launch — right-click → Open to bypass)"
else
    log "success"
    if [[ "$SKIP_PKG" != "1" ]]; then
        printf '\n  %s\n  %s\n\n' "$APP_BUNDLE" "$OUT_PKG"
        echo "next steps:"
        echo "  1. Upload to App Store Connect via Transporter.app:"
        echo "       open -a Transporter \"$OUT_PKG\""
        echo "     (xcrun altool was removed in Xcode 15; use Transporter,"
        echo "      xcrun iTMSTransporter, or the App Store Connect REST API.)"
        echo "  2. After processing, test in real sandbox via TestFlight."
        echo "     (Do not 'sudo installer -pkg …' an MAS-signed .pkg locally —"
        echo "      macOS silently refuses to write the .app since the embedded"
        echo "      provisioning profile is only valid via the App Store /"
        echo "      TestFlight delivery path. Receipt registers anyway, making"
        echo "      it look broken when nothing is wrong.)"
    else
        printf '\n  %s\n\n' "$APP_BUNDLE"
        echo "SKIP_PKG=1 mode — build smoke test only, no .pkg or .zip."
        echo "For a real MAS submission, unset SKIP_PKG and re-run locally."
    fi
fi
