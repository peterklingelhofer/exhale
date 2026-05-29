# Deployment

How to ship a new exhale release to the three stores plus the GitHub Releases page. Each section is independent. Run them in any order; nothing depends on anything else cross-platform.

## At a glance

| Target | Status | One-time | Per-release |
|---|---|---|---|
| Mac App Store | live (Swift listing id6447758995, being migrated to Rust) | Apple Developer membership (have), bundle ID + certs + profile | `bundle-mas.sh` → Transporter (must run locally, see "CI caveat" below) |
| Windows Microsoft Store | listing live (Store ID `9P79Z1NJMZB3`) | Partner Center listing | `bundle-msix.ps1` → Partner Center |
| Snap Store | published, manual upload | Snapcraft developer account, `snap-creds` Multipass VM | CI builds `.snap`, `multipass exec snap-creds -- snapcraft upload` |
| Windows standalone `.exe` | direct ship | none | GitHub Release artifact from `release.yml` |
| Linux `.deb` / AppImage | direct ship | none | GitHub Release artifact from `release.yml` |
| macOS standalone | not shipped (MAS only) | n/a | n/a |

CI in [.github/workflows/release.yml](.github/workflows/release.yml) builds every artifact on a `v*` tag and attaches them to a draft GitHub Release. Store uploads still run by hand.

## Version bump

The version string is pinned in eight places (`Cargo.toml`, `snapcraft.yaml`, `AppxManifest.xml`, the three bundle scripts' default fallbacks, `release.yml`, and `Cargo.lock`). [rust/scripts/release.sh](rust/scripts/release.sh) bumps all of them in lockstep:

```sh
rust/scripts/release.sh 2.0.20 --dry-run   # show the diff, no writes
rust/scripts/release.sh 2.0.20              # bump files only, stop
rust/scripts/release.sh 2.0.20 --tag        # bump + commit + push branch + tag + push tag
```

`--tag` mode stages only the version-bearing files (unrelated dirty files stay untouched), commits as `release: vX.Y.Z`, pushes the current branch, then creates and pushes the tag, which fires [release.yml](.github/workflows/release.yml) and produces every artifact below as a downloadable CI artifact and as a draft GitHub Release. You still upload to the three stores yourself.

For a store-only deploy (e.g. shipping MAS first and tagging later), use the no-flag form so you can stage the submission, then run with `--tag` once you're ready for the full multi-platform cut.

---

## macOS — Mac App Store

You already have a paid Apple Developer Program membership and the App Store Connect record (id6447758995, originally created for the Swift build). The Rust port reuses the same listing, so most of the "one-time setup" was done years ago. The checklist below is the bits you'll touch when the certs roll over or when bringing up a fresh machine.

### One-time setup (per dev machine)

1. **Bundle ID** at https://developer.apple.com → Certificates, Identifiers & Profiles → Identifiers. The ID `peterklingelhofer.exhale` already exists from the Swift app and is what [rust/scripts/bundle-mas.sh](rust/scripts/bundle-mas.sh#L51) embeds. Capabilities: only **App Sandbox**.
2. **Certificates** in the same portal:
   - **Apple Distribution** (single cert covers iOS + macOS, replaces the legacy "3rd Party Mac Developer Application")
   - **Mac Installer Distribution** (a.k.a. "3rd Party Mac Developer Installer")

   Download each `.cer`, double-click to install into the login keychain.
3. **Provisioning profile** → Profiles → "Mac App Store" → pick `peterklingelhofer.exhale` + the Apple Distribution cert. Save as [rust/signing/exhale.provisionprofile](rust/signing/exhale.provisionprofile) (already in place on this machine).
4. **Verify**:

   ```sh
   security find-identity -v -p basic
   ```

   Two lines must appear for team `VZCHHV7VNW`:

   ```
   Apple Distribution: Peter Klingelhofer (VZCHHV7VNW)
   3rd Party Mac Developer Installer: Peter Klingelhofer (VZCHHV7VNW)
   ```

### Build, sign, package

Everything below is one script:

```sh
rust/scripts/bundle-mas.sh
# or with explicit version:
VERSION=2.0.20 BUILD=2020 rust/scripts/bundle-mas.sh
```

What it does (read the source for line-by-line; [bundle-mas.sh](rust/scripts/bundle-mas.sh)):

1. Builds the Rust binary `--release --no-default-features` for both `aarch64-apple-darwin` and `x86_64-apple-darwin`, `lipo`'d into a universal binary. The `--no-default-features` build drops the global-hotkey crate since the Carbon hotkey API is sandbox-prohibited
2. Generates `AppIcon.icns` from [swift/exhale/Assets.xcassets/AppIcon.appiconset/exhaleColorGradient1024.png](swift/exhale/Assets.xcassets/AppIcon.appiconset/exhaleColorGradient1024.png) (the canonical 1024 master shared with the Swift project)
3. Assembles `exhale.app` with `Info.plist` (LSUIElement, category `healthcare-fitness`), entitlements (`app-sandbox` + `files.user-selected.read-only`), and the embedded provisioning profile
4. Signs the `.app` with the Apple Distribution identity
5. Wraps in `exhale.pkg` signed with the 3rd Party Mac Developer Installer identity

Output:

```
rust/target/mas/exhale.app
rust/target/mas/exhale.pkg
```

### Pre-flight checks (local)

```sh
# Signature attached cleanly?
codesign --verify --deep --strict --verbose=2 rust/target/mas/exhale.app

# Entitlements embedded?
codesign -d --entitlements - rust/target/mas/exhale.app
```

**Do NOT try `sudo installer -pkg … -target /` on an MAS-signed `.pkg`.** The 3rd-Party-Mac-Developer-Installer signature + embedded provisioning profile only validate when the package is delivered through the Mac App Store / TestFlight pipeline. `installer(8)` claims success and writes a `pkgutil` receipt, but macOS silently refuses to drop the `.app` into `/Applications`, so the install looks broken when nothing is actually wrong. For sandbox / runtime verification, run the unsigned `cargo run --no-default-features` build directly (no sandbox), or wait for TestFlight after Transporter upload (real sandbox + real delivery).

If a TestFlight install crashes immediately but `cargo run` was fine, the sandbox is biting. Look in `~/Library/Logs/DiagnosticReports/exhale*.crash` for `deny(1) file-read-data` or similar.

### Upload to App Store Connect

**Recommended:** Transporter (free, from the Mac App Store).

```sh
open -a Transporter rust/target/mas/exhale.pkg
```

Sign in with the Apple ID on the developer account, click **Deliver**. Apple validates the signature, sandbox, icon set, and entitlements server-side; you get an email within ~15 minutes with either "Processed by App Store Connect" or a list of validation failures.

**Scripted alternative:** `xcrun iTMSTransporter`. The older `xcrun altool --upload-app` was removed in Xcode 15, so it is no longer an option. `xcrun notarytool` is for non-Store notarization and is not used for MAS submissions. If you want CI to upload automatically, plumb iTMSTransporter (or the App Store Connect REST API with a JWT) in a new step on the `macos` job in [release.yml](.github/workflows/release.yml).

### TestFlight (optional, recommended)

Once the processed build appears in App Store Connect → Builds:

1. App Store Connect → exhale → TestFlight → enable for **Internal Testing** (your team)
2. Install via the TestFlight app on macOS for a sandboxed real-install QA pass

External testing requires Beta App Review (~1 day) the first time. For a low-risk utility like exhale, internal-only is usually enough.

### Submit for review

App Store Connect → exhale → macOS App → Prepare for Submission:

1. Pick the uploaded build
2. Fill in **What's New in This Version**
3. Confirm pricing (free), age rating, availability
4. Add for Review → Submit to App Review

Typical SLA is 24–48 hours. First-time submissions can take 1–3 days.

Common rejection reasons for exhale specifically:
- **App Store Connect agreements unsigned.** First-time-each-year hurdle. Check Agreements, Tax, and Banking before submitting
- **Reviewer "can't find the UI".** Add a note in App Review Information: "App runs in the menu bar; click the ring icon for Preferences."
- **Sandbox violations.** Almost always a new entitlement we added without updating [bundle-mas.sh](rust/scripts/bundle-mas.sh#L196-L207)

### Update cycle

For every subsequent release:

1. Bump `version` in `rust/crates/exhale-app/Cargo.toml` (or run `rust/scripts/release.sh X.Y.Z`)
2. `VERSION=… rust/scripts/bundle-mas.sh` **locally** (see CI caveat)
3. Upload via Transporter
4. App Store Connect → new version → pick the build → release notes → submit

No new certs / profile unless entitlements change.

### CI caveat: MAS `.pkg` is built locally only

`productbuild --sign` deterministically hangs on the `macos-latest` GitHub Actions runners even with every Apple WWDR intermediate imported into the temp keychain. Suspected root cause is an unreachable OCSP / CRL endpoint during installer-cert chain validation, but root-causing it further hasn't been worth the time given there's a clean workaround.

CI sets `SKIP_PKG=1` in [release.yml](.github/workflows/release.yml) so the macOS job emits a signed `.app.zip` for the GitHub Release page (sideload-friendly) instead of running `productbuild`. The actual `.pkg` for MAS submission is built locally by running `rust/scripts/bundle-mas.sh` without `SKIP_PKG` and uploaded via Transporter, as documented above. If you ever need to revisit the CI `.pkg` path, the investigation starting point is whether `productbuild` is blocked on `ocsp.apple.com` — `sample $(pgrep productbuild)` inside a 60-second timeout would confirm or rule it out.

---

## Windows — Microsoft Store

The Partner Center listing already exists (Store ID `9P79Z1NJMZB3`, Package Family `PeterKlingelhofer.exhale_rrj7wxvvetjy2`). The identity values are baked into [rust/packaging/windows/AppxManifest.xml](rust/packaging/windows/AppxManifest.xml#L25-L28). You don't need to reserve a new identity for updates.

### Build the MSIX

From a Windows machine (or the `windows` job in `release.yml`):

```powershell
rust\scripts\bundle-msix.ps1
# explicit version:
rust\scripts\bundle-msix.ps1 -Version 2.0.20 -Build 2020
```

Output: `rust\target\msix\exhale.msix`.

The script ([bundle-msix.ps1](rust/scripts/bundle-msix.ps1)) builds the binary `--release --no-default-features` for `x86_64-pc-windows-msvc`, stages the MSIX layout (binary + assets + manifest), patches the version into the manifest's `<Identity>` element, and packs with `makeappx.exe` from the Windows 10 SDK.

### Code signing

For Microsoft Store submissions, code signing is **optional** because Partner Center re-signs the submitted MSIX with the Store's own certificate. The script's `-CertPath` / `-CertPassword` flags are only needed for:

- Sideload installation (`Add-AppxPackage -Path …` without going through the Store)
- CI smoke tests

To self-sign for sideload testing:

```powershell
rust\scripts\bundle-msix.ps1 `
    -CertPath C:\certs\self.pfx `
    -CertPassword (ConvertTo-SecureString "pw" -AsPlainText -Force)
```

### Upload to Partner Center

1. https://partner.microsoft.com/dashboard/windows/overview → exhale → Packages
2. Drag `exhale.msix` into the package upload zone
3. Validate (Partner Center checks signature, manifest, asset sizes server-side)
4. Submit for certification

Microsoft cert review is usually faster than Apple: most updates clear in 6–12 hours.

### Standalone `.exe`

`cargo build --release -p exhale-app` from the `rust/` dir produces a fully self-contained `target/release/exhale.exe`. This is what the GitHub Release attaches (no MSIX wrapping). Users get a "publisher unknown" SmartScreen warning since the standalone exe isn't code-signed; the warning is bypassable via "More info → Run anyway" but is the cost of not buying a Windows code-signing cert (~$200–400/year). The Store MSIX is the no-warning install path.

---

## Linux — Snap Store

The snap is published as `exhale-app` on https://snapcraft.io.

### Why upload is manual

Every snapcraft auth path tried in CI (snap @ latest/7.x/8.x, the `snapcore/action-publish` action, `pip install snapcraft`, direct REST API calls) hit the same `018h` byte in the discharge macaroon and crashed. Until snapcraft's auth story works in headless CI again, CI builds the `.snap` artifact and upload happens from a Multipass VM with the credentials pre-stored. See the long comment block above the `linux-snap` job in [release.yml](.github/workflows/release.yml#L181-L197) for the exact failure mode.

### One-time setup

On the dev Mac:

```sh
brew install multipass
multipass launch --name snap-creds --memory 2G --disk 5G 22.04
multipass shell snap-creds
# inside the VM:
sudo snap install snapcraft --classic
snapcraft login   # browser flow, paste the URL back
```

The login cookie persists across `multipass stop` / `start`, so this is a one-time step.

### Per-release upload

CI produces `exhale-app_<VERSION>_amd64.snap` as the `linux-snap-<VERSION>` artifact on every `v*` tag. To ship it:

```sh
# 1. Download the snap from the CI run
gh run download <run-id> -n linux-snap-2.0.20

# 2. Push it into the VM
multipass transfer exhale-app_2.0.20_amd64.snap snap-creds:/tmp/

# 3. Upload to the Snap Store (edge channel)
multipass exec snap-creds -- snapcraft upload \
    --release=edge /tmp/exhale-app_2.0.20_amd64.snap
```

Then promote `edge → stable` from https://snapcraft.io/exhale-app/releases once you've smoke-tested edge on a Linux box.

### Local snap build (no CI)

To build a `.snap` locally for testing, use the same commands the CI runs:

```sh
sudo snap install snapcraft --classic
sudo snap install core22 gnome-42-2204 gtk-common-themes
sudo snapcraft pack --destructive-mode --verbose
```

`--destructive-mode` is required because GitHub Actions kernels block `/sys/kernel/mm/page_idle/bitmap` inside LXD, and snapcraft's default managed mode needs it. The same flag is what the `linux-snap` CI job uses.

---

## Linux — `.deb` and AppImage

These ship directly on the GitHub Releases page (no store flow). CI builds them automatically via the `linux-direct` job. To produce them locally:

```sh
# .deb
cd rust
cargo install cargo-deb --locked
cargo build --release --no-default-features -p exhale-app
cargo deb --no-build -p exhale-app
# → rust/target/debian/exhale-app_<VERSION>_amd64.deb

# AppImage (Linux only, appimagetool is x86_64-Linux)
rust/scripts/bundle-appimage.sh
# → rust/target/appimage/exhale-<VERSION>-x86_64.AppImage
```

On macOS, the AppImage step has to run via CI or a Linux VM. The `linux-direct` CI job handles both artifacts on every `v*` tag.

---

## CI secrets

[release.yml](.github/workflows/release.yml) gracefully degrades when secrets are missing (each platform job falls back to an unsigned dry-run build), but for a signed release configure these in repo Settings → Secrets:

| Secret | Used by | What it is |
|---|---|---|
| `MACOS_APP_IDENT_P12` | macOS | base64 of the Apple Distribution `.p12` |
| `MACOS_APP_IDENT_PASSWORD` | macOS | password for the above |
| `MACOS_INSTALLER_P12` | macOS | base64 of the 3rd Party Mac Developer Installer `.p12` |
| `MACOS_INSTALLER_PASSWORD` | macOS | password for the above |
| `MACOS_PROVISION_PROFILE` | macOS | base64 of `exhale.provisionprofile` |
| `MACOS_KEYCHAIN_PASSWORD` | macOS | any string, used for the temporary CI keychain |
| `WINDOWS_CERT_PFX` | Windows | base64 of a code-signing PFX (optional, Store re-signs) |
| `WINDOWS_CERT_PASSWORD` | Windows | password for the PFX |

Export a local `.p12` to base64 with `base64 -i cert.p12 -o cert.p12.b64` then paste into the secret.

---

## What is not automated

Per release, after CI is green:

- Transporter upload + Submit for Review on App Store Connect
- Partner Center MSIX upload + Submit on Microsoft
- `multipass exec snap-creds -- snapcraft upload` for the snap

Per cert rotation (~yearly):

- Re-download Apple Distribution + Mac Installer Distribution certs, re-import to keychain
- Re-download `exhale.provisionprofile` into `rust/signing/`
- Re-base64 the new `.p12`s into the GitHub repo secrets
