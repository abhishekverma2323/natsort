# Python Test Inventory → Rust Coverage Map

## Scope

The retained Python inventory contains 19 source test files and 135 named
`test_*` functions. Parametrization and Hypothesis expand those functions into
additional runtime cases.

This document maps observable behavior. It does not require Rust to duplicate
Python's private factories, tuple-shaped key representation, pytest fixtures,
or profiling utilities.

## Status legend

- **Direct** — the same observable behavior has a corresponding Rust test.
- **Semantic** — an idiomatic typed Rust API tests the same outcome.
- **Rust equivalent** — the Python capability exists through a Rust-native
  interface such as `Path`, `Result`, or `Iterator`.
- **N/A / Replaced** — Python-only infrastructure with no runtime API to port.

## Verified evidence

```text
Rust library tests:                  377 passing
Python-reference integration tests: 176 passing
Local deterministic differential:   1,000 / 1,000 passing
CI deterministic differential:      200 / 200 passing
Differential seed:                   20260801
```

Additional evidence:

- Ubuntu and Windows format/test/Clippy/release checks.
- Linux and Windows CLI smoke tests.
- Exhaustive validation over all valid Unicode scalar values.
- Rust faster in all 15 committed benchmark configurations.

## File-level mapping

| Python source | Named tests | Status | Rust coverage |
|---|---:|---|---|
| `tests/conftest.py` | 0 | N/A | Pytest/Hypothesis fixtures only |
| `tests/profile_natsorted.py` | 0 | Replaced | Deterministic benchmark harness |
| `tests/test_fake_fastnumbers.py` | 15 | Semantic | Numeric parsing, Unicode numerics, NaN/infinity, fallback behavior |
| `tests/test_final_data_transform_factory.py` | 3 | Semantic | Typed final keys, locale/path/value shaping |
| `tests/test_input_string_transform_factory.py` | 7 | Direct/Semantic | casefold, lowercase-first, normalization, locale grouping/decimals |
| `tests/test_main.py` | 18 | Direct | CLI parsing, stdin/NUL, filters, ranges, reverse, paths, modes, whitespace, errors |
| `tests/test_natsorted.py` | 29 | Direct/Semantic | integer, float, real, path, locale, case, NUMAFTER, PRESORT, mixed/nested values |
| `tests/test_natsorted_convenience.py` | 13 | Direct/Rust equivalent | convenience sorts, indexes, key APIs, decoders, eager/lazy index ordering |
| `tests/test_natsort_key.py` | 5 | Rust equivalent | typed text/bytes/number/sequence key dispatch |
| `tests/test_natsort_keygen.py` | 8 | Rust equivalent | reusable typed key generators and stable keys |
| `tests/test_ns_enum.py` | 1 | Direct | flag values, aliases, compound flags, raw-bit conversion |
| `tests/test_os_sorted.py` | 5 | Direct/Semantic | Windows/Unix profiles, punctuation, paths, keys, indexes, reverse |
| `tests/test_parse_bytes_function.py` | 1 | Direct/Semantic | bytes keys, transforms, paths, ASCII/UTF-8/Latin-1 decoding |
| `tests/test_parse_number_function.py` | 2 | Direct/Semantic | numbers, None, NaN, infinities, NANLAST, NUMAFTER |
| `tests/test_parse_string_function.py` | 2 | Semantic | tokenizer, transforms, normalization, numeric conversion |
| `tests/test_regex.py` | 2 | Direct | chooser plus complete six-family split corpus |
| `tests/test_string_component_transform_factory.py` | 1 | Direct/Semantic | integer/float transforms, grouping, collation, casefold |
| `tests/test_unicode_numbers.py` | 6 | Direct/Semantic | generated data plus exhaustive Unicode scalar validation |
| `tests/test_utils.py` | 17 | Direct/Semantic | decoding, flags, grouping, separators, paths, extensions, Unicode |

## Capability mapping

| Capability | Status | Main Rust evidence |
|---|---|---|
| Basic natural sorting | Direct | `sort.rs`, parity suite, fuzzer |
| Stable leading-zero ties | Direct | unit and parity tests |
| Arbitrary-size integers | Direct | normalized numeric keys |
| Arbitrary-precision decimals | Direct | numeric comparison tests |
| Signed integer/float sorting | Direct | real/signed test families |
| Scientific notation | Direct | exponent and high-precision tests |
| NOEXP | Direct | flags, tokenizer corpus, fuzzer |
| Reverse and PRESORT | Direct | sort/index/key tests |
| Case, lower-first, casefold | Direct/Semantic | `text.rs`, locale/sort tests |
| Unicode normalization | Direct | text and parity tests |
| Unicode decimal digits | Direct | `grift_unicode` plus exhaustive scan |
| Unicode digit characters | Direct | 128 generated mappings |
| Unicode numeric characters | Direct | 1,242 generated mappings |
| Paths as strings | Direct | path tests, parity, fuzzer |
| `Path` / `PathBuf` input | Rust equivalent | dedicated path APIs |
| Locale sorting | Direct/Semantic | ICU profiles and parity |
| Czech `cs_CZ` regression | Direct | issue #140 corpus |
| OS-aware sorting | Direct/Semantic | `os_sort.rs` profile tests |
| Mixed typed values | Rust equivalent | `NaturalValue` |
| Nested sequences | Rust equivalent | typed nested key tests |
| Bytes/text decoding | Direct/Semantic | ASCII, UTF-8, Latin-1 |
| Default bytes/text distinction | Documented Rust contract | explicit variants; decoders opt in |
| Key APIs | Rust equivalent | `NaturalKey`, generators |
| Index sorting | Direct | string/value/path/OS indexes |
| Lazy `order_by_index` | Rust equivalent | iterator and safe iterator tests |
| CLI positional input | Direct | CLI tests and smoke tests |
| CLI stdin/newline/NUL input | Direct | unit/parity/fuzzer |
| CLI whitespace preservation | Direct | explicit whitespace tests |
| CLI filters/exclusions/ranges | Direct | CLI test corpus |
| Numeric regex chooser | Direct | flags and six-family corpus |
| Differential property coverage | Direct | bounded CI + extended local run |
| Python private factories | N/A | replaced by typed modules |
| Python tuple key shapes | Language-specific | replaced by typed keys |

## Decoder contract

Python can delegate to its entire codec registry. The Rust port intentionally
defines a smaller built-in contract:

```text
ASCII
UTF-8
Latin-1 / ISO-8859-1 aliases
```

Unsupported names return `UnsupportedEncodingError`.

Byte and text values remain distinct typed variants unless the caller supplies
a decoder-enabled API. This makes the conversion explicit instead of relying
on Python's dynamic mixed-type failure surface.

## Path contract

Dedicated path APIs accept `AsRef<Path>` and clone original values into the
result.

The sorting engine is text based, so path keys use `Path::to_string_lossy`.
This is documented behavior for non-UTF-8 paths rather than hidden platform
dependence.

## Key representation contract

Python exposes tuple-shaped keys whose exact structure reflects private
implementation details.

Rust exposes typed:

- `NaturalKey`,
- `KeyAtom`,
- `NumericKey`,
- `NaturalKeyGenerator`.

Tests target ordering, equality, stability, and reuse rather than requiring
identical tuple layouts.

## Final conclusion

The targeted public sorting behavior is covered by direct tests, typed semantic
equivalents, deterministic differential fuzzing, or an explicit
language-specific contract.

The project should be described as:

> A high-coverage, behaviorally faithful Python-to-Rust migration with
> documented Rust-native API differences.

It should not be described as an identical copy of Python internals or as an
implementation of Python's complete codec registry.
