# Build the Windows installer into dist\ (requires Inno Setup 6 and a release build from build-release.ps1 / cargo build --release).
param([string]$Version = "0.1.1")
$ErrorActionPreference = "Stop"
$root = Split-Path $PSScriptRoot -Parent
$iscc = "${env:ProgramFiles(x86)}\Inno Setup 6\ISCC.exe"
if (-not (Test-Path $iscc)) { throw "Inno Setup 6 not found" }
& $iscc "/DMyAppVersion=$Version" "$root\installer\open-backup.iss"
