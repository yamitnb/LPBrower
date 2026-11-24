Add-Type -AssemblyName System.Drawing
$exe = "$env:APPDATA\shardx-launcher\runtime\ShardX-Windows\chrome.exe"
$icon = [System.Drawing.Icon]::ExtractAssociatedIcon($exe)
if ($icon) {
    $bmp = $icon.ToBitmap()
    $out = "C:\Users\Admin\.gemini\antigravity\brain\6659144a-f4cf-41d2-a473-a1d186c1fdb7\extracted_chrome_icon.png"
    $bmp.Save($out, [System.Drawing.Imaging.ImageFormat]::Png)
    Write-Host "Saved extracted icon to $out"
}
