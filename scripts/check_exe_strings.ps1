$exe = "$env:APPDATA\shardx-launcher\runtime\ShardX-Windows\chrome.exe"
$bytes = [System.IO.File]::ReadAllBytes($exe)
$utf16 = [System.Text.Encoding]::Unicode.GetString($bytes)
$ascii = [System.Text.Encoding]::ASCII.GetString($bytes)

Write-Host "UTF16 Shard:"
[regex]::Matches($utf16, '[^\x00-\x1F]{0,30}Shard[^\x00-\x1F]{0,30}') | ForEach-Object { $_.Value } | Select-Object -Unique

Write-Host "ASCII Shard:"
[regex]::Matches($ascii, '[^\x00-\x1F]{0,30}Shard[^\x00-\x1F]{0,30}') | ForEach-Object { $_.Value } | Select-Object -Unique
