# Judge-Facing Benchmarks

Build and commit the benchmark harness first, then run from a clean repository:

```bash
make bench-final
```

Generated evidence:

- `bench/results.json`
- `bench/results.csv`
- `bench/raw_latency_samples.json`
- `bench/raw_rss_samples.json`
- `bench/environment.json`
- `bench/report.md`

Methodology is documented in `bench/methodology.md`. The existing in-process
benchmark implementation and historical optimization results remain under
`parity/`.
