# Performance: Criterion + Flamegraphs

## Benchmarking (Criterion)

- Run all Criterion benchmarks:
  - `cargo bench`
- Run only the fingerprinting benchmarks:
  - `cargo bench --bench fingerprinting`

Notes:
- Benchmarks live under `benches/` and use deterministic synthetic inputs (no I/O).
- Keep setup outside `b.iter` and use `black_box` to avoid dead-code elimination.

## Profiling (Flamegraphs)

### Install tooling

- Install `cargo-flamegraph`:
  - `cargo install flamegraph`

Platform notes:
- Linux: needs `perf` and permissions to sample.
- macOS: `cargo flamegraph` uses system tooling; you may be prompted for permissions.

### Profile a standalone binary workload

This project includes a deterministic workload *bench target* that exercises peak extraction and fingerprint building:

- `cargo flamegraph --bench fingerprint_workload -- 20 1`
  - args: `<seconds> <iterations>`

### Profile Criterion benchmarks

- `cargo flamegraph --bench fingerprinting`

Tip: If you want shorter runs, pass Criterion CLI args to the bench binary:
- `cargo flamegraph --bench fingerprinting -- --sample-size 10 --measurement-time 3`

## How to use results

- Use Criterion to confirm “it got faster/slower”.
- Use flamegraphs to see *where* time/allocations are concentrated.
- Don’t optimize code that doesn’t show up in either.
