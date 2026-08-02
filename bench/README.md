# Judge-Facing Benchmarks

The final benchmark compares the original Python CLI and standalone Rust
release CLI on identical input bytes and equivalent flags.

```bash
make bench-final
```

## What is measured

- fresh process startup;
- end-to-end p50, p95, and p99 latency;
- median throughput;
- peak resident set size via GNU `time`;
- release binary size;
- deterministic corpus hashes and environment metadata.

Process launch, argument parsing, stdin reading, sorting, output generation,
and process termination are inside the timed region. Compilation and corpus
loading are outside it.

## Headline results

- startup p50: **17.1× faster** in Rust;
- default 1,000-item p50: **12.5× faster**;
- default 50,000-item p50: **2.2× faster**;
- default 1,000-item median RSS: **82.4% lower**;
- default 50,000-item median RSS: **10.1% lower**.

The narrower margin on the largest corpus is retained and reported.

## Evidence

- `bench/methodology.md`
- `bench/results.json`
- `bench/results.csv`
- `bench/raw_latency_samples.json`
- `bench/raw_rss_samples.json`
- `bench/environment.json`
- `bench/report.md`
- `bench/run.log`

Historical in-process optimization benchmarks remain under `parity/`; they are
not substituted for the final startup/p99/RSS evidence.
