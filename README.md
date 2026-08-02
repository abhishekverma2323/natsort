# natsort — Python → Rust Working Port

**Port Mortem 2026 · Track D · Python → Rust**

This repository contains a standalone Rust port of
[`SethMMorton/natsort`](https://github.com/SethMMorton/natsort) together with
reproducible evidence that the unmodified original Python test suite runs
against Rust-backed behavior.

> The original Python project remains untouched at the repository root.
> Production Rust code lives under `rust-port/`. The Python compatibility
> boundary is validation infrastructure—not a behavioral fallback.

## 60-second judge summary

| Requirement | Verified result |
|---|---:|
| Original source baseline | `b543bdce8771b6e7a7dae0c6745ddf7e80299797` |
| Original test integrity | **19 / 19 canonical Git blobs identical** |
| Original suite against Rust | **344 / 344 passed** |
| Rust library tests | **381 passed** |
| Rust Python-reference parity tests | **176 passed** |
| Shared Python/Rust CLI cases | **20 / 20 matched; both diffs empty** |
| Differential fuzzing | **11,727 / 11,727 matched; 0 divergences** |
| Fresh-clone verification | **Passed** |
| Docker build and full verification | **Passed** |
| First-party unsafe surface | **1 Windows-only FFI call site** |
| Rust release CLI size | **2,194,280 bytes (≈2.09 MiB)** |

Every headline number is traceable from [EVIDENCE_INDEX.md](EVIDENCE_INDEX.md).

## Build and verify

From the repository root:

```bash
make build
make verify
```

`make verify` performs all of the following:

1. verifies the original `tests/` tree against the pinned source commit;
2. checks Rust formatting;
3. runs all Rust unit and integration tests;
4. runs Clippy with warnings denied;
5. executes the complete unmodified original Python suite through the
   Rust-backed compatibility adapter.

Containerized reproduction:

```bash
docker build -t natsort-rust-port .
docker run --rm natsort-rust-port
```

Additional judge-facing commands:

```bash
make cli-diff         # Python CLI vs standalone Rust CLI
make fuzz             # deterministic differential smoke session
make bench-final      # startup, p50/p95/p99, throughput, peak RSS
make audit-final      # tests, LOC, dependencies, binary size, unsafe inventory
make submission-check # full suite + CLI/fuzz checks + evidence checks
```

## What was ported

The Rust crate covers the difficult behavior behind natural sorting, including:

- arbitrary-precision integers and decimals;
- signed numbers, floating-point values, scientific notation, and `NOEXP`;
- Unicode decimal, digit, and broader numeric characters;
- Unicode normalization, case folding, lowercase-first, and grouping modes;
- locale-aware numeric normalization and ICU4X collation;
- lexical path sorting and operating-system-aware ordering;
- typed mixed values, nested sequences, `None`, NaN, and infinities;
- ASCII, UTF-8, and Latin-1 byte decoding;
- natural keys, reusable key generators, sorted indexes, and lazy reordering;
- positional, newline, and NUL-delimited CLI input;
- CLI filters, exclusions, ranges, reverse order, paths, and numeric modes.

The implementation is a typed Rust redesign rather than a line-for-line copy
of Python internals.

## Proof architecture

```text
Pinned upstream source commit
        │
        ├── canonical Git-blob integrity check ──► original tests unchanged
        │
        ▼
Unmodified original pytest suite
        │
        ▼
Thin Python transport / callback / representation boundary
        │
        ▼
Rust original-suite-adapter binary
        │
        ▼
Rust modules: options → text/path/locale → tokenization
              → typed keys → stable comparison → APIs / CLI
```

The Python boundary may serialize values, invoke arbitrary Python callbacks,
preserve Python-specific wrappers, and adapt host Unicode-version differences.
Sorting, parsing, numeric ordering, option selection, path/locale decisions,
CLI filtering, and range validation are answered by Rust.

The standalone Rust library and CLI have no Python runtime dependency.

## Behavioral equivalence evidence

### Unmodified original suite

```text
19 / 19 original test files match the pinned source commit
344 passed, 3 warnings
PYTEST_EXIT_CODE=0
```

Artifacts:

- `parity/test_hashes/verification.txt`
- `parity/test_hashes/source.sha256`
- `parity/test_hashes/submission.sha256`
- `parity/evidence/genuine_full_suite_final.txt`
- `parity/evidence/fresh_clone_verify.txt`
- `parity/evidence/docker_verify.txt`

### Explicit CLI output diff

Twenty shared CLI cases cover successful output and error behavior:

```text
17 successful cases: exact exit code + stdout bytes + stderr bytes
3 error cases: exact exit/stdout + normalized final diagnostic
20 / 20 matched
0 mismatches
both unified diff files are empty
```

Artifacts: `parity/evidence/cli/`.

### Differential fuzzing

```text
Seed: 20260802
Fixed-count run: 5,000 / 5,000 matched
120-second run: 6,727 / 6,727 matched
Combined: 11,727 cases
Divergences: 0
Modes: default, float, real, signed_int, float_noexp, path
```

Artifacts: `fuzz/log.txt`, `fuzz/results.json`, and `fuzz/evidence/`.

## Performance

The judge-facing benchmark launches fresh processes and includes startup,
argument parsing, stdin reading, sorting, output generation, and termination.
Python and Rust receive identical corpus bytes and equivalent flags.

| Scenario | Rust p50 speedup | Rust p99 speedup |
|---|---:|---:|
| Empty-input startup | **17.1×** | **13.8×** |
| Default, 1,000 items | **12.5×** | **10.2×** |
| Float, 1,000 items | **13.0×** | **9.3×** |
| Real, 1,000 items | **13.0×** | **9.3×** |
| Path, 1,000 items | **12.8×** | **13.5×** |
| Locale, 1,000 items | **14.3×** | **8.2×** |
| Default, 10,000 items | **8.0×** | **6.9×** |
| Default, 50,000 items | **2.2×** | **3.9×** |

Median peak-RSS reduction:

| Corpus | Python | Rust | Reduction |
|---|---:|---:|---:|
| Default 1,000 | 17,496 KiB | 3,076 KiB | **82.4%** |
| Default 10,000 | 20,136 KiB | 7,928 KiB | **60.6%** |
| Default 50,000 | 31,704 KiB | 28,504 KiB | **10.1%** |

Large-input gains are reported without hiding the narrower margin. Raw samples,
environment metadata, corpus hashes, and methodology are committed under
`bench/`.

## Safety and honest boundaries

The generated audit reports:

```text
Rust production source:       19 files, 11,127 physical lines
Generated Unicode table:       1 file,     457 physical lines
Python compatibility boundary: 11 files,  1,910 physical lines
Direct Cargo dependencies:     7
Locked Cargo packages:         49
Unsafe blocks:                 1
Foreign ABI declarations:      1
Unsafe functions:              0
```

The only first-party unsafe call is the Windows-only `StrCmpLogicalW` FFI
boundary used for native Windows logical filename ordering. CI permits exactly
that documented boundary and rejects additional unsafe occurrences.

The port intentionally does **not** claim:

- identical Python internal tuple shapes;
- the complete Python codec registry;
- lossless handling of arbitrary non-UTF-8 paths;
- zero first-party unsafe;
- performance metrics outside the committed methodology.

See [HONEST_NUMBERS.md](HONEST_NUMBERS.md) and
[UNSAFE_AUDIT.md](UNSAFE_AUDIT.md).

## Repository map

```text
natsort/
├── natsort/                    # Original Python implementation, unchanged
├── tests/                      # Original Python tests, unchanged
├── rust-port/
│   ├── src/                    # Rust library, CLI, and adapter
│   ├── tests/python_parity.rs  # Rust/Python-reference integration tests
│   ├── README.md
│   └── ARCHITECTURE.md
├── parity/
│   ├── original_suite_adapter/ # Thin Python compatibility boundary
│   ├── cli/                    # Explicit CLI equivalence harness
│   ├── test_hashes/            # Canonical source/submission manifests
│   └── evidence/               # Working-port, CLI, Docker, and audit evidence
├── fuzz/                       # Harness, final logs, summaries, checksums
├── bench/                      # Methodology, raw samples, p99/RSS results
├── audit/                      # Reproducible honest-number generator
├── DECISIONS.md
├── PORT_MORTEM_2026.md
├── EVIDENCE_INDEX.md
├── HONEST_NUMBERS.md
├── Dockerfile
└── Makefile
```

## Documentation

- [Evidence index](EVIDENCE_INDEX.md)
- [Migration report](PORT_MORTEM_2026.md)
- [Engineering decisions](DECISIONS.md)
- [Rust crate guide](rust-port/README.md)
- [Rust architecture](rust-port/ARCHITECTURE.md)
- [Python-test coverage map](parity/PYTHON_TEST_COVERAGE.md)
- [Benchmark report](bench/report.md)
- [Honest numbers](HONEST_NUMBERS.md)
- [Unsafe audit](UNSAFE_AUDIT.md)

The original upstream project README is preserved as [README.rst](README.rst).
