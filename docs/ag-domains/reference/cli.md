# CLI reference — ag domains

Phase A control-plane commands. The manual flow requires no provider
credentials. State is kept in a local JSON store (default `.ag/domains.json`,
override with `--state`).

## ag domains attach

Attach an external domain to a project and print DNS instructions.

```
ag domains attach <DOMAIN> --project <P> [--env <E>] [--service <S>]
                   --edge-host <HOST> [--ip <IP>]... [--state <PATH>]
```

- `<DOMAIN>`: `example.com`, `api.example.com` or `*.example.com`.
- `--edge-host` (env `AG_EDGE_HOST`): CNAME target for subdomains/wildcards.
- `--ip` (repeatable): apex A/AAAA addresses.

Creates the attachment in `pending_ownership`, generates a TXT ownership token
and prints the records to publish.

## ag domains instructions

Reprint DNS instructions for an attached domain.

```
ag domains instructions <DOMAIN> --edge-host <HOST> [--ip <IP>]... [--state <PATH>]
```

## ag domains export-zone

Print a BIND zone-file fragment.

```
ag domains export-zone <DOMAIN> --edge-host <HOST> [--ip <IP>]... [--state <PATH>]
```

## ag domains status / list

```
ag domains status <DOMAIN> [--state <PATH>]
ag domains list [--state <PATH>]
```

Show the four readiness dimensions and the derived lifecycle.

## ag domains verify

Verify the TXT ownership record across public resolvers and, on success, mark
ownership verified.

```
ag domains verify <DOMAIN> [--min-confirmed <N>] [--state <PATH>]
```

## ag domains detach

Remove routing, stop renewal and write a takeover tombstone.

```
ag domains detach <DOMAIN> [--tombstone-days <N>] [--state <PATH>]
```

Re-attaching a tombstoned hostname is blocked until the tombstone expires.

## ag domains diagnose

Compare the records the attachment expects against what public resolvers
observe, and report findings (missing, wrong value, CNAME/A conflict).
Read-only; no credentials.

```
ag domains diagnose <DOMAIN> --edge-host <HOST> [--ip <IP>]... [--state <PATH>]
```

Each finding is tagged `[ok]` or `[error]` with a suggested action; a healthy
domain prints "No action required."

## Pre-existing commands (Phase 4.5)

- `ag domains check` — TXT propagation check.
- `ag domains sync` — apply SPF/DKIM/DMARC via a DNS provider.
