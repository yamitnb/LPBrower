$ErrorActionPreference = "Stop"

$cmakePath = "C:\Program Files (x86)\Microsoft Visual Studio\2022\BuildTools\Common7\IDE\CommonExtensions\Microsoft\CMake\CMake\bin"
$nasmPath = "C:\Program Files\NASM"
$llvmBin = "C:\Program Files\LLVM\bin"

$env:LIBCLANG_PATH = $llvmBin
$env:Path = "$llvmBin;$nasmPath;$cmakePath;" + $env:Path

Write-Host "CMake path: $(Get-Command cmake | Select-Object -ExpandProperty Source)"
Write-Host "NASM path: $(Get-Command nasm | Select-Object -ExpandProperty Source)"

# Ensure browser engine is branded with LP icon and LPBrower metadata
if (Get-Command python -ErrorAction SilentlyContinue) {
    Write-Host "Applying LPBrower branding to browser engine..."
    python scripts/patch_browser_exe.py
    python scripts/patch_nh_icon.py
    python scripts/patch_version_info.py
    python scripts/brand_browser_engine.py
}

# Stop any running LPBrower instances before building/copying
Get-Process "LPBrower*" -ErrorAction SilentlyContinue | Stop-Process -Force -ErrorAction SilentlyContinue
Start-Sleep -Milliseconds 500

npx tauri build

# Copy release files to dist-desktop
$distDesktop = "g:\Vibe Tool\LPBrower\dist-desktop"
if (!(Test-Path $distDesktop)) {
    New-Item -ItemType Directory -Path $distDesktop -Force
}

$targetRelease = "g:\Vibe Tool\LPBrower\src-tauri\target\release"
if (Test-Path "$targetRelease\shardbrowser-launcher.exe") {
    Copy-Item "$targetRelease\shardbrowser-launcher.exe" "$distDesktop\LPBrower.exe" -Force
    Write-Host "Copied LPBrower.exe to dist-desktop"
} elseif (Test-Path "$targetRelease\LPBrower Launcher.exe") {
    Copy-Item "$targetRelease\LPBrower Launcher.exe" "$distDesktop\LPBrower.exe" -Force
    Write-Host "Copied LPBrower.exe to dist-desktop"
}

$bundleNsis = "$targetRelease\bundle\nsis"
if (Test-Path $bundleNsis) {
    $setupExe = Get-ChildItem -Path $bundleNsis -Filter "*.exe" | Select-Object -First 1
    if ($setupExe) {
        Copy-Item $setupExe.FullName "$distDesktop\LPBrower_Setup_x64.exe" -Force
        Write-Host "Copied $($setupExe.Name) to dist-desktop\LPBrower_Setup_x64.exe"
    }
}

$bundleMsi = "$targetRelease\bundle\msi"
if (Test-Path $bundleMsi) {
    $setupMsi = Get-ChildItem -Path $bundleMsi -Filter "*.msi" | Select-Object -First 1
    if ($setupMsi) {
        Copy-Item $setupMsi.FullName "$distDesktop\LPBrower_Setup_x64.msi" -Force
        Write-Host "Copied $($setupMsi.Name) to dist-desktop\LPBrower_Setup_x64.msi"
    }
}

Write-Host "Build and release packaging complete!"
