# Build the Android app: the Rust engine for arm64 and x86_64, then the APK.
# Needs: Rust + cargo-ndk + the android targets, Android SDK + NDK, JDK 17, Gradle 8.7+ (or a wrapper).
# Usage: powershell -ExecutionPolicy Bypass -File android\build-android.ps1 [-Gradle <path to gradle.bat>]
param([string]$Gradle = "gradle")
$ErrorActionPreference = "Stop"
$root = Split-Path $PSScriptRoot -Parent
$sdk = if ($env:ANDROID_HOME) { $env:ANDROID_HOME } else { "$env:LOCALAPPDATA\Android\Sdk" }
if (-not $env:ANDROID_NDK_HOME) {
  $ndk = Get-ChildItem "$sdk\ndk" -Directory | Sort-Object Name -Descending | Select-Object -First 1
  $env:ANDROID_NDK_HOME = $ndk.FullName
}
$env:ANDROID_HOME = $sdk
$env:Path = "$env:USERPROFILE\.cargo\bin;$env:Path"

Push-Location $root
cargo ndk -t arm64-v8a -t x86_64 build -p open-backup --release
Pop-Location

# The engine is an executable shipped as lib*.so and run from the app's native library directory.
$jni = Join-Path $PSScriptRoot "app\src\main\jniLibs"
foreach ($p in @(@("arm64-v8a", "aarch64-linux-android"), @("x86_64", "x86_64-linux-android"))) {
  New-Item -ItemType Directory -Force "$jni\$($p[0])" | Out-Null
  Copy-Item "$root\target\$($p[1])\release\open-backup" "$jni\$($p[0])\libopen_backup.so" -Force
}

"sdk.dir=" + ($sdk -replace '\\', '\\') | Set-Content (Join-Path $PSScriptRoot "local.properties") -Encoding ascii
Push-Location $PSScriptRoot
& $Gradle --no-daemon assembleDebug
Pop-Location
Write-Host "APK: $PSScriptRoot\app\build\outputs\apk\debug\app-debug.apk"
