"""Serve only this capture goal's built dist trees on owned loopback ports."""
from pathlib import Path
from http.server import SimpleHTTPRequestHandler, ThreadingHTTPServer
from urllib.parse import urlparse
import functools
import hashlib
import json
import signal
import socket
import sys
import threading

owner = Path(sys.argv[1]).resolve(strict=True)
marker = json.loads((owner / "owner.json").read_text())
repo = Path(__file__).resolve().parents[2]
assert marker.get("owner") == "gomoku-history-implementation" and marker.get("root") == str(owner) and marker.get("repo") == str(repo)
receipts = sorted((owner / "builds").glob("*/receipt.json"), key=lambda p: [int(x) for x in p.parent.name.lstrip("v").split(".")])
assert len(receipts) == 25, "Expected the full prepared source set"
for index in range(len(receipts)):
    with socket.socket() as probe:
        probe.bind(("127.0.0.1", 47200 + index))

class Handler(SimpleHTTPRequestHandler):
    def send_head(self):
        path = Path(self.translate_path(self.path))
        if not path.exists() and not Path(urlparse(self.path).path).suffix:
            # Plain static SPA fallback; no application/asset bytes are changed.
            self.path = "/index.html"
        return super().send_head()

    def list_directory(self, path):
        self.send_error(404)
        return None

    def log_message(self, format, *args):
        pass

servers = []
sources = {}
stop = threading.Event()
try:
    for index, path in enumerate(receipts):
        receipt = json.loads(path.read_text())
        tag = receipt["tag"]
        worktree = owner / "sources" / tag.replace(".", "_")
        dist = worktree / "gomoku-web/dist"
        assert hashlib.sha256((dist / "index.html").read_bytes()).hexdigest() == receipt["indexSha256"]
        port = 47200 + index
        server = ThreadingHTTPServer(("127.0.0.1", port), functools.partial(Handler, directory=str(dist)))
        servers.append(server)
        threading.Thread(target=server.serve_forever, daemon=True).start()
        sources[tag] = {"baseUrl": f"http://127.0.0.1:{port}/", "worktree": str(worktree), "receipt": str(path)}
    (owner / "previews.json").write_text(json.dumps(sources, indent=2) + "\n")
    print(json.dumps({"status": "ready", "sources": len(sources), "ports": [47200, 47224], "mapping": str(owner / "previews.json")}), flush=True)
    signal.signal(signal.SIGINT, lambda *_: stop.set())
    signal.signal(signal.SIGTERM, lambda *_: stop.set())
    stop.wait()
finally:
    for server in servers:
        server.shutdown()
        server.server_close()
