# Python Test Inventory → Rust Coverage Map

## Scope

The retained upstream inventory contains **19 source test files** and 135 named
`test_*` functions. Parametrization and Hypothesis expand those functions to
**344 runtime test cases**.

All 19 source files match the pinned upstream commit by canonical Git blob
content:

```text
source_commit=b543bdce8771b6e7a7dae0c6745ddf7e80299797
source_test_files=19
submission_test_files=19
problems=0
```

The complete unmodified suite runs through the Rust-backed original-suite
adapter:

```text
344 passed, 3 warnings
PYTEST_EXIT_CODE=0
```

This document maps observable behavior. Rust does not need to duplicate private
Python factories, tuple-shaped internal keys, pytest fixtures, or profiling
utilities when a typed Rust interface proves the same outcome.

## Status legend

- **Direct** — the same observable behavior has a corresponding Rust test/API.
- **Semantic** — an idiomatic typed Rust path tests the same outcome.
- **Rust equivalent** — the capability exists through a Rust-native type such
  as `Path`, `Result`, or `Iterator`.
- **N/A / Replaced** — Python-only implementation or test infrastructure.

## Verified evidence

| Layer | Result |
|---|---:|
| Original test-file integrity | 19 / 19 identical |
| Original suite against Rust | 344 / 344 passed |
| Rust library tests | 381 passed |
| Rust Python-reference integration tests | 176 passed |
| Explicit shared CLI cases | 20 / 20 matched |
| Final differential fuzz cases | 11,727 matched |
| Differential divergences | 0 |
| Fresh-clone verification | passed |
| Docker full verification | passed |

Primary artifacts are indexed in `EVIDENCE_INDEX.md`.

## Original-suite adapter boundary

```text
unmodified original pytest
        ↓
thin Python transport / callback / representation layer
        ↓
Rust original-suite-adapter
        ↓
Rust implementation
```

The Python boundary is allowed to:

- serialize dynamic Python values;
- invoke arbitrary Python callbacks required by original tests;
- preserve Python identity, tuple/list/callable, and regex wrappers;
- adapt host-Python Unicode-version differences;
- launch the Rust adapter and decode its response.

It does not provide an alternative Python natural-sorting algorithm.
Sorting, parsing, numeric conversion, algorithm selection, path/locale
decisions, CLI filtering, and range validation are answered by Rust.

## File-level mapping

| Python source | Named tests | Status | Main Rust coverage |
|---|---:|---|---|
| `tests/conftest.py` | 0 | N/A | Pytest/Hypothesis fixtures |
| `tests/profile_natsorted.py` | 0 | Replaced | Deterministic benchmark harnesses |
| `tests/test_fake_fastnumbers.py` | 15 | Semantic | numeric parsing, Unicode numerics, NaN/infinity, fallback behavior |
| `tests/test_final_data_transform_factory.py` | 3 | Semantic | typed final keys, locale/path/value shaping |
| `tests/test_input_string_transform_factory.py` | 7 | Direct/Semantic | casefold, lowercase-first, normalization, locale grouping/decimals |
| `tests/test_main.py` | 18 | Direct | CLI parsing, stdin/NUL, filters, ranges, reverse, paths, whitespace, errors |
| `tests/test_natsorted.py` | 29 | Direct/Semantic | integer, float, real, path, locale, case, NUMAFTER, PRESORT, mixed/nested values |
| `tests/test_natsorted_convenience.py` | 13 | Direct/Rust equivalent | convenience sorts, indexes, keys, decoders, eager/lazy ordering |
| `tests/test_natsort_key.py` | 5 | Rust equivalent | typed text/bytes/number/sequence key dispatch |
| `tests/test_natsort_keygen.py` | 8 | Rust equivalent | reusable typed key generators and stable keys |
| `tests/test_ns_enum.py` | 1 | Direct | flag values, aliases, compounds, raw-bit conversion |
| `tests/test_os_sorted.py` | 5 | Direct/Semantic | Windows/Unix profiles, punctuation, paths, keys, indexes, reverse |
| `tests/test_parse_bytes_function.py` | 1 | Direct/Semantic | byte keys, paths, ASCII/UTF-8/Latin-1 decoding |
| `tests/test_parse_number_function.py` | 2 | Direct/Semantic | numbers, `None`, NaN, infinities, NANLAST, NUMAFTER |
| `tests/test_parse_string_function.py` | 2 | Semantic | tokenizer, transforms, normalization, numeric conversion |
| `tests/test_regex.py` | 2 | Direct | chooser plus complete six-family split corpus |
| `tests/test_string_component_transform_factory.py` | 1 | Direct/Semantic | integer/float transforms, grouping, collation, casefold |
| `tests/test_unicode_numbers.py` | 6 | Direct/Semantic | generated data plus exhaustive Unicode scalar validation |
| `tests/test_utils.py` | 17 | Direct/Semantic | decoding, flags, grouping, separators, paths, extensions, Unicode |

## Capability mapping

| Capability | Status | Main Rust evidence |
|---|---|---|
| Basic natural sorting | Direct | `sort.rs`, original suite, parity, fuzz |
| Stable leading-zero ties | Direct | unit + original suite |
| Arbitrary-size integers | Direct | normalized numeric keys |
| Arbitrary-precision decimals | Direct | comparison tests + original suite |
| Signed integer/float sorting | Direct | real/signed families |
| Scientific notation | Direct | exponent/high-precision tests |
| `NOEXP` | Direct | flags, tokenizer corpus, fuzz |
| Reverse and `PRESORT` | Direct | sort/index/key tests |
| Case, lower-first, casefold | Direct/Semantic | `text.rs`, original suite |
| Unicode normalization | Direct | text + original suite |
| Unicode decimal digits | Direct | `grift_unicode` + exhaustive scan |
| Unicode digit characters | Direct | generated lookup table |
| Unicode numeric characters | Direct | generated lookup table |
| Paths as strings | Direct | path tests, original suite, fuzz |
| `Path` / `PathBuf` input | Rust equivalent | dedicated path APIs |
| Locale sorting | Direct/Semantic | ICU profiles + parity |
| Czech issue #140 regression | Direct | dedicated corpus |
| OS-aware sorting | Direct/Semantic | `os_sort.rs` profile tests |
| Mixed typed values | Rust equivalent | `NaturalValue` |
| Nested sequences | Rust equivalent | typed nested keys |
| Bytes/text decoding | Direct/Semantic | ASCII, UTF-8, Latin-1 |
| Default bytes/text distinction | Documented Rust contract | explicit variants |
| Key APIs | Rust equivalent | `NaturalKey`, generators |
| Index sorting | Direct | string/value/path/OS indexes |
| Lazy `order_by_index` | Rust equivalent | iterator and safe iterator tests |
| CLI positional input | Direct | original suite + explicit CLI diff |
| CLI newline/NUL input | Direct | original suite + CLI diff |
| CLI whitespace preservation | Direct | original suite + CLI diff |
| CLI filters/exclusions/ranges | Direct | original suite + CLI diff |
| Numeric regex chooser | Direct | six-family corpus |
| Differential properties | Direct | 11,727 final fuzz cases |
| Python private factories | N/A | replaced by typed modules |
| Python tuple key shapes | Language-specific | typed keys |

## Decoder contract

Python can delegate to its complete codec registry. The Rust port intentionally
defines a smaller built-in contract:

```text
ASCII
UTF-8
Latin-1 / ISO-8859-1 aliases
```

Unsupported names return `UnsupportedEncodingError`. Byte and text values
remain distinct typed variants unless the caller selects a decoder-enabled API.

## Path contract

Dedicated path APIs accept `AsRef<Path>` and preserve the original Rust values
in the output. The core sorting engine is text based, so non-UTF-8 path keys
use the documented `Path::to_string_lossy` behavior.

## Key representation contract

Python exposes tuple-shaped keys that reflect private implementation details.
Rust exposes typed `NaturalKey`, `KeyAtom`, `NumericKey`, and
`NaturalKeyGenerator` values. Tests target ordering, equality, stability, and
reuse—not an internal tuple layout.

## Final conclusion

The complete original behavioral inventory passes through Rust-backed
execution. Public sorting behavior is additionally covered by Rust tests,
explicit CLI diffs, deterministic differential fuzzing, fresh-clone and Docker
verification, or a documented Rust-native contract.

Accurate description:

> A standalone, behaviorally faithful Python-to-Rust migration with the
> unmodified original suite passing and language-specific API differences
> documented explicitly.
