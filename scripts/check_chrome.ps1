$path = "$env:APPDATA\shardx-launcher\runtime\ShardX-Windows\chrome.exe"
if (Test-Path $path) {
    $bytes = [System.IO.File]::ReadAllBytes($path)
    $ascii = [System.Text.Encoding]::ASCII.GetString($bytes)
    $unicode = [System.Text.Encoding]::Unicode.GetString($bytes)
    Write-Host "ASCII matches:"
    [regex]::Matches($ascii, 'Shard[A-Za-z0-9_]*') | ForEach-Object { $_.Value } | Select-Object -Unique -First 20
    Write-Host "Unicode matches:"
    [regex]::Matches($unicode, 'Shard[A-Za-z0-9_]*') | ForEach-Object { $_.Value } | Select-Object -Unique -First 20
}
