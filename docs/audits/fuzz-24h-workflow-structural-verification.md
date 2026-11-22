# Structural verification of the scheduled 24h fuzz workflow (issue #156)

English canonical (ADR-0008). Date: 2026-11-22. Branch: `community12/i156`.
Commit under test: `72b97ad` (equal to `main` at verification time). Scope:
confirm the wave-11 workflow is present on the branch base and structurally
correct, and record reproducible verification evidence. This is a **capability
and structure check only**; no fuzz target was executed for 86400 s here, no
gate row was moved, and issue #156 remains open.

## 1. Presence on the branch base

```sh
git rev-parse HEAD          # 72b97adbdccf8cab3fbd5f411f54343beab40672
git rev-parse main          # 72b97adbdccf8cab3fbd5f411f54343beab40672
git cat-file -e main:.github/workflows/fuzz-24h.yml && echo 'PRESENT on main'
git log --oneline main --grep='fuzz gate|scheduled 24h' -i
```

Output:

```
PRESENT on main
c0a695d docs(rfc): renumber scheduled fuzz gate RFC to 0036
978c413 merge: integrate community11/gatefuzz (conflicts resolved)
5da4087 feat(ci): scheduled 24h fuzz gate on self-hosted runner (RFC-0035)
```

`git merge-base HEAD main` equals `HEAD`, so the workflow is on this branch's
base. The wave-11 merge commit is `978c413`, renumbered to RFC-0036 by
`c0a695d`. The workflow file needs no change: every assertion in section 2
passed. No workflow defect was found.

## 2. YAML parse and structural assertions

Command (reproducible from the repository root; PyYAML is the parser used by
RFC-0036 section 9, fallback Ruby `psych`):

```sh
python3 - <<'PY'
import re, sys, yaml
path = ".github/workflows/fuzz-24h.yml"
raw = open(path, encoding="utf-8").read()
d = yaml.safe_load(raw)
print("PARSE: OK (yaml.safe_load)")
triggers = d.get(True) if True in d else d.get("on")  # PyYAML maps `on` -> True
assert isinstance(triggers, dict)
print("TRIGGERS keys:", sorted(triggers.keys()))
assert "workflow_dispatch" in triggers and "schedule" in triggers
cron = [c.get("cron") for c in triggers["schedule"]]
print("CRON:", cron)
assert "0 3 1 * *" in cron
jobs = d["jobs"]
print("JOBS:", sorted(jobs.keys()))
assert set(jobs) == {"fuzz", "notify-failure"}
fz, nf = jobs["fuzz"], jobs["notify-failure"]
print("runs-on:", fz["runs-on"])
assert fz["runs-on"] == ["self-hosted", "linux", "x64", "fuzz-dedicated"]
print("timeout-minutes:", fz.get("timeout-minutes"))
print("permissions top:", d.get("permissions"))
assert d.get("permissions") == {}
print("permissions fuzz:", fz.get("permissions"))
assert fz.get("permissions") == {"contents": "read"}
print("permissions notify:", nf.get("permissions"))
assert nf.get("permissions") == {"contents": "read", "issues": "write"}
guards = {n: any("runner.environment" in str(s.get("if","")) and "self-hosted" in str(s.get("if",""))
                 for s in j.get("steps",[]) if isinstance(s,dict)) for n,j in jobs.items()}
print("runner.environment guards:", guards)
assert all(guards.values())
uses = [s["uses"] for j in jobs.values() for s in j.get("steps",[]) if isinstance(s,dict) and "uses" in s]
print("uses (parsed):", uses)
assert len(uses) == 2
for u in uses:
    assert re.match(r"^actions/(checkout|upload-artifact)@[0-9a-f]{40}$", u)
for name in ("checkout", "upload-artifact"):
    assert re.search(r"actions/%s@[0-9a-f]{40} # v4" % name, raw)
print("pinned actions: 2 SHA pins with # v4 comment in raw text: OK")
print("env.DURATION:", d["env"].get("DURATION"))
assert d["env"].get("DURATION") == "86400"
targets = fz["strategy"]["matrix"]["target"]
expected = ["fuzz_lexer","fuzz_parser","fuzz_compile","fuzz_resp",
            "fuzz_storage_key","fuzz_signed_url","fuzz_workers_payload"]
print("matrix targets (%d):" % len(targets), targets)
assert targets == expected
print("max-parallel:", fz["strategy"].get("max-parallel"), "fail-fast:", fz["strategy"].get("fail-fast"))
assert fz["strategy"].get("max-parallel") == 1 and fz["strategy"].get("fail-fast") is False
print("fuzz env.FUZZ_TARGETS:", fz["env"].get("FUZZ_TARGETS"))
assert "matrix.target" in str(fz["env"].get("FUZZ_TARGETS"))
print("concurrency:", d.get("concurrency"))
secrets = re.findall(r"secrets\.[A-Za-z0-9_]+", raw)
print("secrets.* references:", secrets)
assert secrets == []
assert "${{ github.token }}" in raw and "GITHUB_TOKEN" in raw
print("token refs present: github.token + GITHUB_TOKEN (built-in only)")
assert "<!-- fuzz-24h-gate run ${RUN_ID} -->" in raw
assert "gh issue comment" in raw and "if: ${{ failure() }}" in raw
assert "stays pending" in raw and "stays open" in raw
print("notify: marker + gh comment + failure() + open/pending language: OK")
assert 'bash scripts/run-fuzz-window.sh "$DURATION"' in raw
print("harness invocation with $DURATION: OK")
print("ALL STRUCTURAL ASSERTIONS PASSED")
PY
```

Captured output:

```
PARSE: OK (yaml.safe_load)
TRIGGERS keys: ['schedule', 'workflow_dispatch']
CRON: ['0 3 1 * *']
JOBS: ['fuzz', 'notify-failure']
runs-on: ['self-hosted', 'linux', 'x64', 'fuzz-dedicated']
timeout-minutes: 1500
permissions top: {}
permissions fuzz: {'contents': 'read'}
permissions notify: {'contents': 'read', 'issues': 'write'}
runner.environment guards: {'fuzz': True, 'notify-failure': True}
uses (parsed): ['actions/checkout@34e114876b0b11c390a56381ad16ebd13914f8d5', 'actions/upload-artifact@ea165f8d65b6e75b540449e92b4886f43607fa02']
pinned actions: 2 SHA pins with # v4 comment in raw text: OK
env.DURATION: 86400
matrix targets (7): ['fuzz_lexer', 'fuzz_parser', 'fuzz_compile', 'fuzz_resp', 'fuzz_storage_key', 'fuzz_signed_url', 'fuzz_workers_payload']
max-parallel: 1 fail-fast: False
fuzz env.FUZZ_TARGETS: ${{ matrix.target }}
concurrency: {'group': 'fuzz-24h-gate', 'cancel-in-progress': False}
secrets.* references: []
token refs present: github.token + GITHUB_TOKEN (built-in only)
notify: marker + gh comment + failure() + open/pending language: OK
harness invocation with $DURATION: OK
ALL STRUCTURAL ASSERTIONS PASSED
```

Notes on two verification-tooling details that are not workflow defects:

- PyYAML (YAML 1.1) parses the mapping key `on` as boolean `True`; GitHub Actions
  parses YAML 1.2 where it is the string `on`. The snippet accepts either key.
- The ` # v4` suffix on a pinned action is a YAML comment and is stripped by the
  parser, so it is asserted against the raw file text, not the parsed value.

## 3. Harness checks

```sh
bash -n scripts/run-fuzz-window.sh && echo 'bash -n: OK'
bash scripts/run-fuzz-window.sh --help >/dev/null && echo 'help: OK'
FUZZ_TARGETS=bogus_target bash scripts/run-fuzz-window.sh 1 2>&1; echo "exit=$?"
```

Output:

```
bash -n: OK
help: OK
ERROR: unknown fuzz target in FUZZ_TARGETS: bogus_target
exit=2
```

The unknown target fails closed before any fuzzing starts; the seven canonical
names are the only accepted subset values.

## 4. Conclusion and evidence required to close #156

The wave-11 workflow is present on `main` and structurally correct: triggers,
label gating plus `runner.environment` guard, least privilege, SHA-pinned
actions, `DURATION=86400`, a seven-target matrix, and no secret beyond
`GITHUB_TOKEN`. No workflow change was made because no real defect was found.
Two stale `RFC-0035` navigation references left by the renumber commit were
corrected to RFC-0036 in `docs/fuzz/README.md` and `docs/INDEX.md`.

This document does not and cannot close the gate. Issue #156 stays open with
`blocked`, and the `Fuzz 24h` row stays `pending`. The exact evidence required
to close #156 is listed in section 6 of `docs/fuzz/24h-gate-runbook.md` and in
`docs/fuzz/24h-gate-verification.md`: seven uninterrupted 86400 s runs at one
recorded commit on the dedicated host, zero or triaged-and-fixed crashes with a
fresh re-run, archived logs/metadata/timeline/filled `RESULTS.md`, the gate row
updated with date+commit+result path before moving to `pass`, the README blocker
text updated, and `Closes #156` in the change that carries the full-length
evidence.
