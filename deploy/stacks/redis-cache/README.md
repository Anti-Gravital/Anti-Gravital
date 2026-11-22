# Redis cache stack

Local Redis for the `ag-cache` external backend. The crate's default native
cache is in-process (`CACHE_NATIVE_SERVER` / `CACHE_NATIVE_PORT`); Redis is an
opt-in backend selected by the project, not a runtime requirement.

## Ports

| Host port | Service | Purpose |
| --- | --- | --- |
| 6379 | redis | Redis RESP protocol |

Override with `REDIS_PORT` in `.env`.

## Run

```sh
cd deploy/stacks/redis-cache
cp .env.example .env
docker compose up -d
```

## Point a generated project at the stack

```sh
export REDIS_URL='redis://127.0.0.1:6379'
```

The AOF log persists in the named volume `redisdata`. `redis-cli ping` is the
container healthcheck.

## Limits and security

No password is configured: this stack is for localhost development only.
Do not publish port 6379 on an untrusted network, and do not use it for
production data. To require authentication, add `--requirepass` to the
`command:` list and use `redis://:<password>@127.0.0.1:6379` in `REDIS_URL`.
