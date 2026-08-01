# Port Mortem 2026 Submission — natsort Python → Rust

## Submission identity

| Field | Value |
|---|---|
| Source project | `SethMMorton/natsort` |
| Migration track | Python → Rust |
| Rust crate | `rust-port` |
| License | MIT |
| Working branch | `python-to-rust-port` |
| Migration approach | Behavior-first, oracle-driven, typed Rust redesign |

## Executive summary

This submission migrates the core observable behavior of Python's `natsort`
library into an idiomatic Rust crate and CLI.

The port handles substantially more than the basic `"file2" < "file10"`
example. It includes:

- arbitrary-precision integer and decimal comparison,
- signed numbers and scientific notation,
- Unicode decimal, digit, and numeric characters,
- locale-aware and Czech-specific collation,
- path-aware and OS-aware ordering,
- mixed typed values and nested sequences,
- bytes with ASCII, UTF-8, and Latin-1 decoders,
- Python-compatible CLI behavior,
- sorting indexes and lazy index reordering,
- cross-platform CI,
- deterministic Python-vs-Rust differential fuzzing,
- reproducible performance benchmarks.

The original Python package remains in the repository and is used as the
behavioral oracle.

## Why this migration is difficult

Natural sorting is not equivalent to splitting ASCII digits and calling
`parse::<u64>()`.

Compatibility requires correct behavior for:

- values larger than fixed-width integers,
- numerically equivalent spellings with stable order,
- signed zero and extreme exponents,
- Unicode numeric categories beyond `0`–`9`,
- locale grouping and decimal separators,
- ICU collation and language-specific ordering,
- Windows and Unix path conventions,
- mixed bytes, text, numbers, `None`, NaN, and infinities,
- Python CLI input/output edge cases.

The Rust implementation addresses these as typed subsystems rather than a
single ad-hoc comparator.

## Evidence

### Automated tests

Latest verified local result:

```text
377 Rust library tests passed
176 Python-reference parity tests passed
0 failed
```

The original Python suite is retained as the source behavior inventory.

### Differential fuzzing

Extended local run:

```text
Seed: 20260801
Cases: 1,000
Result: 1,000 / 1,000 passed
```

Bounded CI run:

```text
Seed: 20260801
Cases: 200
Result: 200 / 200 passed
Modes: default, float, real, signed_int, float_noexp, path
```

A failure produces a JSON reproduction containing the seed, exact case,
arguments, stdin bytes, Python result, Rust result, return code, and stderr.

### Cross-platform CI

The `Rust CI` workflow runs:

- formatting,
- all-target tests,
- Clippy with warnings denied,
- release CLI build,
- release benchmark build,
- Linux CLI smoke test,
- Windows CLI smoke test,
- Ubuntu Python-vs-Rust differential fuzzing.

### Performance

Rust is faster in all 15 committed benchmark configurations.

| Mode | Median speedup |
|---|---:|
| Default | 2.12× |
| Float | 4.10× |
| Real | 4.47× |
| Path | 5.04× |
| Locale | 1.67× |

Largest individual measured result:

```text
Path mode, 1,000 entries: 5.51× faster
```

The comparison uses identical deterministic datasets and measures only
in-process sorting after warm-up.

## Compatibility overview

| Python behavior | Rust status | Evidence |
|---|---|---|
| Natural integer sorting | Direct | unit + parity + fuzz |
| Leading zeros and stable ties | Direct | unit + parity |
| Arbitrary-size integers | Direct | unit + parity |
| Float, real, signs, exponent, NOEXP | Direct | unit + parity + fuzz |
| Unicode decimals/digits/numerics | Direct | generated data + exhaustive scalar scan |
| Case transforms and normalization | Direct/Semantic | unit + parity |
| Locale alphabetic/numeric sorting | Direct | ICU profiles + parity |
| Czech `cs_CZ` regression | Direct | issue #140 corpus |
| Path strings | Direct | unit + parity + fuzz |
| Direct `Path`/`PathBuf` input | Rust API equivalent | dedicated APIs |
| OS-aware sorting | Direct/Semantic | Windows/Unix profiles |
| Mixed and nested values | Typed Rust equivalent | `NaturalValue` tests |
| Bytes decoding | Direct/Semantic | ASCII, UTF-8, Latin-1 |
| Key generation | Typed Rust equivalent | key/keygen tests |
| Index sorting | Direct | unit + parity |
| Lazy `order_by_index` | Rust iterator equivalent | iterator tests |
| CLI | Mostly direct | unit + parity + smoke + fuzz |
| Python internal factories | N/A | replaced by typed modules |
| Python key tuple representation | Language-specific | `NaturalKey` |

Full mapping: `parity/PYTHON_TEST_COVERAGE.md`.

## Architecture highlights

The migration is divided into independently tested layers:

```text
options
  → text/path/locale transformation
  → numeric tokenization
  → typed natural keys
  → stable index comparison
  → cloned values or sorted indexes
```

Important design decisions:

- Cache keys before sorting to avoid repeated tokenization.
- Preserve arbitrary numeric precision.
- Keep mixed values typed through `NaturalValue`.
- Use ICU4X for locale sort keys.
- Expose dedicated path and OS-sort APIs.
- Pair convenience APIs with safe `Result` alternatives.
- Treat Python as an oracle, not as an architecture template.

Detailed design: `rust-port/ARCHITECTURE.md`.

## Repository guide

```text
rust-port/src/                     Rust library and CLI implementation
rust-port/tests/python_parity.rs   Python-reference integration suite
rust-port/README.md                User-facing crate guide
rust-port/ARCHITECTURE.md          Technical design
parity/differential_fuzz.py        Live Python-vs-Rust fuzz oracle
parity/PYTHON_TEST_COVERAGE.md     Compatibility inventory
parity/BENCHMARK_RESULTS.md        Human-readable performance report
parity/benchmark_results.json      Raw benchmark evidence
.github/workflows/rust-ci.yml      Linux, Windows, and fuzz CI
```

## Reproduction commands

### Verify the Rust port

```bash
cd rust-port
cargo fmt --all -- --check
cargo test --all-targets
cargo clippy --all-targets -- -D warnings
```

### Build and demonstrate the CLI

```bash
cargo build --release --bin natsort

./target/release/natsort \
  file10 file2 file1 value20 value3
```

### Run differential fuzzing

From the repository root with the Python package installed:

```bash
python parity/differential_fuzz.py \
  --cases 1000 \
  --seed 20260801
```

### Re-run benchmarks

```bash
python parity/run_benchmarks.py \
  --seed 20260801 \
  --sizes 1000 10000 50000
```

## Suggested judge demo

A compact five-minute walkthrough:

1. **Show the migration boundary**
   - original Python package in the root,
   - Rust crate under `rust-port/`.

2. **Run basic and advanced CLI examples**
   - natural filenames,
   - signed scientific numbers,
   - mixed path separators.

3. **Show typed Rust APIs**
   - `NaturalValue`,
   - `natsorted_paths`,
   - lazy `order_by_index_iter`.

4. **Run tests**
   - `cargo test --all-targets`.

5. **Run a short live differential check**
   - 50 deterministic Python-vs-Rust cases.

6. **Open benchmark results**
   - emphasize all 15 configurations,
   - highlight 5.04× median path speedup.

7. **Open CI**
   - Ubuntu,
   - Windows,
   - deterministic differential fuzz job.

## Honest language-specific differences

This is not a line-for-line rewrite and does not claim identical Python
internals.

Documented differences:

- Python's dynamic values become `NaturalValue`.
- Python tuple keys become typed `NaturalKey` values.
- Python exceptions become Rust errors or documented convenience panics.
- Only ASCII, UTF-8, and Latin-1 built-in decoders are provided.
- `Path` values are sorted through text and therefore use lossy conversion for
  non-UTF-8 paths.
- Python-only internal factories, fixtures, and profiling tools are not public
  Rust APIs.

These differences do not weaken the targeted sorting behavior; they make the
migration explicit and idiomatic for Rust.

## Submission checklist

- [x] Original source project retained
- [x] Rust implementation isolated in `rust-port`
- [x] Core and advanced sorting behavior migrated
- [x] Python-reference parity suite
- [x] Deterministic differential fuzzing
- [x] Linux CI
- [x] Windows CI
- [x] Clippy warnings denied
- [x] Release CLI
- [x] Reproducible benchmark harness
- [x] Before/after optimization evidence
- [x] Compatibility inventory
- [x] Architecture documentation
- [x] Known differences documented
- [x] Judge demo flow documented

## Key implementation milestones

```text
CLI compatibility
Differential fuzzing and path normalization
Cached-key performance optimization
Cross-platform CI
CLI whitespace and full numeric corpus parity
Latin-1 decoder compatibility
Direct Path/PathBuf APIs and lazy index ordering
Czech locale and exhaustive Unicode validation
Deterministic differential fuzzing in CI
```

## Final claim

The submission demonstrates a high-coverage, behaviorally faithful migration
of a mature Python natural-sorting library into an idiomatic, tested, faster,
cross-platform Rust implementation.

It deliberately avoids claiming byte-for-byte or internal API identity.
Instead, it provides reproducible evidence for the public sorting behaviors
that matter.
