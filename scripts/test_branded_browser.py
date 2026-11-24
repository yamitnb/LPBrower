import os, time, json, shutil, tempfile, subprocess, urllib.request, ctypes, base64, socket, hashlib, struct
from ctypes import wintypes

CHROME = os.path.expandvars(r"%APPDATA%\LPBrower-launcher\runtime\ShardX-Windows\chrome.exe")
OUT_DIR = r"C:\Users\Admin\.gemini\antigravity\brain\6659144a-f4cf-41d2-a473-a1d186c1fdb7"

# Minimal sync RFC6455 WebSocket client for CDP (zero external deps!)
class MiniCDP:
    def __init__(self, ws_url):
        # ws://127.0.0.1:PORT/path
        rest = ws_url[len("ws://"):]
        hostport, path = rest.split("/", 1)
        host, port = hostport.split(":")
        self.sock = socket.create_connection((host, int(port)), timeout=10)
        key = base64.b64encode(os.urandom(16)).decode()
        req = (
            f"GET /{path} HTTP/1.1\r\n"
            f"Host: {hostport}\r\n"
            f"Upgrade: websocket\r\n"
            f"Connection: Upgrade\r\n"
            f"Sec-WebSocket-Key: {key}\r\n"
            f"Sec-WebSocket-Version: 13\r\n\r\n"
        )
        self.sock.sendall(req.encode())
        resp = b""
        while b"\r\n\r\n" not in resp:
            resp += self.sock.recv(4096)
        self.buf = resp.split(b"\r\n\r\n", 1)[1]
        self.msg_id = 0

    def _recv_exact(self, n):
        while len(self.buf) < n:
            chunk = self.sock.recv(65536)
            if not chunk:
                raise EOFError("ws closed")
            self.buf += chunk
        out = self.buf[:n]
        self.buf = self.buf[n:]
        return out

    def send_text(self, text):
        payload = text.encode("utf-8")
        mask = os.urandom(4)
        header = bytearray([0x81])
        L = len(payload)
        if L < 126:
            header.append(0x80 | L)
        elif L < 65536:
            header.append(0x80 | 126)
            header += struct.pack("!H", L)
        else:
            header.append(0x80 | 127)
            header += struct.pack("!Q", L)
        header += mask
        masked = bytes(b ^ mask[i % 4] for i, b in enumerate(payload))
        self.sock.sendall(header + masked)

    def recv_msg(self):
        chunks = []
        while True:
            b1, b2 = self._recv_exact(2)
            fin = (b1 & 0x80) != 0
            opcode = b1 & 0x0F
            length = b2 & 0x7F
            if length == 126:
                length = struct.unpack("!H", self._recv_exact(2))[0]
            elif length == 127:
                length = struct.unpack("!Q", self._recv_exact(8))[0]
            data = self._recv_exact(length)
            if opcode == 0x8:
                raise EOFError("ws close frame")
            if opcode == 0x9:
                continue
            chunks.append(data)
            if fin:
                return b"".join(chunks).decode("utf-8", errors="replace")

    def call(self, method, params=None):
        self.msg_id += 1
        cid = self.msg_id
        self.send_text(json.dumps({"id": cid, "method": method, "params": params or {}}))
        while True:
            msg = json.loads(self.recv_msg())
            if msg.get("id") == cid:
                return msg.get("result", {})

    def close(self):
        try: self.sock.close()
        except: pass


tmp_udd = tempfile.mkdtemp(prefix="lpbrower_live_")
fp_path = os.path.join(tmp_udd, "fp.json")
with open(fp_path, "w") as f:
    json.dump({"name": "LPB|DemoProfile"}, f)

proc = subprocess.Popen([
    CHROME,
    f"--user-data-dir={tmp_udd}",
    f"--fingerprint-profile={fp_path}",
    "--no-first-run",
    "--remote-debugging-port=19333",
    "--remote-allow-origins=*",
    "--window-size=1100,820",
], stdout=subprocess.PIPE, stderr=subprocess.PIPE)

try:
    targets = None
    for _ in range(30):
        time.sleep(0.3)
        try:
            with urllib.request.urlopen("http://127.0.0.1:19333/json", timeout=2) as r:
                targets = json.loads(r.read().decode())
                if any(t.get("type") == "page" for t in targets):
                    break
        except Exception:
            pass

    if not targets:
        print("FAILED to connect to CDP!")
    else:
        time.sleep(1.5)
        # Re-read targets after load
        with urllib.request.urlopen("http://127.0.0.1:19333/json", timeout=2) as r:
            targets = json.loads(r.read().decode())
        pages = [t for t in targets if t.get("type") == "page"]
        print("Opened pages:")
        for t in pages:
            print(f"  title={t.get('title')!r} url={t.get('url')!r}")

        # Enumerate Win32 window titles for proc.pid
        user32 = ctypes.windll.user32
        titles = []
        @ctypes.WINFUNCTYPE(wintypes.BOOL, wintypes.HWND, wintypes.LPARAM)
        def enum_cb(hwnd, lparam):
            pid = wintypes.DWORD()
            user32.GetWindowThreadProcessId(hwnd, ctypes.byref(pid))
            if pid.value == proc.pid:
                n = user32.GetWindowTextLengthW(hwnd)
                if n > 0:
                    buf = ctypes.create_unicode_buffer(n + 1)
                    user32.GetWindowTextW(hwnd, buf, n + 1)
                    titles.append(buf.value)
            return True
        user32.EnumWindows(enum_cb, 0)
        print("Win32 window titles:", titles)

        # Connect CDP to first page & capture screenshots
        cdp = MiniCDP(pages[0]["webSocketDebuggerUrl"])
        cdp.call("Page.enable")
        res = cdp.call("Page.captureScreenshot", {"format": "png"})
        if "data" in res:
            p_start = os.path.join(OUT_DIR, "browser_startpage_verified.png")
            open(p_start, "wb").write(base64.b64decode(res["data"]))
            print("Saved", p_start)

        for fname, url in [
            ("browser_shardx_page_verified.png", "chrome://shardx/"),
            ("browser_about_verified.png", "chrome://settings/help"),
            ("browser_version_verified.png", "chrome://version"),
        ]:
            cdp.call("Page.navigate", {"url": url})
            time.sleep(1.8)
            res = cdp.call("Page.captureScreenshot", {"format": "png"})
            if "data" in res:
                out_p = os.path.join(OUT_DIR, fname)
                open(out_p, "wb").write(base64.b64decode(res["data"]))
                print("Saved", out_p)
        cdp.close()
finally:
    proc.terminate()
    try: proc.wait(timeout=5)
    except: proc.kill()
    time.sleep(0.5)
    shutil.rmtree(tmp_udd, ignore_errors=True)
