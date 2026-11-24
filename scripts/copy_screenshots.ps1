$ErrorActionPreference = "Stop"

$base = "C:\Users\Admin\.gemini\antigravity\brain\e2440e97-671e-40c6-be08-fdb4066b123f\browser_sandbox\screenshots"
$dest = "C:\Users\Admin\.gemini\antigravity\brain\6659144a-f4cf-41d2-a473-a1d186c1fdb7"

Copy-Item "$base\step_134_shot_1_lp_dashboard_main_brand.png" "$dest\lp_ux_dashboard.png" -Force
Copy-Item "$base\step_138_shot_1_lp_modal_basics_balanced.png" "$dest\lp_ux_modal_basics.png" -Force
Copy-Item "$base\step_138_shot_2_lp_modal_fingerprint_balanced.png" "$dest\lp_ux_modal_fingerprint.png" -Force
Copy-Item "$base\step_142_shot_2_lp_proxy_page.png" "$dest\lp_ux_proxy_page.png" -Force
Copy-Item "$base\step_142_shot_1_lp_fingerprints_page.png" "$dest\lp_ux_fingerprints_page.png" -Force

Get-Item "$dest\lp_ux_*.png" | Select-Object Name, Length
