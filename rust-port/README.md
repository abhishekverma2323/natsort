# natsort Rust Port

A behavior-focused Rust migration of
[`SethMMorton/natsort`](https://github.com/SethMMorton/natsort), developed for
the **Port Mortem 2026 — Python → Rust** track.

The original Python implementation remains in the repository root and acts as
the behavioral oracle. The Rust crate lives in `rust-port/`.

## Results at a glance

- Natural sorting for integers, arbitrary-precision decimals, signs, and
  scientific notation.
- Unicode decimal, digit, and numeric-character handling.
- Locale-aware, path-aware, and operating-system-aware sorting.
- Typed mixed-value APIs for text, bytes, numbers, `None`, NaN, infinities,
  and nested sequences.
- ASCII, UTF-8, and Latin-1 decoding paths.
- Python-compatible CLI behavior, including stdin, NUL-delimited input,
  filters, exclusions, ranges, whitespace preservation, paths, reverse order,
  and numeric modes.
- Direct `Path`/`PathBuf` sorting APIs.
- Eager and lazy `order_by_index` APIs.
- Linux and Windows CI plus deterministic Python-vs-Rust differential fuzzing.
- Rust faster in all 15 committed benchmark configurations.

Latest verified evidence:

```text
377 Rust library tests passed
176 Python-reference parity tests passed
1,000 / 1,000 local deterministic differential cases passed
200 / 200 deterministic differential cases run in CI
0 failures
```

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

### Rust and Python-reference tests

```bash
cd rust-port
cargo test --all-targets
```

The integration suite in `rust-port/tests/python_parity.rs` uses reference
outputs and behavior captured from the original Python implementation.

### Deterministic differential fuzzing

A Python controller generates randomized inputs, asks the Python package for
the expected order, runs the Rust CLI, and compares the complete output.

Local extended run:

```bash
python parity/differential_fuzz.py \
  --cases 1000 \
  --seed 20260801
```

CI-bounded run:

```bash
python parity/differential_fuzz.py \
  --cases 200 \
  --seed 20260801 \
  --python-oracle python
```

On failure, the exact seed, case, mode, arguments, input, expected output, and
actual output are written to:

```text
parity/differential_fuzz_failure.json
```

## Performance

The committed benchmark compares in-process Python and Rust sorting on the
same deterministic UTF-8 datasets. Loading, process startup, and compilation
are outside the timed region.

Median speedup by mode:

| Mode | Median Rust speedup |
|---|---:|
| Default | 2.12× |
| Float | 4.10× |
| Real | 4.47× |
| Path | 5.04× |
| Locale | 1.67× |

Rust was faster in all 15 measured mode/size combinations from 1,000 through
50,000 entries.

Full methodology and raw results:

- `parity/BENCHMARK_RESULTS.md`
- `parity/benchmark_results.json`

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
│   ├── src/                         # Rust library and CLI
│   ├── tests/python_parity.rs       # Python-reference parity tests
│   ├── README.md
│   └── ARCHITECTURE.md
├── parity/
│   ├── differential_fuzz.py
│   ├── PYTHON_TEST_COVERAGE.md
│   ├── BENCHMARK_RESULTS.md
│   └── benchmark_results.json
└── PORT_MORTEM_2026.md              # Hackathon submission guide
```

## License

The original repository is MIT licensed. This migration follows the existing
repository license.
