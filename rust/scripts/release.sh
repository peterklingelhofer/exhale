#!/usr/bin/env bash
#
# Bump exhale's version string across every place it's pinned, and
# optionally commit, tag, and push the release.
#
# Files touched:
#   rust/crates/exhale-app/Cargo.toml          version = "X.Y.Z"
#   snap/snapcraft.yaml                        version: 'X.Y.Z'
#   rust/packaging/windows/AppxManifest.xml    Version="X.Y.Z.0"
#   rust/scripts/bundle-msix.ps1               $Version / $Build
#   rust/scripts/bundle-appimage.sh            VERSION="${VERSION:-X.Y.Z}"
#   rust/scripts/bundle-mas.sh                 VERSION="${VERSION:-X.Y.Z}"
#   .github/workflows/release.yml              V="X.Y.Z" fallback
#   rust/Cargo.lock                            via `cargo update -p exhale-app`
#
# Usage:
#   rust/scripts/release.sh 2.0.16              # bump files only
#   rust/scripts/release.sh 2.0.16 --dry-run    # show diffs, no writes
#   rust/scripts/release.sh 2.0.16 --tag        # bump + commit + tag + push tag
#
# --tag mode: stages only the files above (so unrelated dirty files are safe),
# commits as "release: vX.Y.Z", pushes the current branch, creates tag
# vX.Y.Z, and pushes the tag (which triggers the release workflow).
#

set -euo pipefail

VERSION="${1:-}"
MODE="${2:-}"

if ! [[ "$VERSION" =~ ^[0-9]+\.[0-9]+\.[0-9]+$ ]]; then
    echo "usage: $(basename "$0") X.Y.Z [--dry-run|--tag]" >&2
    exit 2
fi

case "$MODE" in ""|--dry-run|--tag) ;; *)
    echo "unknown mode: $MODE (expected --dry-run or --tag)" >&2
    exit 2
;; esac

# MAS / MSIX / release.yml short build number. Apple App Store requires
# CFBundleVersion to be monotonically increasing across every upload for
# the bundle ID, including rejected/resubmitted ones. The old formula
# `${VERSION//./}` was deterministic-per-VERSION which meant every
# resubmission burned the next version's BUILD slot (shipped 2020 then
# 2021 as v2.0.20 resubmissions, then v2.0.21's default 2021 collided
# with App Store Connect rejection ID 8a9458f3-...). Using commit count
# instead gives a monotonic value that doesn't depend on VERSION, plus a
# 10000 offset to clear the historical 2020–2022 range we already burned.
# Matches the computation in release.yml + bundle-mas.sh
BUILD="$(( $(git rev-list --count HEAD) + 10000 ))"

REPO_ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
cd "$REPO_ROOT"

# sed_inplace <extended-regex> <file>
sed_inplace() {
    local re="$1" file="$2"
    if [[ "$MODE" == "--dry-run" ]]; then
        if diff -u "$file" <(sed -E "$re" "$file") >/dev/null 2>&1; then
            printf '  %-52s (no change)\n' "$file"
        else
            printf '  %-52s\n' "$file"
            { diff -u "$file" <(sed -E "$re" "$file") || true; } | sed 's/^/      /'
        fi
        return
    fi
    case "$(uname -s)" in
        Darwin) sed -i '' -E "$re" "$file" ;;
        *)      sed -i    -E "$re" "$file" ;;
    esac
}

printf 'bumping exhale → %s (build %s)\n' "$VERSION" "$BUILD"
[[ "$MODE" == "--dry-run" ]] && echo "(dry run — no files will be modified)"

sed_inplace 's/(^version[[:space:]]+=[[:space:]]+")[^"]+(")/\1'"$VERSION"'\2/' \
    rust/crates/exhale-app/Cargo.toml

sed_inplace "s/(^version: ')[^']+(')/\\1${VERSION}\\2/" \
    snap/snapcraft.yaml

# Anchor to line-start whitespace so we only match the <Identity Version="…">
# attribute, not the <TargetDeviceFamily Min/MaxVersion="…"> siblings.
sed_inplace 's/(^[[:space:]]+Version=")[0-9]+\.[0-9]+\.[0-9]+\.0(")/\1'"${VERSION}"'.0\2/' \
    rust/packaging/windows/AppxManifest.xml

sed_inplace 's/(\$Version[[:space:]]+=[[:space:]]+")[^"]+(")/\1'"$VERSION"'\2/' \
    rust/scripts/bundle-msix.ps1
sed_inplace 's/(\$Build[[:space:]]+=[[:space:]]+")[^"]+(")/\1'"$BUILD"'\2/' \
    rust/scripts/bundle-msix.ps1

sed_inplace 's/(VERSION="\$\{VERSION:-)[^}]+(\}")/\1'"$VERSION"'\2/' \
    rust/scripts/bundle-appimage.sh

sed_inplace 's/(VERSION="\$\{VERSION:-)[^}]+(\}")/\1'"$VERSION"'\2/' \
    rust/scripts/bundle-mas.sh

sed_inplace 's/(^[[:space:]]*V=")[0-9]+\.[0-9]+\.[0-9]+(")/\1'"$VERSION"'\2/' \
    .github/workflows/release.yml

if [[ "$MODE" == "--dry-run" ]]; then
    echo "[dry-run] would run: (cd rust && cargo update -p exhale-app --workspace)"
    echo "[dry-run] done"
    exit 0
fi

echo "refreshing Cargo.lock"
( cd rust && cargo update -p exhale-app --workspace )

FILES=(
    .github/workflows/release.yml
    rust/Cargo.lock
    rust/crates/exhale-app/Cargo.toml
    rust/packaging/windows/AppxManifest.xml
    rust/scripts/bundle-appimage.sh
    rust/scripts/bundle-mas.sh
    rust/scripts/bundle-msix.ps1
    snap/snapcraft.yaml
)

if [[ "$MODE" != "--tag" ]]; then
    echo
    echo "done. files modified:"
    printf '  %s\n' "${FILES[@]}"
    echo
    echo "next: git add … && git commit -m 'release: v$VERSION'"
    echo "      git tag v$VERSION && git push origin HEAD v$VERSION"
    exit 0
fi

if git rev-parse "v$VERSION" >/dev/null 2>&1; then
    echo "error: tag v$VERSION already exists — pick a new version" >&2
    exit 1
fi

echo "staging release files"
git add "${FILES[@]}"

echo "committing"
git commit -m "release: v$VERSION"

BRANCH="$(git symbolic-ref --short HEAD)"
echo "pushing $BRANCH"
git push origin "$BRANCH"

echo "tagging v$VERSION"
git tag "v$VERSION"
git push origin "v$VERSION"

echo
echo "release v$VERSION cut. watch the CI run:"
echo "  gh run watch \$(gh run list --workflow=release.yml --limit 1 --json databaseId -q '.[0].databaseId')"
