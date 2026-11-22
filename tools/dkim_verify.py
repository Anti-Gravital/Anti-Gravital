#!/usr/bin/env python3
"""Independent DKIM verification with dkimpy for the ag-mail test vectors.

This tool is deliberately NOT a Cargo dependency. mail-auth's own verifier
shares canonicalization, hashing and header-writing lineage with the signer,
so it cannot provide independent evidence that ag-mail emits valid DKIM.
dkimpy (BSD-like / DFSG approved) is a separate implementation; PyNaCl
(Apache-2.0) supplies its ed25519 primitive.

Usage:
    python3 tools/dkim_verify.py MESSAGE [MESSAGE ...]

Each MESSAGE is an RFC 5322 file holding a DKIM-Signature made by selector
`sel` for domain `send.example`. The matching published `p=` key is fixed
below, so no live DNS access is needed. One result line is printed per file
and the exit status is 0 only when every signature verifies.

See docs/security/DKIM_INDEPENDENT_VERIFICATION.md for prerequisites, the
exact invocation and recorded output.
"""

import sys

try:
    import dkim
except ImportError:
    print(
        "dkimpy is not installed; run: python3 -m pip install --user dkimpy",
        file=sys.stderr,
    )
    sys.exit(2)

try:
    import nacl  # noqa: F401 - imported only to fail early with a clear message
except ImportError:
    # dkimpy 1.1.8 silently returns False for ed25519 signatures when PyNaCl
    # is absent. Fail loudly instead of reporting a false negative.
    print(
        "PyNaCl is not installed; ed25519 verification would silently fail. "
        "Run: python3 -m pip install --user pynacl",
        file=sys.stderr,
    )
    sys.exit(2)

SELECTOR = "sel"
DOMAIN = "send.example"
PUBLIC_KEY_RECORD = (
    "v=DKIM1; k=ed25519; p=omzBDludvx6go2PDwfrVAv5i+So6Bc3/GDagUysiu8Q="
)
DNS_NAME = f"{SELECTOR}._domainkey.{DOMAIN}."
DNS_RECORDS = {DNS_NAME: PUBLIC_KEY_RECORD.encode("ascii")}


def dnsfunc(name, **_kwargs):
    """Answer the single _domainkey TXT query; never touch the network."""
    query = name.decode("ascii") if isinstance(name, bytes) else name
    return DNS_RECORDS.get(query, b"")


def main(argv):
    if not argv:
        print(__doc__, file=sys.stderr)
        return 2

    all_ok = True
    for path in argv:
        try:
            with open(path, "rb") as handle:
                message = handle.read()
        except OSError as exc:
            print(f"dkim=error file={path} detail=read failed: {exc}")
            all_ok = False
            continue

        try:
            verified = dkim.verify(message, dnsfunc=dnsfunc)
        except Exception as exc:  # surface any verifier error, never fake a pass
            print(f"dkim=error file={path} detail={type(exc).__name__}: {exc}")
            all_ok = False
            continue

        status = "pass" if verified else "fail"
        print(
            f"dkim={status} file={path} d={DOMAIN} s={SELECTOR} "
            "a=ed25519-sha256 c=relaxed/relaxed"
        )
        all_ok = all_ok and verified

    return 0 if all_ok else 1


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
