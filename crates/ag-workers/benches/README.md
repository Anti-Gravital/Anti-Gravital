# `ag-workers` benchmarks

Microbenchmarks for the in-memory hot paths, run with `criterion`. They follow
`CLAUDE.md` rule 17: no number is published without full context (hardware, OS,
Rust version, commit, configuration, methodology, repetitions, variance).

## Current suite

### `queue_throughput.rs`

| Benchmark | What it measures |
| --- | --- |
| `payload_encode` | `rmp-serde` encoding of a typed payload (per-enqueue cost, §9.3). |
| `payload_decode` | `rmp-serde` decoding back into the typed payload (per-dispatch cost). |
| `enqueue_memory` | Admission + envelope build + insert into a fresh `MemoryQueue` (§12, §18). |
| `lease_memory_batch_100` | Leasing a batch of 100 from a queue pre-filled with 1000 ready jobs (§13.2 in-memory equivalent). |

The PostgreSQL backend is network/DB bound and is **not** benchmarked here: the
default environment has no database. Its throughput is measured against a live
instance as part of the durable-backend verification work (Issue #108).

## How to run

From the repository root:

```sh
# Compile only (verifies the bench builds; this is what CI checks).
cargo bench -p ag-workers --no-run

# Full run (local, on quiet hardware).
cargo bench -p ag-workers
```

## Recording results

When publishing numbers, record alongside them, per rule 17:

- CPU model, core count, RAM.
- OS and kernel version.
- `rustc --version` and the exact commit SHA.
- Backend and configuration used (here: in-memory, defaults).
- `criterion` sample size and the reported mean +/- standard deviation.

Do not commit raw `target/criterion/` output; publish curated results under
`docs/benchmarks/` with the context above.
