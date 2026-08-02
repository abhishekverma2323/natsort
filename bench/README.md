# Benchmarks

The existing deterministic in-process benchmark implementation is maintained in:

- `parity/run_benchmarks.py`
- `parity/benchmark_python.py`
- `parity/benchmark_data/`
- `parity/benchmark_results.json`
- `parity/BENCHMARK_RESULTS.md`

The judge-facing benchmark package in this directory will additionally record
startup latency, p50/p95/p99 latency, peak RSS, throughput, binary size, exact
commands, corpus hashes, and machine metadata.
