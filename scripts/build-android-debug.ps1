# Builds the Rapid Pi Android debug APK.
#
# Usage: scripts\build-android-debug.ps1 [options]
#   -JavaHome PATH    JDK to build with. Default: $env:JAVA_HOME, else F:\toolchain\jdk-21
#   -AndroidHome PATH Android SDK. Default: $env:ANDROID_HOME, else F:\toolchain\android-sdk
#   -Out FILE         copy the finished APK here as well. Default: no copy
# Environment: JAVA_HOME, ANDROID_HOME (or ANDROID_SDK_ROOT)
#
# The APK lands at android\app\build\outputs\apk\debug\app-debug.apk and installs as
# in.opensec.rapidpi.debug next to a release build.
param(
	[string]$JavaHome = '',
	[string]$AndroidHome = '',
	[string]$Out = ''
)
$ErrorActionPreference = 'Stop'
$Root = (Resolve-Path (Join-Path $PSScriptRoot '..')).Path
$AndroidDir = Join-Path $Root 'android'

if (-not $JavaHome) { $JavaHome = $env:JAVA_HOME }
if (-not $JavaHome) { $JavaHome = 'F:\toolchain\jdk-21' }
if (-not (Test-Path $JavaHome)) { Die "JDK not found at $JavaHome (set -JavaHome or JAVA_HOME)" }
$env:JAVA_HOME = $JavaHome
$env:PATH = "$JavaHome\bin;$env:PATH"

if (-not $AndroidHome) { $AndroidHome = $env:ANDROID_HOME }
if (-not $AndroidHome) { $AndroidHome = $env:ANDROID_SDK_ROOT }
if (-not $AndroidHome) { $AndroidHome = 'F:\toolchain\android-sdk' }
if (-not (Test-Path $AndroidHome)) { Die "Android SDK not found at $AndroidHome (set -AndroidHome or ANDROID_HOME)" }
$env:ANDROID_HOME = $AndroidHome
$env:ANDROID_SDK_ROOT = $AndroidHome

function Log($message) { Write-Host "==> $message" -ForegroundColor Cyan }
function Die($message) { Write-Host "error: $message" -ForegroundColor Red; exit 1 }

Log "JAVA_HOME=$env:JAVA_HOME"
Log "ANDROID_HOME=$env:ANDROID_HOME"

Push-Location $AndroidDir
try {
	& ./gradlew :app:assembleDebug
	if ($LASTEXITCODE -ne 0) { Die "gradle build failed (exit $LASTEXITCODE)" }
} finally {
	Pop-Location
}

$Apk = Join-Path $AndroidDir 'app\build\outputs\apk\debug\app-debug.apk'
if (-not (Test-Path $Apk)) { Die "expected APK not produced: $Apk" }

$sizeMB = [math]::Round(((Get-Item $Apk).Length / 1MB), 1)
Log "APK: $Apk ($sizeMB MB)"

if ($Out) {
	Copy-Item $Apk -Destination $Out -Force
	$copied = Resolve-Path $Out
	Log "copied to: $copied"
}
