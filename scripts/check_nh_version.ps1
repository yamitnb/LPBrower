$nh = "$env:APPDATA\shardx-launcher\runtime\ShardX-Windows\notification_helper.exe"
if (Test-Path $nh) {
    (Get-Item $nh).VersionInfo | Format-List *
}
