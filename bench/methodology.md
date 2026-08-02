# Benchmark Methodology

## Current in-process benchmark

The existing benchmark uses identical deterministic UTF-8 datasets for Python
and Rust, performs warm-up runs, and times repeated sorting of the same
unsorted in-memory input. Rust is compiled with the release profile.

## Final judge-facing extension

The final benchmark run will add:

1. Cold process startup latency.
2. End-to-end p50, p95, and p99 latency.
3. Peak resident set size (RSS).
4. Throughput on medium and large deterministic corpora.
5. Binary size.
6. Exact commit, toolchain, operating system, CPU, corpus hashes, warm-up count,
   measured-run count, and raw samples.

Python and Rust will receive the same input corpus and equivalent algorithm
flags. Compilation and corpus generation will remain outside measured regions.
