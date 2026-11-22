[CmdletBinding()]
param(
    [Parameter(Mandatory)][string]$FdkSource,
    [string]$SdkDirectory,
    [string]$Generator = 'Visual Studio 17 2022'
)

$ErrorActionPreference = 'Stop'
$root = $PSScriptRoot
$build = Join-Path $root 'windows-build'
$app = Join-Path $build 'app'
$target = 'x86_64-pc-windows-msvc'
$fdk = (Resolve-Path -LiteralPath $FdkSource).Path

function Invoke-Tool {
    param([string]$Program, [string[]]$Arguments)
    & $Program @Arguments
    if ($LASTEXITCODE -ne 0) {
        throw "$Program exited with code $LASTEXITCODE"
    }
}

if (!(Test-Path -LiteralPath (Join-Path $fdk 'CMakeLists.txt') -PathType Leaf)) {
    throw 'FdkSource must point to the FDK AAC source tree (v2.0.3 or compatible).'
}
if ($SdkDirectory) {
    $sdk = (Resolve-Path -LiteralPath $SdkDirectory).Path
    foreach ($file in 'htra_api.h', 'htra_api_pnm.h', 'htra_api.lib', 'htra_api.dll') {
        if (!(Test-Path -LiteralPath (Join-Path $sdk $file) -PathType Leaf)) {
            throw "SdkDirectory must point to SA_Windows_API\x64\htra_api (missing $file)."
        }
    }
}

New-Item -ItemType Directory -Path $app -Force | Out-Null
$fdkBuild = Join-Path $build 'fdk-build'
$fdkInstall = Join-Path $build 'fdk-install'
$rustBuild = Join-Path $build 'cargo'
Invoke-Tool 'cmake' @('-S', $fdk, '-B', $fdkBuild, '-G', $Generator, '-A', 'x64', '-DBUILD_SHARED_LIBS=OFF')
Invoke-Tool 'cmake' @('--build', $fdkBuild, '--config', 'Release', '--target', 'fdk-aac')
Invoke-Tool 'cmake' @('--install', $fdkBuild, '--config', 'Release', '--prefix', $fdkInstall)

$previousFdk = $env:WEBSA_FDK_AAC_LIB_DIR
try {
    $env:WEBSA_FDK_AAC_LIB_DIR = Join-Path $fdkInstall 'lib'
    if (!(Test-Path -LiteralPath (Join-Path $env:WEBSA_FDK_AAC_LIB_DIR 'fdk-aac.lib') -PathType Leaf)) {
        throw 'FDK AAC installation is missing fdk-aac.lib.'
    }
    Invoke-Tool 'cargo' @('build', '--locked', '--release', '--target', $target, '--bin', 'websad',
        '--manifest-path', (Join-Path $root 'backend/Cargo.toml'), '--target-dir', $rustBuild, '-j', '1')
} finally {
    $env:WEBSA_FDK_AAC_LIB_DIR = $previousFdk
}

Push-Location (Join-Path $root 'frontend')
try {
    Invoke-Tool 'npm.cmd' @('ci', '--prefer-offline', '--no-audit')
    Invoke-Tool 'npm.cmd' @('run', 'build')
} finally {
    Pop-Location
}

$frontend = Join-Path $app 'frontend'
$dist = Join-Path $frontend 'dist'
New-Item -ItemType Directory -Path $frontend -Force | Out-Null
if (Test-Path -LiteralPath $dist) {
    Remove-Item -LiteralPath $dist -Recurse -Force
}
Copy-Item -LiteralPath (Join-Path $root 'frontend/dist') -Destination $dist -Recurse
Copy-Item -LiteralPath (Join-Path $rustBuild "$target/release/websad.exe") -Destination $app -Force
Copy-Item -LiteralPath (Join-Path $root 'Start-WebSA.bat') -Destination $app -Force

if ($SdkDirectory) {
    $shimBuild = Join-Path $build 'shim'
    Invoke-Tool 'cmake' @('-S', (Join-Path $root 'backend/sdk'), '-B', $shimBuild, '-G', $Generator,
        '-A', 'x64', "-DHTRA_SDK_DIR=$sdk")
    Invoke-Tool 'cmake' @('--build', $shimBuild, '--config', 'Release')
    Invoke-Tool 'cmake' @('--install', $shimBuild, '--config', 'Release', '--prefix', (Join-Path $app 'htra_api'))
}

Write-Host "WebSA for Windows is available in $app"
