# Judge-Facing Benchmark Results

All measurements compare the original Python CLI with the standalone
Rust release CLI on identical input bytes and equivalent flags.

## End-to-end latency

| Scenario | Impl | Samples | p50 ms | p95 ms | p99 ms | Median items/s |
|---|---:|---:|---:|---:|---:|---:|
| startup_empty | python | 100 | 118.621 | 306.718 | 366.159 | 0.0 |
| startup_empty | rust | 100 | 6.951 | 18.433 | 26.491 | 0.0 |
| default_1000 | python | 100 | 115.696 | 250.175 | 283.284 | 8643.3 |
| default_1000 | rust | 100 | 9.262 | 22.871 | 27.891 | 107964.6 |
| float_1000 | python | 50 | 126.725 | 250.720 | 379.479 | 7891.1 |
| float_1000 | rust | 50 | 9.711 | 13.662 | 40.832 | 102975.6 |
| real_1000 | python | 50 | 126.420 | 293.640 | 334.210 | 7910.1 |
| real_1000 | rust | 50 | 9.693 | 24.091 | 35.926 | 103162.9 |
| path_1000 | python | 50 | 143.403 | 289.248 | 321.343 | 6973.3 |
| path_1000 | rust | 50 | 11.168 | 23.342 | 23.793 | 89544.8 |
| locale_1000 | python | 50 | 254.058 | 321.048 | 343.741 | 3936.1 |
| locale_1000 | rust | 50 | 17.743 | 34.444 | 41.778 | 56359.2 |
| default_10000 | python | 30 | 290.632 | 361.270 | 394.934 | 34407.8 |
| default_10000 | rust | 30 | 36.319 | 54.275 | 56.867 | 275339.7 |
| default_50000 | python | 10 | 284.070 | 533.169 | 533.169 | 176013.1 |
| default_50000 | rust | 10 | 128.677 | 137.231 | 137.231 | 388570.1 |

## Peak RSS

| Scenario | Impl | Samples | Median KiB | Max KiB |
|---|---:|---:|---:|---:|
| default_1000 | python | 5 | 17496.0 | 17612 |
| default_1000 | rust | 5 | 3076.0 | 3120 |
| default_10000 | python | 5 | 20136.0 | 20212 |
| default_10000 | rust | 5 | 7928.0 | 7960 |
| default_50000 | python | 5 | 31704.0 | 31732 |
| default_50000 | rust | 5 | 28504.0 | 28528 |

Percentiles use the nearest-rank method. Process launch, argument
parsing, stdin reading, sorting, and output generation are inside
the timed region; compilation and corpus loading are outside it.
Output is redirected to `/dev/null` equally for both programs.
