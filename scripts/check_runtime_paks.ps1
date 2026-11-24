$dir = "$env:APPDATA\shardx-launcher\runtime\ShardX-Windows"
Get-ChildItem -Path $dir -Recurse -File -Include "*.pak","*.json","*.manifest" | ForEach-Object {
    $matches = Select-String -Path $_.FullName -Pattern "ShardX|ShardBrowser|ProxyShard" -SimpleMatch
    if ($matches) {
        Write-Host "$($_.Name) has $($matches.Count) matches"
    }
}
