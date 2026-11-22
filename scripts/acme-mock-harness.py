#!/usr/bin/env python3
"""Local, credential-free ACME order-flow harness.

This script emulates the RFC 8555 order state machine on 127.0.0.1 and drives
it end to end. It exists so the ACME flow documented for issue #87 can be
exercised offline: new order -> authorization pending/valid -> challenge ->
finalize -> certificate download.

It is NOT an ACME certification authority. No real CA is contacted, no domain
control is proven, and the downloaded chain is a self-signed test certificate
generated locally with openssl. Never present its output as evidence that a
certificate was issued by a trusted CA. The credentialed staging run described
in docs/ag-domains/how-to/acme-staging-test.md remains the gate for #87/#179.

Requirements: Python 3 (stdlib only) and `openssl` on PATH.
Run directly or through scripts/acme-mock-harness.sh.
"""

from __future__ import annotations

import argparse
import datetime as dt
import json
import secrets
import shutil
import subprocess
import sys
import tempfile
import threading
import urllib.error
import urllib.request
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer
from pathlib import Path
from typing import Any

MOCK_DOMAIN = "mock.example.test"  # RFC 2606 reserved .test TLD; no real DNS.
ORDER_ID = "order-1"
EXPECTED_SEQUENCE = [
    "new_order:pending",
    "authorization:pending",
    "challenge:pending",
    "challenge:valid",
    "authorization:valid",
    "order:ready",
    "order:processing",
    "order:valid",
]


def now() -> str:
    return dt.datetime.now(dt.timezone.utc).isoformat()


class MockState:
    """In-memory state of the single mock order and its authorization."""

    def __init__(self, domain: str, key_auth: str, certificate_pem: str) -> None:
        self.domain = domain
        self.key_auth = key_auth
        self.certificate_pem = certificate_pem
        self.order_status = "pending"
        self.authorization_status = "pending"
        self.challenge_status = "pending"
        self.certificate_url: str | None = None
        self.events: list[dict[str, str]] = []
        self.lock = threading.Lock()

    def record(self, event: str, detail: str = "") -> None:
        with self.lock:
            self.events.append({"at": now(), "event": event, "detail": detail})

    def order_json(self, base: str) -> dict[str, Any]:
        result: dict[str, Any] = {
            "status": self.order_status,
            "identifiers": [{"type": "dns", "value": self.domain}],
            "authorizations": [f"{base}/authz/{ORDER_ID}"],
            "finalize": f"{base}/finalize/{ORDER_ID}",
        }
        if self.certificate_url is not None:
            result["certificate"] = self.certificate_url
        return result

    def authorization_json(self, base: str) -> dict[str, Any]:
        return {
            "status": self.authorization_status,
            "identifier": {"type": "dns", "value": self.domain},
            "challenges": [
                {
                    "type": "dns-01",
                    "status": self.challenge_status,
                    "url": f"{base}/challenge/{ORDER_ID}",
                    "token": self.key_auth,
                }
            ],
        }


class MockAcmeHandler(BaseHTTPRequestHandler):
    """Minimal RFC 8555-shaped HTTP handler for the local mock server."""

    state: MockState

    def log_message(self, fmt: str, *args: Any) -> None:
        # Keep stdout as machine-readable JSON lines only.
        return

    def _base(self) -> str:
        return f"http://{self.server.server_address[0]}:{self.server.server_address[1]}"

    def _send_json(self, payload: dict[str, Any], status: int = 200) -> None:
        body = json.dumps(payload).encode("utf-8")
        self.send_response(status)
        self.send_header("Content-Type", "application/json")
        self.send_header("Content-Length", str(len(body)))
        self.end_headers()
        self.wfile.write(body)

    def _send_pem(self, pem: str) -> None:
        body = pem.encode("utf-8")
        self.send_response(200)
        self.send_header("Content-Type", "application/pem-certificate-chain")
        self.send_header("Content-Length", str(len(body)))
        self.end_headers()
        self.wfile.write(body)

    def do_GET(self) -> None:  # noqa: N802 - BaseHTTPRequestHandler API
        base = self._base()
        path = self.path.split("?", 1)[0]
        if path == "/directory":
            self._send_json(
                {"newAccount": f"{base}/new-acct", "newOrder": f"{base}/new-order", "mock": True}
            )
            return
        if path == f"/authz/{ORDER_ID}":
            self._send_json(self.state.authorization_json(base))
            return
        if path == f"/order/{ORDER_ID}":
            # A finalize POST sets 'processing'; the first subsequent refresh
            # simulates the CA finishing and publishes the certificate URL.
            if self.state.order_status == "processing":
                self.state.order_status = "valid"
                self.state.certificate_url = f"{base}/certificate/{ORDER_ID}"
                self.state.record("order:valid", "certificate URL published")
            self._send_json(self.state.order_json(base))
            return
        if path == f"/certificate/{ORDER_ID}" and self.state.certificate_url:
            self._send_pem(self.state.certificate_pem)
            return
        if path == "/events":
            with self.state.lock:
                events = list(self.state.events)
            self._send_json({"events": events})
            return
        self._send_json({"error": "not found"}, status=404)

    def do_POST(self) -> None:  # noqa: N802 - BaseHTTPRequestHandler API
        base = self._base()
        length = int(self.headers.get("Content-Length", "0"))
        raw = self.rfile.read(length) if length else b"{}"
        try:
            body = json.loads(raw or b"{}")
        except json.JSONDecodeError:
            body = {}
        path = self.path.split("?", 1)[0]

        if path == "/new-acct":
            self.state.record("account:valid", "mock account created")
            self._send_json({"status": "valid", "mock": True})
            return
        if path == "/new-order":
            identifiers = body.get("identifiers", [])
            if not any(
                i.get("type") == "dns" and i.get("value") == self.state.domain
                for i in identifiers
            ):
                self._send_json({"error": "unexpected identifier"}, status=400)
                return
            self.state.order_status = "pending"
            self.state.authorization_status = "pending"
            self.state.challenge_status = "pending"
            self.state.certificate_url = None
            self.state.record("new_order:pending", self.state.domain)
            self.state.record("authorization:pending", self.state.domain)
            self.state.record("challenge:pending", "dns-01")
            self._send_json(self.state.order_json(base))
            return
        if path == f"/challenge/{ORDER_ID}":
            if self.state.order_status != "pending":
                self._send_json({"error": "challenge not pending"}, status=409)
                return
            self.state.challenge_status = "valid"
            self.state.authorization_status = "valid"
            self.state.order_status = "ready"
            self.state.record("challenge:valid", "DNS-01 accepted by mock")
            self.state.record("authorization:valid", self.state.domain)
            self.state.record("order:ready", "authorization satisfied")
            self._send_json(
                {"type": "dns-01", "status": "valid", "url": f"{base}/challenge/{ORDER_ID}"}
            )
            return
        if path == f"/finalize/{ORDER_ID}":
            if self.state.order_status != "ready":
                self._send_json({"error": "order not ready"}, status=409)
                return
            self.state.order_status = "processing"
            self.state.record("order:processing", "CSR accepted by mock")
            self._send_json(self.state.order_json(base))
            return
        self._send_json({"error": "not found"}, status=404)


def request(url: str, method: str = "GET", payload: dict[str, Any] | None = None) -> Any:
    """Small JSON/bytes HTTP client using only the standard library."""
    data = json.dumps(payload).encode("utf-8") if payload is not None else None
    req = urllib.request.Request(url, data=data, method=method)
    if data is not None:
        req.add_header("Content-Type", "application/json")
    try:
        with urllib.request.urlopen(req, timeout=5) as resp:  # noqa: S310 - local URL
            raw = resp.read()
            ctype = resp.headers.get("Content-Type", "")
            return raw.decode("utf-8") if "pem" in ctype else json.loads(raw)
    except urllib.error.HTTPError as exc:
        raise RuntimeError(f"HTTP {exc.code} for {method} {url}") from exc


def generate_mock_certificate(workdir: Path) -> Path:
    """Create a self-signed test certificate; never a CA-issued certificate."""
    key = workdir / "mock-key.pem"
    cert = workdir / "mock-chain.pem"
    proc = subprocess.run(
        [
            "openssl", "req", "-x509", "-newkey", "rsa:2048", "-nodes",
            "-keyout", str(key), "-out", str(cert), "-days", "3650",
            "-subj", f"/CN={MOCK_DOMAIN}",
        ],
        capture_output=True,
        text=True,
        check=False,
    )
    if proc.returncode != 0:
        raise RuntimeError(f"openssl failed: {proc.stderr.strip()}")
    return cert


def inspect_certificate(pem_path: Path) -> dict[str, str]:
    """Read subject/issuer/serial/fingerprint of the locally generated cert."""
    proc = subprocess.run(
        [
            "openssl", "x509", "-in", str(pem_path), "-noout",
            "-subject", "-issuer", "-serial", "-fingerprint", "-sha256",
        ],
        capture_output=True,
        text=True,
        check=False,
    )
    if proc.returncode != 0:
        raise RuntimeError(f"openssl x509 failed: {proc.stderr.strip()}")
    fields: dict[str, str] = {}
    for line in proc.stdout.splitlines():
        if "=" in line:
            key, value = line.split("=", 1)
            fields[key.strip()] = value.strip()
    fields["raw"] = proc.stdout.strip()
    return fields


def check(condition: bool, message: str) -> None:
    if not condition:
        raise AssertionError(message)


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "--evidence-out",
        type=Path,
        default=None,
        help="Optional path to write the transcript (defaults to stdout only).",
    )
    args = parser.parse_args()
    transcript: list[str] = []

    def log(event: str, **fields: Any) -> None:
        record = {"at": now(), "event": event, **fields}
        line = json.dumps(record, sort_keys=True)
        transcript.append(line)
        print(line, flush=True)

    if shutil.which("openssl") is None:
        log("harness:result", result="FAIL", reason="openssl not found on PATH")
        result_code = 2
    else:
        try:
            log(
                "harness:start",
                kind="local-mock-only",
                domain=MOCK_DOMAIN,
                python=sys.version.split()[0],
                openssl=subprocess.run(
                    ["openssl", "version"], capture_output=True, text=True, check=True
                ).stdout.strip(),
                note="No real CA, no credentials, no external network.",
            )

            with tempfile.TemporaryDirectory(prefix="ag-acme-mock-") as tmp:
                workdir = Path(tmp)
                cert_path = generate_mock_certificate(workdir)
                state = MockState(
                    domain=MOCK_DOMAIN,
                    key_auth=secrets.token_urlsafe(32),
                    certificate_pem=cert_path.read_text(encoding="utf-8"),
                )
                MockAcmeHandler.state = state

                server = ThreadingHTTPServer(("127.0.0.1", 0), MockAcmeHandler)
                thread = threading.Thread(target=server.serve_forever, daemon=True)
                thread.start()
                base = f"http://127.0.0.1:{server.server_address[1]}"

                try:
                    log("server:ready", url=base, bind="127.0.0.1", tls=False)

                    # 1. Directory discovery.
                    directory = request(f"{base}/directory")
                    check(
                        "newAccount" in directory and "newOrder" in directory, "bad directory"
                    )
                    log("directory:ok", new_account=True, new_order=True)

                    # 2. Account creation (mock always accepts).
                    account = request(f"{base}/new-acct", method="POST", payload={})
                    check(account.get("status") == "valid", "account not valid")
                    log("account:valid", status=account["status"])

                    # 3. New order -> pending.
                    order = request(
                        f"{base}/new-order",
                        method="POST",
                        payload={"identifiers": [{"type": "dns", "value": MOCK_DOMAIN}]},
                    )
                    check(order.get("status") == "pending", "order not pending")
                    check(len(order.get("authorizations", [])) == 1, "no authorization")
                    log("new_order:pending", status=order["status"], domain=MOCK_DOMAIN)

                    # 4. Authorization -> pending, DNS-01 challenge present.
                    authz_url = order["authorizations"][0]
                    authz = request(authz_url)
                    check(authz.get("status") == "pending", "authorization not pending")
                    challenges = authz.get("challenges", [])
                    dns_challenges = [c for c in challenges if c.get("type") == "dns-01"]
                    check(len(dns_challenges) == 1, "no DNS-01 challenge")
                    check(dns_challenges[0].get("status") == "pending", "challenge not pending")
                    log(
                        "authorization:pending",
                        status=authz["status"],
                        challenge_type="dns-01",
                        challenge_status=dns_challenges[0]["status"],
                    )

                    # 5. Signal ready -> challenge/authorization valid, order ready.
                    challenge = request(
                        dns_challenges[0]["url"],
                        method="POST",
                        payload={"mock": "dns01-ready"},
                    )
                    check(challenge.get("status") == "valid", "challenge not valid")
                    log("challenge:valid", status=challenge["status"])

                    authz = request(authz_url)
                    check(authz.get("status") == "valid", "authorization still pending")
                    log("authorization:valid", status=authz["status"])

                    order = request(f"{base}/order/{ORDER_ID}")
                    check(order.get("status") == "ready", "order not ready")
                    log("order:ready", status=order["status"])

                    # 6. Finalize -> processing, then refresh -> valid + cert URL.
                    order = request(
                        order["finalize"], method="POST", payload={"csr": "mock-csr"}
                    )
                    check(order.get("status") == "processing", "order not processing")
                    log("order:processing", status=order["status"])

                    order = request(f"{base}/order/{ORDER_ID}")
                    check(order.get("status") == "valid", "order not valid after refresh")
                    check("certificate" in order, "no certificate URL")
                    log(
                        "order:valid",
                        status=order["status"],
                        certificate_url=order["certificate"],
                    )

                    # 7. Download the chain and verify it is LOCAL and self-signed.
                    downloaded = request(order["certificate"])
                    check("-----BEGIN CERTIFICATE-----" in downloaded, "no PEM chain")
                    downloaded_path = workdir / "downloaded-chain.pem"
                    downloaded_path.write_text(downloaded, encoding="utf-8")
                    info = inspect_certificate(downloaded_path)
                    check(MOCK_DOMAIN in info.get("subject", ""), "certificate CN mismatch")
                    check(
                        info.get("subject") == info.get("issuer"),
                        "certificate is not self-signed",
                    )
                    log(
                        "certificate:downloaded",
                        pem_header="-----BEGIN CERTIFICATE-----",
                        self_signed=True,
                        subject_equals_issuer=True,
                        openssl_x509=info["raw"],
                    )

                    # 8. Assert the exact state sequence recorded by the mock.
                    observed = [
                        e["event"] for e in state.events if e["event"] in EXPECTED_SEQUENCE
                    ]
                    check(observed == EXPECTED_SEQUENCE, f"state sequence mismatch: {observed}")

                    log(
                        "harness:result",
                        result="PASS",
                        sequence=observed,
                        statement=(
                            "Local mock flow only. No real CA issued a certificate; "
                            "the chain was self-signed on this machine."
                        ),
                    )
                    result_code = 0
                finally:
                    server.shutdown()
                    server.server_close()
                    thread.join(timeout=5)
        except (AssertionError, RuntimeError, OSError, KeyError) as exc:
            log("harness:result", result="FAIL", reason=str(exc))
            result_code = 1

    if args.evidence_out is not None:
        args.evidence_out.write_text("\n".join(transcript) + "\n", encoding="utf-8")
    return result_code


if __name__ == "__main__":
    sys.exit(main())
