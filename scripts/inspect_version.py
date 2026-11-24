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

LOAD_LIBRARY_AS_DATAFILE = 0x00000002

def inspect_version(exe_path):
    h_mod = LoadLibraryExW(exe_path, None, LOAD_LIBRARY_AS_DATAFILE)
    if not h_mod:
        print("Failed to load library")
        return
    try:
        # RT_VERSION = 16, ID = 1
        h_res = FindResourceExW(h_mod, ctypes.cast(16, wintypes.LPCWSTR), ctypes.cast(1, wintypes.LPCWSTR), 1033)
        if not h_res:
            h_res = FindResourceExW(h_mod, ctypes.cast(16, wintypes.LPCWSTR), ctypes.cast(1, wintypes.LPCWSTR), 0)
        if not h_res:
            print("Resource not found")
            return
        size = SizeofResource(h_mod, h_res)
        h_glob = LoadResource(h_mod, h_res)
        ptr = LockResource(h_glob)
        data = ctypes.string_at(ptr, size)
        print(f"Version resource size: {size}")
        
        # Check if "ShardX" is in data
        utf16_shard = "ShardX".encode('utf-16le')
        print(f"Occurrences of 'ShardX' in RT_VERSION: {data.count(utf16_shard)}")
    finally:
        FreeLibrary(h_mod)

inspect_version(r"C:\Users\Admin\AppData\Roaming\shardx-launcher\runtime\ShardX-Windows\chrome.exe")
inspect_version(r"C:\Users\Admin\AppData\Roaming\shardx-launcher\runtime\ShardX-Windows\chrome.dll")
