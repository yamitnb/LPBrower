$ErrorActionPreference = "Stop"

$srcRelease = "G:\Vibe Tool\LPBrower\src-tauri\target\release"
$destDir = "G:\Vibe Tool\LPBrower\dist-desktop"

if (-not (Test-Path $destDir)) {
    New-Item -ItemType Directory -Path $destDir | Out-Null
}

Copy-Item "$srcRelease\shardbrowser-launcher.exe" "$destDir\LPBrower.exe" -Force
Copy-Item "$srcRelease\bundle\nsis\LPBrower Launcher_2.1.5_x64-setup.exe" "$destDir\LPBrower_Setup_x64.exe" -Force
Copy-Item "$srcRelease\bundle\msi\LPBrower Launcher_2.1.5_x64_en-US.msi" "$destDir\LPBrower_Setup_x64.msi" -Force

Get-Item "$destDir\*" | Select-Object Name, Length, LastWriteTime
