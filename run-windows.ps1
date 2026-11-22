[CmdletBinding()]
param(
    [switch]$Fake,
    [string]$SdkDirectory,
    [ValidateRange(1, 65535)][int]$Port = 8080,
    [string]$BindAddress = '127.0.0.1'
)

$ErrorActionPreference = 'Stop'
$app = $PSScriptRoot
if (!(Test-Path -LiteralPath (Join-Path $app 'websad.exe') -PathType Leaf)) {
    $app = Join-Path $app 'windows-build/app'
}
$server = Join-Path $app 'websad.exe'
$frontend = Join-Path $app 'frontend'
if (!(Test-Path -LiteralPath $server -PathType Leaf) -or
    !(Test-Path -LiteralPath (Join-Path $frontend 'dist/index.html') -PathType Leaf)) {
    throw 'Windows service or frontend assets are missing. Build the app first.'
}

$shim = $null
if (!$Fake) {
    $runtime = Join-Path $app 'htra_api'
    $shim = Join-Path $runtime 'websa_san90_sdk_session.dll'
    if (!(Test-Path -LiteralPath $shim -PathType Leaf)) {
        throw 'The Windows SDK shim is missing. Run build-windows.ps1 with -SdkDirectory.'
    }
    if ($SdkDirectory) {
        $sdk = (Resolve-Path -LiteralPath $SdkDirectory).Path
        if (!(Test-Path -LiteralPath (Join-Path $sdk 'htra_api.dll') -PathType Leaf)) {
            throw 'SdkDirectory must point to the extracted SA_Windows_API\x64\htra_api directory.'
        }
        if ($sdk -ne (Resolve-Path -LiteralPath $runtime).Path) {
            Get-ChildItem -LiteralPath $sdk -File -Filter '*.dll' | ForEach-Object {
                Copy-Item -LiteralPath $_.FullName -Destination $runtime -Force
            }
            $cal = Join-Path $sdk 'CalFile'
            if (Test-Path -LiteralPath $cal -PathType Container) {
                Copy-Item -LiteralPath $cal -Destination $runtime -Recurse -Force
            }
        }
    }
    if (!(Test-Path -LiteralPath (Join-Path $runtime 'htra_api.dll') -PathType Leaf)) {
        throw 'HAROGIC htra_api.dll is missing. Pass -SdkDirectory with the extracted Windows x64 SDK.'
    }
}

$previousStatic = $env:WEBSA_STATIC
$previousHost = $env:WEBSA_HOST
$previousPort = $env:WEBSA_PORT
$previousShim = $env:WEBSA_RUST_IQS_SHIM
try {
    $env:WEBSA_STATIC = $frontend
    $env:WEBSA_HOST = $BindAddress
    $env:WEBSA_PORT = [string]$Port
    $env:WEBSA_RUST_IQS_SHIM = $shim
    $urlHost = if ($BindAddress.Contains(':')) { "[$BindAddress]" } else { $BindAddress }
    Write-Host "Open http://${urlHost}:${Port}/ in your browser. Press Ctrl+C to stop."
    & $server
    $code = $LASTEXITCODE
    if ($code -ne 0) {
        throw "websad exited with code $code"
    }
} finally {
    $env:WEBSA_STATIC = $previousStatic
    $env:WEBSA_HOST = $previousHost
    $env:WEBSA_PORT = $previousPort
    $env:WEBSA_RUST_IQS_SHIM = $previousShim
}
