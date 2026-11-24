import os
import struct
import ctypes
from ctypes import wintypes

# Win32 API constants
RT_ICON = 3
RT_GROUP_ICON = 14
RT_VERSION = 16

kernel32 = ctypes.WinDLL('kernel32', use_last_error=True)

BeginUpdateResourceW = kernel32.BeginUpdateResourceW
BeginUpdateResourceW.argtypes = [wintypes.LPCWSTR, wintypes.BOOL]
BeginUpdateResourceW.restype = wintypes.HANDLE

UpdateResourceW = kernel32.UpdateResourceW
UpdateResourceW.argtypes = [
    wintypes.HANDLE,
    wintypes.LPCWSTR,
    wintypes.LPCWSTR,
    wintypes.WORD,
    wintypes.LPVOID,
    wintypes.DWORD
]
UpdateResourceW.restype = wintypes.BOOL

EndUpdateResourceW = kernel32.EndUpdateResourceW
EndUpdateResourceW.argtypes = [wintypes.HANDLE, wintypes.BOOL]
EndUpdateResourceW.restype = wintypes.BOOL

def update_exe_icon(exe_path, ico_path):
    with open(ico_path, 'rb') as f:
        ico_data = f.read()

    # Parse ICO header
    reserved, ico_type, count = struct.unpack_from('<HHH', ico_data, 0)
    if ico_type != 1:
        raise ValueError("Not a valid ICO file")

    print(f"ICO contains {count} images")

    # Read directory entries
    entries = []
    offset = 6
    for i in range(count):
        width, height, colors, res, planes, bpp, size, img_offset = struct.unpack_from('<BBBBHHII', ico_data, offset)
        offset += 16
        img_bytes = ico_data[img_offset:img_offset + size]
        entries.append({
            'width': width,
            'height': height,
            'colors': colors,
            'reserved': res,
            'planes': planes,
            'bpp': bpp,
            'size': size,
            'data': img_bytes,
            'id': i + 1
        })

    # Build GRPICONDIR structure
    # Header: idReserved (WORD), idType (WORD), idCount (WORD)
    grp_dir = struct.pack('<HHH', 0, 1, count)
    for entry in entries:
        # GRPICONDIRENTRY:
        # BYTE bWidth, bHeight, bColorCount, bReserved;
        # WORD wPlanes, wBitCount;
        # DWORD dwBytesInRes;
        # WORD nID;
        grp_dir += struct.pack(
            '<BBBBHHIH',
            entry['width'],
            entry['height'],
            entry['colors'],
            entry['reserved'],
            entry['planes'],
            entry['bpp'],
            entry['size'],
            entry['id']
        )

    # Open PE for update
    h_update = BeginUpdateResourceW(exe_path, False)
    if not h_update:
        err = ctypes.get_last_error()
        raise WindowsError(err, f"BeginUpdateResourceW failed: {err}")

    try:
        # Update each RT_ICON
        for entry in entries:
            c_data = (ctypes.c_char * len(entry['data'])).from_buffer_copy(entry['data'])
            # MAKEINTRESOURCE(RT_ICON) = 3
            # MAKEINTRESOURCE(entry['id'])
            res_type = ctypes.cast(RT_ICON, wintypes.LPCWSTR)
            res_name = ctypes.cast(entry['id'], wintypes.LPCWSTR)
            ok = UpdateResourceW(h_update, res_type, res_name, 0, c_data, len(entry['data']))
            if not ok:
                err = ctypes.get_last_error()
                print(f"Failed to update RT_ICON {entry['id']}: error {err}")

        # Update RT_GROUP_ICON ID 1 (or 101)
        # IDI_APPLICATION in Chromium is usually ID 1 or 101 or 1
        c_grp = (ctypes.c_char * len(grp_dir)).from_buffer_copy(grp_dir)
        res_type_grp = ctypes.cast(RT_GROUP_ICON, wintypes.LPCWSTR)
        
        # In Chromium chrome.exe, main icon is 1 and also 101
        for icon_group_id in [1, 101]:
            res_name_grp = ctypes.cast(icon_group_id, wintypes.LPCWSTR)
            UpdateResourceW(h_update, res_type_grp, res_name_grp, 0, c_grp, len(grp_dir))
            # Also language neutral / US English (1033)
            UpdateResourceW(h_update, res_type_grp, res_name_grp, 1033, c_grp, len(grp_dir))

        if not EndUpdateResourceW(h_update, False):
            err = ctypes.get_last_error()
            raise WindowsError(err, f"EndUpdateResourceW failed: {err}")
        print(f"Successfully updated icons in {exe_path}")
    except Exception:
        EndUpdateResourceW(h_update, True)
        raise

if __name__ == "__main__":
    import sys
    test_exe = os.path.expandvars(r"%APPDATA%\LPBrower-launcher\runtime\ShardX-Windows\chrome.exe")
    ico = r"g:\Vibe Tool\LPBrower\src-tauri\icons\browser-icon.ico"
    update_exe_icon(test_exe, ico)
