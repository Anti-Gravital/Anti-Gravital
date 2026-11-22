# Verification evidence for the reusable Docker packaging

Date: 2026-11-21. All commands were run from the repository root of the
`community7/docker` worktree. Nothing was pushed and no image was published.

## Reproduce

```sh
bash deploy/docker/verify.sh
```

The script prints `PASS`, `FAIL` and `SKIP` lines and exits non-zero on any
failure. Placeholder values for required variables are exported only inside
the script process; they are not stored in any file.

## Tool versions (real output)

```
Docker version 29.10.0, build fdd4b8e
Docker Compose version v5.5.1
Python 3.14.4
```

PyYAML 6.0.3 is installed (`python3 -c 'import yaml; print(yaml.__version__)'`).

## Docker daemon availability (reason for the build SKIP)

The Docker CLI and the Compose plugin are installed, but no daemon is
reachable and no `dockerd` binary exists in this environment. The Docker
Desktop socket is present but not responding:

```
$ docker info
Server:
Cannot connect to the Docker daemon at unix:///home/angel-nereira/.docker/desktop/docker.sock. Is the docker daemon running?

$ docker pull busybox:1.36-musl
Cannot connect to the Docker daemon at unix:///home/angel-nereira/.docker/desktop/docker.sock. Is the docker daemon running?
```

Result: **SKIP** for a real `docker build`. No image was built, run or
pushed. This is recorded as a skip, never as a successful build.

## Real checks executed

`bash deploy/docker/verify.sh` returned `PASS=23 FAIL=0 SKIP=2`.

| Check | Result |
| --- | --- |
| `docker compose -f <file> config -q` for all five compose files | PASS (5/5); no daemon needed |
| PyYAML `safe_load` of all five compose files, asserting `services` and `volumes` | PASS (5/5) |
| Compose contract: interpolation, named volumes, build section | PASS (5/5) |
| Dockerfile static lint: `rust:1.99.1`, `scratch`, non-root `USER`, `EXPOSE`, `HEALTHCHECK`, multi-stage, musl target | PASS (8/8) |
| No secret-like `ENV` in the Dockerfile | PASS |
| Real image build | SKIP (no reachable daemon) |

## Secret handling checked

The Dockerfile contains no credential literal; the only matches for
password/secret/token words are explanatory comments. The compose files
reference secrets only through required or optional environment variables:

- `api.yml`: `${POSTGRES_PASSWORD:?set POSTGRES_PASSWORD}`
- `ai.yml`: `${AI_API_KEY:?set AI_API_KEY}`

`compose/.env.example` uses the literal placeholder `CHANGE_ME` and is not a
real credential. Users must copy it to `.env` and supply their own values; a
populated `.env` must not be committed.

## What is not verified here

The static musl build of the workspace inside the builder stage, the runtime
behaviour of the busybox-backed HEALTHCHECK in `scratch`, and the pull of the
`rust:1.99.1` and `busybox:1.36-musl` base images. These require a running
daemon; the exact commands are documented in `README.md` so they can be
reproduced on a machine with Docker available.
