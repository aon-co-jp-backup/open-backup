# Build the release assets (Windows exe, Linux binary via WSL, Android debug APK) into dist\ with SHA256SUMS.txt.
# Local paths (user name, cargo registry, repository path) are remapped so they are not embedded in the binaries.
# Usage: powershell -ExecutionPolicy Bypass -File scripts\build-release.ps1 -Version 0.1.0 [-Gradle <path to gradle.bat>] [-Wsl Ubuntu]
param([string]$Version = "0.1.0", [string]$Gradle = "gradle", [string]$Wsl = "Ubuntu")
$ErrorActionPreference = "Stop"
$root = Split-Path $PSScriptRoot -Parent
$dist = Join-Path $root "dist"
New-Item -ItemType Directory -Force $dist | Out-Null
$env:Path = "$env:USERPROFILE\.cargo\bin;$env:Path"

function Remap($pairs) { ($pairs | ForEach-Object { "--remap-path-prefix=$($_[0])=$($_[1])" }) -join " " }

# Windows
$env:RUSTFLAGS = Remap @(@("$env:USERPROFILE\.cargo", "/cargo"), @("$env:USERPROFILE\.rustup", "/rustup"), @($root, "."))
Push-Location $root
cargo build --release -p open-backup
Pop-Location
Copy-Item "$root\target\release\open-backup.exe" "$dist\open-backup-$Version-windows-x86_64.exe" -Force

# Android (the engine inside the APK is built with the same remapping; build-android.ps1 uses RUSTFLAGS)
powershell -NoProfile -ExecutionPolicy Bypass -File "$root\android\build-android.ps1" -Gradle $Gradle
Copy-Item "$root\android\app\build\outputs\apk\debug\app-debug.apk" "$dist\open-backup-$Version-android-debug.apk" -Force

# Linux (built inside WSL)
$wslRoot = (wsl -d $Wsl -- wslpath -a ($root -replace '\\', '/')).Trim()
$wslHome = (wsl -d $Wsl -- bash -lc 'echo $HOME').Trim()
$flags = "--remap-path-prefix=$wslHome/.cargo=/cargo --remap-path-prefix=$wslHome/.rustup=/rustup --remap-path-prefix=$wslRoot=."
$wslDist = (wsl -d $Wsl -- wslpath -a ($dist -replace '\\', '/')).Trim()
# Single-quoted bash code, so PowerShell does not expand $HOME (it would become the Windows home).
$build = 'cd "__ROOT__" && RUSTFLAGS="__FLAGS__" CARGO_TARGET_DIR="$HOME/ob-target-release" cargo build --release -p open-backup'
$build = $build.Replace('__ROOT__', $wslRoot).Replace('__FLAGS__', $flags)
wsl -d $Wsl -- bash -lc $build
$copy = 'cp "$HOME/ob-target-release/release/open-backup" "__DIST__/open-backup-__VER__-linux-x86_64"'
$copy = $copy.Replace('__DIST__', $wslDist).Replace('__VER__', $Version)
wsl -d $Wsl -- bash -lc $copy

# Checksums
$lines = Get-ChildItem $dist -File | Where-Object { $_.Name -ne "SHA256SUMS.txt" } | ForEach-Object { (Get-FileHash $_.FullName -Algorithm SHA256).Hash.ToLower() + "  " + $_.Name }
[IO.File]::WriteAllText("$dist\SHA256SUMS.txt", (($lines -join "`n") + "`n"), (New-Object Text.UTF8Encoding($false)))
Get-ChildItem $dist | Select-Object Name, Length
