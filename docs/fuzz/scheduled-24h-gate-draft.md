# Draft scheduled runner for the 24-hour fuzz gate (not enabled)

**Status: DRAFT / NOT ENABLED.** No file is added under `.github/workflows/`.
This note only describes how a funded runner would execute the gate defined in
`docs/fuzz/24h-gate-verification.md`. Publishing a workflow from this draft is
a separate decision and does not by itself satisfy or prove issue #156.

Public hosted runners are unsuitable: their job timeouts are far shorter than
86400 s and seven sequential targets need at least 168 h of wall time. The gate
must run on dedicated, self-hosted infrastructure.

## Host requirements

- Linux x86-64, kernel 6.0 or newer; no containers with throttled CPU unless
  the same limits are documented for every target.
- Quiet machine: no unrelated heavy load. Record CPU model, physical and
  logical cores, RAM, storage type and CPU governor before the run.
- Disk: enough free space for seven libFuzzer corpora, raw logs and crash
  artifacts; use a scratch volume so a full disk cannot corrupt the source
  tree.
- Toolchain: `fuzz/rust-toolchain.toml` selects floating `nightly`. Pin a dated
  nightly (for example `nightly-YYYY-MM-DD`) for the official run and use it
  consistently for all seven targets; record `rustc` and `cargo fuzz` versions.
- Workspace: clean tree at the stabilization commit with a synchronized
  `fuzz/Cargo.lock` (see the stale-lock finding in
  `docs/fuzz/wave9-window-evidence.md`), otherwise cargo mutates the tree.
- Isolation: either one host running the seven targets sequentially for at
  least 168 h, or seven hosts with CPU/RSS isolated and one full 86400 s window
  each. Record the allocation per target when running in parallel because
  `exec/s` and RSS are resource-sensitive.
- Reliability: power and network stability sufficient for an uninterrupted
  86400 s per target. Any log that ends without the final `DONE` and
  `Done N runs in M second(s)` lines requires a full-length restart of that
  target.

## Runner outline (illustrative YAML, not a workflow file)

```yaml
# DRAFT ONLY - do not copy into .github/workflows/ without an RFC/decision.
name: fuzz-24h-gate
on:
  workflow_dispatch:
  schedule:
    - cron: "0 3 1 * *"   # monthly; cadence is a policy decision
permissions:
  contents: read
jobs:
  fuzz:
    runs-on: [self-hosted, linux, x64, fuzz-dedicated]
    timeout-minutes: 10080   # 168 h plus setup; hosted runners cannot do this
    concurrency:
      group: fuzz-24h-gate
      cancel-in-progress: false
    strategy:
      fail-fast: false
      max-parallel: 1
      matrix:
        target:
          - fuzz_lexer
          - fuzz_parser
          - fuzz_compile
          - fuzz_resp
          - fuzz_storage_key
          - fuzz_signed_url
          - fuzz_workers_payload
    steps:
      - uses: actions/checkout@v4
      - name: Verify clean tree and pinned toolchain
        run: |
          test -z "$(git status --porcelain)"
          rustup toolchain install "$FUZZ_TOOLCHAIN"
          cargo +"$FUZZ_TOOLCHAIN" fuzz --version
      - name: Run one full-length target
        env:
          FUZZ_TOOLCHAIN: nightly-YYYY-MM-DD
          CARGO_INCREMENTAL: "0"
          CARGO_PROFILE_TEST_DEBUG: "0"
        run: |
          mkdir -p evidence
          cd fuzz
          set -o pipefail
          cargo +"$FUZZ_TOOLCHAIN" fuzz run "${{ matrix.target }}" \
            -- -max_total_time=86400 2>&1 | tee "../evidence/${{ matrix.target }}.log"
          echo "${PIPESTATUS[0]}" > "../evidence/${{ matrix.target }}.rc"
      - name: Archive logs and crash artifacts
        if: always()
        run: |
          # Upload evidence/<target>.log, .rc and any fuzz/artifacts/<target>/
          # input to the run record; never convert a missing log into a pass.
          echo "archive step: configure artifact storage before enabling"
      - name: Triage
        if: always()
        run: |
          # A non-zero .rc or any crash input blocks the gate. Open a GitHub
          # Issue for each crash, fix it with a regression test, and re-run the
          # affected target for a full 86400 s before closing the issue.
          echo "triage is manual; this step only preserves evidence"
```

## Post-run obligations

A scheduled run does not close issue #156 by itself. A human must verify the
acceptance criteria in `docs/fuzz/24h-gate-verification.md`, fill `RESULTS.md`,
update the `Fuzz 24h` release-gate row, and update the README blocker text.
Until then the row stays `pending` and issue #156 stays open with `blocked`.
