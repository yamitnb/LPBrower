import struct
import ctypes
from ctypes import wintypes

kernel32 = ctypes.WinDLL('kernel32', use_last_error=True)

FindResourceExW = kernel32.FindResourceExW
FindResourceExW.argtypes = [wintypes.HMODULE, wintypes.LPCWSTR, wintypes.LPCWSTR, wintypes.WORD]
FindResourceExW.restype = wintypes.HRSRC

LoadResource = kernel32.LoadResource
LoadResource.argtypes = [wintypes.HMODULE, wintypes.HRSRC]
LoadResource.restype = wintypes.HGLOBAL

LockResource = kernel32.LockResource
LockResource.argtypes = [wintypes.HGLOBAL]
LockResource.restype = wintypes.LPVOID

SizeofResource = kernel32.SizeofResource
SizeofResource.argtypes = [wintypes.HMODULE, wintypes.HRSRC]
SizeofResource.restype = wintypes.DWORD

LoadLibraryExW = kernel32.LoadLibraryExW
LoadLibraryExW.argtypes = [wintypes.LPCWSTR, wintypes.HANDLE, wintypes.DWORD]
LoadLibraryExW.restype = wintypes.HMODULE

FreeLibrary = kernel32.FreeLibrary
FreeLibrary.argtypes = [wintypes.HMODULE]
FreeLibrary.restype = wintypes.BOOL

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

LOAD_LIBRARY_AS_DATAFILE = 0x00000002

class VerNode:
    def __init__(self, key="", val_type=1, value=b"", children=None):
        self.key = key
        self.val_type = val_type
        self.value = value
        self.children = children or []

def align4(n):
    return (n + 3) & ~3

def parse_ver_node(data, offset):
    start = offset
    wLength, wValueLength, wType = struct.unpack_from('<HHH', data, offset)
    offset += 6
    
    # Read szKey (null-terminated UTF-16LE string)
    key_chars = []
    while offset < start + wLength:
        ch = struct.unpack_from('<H', data, offset)[0]
        offset += 2
        if ch == 0:
            break
        key_chars.append(chr(ch))
    key = "".join(key_chars)
    
    # Align to 4 bytes
    offset = align4(offset)
    
    # Value
    value = b""
    if wValueLength > 0:
        if wType == 1:
            val_bytes_len = wValueLength * 2
            value = data[offset:offset + val_bytes_len]
            offset += val_bytes_len
        else:
            val_bytes_len = wValueLength
            value = data[offset:offset + val_bytes_len]
            offset += val_bytes_len
        offset = align4(offset)
        
    # Children
    children = []
    while offset < start + wLength:
        child_node, next_offset = parse_ver_node(data, offset)
        children.append(child_node)
        offset = align4(next_offset)
        
    return VerNode(key, wType, value, children), start + wLength

def serialize_ver_node(node):
    # key bytes
    key_bytes = (node.key + "\0").encode('utf-16le')
    
    # value bytes
    val_bytes = node.value
    if node.val_type == 1:
        wValueLength = len(val_bytes) // 2
    else:
        wValueLength = len(val_bytes)
        
    # children bytes
    children_bytes = b""
    for child in node.children:
        c_bytes = serialize_ver_node(child)
        children_bytes += c_bytes
        pad_len = align4(len(children_bytes)) - len(children_bytes)
        children_bytes += b"\0" * pad_len
        
    # Build node
    header_len = 6 + len(key_bytes)
    aligned_header_len = align4(header_len)
    val_offset_pad = aligned_header_len - header_len
    
    val_len = len(val_bytes)
    aligned_val_len = align4(val_len)
    child_offset_pad = aligned_val_len - val_len if (val_len > 0 and len(children_bytes) > 0) else 0
    
    total_len = aligned_header_len + val_len + child_offset_pad + len(children_bytes)
    
    res = struct.pack('<HHH', total_len, wValueLength, node.val_type)
    res += key_bytes
    res += b"\0" * val_offset_pad
    res += val_bytes
    res += b"\0" * child_offset_pad
    res += children_bytes
    return res

def modify_strings(node, replacements):
    # Check if this node is a String
    if node.val_type == 1 and len(node.value) >= 2:
        try:
            val_str = node.value.decode('utf-16le').rstrip('\0')
            if node.key in replacements:
                new_val = replacements[node.key]
                node.value = (new_val + "\0").encode('utf-16le')
            elif val_str in replacements:
                new_val = replacements[val_str]
                node.value = (new_val + "\0").encode('utf-16le')
        except Exception:
            pass
            
    for child in node.children:
        modify_strings(child, replacements)

def patch_file_version(exe_path, replacements):
    h_mod = LoadLibraryExW(exe_path, None, LOAD_LIBRARY_AS_DATAFILE)
    if not h_mod:
        raise WindowsError(f"Cannot load {exe_path}")
    try:
        lang_id = 1033
        h_res = FindResourceExW(h_mod, ctypes.cast(16, wintypes.LPCWSTR), ctypes.cast(1, wintypes.LPCWSTR), lang_id)
        if not h_res:
            lang_id = 0
            h_res = FindResourceExW(h_mod, ctypes.cast(16, wintypes.LPCWSTR), ctypes.cast(1, wintypes.LPCWSTR), lang_id)
        if not h_res:
            raise ValueError("Version resource not found")
        size = SizeofResource(h_mod, h_res)
        h_glob = LoadResource(h_mod, h_res)
        ptr = LockResource(h_glob)
        data = ctypes.string_at(ptr, size)
    finally:
        FreeLibrary(h_mod)
        
    root, _ = parse_ver_node(data, 0)
    modify_strings(root, replacements)
    new_data = serialize_ver_node(root)
    
    h_update = BeginUpdateResourceW(exe_path, False)
    if not h_update:
        err = ctypes.get_last_error()
        raise WindowsError(err, f"BeginUpdateResourceW failed: {err}")
    try:
        c_data = (ctypes.c_char * len(new_data)).from_buffer_copy(new_data)
        res_type = ctypes.cast(16, wintypes.LPCWSTR)
        res_name = ctypes.cast(1, wintypes.LPCWSTR)
        ok = UpdateResourceW(h_update, res_type, res_name, lang_id, c_data, len(new_data))
        if not ok:
            err = ctypes.get_last_error()
            raise WindowsError(err, f"UpdateResourceW failed: {err}")
        if not EndUpdateResourceW(h_update, False):
            err = ctypes.get_last_error()
            raise WindowsError(err, f"EndUpdateResourceW failed: {err}")
        print(f"Patched version info in {exe_path}")
    except Exception:
        EndUpdateResourceW(h_update, True)
        raise

replacements = {
    "ProductName": "LPBrower",
    "FileDescription": "LPBrower",
    "CompanyName": "LPBrower",
    "LegalCopyright": "Copyright 2026 LPBrower. All rights reserved.",
    "ShardX": "LPBrower"
}

import os
base_rt = os.path.expandvars(r"%APPDATA%\LPBrower-launcher\runtime\ShardX-Windows")
patch_file_version(os.path.join(base_rt, "chrome.exe"), replacements)
patch_file_version(os.path.join(base_rt, "chrome.dll"), replacements)
patch_file_version(os.path.join(base_rt, "notification_helper.exe"), replacements)
