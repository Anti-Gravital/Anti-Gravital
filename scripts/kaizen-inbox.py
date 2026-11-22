#!/usr/bin/env python3
"""Buzón local: recibe instrucciones de WhatsApp/n8n y las convierte en archivos para el equipo.

Solo escucha en 127.0.0.1. Requiere cabecera X-Kaizen-Secret igual a la variable de entorno
cuyo nombre se declara en notify.env (KAZEN_INBOX_SECRET por defecto). Nunca imprime secretos.
"""
import datetime
import json
import os
import re
import subprocess
import sys
from http.server import BaseHTTPRequestHandler, HTTPServer

PORT = int(os.environ.get("KI_PORT", "8787"))
SECRET_NAME = os.environ.get("KI_SHARED_SECRET_VAR", "KAZEN_INBOX_SECRET")
SECRET = os.environ.get(SECRET_NAME, "")
INBOX = os.environ.get("KI_INBOX_DIR", ".kaizen/team/inbox")
ROOT = os.environ.get("KAZEN_ROOT", os.getcwd())


def safe_name(value):
    return re.sub(r"[^A-Za-z0-9_-]", "_", str(value))[:40] or "mensaje"


def sanitize(text):
    """Usa el redactor del framework si existe; si no, recorta longitud."""
    redactor = os.path.join(ROOT, "scripts", "kaizen-security")
    cleaned = str(text)[:4000]
    if os.path.exists(redactor):
        try:
            p = subprocess.run([redactor, "redact"], input=cleaned, text=True,
                               capture_output=True, timeout=10)
            if p.returncode == 0 and p.stdout:
                return p.stdout
        except Exception:
            pass
    return cleaned


class Handler(BaseHTTPRequestHandler):
    def _json(self, code, payload):
        body = json.dumps(payload).encode()
        self.send_response(code)
        self.send_header("Content-Type", "application/json")
        self.send_header("Content-Length", str(len(body)))
        self.end_headers()
        self.wfile.write(body)

    def do_GET(self):
        if self.path.startswith("/health"):
            self._json(200, {"ok": True, "service": "kaizen-inbox"})
        else:
            self._json(404, {"error": "not found"})

    def do_POST(self):
        if not SECRET or self.headers.get("X-Kaizen-Secret") != SECRET:
            self._json(401, {"error": "unauthorized"})
            return
        if self.path not in ("/inbox", "/message"):
            self._json(404, {"error": "not found"})
            return
        try:
            length = int(self.headers.get("Content-Length", "0"))
            data = json.loads(self.rfile.read(length).decode("utf-8"))
        except Exception:
            self._json(400, {"error": "invalid json"})
            return
        sender = safe_name(data.get("from", "whatsapp"))
        channel = safe_name(data.get("channel", "whatsapp"))
        text = sanitize(data.get("text", ""))
        if not text.strip():
            self._json(400, {"error": "empty message"})
            return
        d = os.path.join(ROOT, INBOX)
        os.makedirs(d, exist_ok=True)
        ts = datetime.datetime.utcnow().strftime("%Y%m%d-%H%M%S")
        path = os.path.join(d, f"{ts}-{channel}-{sender}.md")
        with open(path, "w", encoding="utf-8") as fh:
            fh.write("---\n")
            fh.write(f"from: {json.dumps(sender, ensure_ascii=False)}\n")
            fh.write(f"channel: {json.dumps(channel, ensure_ascii=False)}\n")
            fh.write(f"received: {datetime.datetime.utcnow().isoformat()}Z\n")
            fh.write("status: pending\n")
            fh.write("---\n\n")
            fh.write("# Instrucción de El Presidente\n\n")
            fh.write(text + "\n\n")
            fh.write("## Acción del equipo\n- [ ] kaizen-equipo convierte esto en proyecto/tarea con KPI y source_ids.\n")
        self._json(200, {"ok": True, "file": os.path.relpath(path, ROOT)})

    def log_message(self, fmt, *args):
        """No registra cuerpos ni cabeceras: podrían contener datos sensibles (S-8)."""
        sys.stderr.write(f"[kaizen-inbox] {self.command} {self.path}\n")


if __name__ == "__main__":
    server = HTTPServer(("127.0.0.1", PORT), Handler)
    sys.stderr.write(f"[kaizen-inbox] escuchando en 127.0.0.1:{PORT}\n")
    server.serve_forever()
