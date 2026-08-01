# Python vs Rust Natural-Sort Benchmark

The benchmark measures only in-process sorting. Dataset loading, process startup, and Rust compilation are outside the timed region.

## Environment

- Generated: `2026-08-01T16:56:11.000819+00:00`
- Platform: `Linux-6.18.33.2-microsoft-standard-WSL2-x86_64-with-glibc2.43`
- Logical CPUs: `12`
- Seed: `20260801`
- Warm-up runs: `1`
- Measured runs: `3`
- Rust profile: `release`

## Results

| Mode | Entries | Python median (ms) | Rust median (ms) | Speedup |
|---|---:|---:|---:|---:|
| default | 1,000 | 3.500 | 12.892 | 0.27× |
| float | 1,000 | 9.980 | 12.465 | 0.80× |
| real | 1,000 | 10.177 | 13.255 | 0.77× |
| path | 1,000 | 18.722 | 17.044 | 1.10× |
| locale | 1,000 | 11.185 | 97.808 | 0.11× |

## Median speedup by mode

| Mode | Median speedup |
|---|---:|
| default | 0.27× |
| float | 0.80× |
| real | 0.77× |
| path | 1.10× |
| locale | 0.11× |

## Methodology

- Identical deterministic UTF-8 datasets are used by both implementations.
- Each implementation performs warm-up runs before measurement.
- Every measured run sorts the same unsorted in-memory input.
- Reported time is the median of the measured runs.
- Python uses the repository virtual environment and `natsorted`.
- Rust uses `natsorted_with_options` compiled with `--release`.
- Locale mode uses each implementation's system locale configuration.
