# Port Mortem 2026 Submission — natsort Python → Rust

## Submission identity

| Field | Value |
|---|---|
| Source project | `SethMMorton/natsort` |
| Source baseline | `b543bdce8771b6e7a7dae0c6745ddf7e80299797` |
| Migration track | Track D · Python → Rust |
| Port repository | `abhishekverma2323/natsort` |
| Submission branch | `python-to-rust-port` |
| Rust crate | `rust-port` |
| License | MIT |
| Approach | Behavior-first, oracle-driven, typed Rust redesign |

## Executive summary

This submission ports the observable behavior of Python's mature `natsort`
library into a standalone Rust crate and CLI.

The project is not a toy `"file2" < "file10"` comparator. It implements:

- arbitrary-precision integers and decimals;
- signed values, floating-point values, scientific notation, and `NOEXP`;
- Unicode decimal, digit, and broad numeric-character behavior;
- Unicode normalization and case modes;
- locale-aware numeric normalization and ICU4X collation;
- cross-platform path parsing and OS-aware ordering;
- mixed typed values and nested sequences;
- bytes with explicit ASCII, UTF-8, and Latin-1 decoding;
- key generation, index sorting, and lazy index reordering;
- Python-compatible CLI input, output, filters, exclusions, and errors.

The original Python implementation and tests are retained unchanged. They act
as the behavioral oracle; production Rust remains independent of Python.

## The migration challenge

Natural sorting combines several domains whose edge cases interact:

```text
dynamic Python values
+ arbitrary numeric precision
+ Unicode normalization and numeric categories
+ locale collation and number punctuation
+ paths and operating-system conventions
+ stable ordering for equivalent values
+ CLI byte-level behavior
```

A fixed-width integer parser or ASCII-only split is insufficient. For example,
the port must distinguish or equate values correctly across:

- huge integers beyond `u64`;
- `1`, `01`, and `1.0` under stable ordering;
- signed zero, infinities, NaN, and extreme exponents;
- Arabic-Indic, Devanagari, fullwidth, circled, fractional, and Roman numerics;
- German, French, English, Czech, C/POSIX, and system-locale profiles;
- nested directories, Windows separators, hidden files, and suffix heuristics;
- text, bytes, direct numbers, `None`, and nested sequences.

The Rust implementation addresses these as typed, independently tested modules.

## A critical engineering correction

During development, an early Python-side compatibility facade could make tests
pass without proving a real port. That approach was rejected and removed.

The final validation architecture is:

```text
Unmodified original tests
        ↓
Thin Python transport / callback / representation boundary
        ↓
Rust original-suite-adapter executable
        ↓
Rust sorting, parsing, numeric, Unicode, locale, path, and CLI logic
```

The compatibility boundary does not contain a copied natural-sorting
algorithm. It may serialize Python values, invoke arbitrary Python callbacks,
and preserve Python-specific wrappers, but behavioral decisions are returned
by Rust.

This redesign is why the final **344 / 344** result is meaningful.

## Working-port proof

### Original tests are unmodified

The repository pins the source baseline and compares every tracked file in
`tests/` using canonical Git blob content:

```text
source_test_files=19
submission_test_files=19
problems=0
```

This avoids false differences caused by CRLF/LF checkout conversion while
still detecting added, removed, staged, unstaged, or modified test files.

### Complete original suite runs against Rust

```text
344 passed, 3 warnings
PYTEST_EXIT_CODE=0
```

The runner prints both the Rust-backed Python package path and the Rust adapter
binary path before invoking the original suite.

### Independent reproduction

The same verification passed in:

- the development checkout;
- a fresh single-branch clone;
- the supplied Docker image.

The Docker run produced:

```text
381 Rust library tests passed
176 Rust Python-reference parity tests passed
344 original tests passed
DOCKER_VERIFY_EXIT_CODE=0
```

Primary artifacts are indexed in `EVIDENCE_INDEX.md`.

## Actual behavioral equivalence

### Rust tests

```text
381 Rust library tests
176 Python-reference integration tests
0 failures
```

The integration suite contains committed reference behavior from the original
Python implementation and covers flags, numeric families, Unicode, locales,
paths, mixed values, bytes, OS profiles, and CLI behavior.

### Explicit CLI diff

The original Python CLI and standalone Rust release CLI receive identical
arguments, stdin bytes, locale, and working directory.

```text
20 shared cases
17 successful cases compared byte-for-byte
3 error cases compared by exit/stdout and normalized final diagnostic
20 matched
0 mismatches
both unified diffs empty
```

Raw stdout, stderr, and exit codes remain committed for inspection.

### Differential fuzzing

Final session:

```text
Seed: 20260802
Fixed-count: 5,000 / 5,000 matched
Duration: 120.022739 seconds
Duration cases: 6,727 / 6,727 matched
Combined: 11,727 cases
Divergences: 0
```

All six modes were exercised in both runs:

```text
default · float · real · signed_int · float_noexp · path
```

Failures would create a reproduction payload containing seed, case, mode,
arguments, stdin bytes, expected output, actual output, exit code, and stderr.

## Architecture

### Core pipeline

```text
SortOptions / AlgorithmFlags
        ↓
text, Unicode, locale, and path transforms
        ↓
numeric-token recognition
        ↓
typed natural keys
        ↓
stable cached-key comparison
        ↓
sorted clones, values, or indexes
```

### Key design choices

- **Typed dynamic values:** `NaturalValue` replaces arbitrary Python object
  dispatch without using `Any` or downcasting.
- **Arbitrary precision:** normalized numeric parts preserve huge integers,
  high-precision decimals, and extreme exponents.
- **Stable equivalence:** equal natural keys retain original input order.
- **Key caching:** inputs are tokenized once before comparison.
- **Unicode separation:** decimal, digit, and broader numeric categories remain
  distinct.
- **ICU4X locale keys:** explicit locale profiles replace global Python locale
  state.
- **Dedicated paths and OS profiles:** generic lexical paths and native
  OS-aware ordering remain separate APIs.
- **Safe alternatives:** fallible operations expose `Result` forms.

Detailed module responsibilities are in `rust-port/ARCHITECTURE.md`.

## Compatibility matrix

| Behavior | Rust status | Main evidence |
|---|---|---|
| Natural integer sorting | Direct | unit + parity + fuzz |
| Leading-zero stability | Direct | unit + original suite |
| Arbitrary-size numbers | Direct | unit + original suite |
| Float/real/sign/exponent/NOEXP | Direct | unit + parity + fuzz |
| Unicode decimal/digit/numeric | Direct | generated data + exhaustive tests |
| Case and normalization modes | Direct/Semantic | unit + original suite |
| Locale sorting | Direct/Semantic | ICU profiles + parity |
| Czech issue #140 regression | Direct | dedicated corpus |
| Path strings | Direct | unit + parity + fuzz |
| Direct `Path`/`PathBuf` input | Rust equivalent | dedicated APIs |
| OS-aware sorting | Direct/Semantic | Windows/Unix profile tests |
| Mixed and nested values | Rust equivalent | `NaturalValue` |
| Bytes decoding | Direct/Semantic | ASCII, UTF-8, Latin-1 |
| Key generation | Rust equivalent | typed keys/generators |
| Index sorting and reordering | Direct/Rust equivalent | eager + lazy APIs |
| CLI | Direct/Semantic | original suite + explicit diff |
| Python private factories | Replaced | typed Rust modules |
| Python tuple key shape | Language-specific | `NaturalKey` |

Full file-level inventory: `parity/PYTHON_TEST_COVERAGE.md`.

## Performance and memory

Two benchmark layers are retained:

1. historical in-process optimization benchmarks under `parity/`;
2. final judge-facing end-to-end CLI benchmarks under `bench/`.

The final benchmark includes process startup, argument parsing, stdin reading,
sorting, output generation, and termination.

### Latency

| Scenario | Rust p50 speedup | Rust p99 speedup |
|---|---:|---:|
| Startup | 17.1× | 13.8× |
| Default 1,000 | 12.5× | 10.2× |
| Float 1,000 | 13.0× | 9.3× |
| Real 1,000 | 13.0× | 9.3× |
| Path 1,000 | 12.8× | 13.5× |
| Locale 1,000 | 14.3× | 8.2× |
| Default 10,000 | 8.0× | 6.9× |
| Default 50,000 | 2.2× | 3.9× |

### Peak RSS

| Corpus | Python median | Rust median | Reduction |
|---|---:|---:|---:|
| Default 1,000 | 17,496 KiB | 3,076 KiB | 82.4% |
| Default 10,000 | 20,136 KiB | 7,928 KiB | 60.6% |
| Default 50,000 | 31,704 KiB | 28,504 KiB | 10.1% |

The reduced advantage on the largest corpus is reported rather than hidden.
Raw samples and environment details are committed.

## Safety and dependency audit

Generated audit:

```text
First-party unsafe blocks:       1
Foreign ABI declarations:       1
Unsafe functions:               0
Direct Cargo dependencies:      7
Locked Cargo packages:          49
Rust release binary:            2,194,280 bytes
```

The only unsafe operation is a narrowly scoped Windows-only call to
`StrCmpLogicalW`. Its UTF-16 buffer lifetime and ownership invariants are
documented adjacent to the call. `rust-port/scripts/check_unsafe.sh` causes CI
to fail if the first-party unsafe surface expands.

## Reproduction commands

```bash
make build
make verify
make cli-diff
make fuzz
make bench-final
make audit-final
```

Full non-mutating submission gate:

```bash
make submission-check
```

Docker:

```bash
docker build -t natsort-rust-port .
docker run --rm natsort-rust-port
```

## Repository guide

| Location | Purpose |
|---|---|
| `rust-port/src/` | Rust library, CLI, compatibility adapter |
| `rust-port/tests/python_parity.rs` | Python-reference integration suite |
| `parity/original_suite_adapter/` | Thin Python test boundary |
| `parity/test_hashes/` | Canonical original/submission test manifests |
| `parity/evidence/` | Working-port, CLI, Docker, and audit evidence |
| `fuzz/` | Differential harness, logs, checksums, summaries |
| `bench/` | Methodology, raw samples, p99/RSS results |
| `audit/` | Reproducible metrics generator |
| `DECISIONS.md` | Engineering trade-offs and rejected alternatives |
| `EVIDENCE_INDEX.md` | Claim-to-proof map |

## Honest limitations

- The Python compatibility boundary uses subprocess IPC and is not presented
  as production performance.
- Error-message comparison normalizes executable display names, line endings,
  wrapping whitespace, and Python-version-specific quote rendering inside
  `argparse` choice lists; rejected values and ordered choices still match.
- Built-in decoders are ASCII, UTF-8, and Latin-1—not Python's entire codec
  registry.
- Non-UTF-8 Rust paths use documented lossy text conversion.
- Locale and OS-sort behavior can depend on platform profile and locale data;
  Docker pins the Linux validation environment.
- Native Windows system-profile sorting retains one documented FFI call.
- Python's private factories and exact tuple key layouts are intentionally
  replaced by typed Rust structures.

## Why this submission is credible

The project does not ask judges to trust a percentage written in a README.
It supplies independent, inspectable proof layers:

```text
canonical source/test hashes
+ complete original suite
+ Rust unit and parity tests
+ exact CLI output diffs
+ differential fuzzing
+ fresh-clone verification
+ Docker verification
+ raw benchmark samples
+ generated safety/metrics audit
```

## Final claim

This is a standalone, behaviorally faithful Rust migration of `natsort` with
the complete unmodified original suite passing against Rust-backed behavior.
The difficult numeric, Unicode, locale, path, mixed-value, and CLI surfaces are
covered by independent evidence, while language-specific differences and the
single native FFI boundary are documented explicitly.
