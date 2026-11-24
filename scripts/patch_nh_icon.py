import os
from patch_browser_exe import update_exe_icon

exe = os.path.expandvars(r"%APPDATA%\LPBrower-launcher\runtime\ShardX-Windows\notification_helper.exe")
ico = r"g:\Vibe Tool\LPBrower\src-tauri\icons\browser-icon.ico"
update_exe_icon(exe, ico)
