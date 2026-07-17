# MarkFly release build script (Windows)
$ErrorActionPreference = 'Stop'

$projectRoot = Split-Path -Parent $PSScriptRoot
Set-Location $projectRoot

# Keep PATH short: only what this build needs
$nodeBin = Join-Path $projectRoot 'node_modules\.bin'
$cargoBin = Join-Path $env:USERPROFILE '.cargo\bin'
$nodeExe = Get-Command node -ErrorAction SilentlyContinue
$nodejsBin = if ($nodeExe) { Split-Path $nodeExe.Source -Parent } else { Join-Path ${env:ProgramFiles} 'nodejs' }
$env:Path = "$nodeBin;$cargoBin;$nodejsBin;C:\Windows\system32;C:\Windows"

Write-Host "==> Building frontend..."
npm run build

Write-Host "==> Building Tauri app (NSIS)..."
$vswhere = "${env:ProgramFiles(x86)}\Microsoft Visual Studio\Installer\vswhere.exe"
if (Test-Path $vswhere) {
  $vsPath = & $vswhere -latest -products * -requires Microsoft.VisualStudio.Component.VC.Tools.x86.x64 -property installationPath
  if ($vsPath) {
    Import-Module "$vsPath\Common7\Tools\Microsoft.VisualStudio.DevShell.dll"
    Enter-VsDevShell -VsInstallPath $vsPath -SkipAutomaticLocation -DevCmdArguments '-arch=x64' | Out-Null
  }
}

npx tauri build --bundles nsis --features custom-protocol

Write-Host ""
Write-Host "Done:" (Join-Path $projectRoot 'src-tauri\target\release\bundle\nsis\MarkFly_0.1.1_x64-setup.exe')
