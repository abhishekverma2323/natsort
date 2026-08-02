# natsort Rust Port

A behavior-focused Rust migration of
[`SethMMorton/natsort`](https://github.com/SethMMorton/natsort), developed for
the **Port Mortem 2026 — Python → Rust** track.

The original Python implementation remains in the repository root and acts as
the behavioral oracle. The Rust crate lives in `rust-port/`.

## Results at a glance

- Complete unmodified original suite: **344 / 344 passed** through the
  Rust-backed compatibility adapter.
- Original test integrity: **19 / 19 canonical Git blobs identical** to the
  pinned upstream source commit.
- Rust library tests: **381 passed**.
- Rust Python-reference integration tests: **176 passed**.
- Explicit CLI equivalence: **20 / 20 shared cases matched** with both unified
  diff files empty.
- Differential fuzzing: **11,727 matched cases, zero divergences** across
  default, float, real, signed-integer, no-exponent, and path modes.
- Fresh-clone and Docker full verification: **passed**.
- Release CLI: **2,194,280 bytes (≈2.09 MiB)**.
- First-party unsafe surface: one documented Windows-only FFI call site.

The original Python implementation remains at the repository root as the
behavioral oracle. Production Rust does not invoke or embed Python.

## Build and verify

From the repository root:

```bash
cd rust-port

cargo fmt --all -- --check
cargo test --all-targets
cargo clippy --all-targets -- -D warnings
cargo build --release --bin natsort
```

The release CLI is written to:

```text
rust-port/target/release/natsort
```

On Windows it is:

```text
rust-port\target\release\natsort.exe
```

## Quick start

### Basic natural sorting

```rust
use rust_port::natsorted;

let input = ["file10", "file2", "file1"];
let output = natsorted(&input);

assert_eq!(output, vec!["file1", "file2", "file10"]);
```

### Signed floating-point sorting

```rust
use rust_port::{realsorted, SortOptions};

let input = ["value5.10", "value-3", "value5.3", "value2"];
let output = realsorted(&input);

assert_eq!(
    output,
    vec!["value-3", "value2", "value5.10", "value5.3"],
);
```

Additional options use the builder API:

```rust
use rust_port::{natsorted_with_options, SortOptions};

let input = ["FILE10", "file2", "File1"];

let options = SortOptions::new()
    .ignore_case(true)
    .reverse(false);

let output = natsorted_with_options(&input, options);

assert_eq!(output, vec!["File1", "file2", "FILE10"]);
```

### Direct path sorting

```rust
use std::path::PathBuf;

use rust_port::natsorted_paths;

let input = vec![
    PathBuf::from("folder/file10.txt"),
    PathBuf::from("folder/file2.txt"),
    PathBuf::from("folder/file1.txt"),
];

let output = natsorted_paths(&input);

assert_eq!(
    output,
    vec![
        PathBuf::from("folder/file1.txt"),
        PathBuf::from("folder/file2.txt"),
        PathBuf::from("folder/file10.txt"),
    ],
);
```

### Mixed typed values

```rust
use rust_port::{natsorted_values, NaturalValue};

let input = vec![
    NaturalValue::from("a2"),
    NaturalValue::from(3),
    NaturalValue::from("a1"),
    NaturalValue::from(2),
];

let output = natsorted_values(&input);

assert_eq!(
    output,
    vec![
        NaturalValue::from(2),
        NaturalValue::from(3),
        NaturalValue::from("a1"),
        NaturalValue::from("a2"),
    ],
);
```

### Explicit byte decoding

```rust
use rust_port::{
    decoder,
    natsorted_values_with_decoder,
    NaturalValue,
};

let input = vec![
    NaturalValue::from(&b"caf\xe9-10"[..]),
    NaturalValue::from(&b"caf\xe9-2"[..]),
];

let latin1 = decoder("latin1").expect("Latin-1 is supported");
let output =
    natsorted_values_with_decoder(&input, latin1).expect("valid Latin-1");

assert_eq!(output.len(), 2);
```

### Lazy index ordering

```rust
use rust_port::order_by_index_iter;

let values = ["num3", "num5", "num2"];
let indexes = [2, 0, 1];

let output: Vec<_> =
    order_by_index_iter(&values, &indexes).collect();

assert_eq!(output, vec!["num2", "num3", "num5"]);
```

## CLI examples

Sort positional arguments:

```bash
cargo run --bin natsort -- file10 file2 file1
```

Output:

```text
file1
file2
file10
```

Read from stdin:

```bash
printf 'file10\nfile2\nfile1\n' |
  cargo run --quiet --bin natsort
```

Real-number sorting:

```bash
cargo run --quiet --bin natsort -- \
  --number-type real \
  value5.10 value-3 value5.3 value2
```

Path sorting:

```bash
cargo run --quiet --bin natsort -- \
  --paths \
  folder/file10.txt folder/file2.txt folder/file1.txt
```

See the complete CLI:

```bash
cargo run --quiet --bin natsort -- --help
```

## Public API groups

| Area | Main APIs |
|---|---|
| String sorting | `natsorted`, `realsorted`, `humansorted` |
| Options | `SortOptions`, `AlgorithmFlags` |
| Key generation | `natsort_key`, `natsort_keygen`, typed key variants |
| Index sorting | `index_natsorted`, `index_realsorted`, locale/value variants |
| Path objects | `natsorted_paths`, `index_natsorted_paths` |
| Mixed values | `NaturalValue`, `natsorted_values`, value-key APIs |
| Decoding | `Decoder`, `decoder`, decoder-enabled value sorting |
| OS sorting | `os_sorted`, `os_sort_key`, `OsSortOptions`, index/key variants |
| Reordering | `order_by_index`, `try_order_by_index`, lazy iterator variants |
| CLI integration | `run_cli`, `CliOptions`, `NumericRange` |

Most API families also provide an explicit `_with_options` form.

## Validation

### Complete original Python suite

From the repository root:

```bash
make verify
```

This verifies canonical original-test blobs, Rust formatting, all Rust tests,
Clippy with warnings denied, and the complete unmodified original Python suite
through the Rust adapter.

Committed result:

```text
19 / 19 original test files identical
381 Rust library tests passed
176 Rust Python-reference tests passed
344 original Python tests passed
```

The compatibility path is:

```text
unmodified original pytest
        ↓
thin Python transport / callback / representation boundary
        ↓
Rust original-suite-adapter
        ↓
Rust implementation
```

The Python boundary does not provide an alternative natural-sorting algorithm.

### Explicit CLI equivalence

```bash
make cli-diff
```

The original Python CLI and standalone Rust release CLI are run with identical
arguments, stdin bytes, locale, and working directory.

```text
20 / 20 shared cases matched
17 successful cases matched byte-for-byte
3 error cases matched under documented diagnostic normalization
both unified diff files empty
```

Evidence: `parity/evidence/cli/`.

### Differential fuzzing

Quick check:

```bash
make fuzz
```

Final committed session:

```text
Seed: 20260802
Fixed-count: 5,000 / 5,000
120-second run: 6,727 / 6,727
Combined: 11,727
Divergences: 0
```

A failure writes a complete reproduction payload to
`parity/differential_fuzz_failure.json`.

## Performance

The final judge-facing benchmark compares fresh Python and Rust CLI processes
on identical deterministic corpus bytes. It includes startup, argument
parsing, stdin reading, sorting, output generation, and termination.

| Scenario | Rust p50 speedup | Rust p99 speedup |
|---|---:|---:|
| Empty-input startup | 17.1× | 13.8× |
| Default 1,000 items | 12.5× | 10.2× |
| Float 1,000 items | 13.0× | 9.3× |
| Real 1,000 items | 13.0× | 9.3× |
| Path 1,000 items | 12.8× | 13.5× |
| Locale 1,000 items | 14.3× | 8.2× |
| Default 10,000 items | 8.0× | 6.9× |
| Default 50,000 items | 2.2× | 3.9× |

Median peak-RSS reduction is 82.4% at 1,000 items, 60.6% at 10,000,
and 10.1% at 50,000. The narrower large-input margin is reported explicitly.

Full methodology, raw samples, corpus hashes, environment metadata, and binary
hash:

- `bench/methodology.md`
- `bench/report.md`
- `bench/results.json`
- `bench/raw_latency_samples.json`
- `bench/raw_rss_samples.json`
- `bench/environment.json`

Historical in-process optimization results remain under `parity/`.

## Architecture

See [`ARCHITECTURE.md`](ARCHITECTURE.md) for module responsibilities, sorting
data flow, design decisions, and migration boundaries.

## Compatibility contract

This project targets observable sorting behavior rather than reproducing
Python internals or Python tuple representations.

Intentional Rust-specific differences include:

- Typed `NaturalValue` and `NaturalKey` representations instead of arbitrary
  Python objects and tuple-shaped keys.
- Rust error types and panic/`Result` pairs instead of Python exceptions.
- Explicit decoder-enabled APIs for bytes/text interoperability.
- Dedicated path APIs; non-UTF-8 paths are converted with
  `Path::to_string_lossy` because the sorting engine is text based.
- Python-only internal factories, profiling helpers, and pytest fixtures are
  not exposed as Rust public APIs.
- Supported built-in decoders are ASCII, UTF-8, and Latin-1 rather than the
  complete Python codec registry.

The detailed mapping is maintained in:

```text
parity/PYTHON_TEST_COVERAGE.md
```

## Repository map

```text
natsort/
├── natsort/                         # Original Python implementation
├── tests/                           # Original Python tests
├── rust-port/
│   ├── src/                         # Rust library, CLI, adapter
│   ├── tests/python_parity.rs       # Python-reference integration tests
│   ├── README.md
│   └── ARCHITECTURE.md
├── parity/
│   ├── original_suite_adapter/      # Thin Python test boundary
│   ├── cli/                         # Explicit CLI equivalence harness
│   ├── test_hashes/                 # Canonical test manifests
│   ├── evidence/                    # Verification and audit evidence
│   └── PYTHON_TEST_COVERAGE.md
├── fuzz/                            # Differential evidence
├── bench/                           # p99/RSS benchmark package
├── audit/                           # Generated metrics tooling
├── EVIDENCE_INDEX.md
└── PORT_MORTEM_2026.md
```

## License

The original repository is MIT licensed. This migration follows the existing
repository license.
