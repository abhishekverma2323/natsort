# Python vs Rust Natural-Sort Benchmark

The benchmark measures only in-process sorting. Dataset loading, process startup, and Rust compilation are outside the timed region.

## Environment

- Generated: `2026-08-01T17:59:23.048353+00:00`
- Platform: `Linux-6.18.33.2-microsoft-standard-WSL2-x86_64-with-glibc2.43`
- Logical CPUs: `12`
- Seed: `20260801`
- Warm-up runs: `2`
- Measured runs: `7`
- Rust profile: `release`

## Results

| Mode | Entries | Python median (ms) | Rust median (ms) | Speedup |
|---|---:|---:|---:|---:|
| default | 1,000 | 4.278 | 1.924 | 2.22× |
| default | 10,000 | 55.012 | 25.955 | 2.12× |
| default | 50,000 | 265.279 | 144.597 | 1.83× |
| float | 1,000 | 9.473 | 2.308 | 4.10× |
| float | 10,000 | 133.306 | 24.418 | 5.46× |
| float | 50,000 | 545.225 | 141.648 | 3.85× |
| real | 1,000 | 13.039 | 2.520 | 5.17× |
| real | 10,000 | 100.504 | 22.488 | 4.47× |
| real | 50,000 | 596.425 | 142.044 | 4.20× |
| path | 1,000 | 18.692 | 3.393 | 5.51× |
| path | 10,000 | 220.798 | 43.824 | 5.04× |
| path | 50,000 | 1144.832 | 238.685 | 4.80× |
| locale | 1,000 | 8.280 | 5.455 | 1.52× |
| locale | 10,000 | 103.177 | 61.857 | 1.67× |
| locale | 50,000 | 594.374 | 335.951 | 1.77× |

## Median speedup by mode

| Mode | Median speedup |
|---|---:|
| default | 2.12× |
| float | 4.10× |
| real | 4.47× |
| path | 5.04× |
| locale | 1.67× |

## Methodology

- Identical deterministic UTF-8 datasets are used by both implementations.
- Each implementation performs warm-up runs before measurement.
- Every measured run sorts the same unsorted in-memory input.
- Reported time is the median of the measured runs.
- Python uses the repository virtual environment and `natsorted`.
- Rust uses `natsorted_with_options` compiled with `--release`.
- Locale mode uses each implementation's system locale configuration.
