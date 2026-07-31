# natsort Rust Port

A Rust implementation of the core natural-sorting behavior provided by the Python [`natsort`](https://github.com/SethMMorton/natsort) project.

This port is being developed for the **Port Mortem 2026 Hackathon** under the **Python → Rust** migration track.

## Project Structure

The original Python project remains in the repository root.

The Rust implementation is located in:

```text
rust-port/
├── Cargo.toml
├── README.md
├── src/
│   ├── lib.rs
│   ├── options.rs
│   ├── sort.rs
│   └── token.rs
└── tests/
    └── python_parity.rs
```

## Implemented Features

The current Rust implementation supports:

* Basic natural sorting
* Multiple numeric components
* Plain-text sorting
* Arbitrary-precision integers
* Leading-zero handling
* Case-insensitive sorting
* Reverse sorting
* Signed integers
* Arbitrary-precision decimal numbers
* Scientific notation
* Combined signed and floating-point sorting
* Python-to-Rust parity tests

## Public API

The crate exposes:

```rust
pub use options::SortOptions;
pub use sort::{natsorted, natsorted_with_options};
```

### Basic Natural Sorting

```rust
use rust_port::natsorted;

fn main() {
    let input = vec!["file10", "file2", "file1"];
    let result = natsorted(&input);

    assert_eq!(result, vec!["file1", "file2", "file10"]);
}
```

### Case-Insensitive Sorting

```rust
use rust_port::{natsorted_with_options, SortOptions};

fn main() {
    let input = vec!["File10", "file2", "FILE1"];

    let options = SortOptions::new().ignore_case(true);
    let result = natsorted_with_options(&input, options);

    assert_eq!(result, vec!["FILE1", "file2", "File10"]);
}
```

### Reverse Sorting

```rust
use rust_port::{natsorted_with_options, SortOptions};

fn main() {
    let input = vec!["file1", "file10", "file2"];

    let options = SortOptions::new().reverse(true);
    let result = natsorted_with_options(&input, options);

    assert_eq!(result, vec!["file10", "file2", "file1"]);
}
```

### Signed Integer Sorting

```rust
use rust_port::{natsorted_with_options, SortOptions};

fn main() {
    let input = vec!["value5", "value-2", "value1", "value-10"];

    let options = SortOptions::new().signed(true);
    let result = natsorted_with_options(&input, options);

    assert_eq!(
        result,
        vec!["value-10", "value-2", "value1", "value5"]
    );
}
```

### Decimal Sorting

```rust
use rust_port::{natsorted_with_options, SortOptions};

fn main() {
    let input = vec!["value1.5", "value1.25", "value10.01", "value2.0"];

    let options = SortOptions::new().float(true);
    let result = natsorted_with_options(&input, options);

    assert_eq!(
        result,
        vec!["value1.25", "value1.5", "value2.0", "value10.01"]
    );
}
```

### Scientific Notation

```rust
use rust_port::{natsorted_with_options, SortOptions};

fn main() {
    let input = vec![
        "value1e3",
        "value2.5e2",
        "value4.2e-3",
        "value1",
        "value10",
    ];

    let options = SortOptions::new().float(true);
    let result = natsorted_with_options(&input, options);

    assert_eq!(
        result,
        vec![
            "value4.2e-3",
            "value1",
            "value10",
            "value2.5e2",
            "value1e3",
        ]
    );
}
```

### Combining Options

Options use a builder-style API and can be combined:

```rust
use rust_port::{natsorted_with_options, SortOptions};

fn main() {
    let input = vec![
        "Value1.5",
        "value-2.25",
        "VALUE-10.5",
        "value0.25",
    ];

    let options = SortOptions::new()
        .ignore_case(true)
        .signed(true)
        .float(true)
        .reverse(false);

    let result = natsorted_with_options(&input, options);

    assert_eq!(
        result,
        vec![
            "VALUE-10.5",
            "value-2.25",
            "value0.25",
            "Value1.5",
        ]
    );
}
```

## Arbitrary-Precision Numeric Comparison

The implementation does not parse numeric tokens into fixed-width integer or floating-point types.

Numeric values are compared using normalized strings, which allows the Rust port to handle values larger than `u64`, `u128`, or the exact precision range of `f64`.

Examples include:

```text
999999999999999999999999
-184467440737095516160000
1.000000000000000000000001
1.25e100
```

## Python Parity Verification

Reference outputs are generated using the original Python implementation:

```text
parity/python_reference.py
parity/python_reference_output.txt
```

Rust integration tests based on those outputs are located in:

```text
rust-port/tests/python_parity.rs
```

The parity suite currently covers:

* Basic natural sorting
* Leading zeros
* Very large integers
* Signed integers
* Decimal numbers
* Scientific notation
* Signed floating-point numbers
* Case-insensitive sorting
* Reverse sorting

## Running the Tests

From WSL or another Linux environment:

```bash
cd rust-port
cargo fmt --check
cargo test
cargo clippy -- -D warnings
```

Current verified test status:

```text
35 Rust unit tests passed
9 Python parity integration tests passed
0 failed
```

## Python Baseline

The original Python test suite can be run from the repository root:

```bash
python -m pytest
```

Verified original Python baseline:

```text
344 tests passed
```

## Current Limitations

This is an incremental migration of the original Python project.

The current Rust implementation focuses on core natural-sorting and numeric behavior. The following Python `natsort` capabilities are not yet fully implemented:

* Locale-aware sorting
* Operating-system path sorting
* Unicode numeric categories
* Bytes-specific sorting behavior
* NaN and infinity handling
* All original `ns` enum combinations
* Python-compatible key-generator APIs
* Command-line interface parity

## Migration Approach

The project follows an incremental migration strategy:

1. Preserve the original Python implementation.
2. Establish the Python test baseline.
3. Implement isolated Rust sorting components.
4. Add unit tests for each migrated behavior.
5. Generate outputs from the Python implementation.
6. Lock those outputs into Rust parity tests.
7. Expand support feature by feature without breaking existing behavior.

## License

The original project is licensed under the MIT License. This Rust port follows the repository’s existing licensing terms.
