# Benchmark Methodology

## Goal

Compare the original Python CLI and the standalone Rust release CLI with
identical input bytes and equivalent options. The benchmark reports startup,
end-to-end latency percentiles, throughput, peak RSS, and binary size.

## Implementations

- Python: `python -X utf8 -m natsort`
- Rust: `rust-port/target/release/natsort`

The Python module path and Rust binary SHA-256 are recorded in
`bench/environment.json`.

## Timing method

Each latency sample launches a fresh process. The timed region includes process
startup, argument parsing, stdin reading, natural sorting, output generation,
and process termination. Output is redirected to `/dev/null` equally for both
implementations. Compilation and corpus loading are outside the timed region.

Runs alternate Python-first and Rust-first ordering to reduce time-order bias.
Warm-up executions occur before samples. Percentiles use the nearest-rank
method.

## Scenario matrix

- Empty stdin: 100 samples, process-startup focused.
- Default 1,000-item corpus: 100 samples.
- Float, real, path, and locale 1,000-item corpora: 50 samples each.
- Default 10,000-item corpus: 30 samples.
- Default 50,000-item corpus: 10 samples.

Every corpus is deterministic and its SHA-256 is stored with the results.

## Peak RSS

GNU `time` `%M` measures maximum resident set size in KiB. Python and Rust each
receive five alternating runs on the deterministic 1,000-, 10,000-, and
50,000-item default corpora.

## Interpretation

The CLI benchmark intentionally includes Python interpreter startup and Rust
binary startup because judges requested startup and end-to-end behavior. It
does not claim that the compatibility test adapter represents production
performance. Existing in-process benchmark data remains under `parity/`.
