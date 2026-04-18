#
# Build + sign the Rust exhale binary into an MSIX package for Microsoft
# Store submission (and sideload testing).
#
# Produces (under `rust\target\msix\`):
#   exhale.msix          — signed MSIX (upload via Partner Center)
#
# Requirements:
#   - Windows 10 SDK (10.0.17763 or newer) — provides makeappx.exe + signtool.exe
#     Install from: https://developer.microsoft.com/windows/downloads/windows-sdk
#   - Rust toolchain with x86_64-pc-windows-msvc target
#   - (Partner-Center upload only) Publisher + Identity Name reserved at
#     https://partner.microsoft.com/dashboard/windows/overview and pasted into
#     rust\packaging\windows\AppxManifest.xml before running
#   - (Signing only) A code-signing cert in PFX form.
#
# Usage (PowerShell):
#   rust\scripts\bundle-msix.ps1                                   # default version
#   rust\scripts\bundle-msix.ps1 -Version 2.0.8 -Build 208         # override
#   rust\scripts\bundle-msix.ps1 -CertPath C:\certs\self.pfx `
#                                -CertPassword (ConvertTo-SecureString "pw" -AsPlainText -Force)
#   rust\scripts\bundle-msix.ps1 -DryRun                           # skip signing
#
# For the Microsoft Store, signing is *optional* here — Partner Center
# re-signs the submitted MSIX with the Store's own cert.  For sideload /
# CI smoke tests, pass -CertPath + -CertPassword to self-sign.
#

[CmdletBinding()]
param(
    [string] $Version         = "2.0.8",
    [string] $Build           = "208",
    [string] $CertPath        = "",
    [SecureString] $CertPassword = $null,
    [switch] $DryRun
)

$ErrorActionPreference = "Stop"

# ── Constants ────────────────────────────────────────────────────────────────
$RepoRoot   = Resolve-Path (Join-Path $PSScriptRoot "..\..")
$RustRoot   = Join-Path $RepoRoot "rust"
$PkgRoot    = Join-Path $RustRoot "packaging\windows"
$OutDir     = Join-Path $RustRoot "target\msix"
$StageDir   = Join-Path $OutDir "stage"
$OutMsix    = Join-Path $OutDir "exhale.msix"

$BundleId   = "PeterKlingelhofer.exhale"   # MUST match <Identity Name> in manifest
$Target     = "x86_64-pc-windows-msvc"

# ── Helpers ──────────────────────────────────────────────────────────────────
function Write-Log($msg)  { Write-Host "[msix] $msg" -ForegroundColor Cyan }
function Write-Fail($msg) { Write-Host "[msix] error: $msg" -ForegroundColor Red; exit 1 }

function Find-SdkTool($name) {
    # Walk every installed Windows 10/11 SDK, pick the newest bin\*\x64\$name.
    $roots = @(
        "${Env:ProgramFiles(x86)}\Windows Kits\10\bin",
        "${Env:ProgramFiles}\Windows Kits\10\bin"
    ) | Where-Object { Test-Path $_ }

    foreach ($root in $roots) {
        $candidates = Get-ChildItem $root -Directory -ErrorAction SilentlyContinue |
            Sort-Object Name -Descending
        foreach ($c in $candidates) {
            $candidate = Join-Path $c.FullName "x64\$name"
            if (Test-Path $candidate) { return $candidate }
        }
    }
    return $null
}

$MakeAppx = Find-SdkTool "makeappx.exe"
$SignTool = Find-SdkTool "signtool.exe"

if (-not $MakeAppx) { Write-Fail "makeappx.exe not found — install Windows 10 SDK" }
if (-not $DryRun -and $CertPath -and -not $SignTool) {
    Write-Fail "signtool.exe not found — install Windows 10 SDK"
}

# ── 1. Build the Rust binary (release, no-default-features for MAS parity) ──
Write-Log "cargo build --release --no-default-features --target $Target"
Push-Location $RustRoot
try {
    & rustup target add $Target | Out-Null
    & cargo build --release --no-default-features -p exhale-app --target $Target
    if ($LASTEXITCODE -ne 0) { Write-Fail "cargo build failed" }
} finally {
    Pop-Location
}

$BinPath = Join-Path $RustRoot "target\$Target\release\exhale.exe"
if (-not (Test-Path $BinPath)) { Write-Fail "binary missing: $BinPath" }

# ── 2. Stage the MSIX layout ─────────────────────────────────────────────────
Write-Log "staging MSIX payload at $StageDir"
if (Test-Path $StageDir) { Remove-Item -Recurse -Force $StageDir }
New-Item -ItemType Directory -Path $StageDir | Out-Null
New-Item -ItemType Directory -Path (Join-Path $StageDir "Assets") | Out-Null

Copy-Item $BinPath (Join-Path $StageDir "exhale.exe")
Copy-Item (Join-Path $PkgRoot "Assets\*") (Join-Path $StageDir "Assets\") -Recurse

# ── 3. Materialise the manifest (substitute version) ────────────────────────
$ManifestSrc = Join-Path $PkgRoot "AppxManifest.xml"
$ManifestDst = Join-Path $StageDir "AppxManifest.xml"

$manifestText = Get-Content $ManifestSrc -Raw
# Version in manifest is A.B.C.D where D must be 0 (Microsoft Store rule).
# Also coerce any pre-release suffix (e.g. "2.0.8-rc.1") down to the numeric
# triple MSIX requires.
$numericVersion = ($Version -split '-')[0]
$manifestText = $manifestText -replace 'Version="[^"]+"', "Version=`"$numericVersion.0`""
# PowerShell 5.1's `Set-Content -Encoding UTF8` writes a BOM, which makeappx
# rejects as "Incorrect xml declaration syntax". Emit plain UTF-8 without BOM.
[System.IO.File]::WriteAllText($ManifestDst, $manifestText, [System.Text.UTF8Encoding]::new($false))

# ── 4. Pack the MSIX ─────────────────────────────────────────────────────────
if (-not (Test-Path $OutDir)) { New-Item -ItemType Directory -Path $OutDir | Out-Null }
if (Test-Path $OutMsix) { Remove-Item -Force $OutMsix }

Write-Log "makeappx pack → $OutMsix"
& $MakeAppx pack /d $StageDir /p $OutMsix /o
if ($LASTEXITCODE -ne 0) { Write-Fail "makeappx pack failed" }

# ── 5. Sign (optional) ───────────────────────────────────────────────────────
if ($DryRun) {
    Write-Log "DryRun: skipping signtool"
} elseif ($CertPath) {
    if (-not (Test-Path $CertPath)) { Write-Fail "cert not found: $CertPath" }
    if (-not $CertPassword)        { Write-Fail "-CertPassword required with -CertPath" }

    $BSTR = [System.Runtime.InteropServices.Marshal]::SecureStringToBSTR($CertPassword)
    $plainPw = [System.Runtime.InteropServices.Marshal]::PtrToStringAuto($BSTR)
    try {
        Write-Log "signtool sign → $OutMsix"
        & $SignTool sign `
            /fd SHA256 `
            /a `
            /f $CertPath `
            /p $plainPw `
            $OutMsix
        if ($LASTEXITCODE -ne 0) { Write-Fail "signtool sign failed" }
    } finally {
        [System.Runtime.InteropServices.Marshal]::ZeroFreeBSTR($BSTR)
    }
} else {
    Write-Log "no -CertPath supplied — MSIX is unsigned"
    Write-Log "Microsoft Store will re-sign on submission; for sideload testing"
    Write-Log "pass -CertPath + -CertPassword with a self-signed PFX."
}

# ── 6. Summary ───────────────────────────────────────────────────────────────
Write-Log "success"
Write-Host ""
Write-Host "  $OutMsix"
Write-Host ""
Write-Host "next steps:"
Write-Host "  - Sideload install:  Add-AppxPackage -Path '$OutMsix'"
Write-Host "  - Store submission:  upload via Partner Center"
Write-Host "    https://partner.microsoft.com/dashboard/windows/overview"
