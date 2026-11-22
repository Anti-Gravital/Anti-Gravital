# {{name}} - auth service

A small authentication service that shows the `ag-auth` primitives most
projects need first: Ed25519 JWTs and BLAKE3 API keys. All routes pass through
the `ag-core` Shield pipeline.

## Stack

| Concern | Choice |
| --- | --- |
| JWT | `ag_auth::JwtSigner` + `ag_auth::Claims` (EdDSA / Ed25519) |
| API keys | `ag_auth::generate_api_key` / `verify_api_key` (BLAKE3 hashing) |
| HTTP and security | `ag-core` (`Shield`), axum 0.7 |
| Key generation | `ed25519-dalek` 2 + `getrandom` (development only) |

## Run

```bash
cargo run
```

On startup the service generates an ephemeral Ed25519 keypair, because no
`JWT_PRIVATE_KEY` / `JWT_PUBLIC_KEY` are set. `/health` reports
`{"status":"ok","mode":"ephemeral"}`. Tokens stop being valid on restart.

```bash
TOKEN=$(curl -s -X POST http://127.0.0.1:8080/login \
  -H 'content-type: application/json' \
  -d '{"sub":"user-1","role":"admin"}' | sed -n 's/.*"access_token":"\([^"]*\)".*/\1/p')

curl http://127.0.0.1:8080/me -H "Authorization: Bearer $TOKEN"

curl -X POST http://127.0.0.1:8080/api-keys
```

## Persistent keys

Provide stable PEM keys in the environment to keep tokens valid across
restarts:

```bash
openssl genpkey -algorithm ed25519 -out jwt-private.pem
openssl pkey -in jwt-private.pem -pubout -out jwt-public.pem

export JWT_PRIVATE_KEY="$(cat jwt-private.pem)"
export JWT_PUBLIC_KEY="$(cat jwt-public.pem)"
export JWT_TTL_SECS=3600
cargo run
```

`/health` then reports `mode: "persistent"`. Never commit private keys.

## API

| Method | Path | Purpose |
| --- | --- | --- |
| POST | `/login` | Issue a signed JWT for `sub` and `role` |
| GET | `/me` | Verify the Bearer token and return its claims |
| POST | `/api-keys` | Generate a key plus its hash (hash is what you persist) |
| POST | `/api-keys/verify` | Check a raw key against a stored hash |
| GET | `/health` | Liveness and key mode |

## Limitations

- `/login` trusts the caller-supplied `sub` and `role`: it is a demo endpoint.
  Replace it with a real credential check (password, passkey or OAuth2) before
  deployment. `ag-auth` also exposes `AgAuth`, `WebAuthnRp` and `OAuthClient`
  for those flows.
- No refresh tokens, revocation list or rate limit on `/login` are wired here.
  Add `ag_auth::RefreshBlacklist` and a Shield rate-limit layer.
- API key hashes are returned once and never stored by the service; the caller
  owns persistence.
- Ephemeral keys are intentionally unsuitable for multi-instance deployments.
