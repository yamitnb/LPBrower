$dll = "$env:APPDATA\shardx-launcher\runtime\ShardX-Windows\chrome.dll"
if (Test-Path $dll) {
    (Get-Item $dll).VersionInfo | Format-List *
}
