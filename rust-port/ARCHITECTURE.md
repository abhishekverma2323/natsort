# Rust Port Architecture

## Design objective

The Rust port preserves the original project's observable natural-sorting
behavior while replacing Python's dynamic dispatch and tuple-based keys with
typed Rust structures.

The migration uses the Python implementation as an oracle, but the Rust design
is intentionally idiomatic:

- explicit options,
- typed values and keys,
- stable sorting,
- reusable key generators,
- `Result`-based safe APIs,
- dedicated path and OS-sorting surfaces.

## Sorting pipeline

```text
Input values
    │
    ├── strings / paths
    ├── NaturalValue typed values
    └── key-function outputs
    │
    ▼
Option normalization
    │
    ├── real = signed + float
    ├── human = locale alpha + locale numeric
    └── dedicated path APIs force path mode
    │
    ▼
Text and path transformation
    │
    ├── Unicode normalization
    ├── case/lowercase/casefold handling
    ├── locale number normalization
    └── path component and suffix splitting
    │
    ▼
Tokenization
    │
    ├── integer
    ├── signed integer
    ├── float
    ├── signed float
    ├── exponent / no-exponent
    └── Unicode decimal/digit/numeric values
    │
    ▼
Typed natural key
    │
    ├── text atoms
    ├── arbitrary-precision numeric atoms
    ├── path component keys
    ├── locale sort keys
    └── nested sequence keys
    │
    ▼
Stable comparison
    │
    ├── optional presort
    ├── natural-key comparison
    └── optional reverse
    │
    ▼
Sorted clones or sorted indexes
```

## Module responsibilities

| Module | Responsibility |
|---|---|
| `algorithm.rs` | Python-compatible algorithm flags and numeric-regex selection |
| `api.rs` | Convenience APIs, key-based sorting, index sorting, path APIs, index reordering |
| `cli.rs` | Command-line parsing, filtering, ranges, input modes, output behavior |
| `decode.rs` | ASCII, UTF-8, and Latin-1 decoding for typed byte values |
| `locale.rs` | Locale profiles, number normalization, ICU collation, Czech regression |
| `options.rs` | Builder-style sort configuration |
| `os_sort.rs` | Platform-aware Windows/Unix sorting profiles and key APIs |
| `path.rs` | Cross-separator path components, dot-path normalization, extension heuristics |
| `separator.rs` | Ordering separators and NUMAFTER behavior |
| `sort.rs` | Core string sorting and comparison orchestration |
| `text.rs` | Unicode normalization, case transforms, locale text behavior |
| `token.rs` | Numeric token recognition and conversion |
| `unicode_numeric.rs` | Generated non-decimal Unicode digit/numeric lookup tables |
| `value.rs` | `NaturalValue`, typed keys, mixed/nested value sorting |
| `value_api.rs` | Mixed-value index and key-function APIs |

## Numeric representation

Fixed-width integer or floating-point parsing would lose compatibility for
very large integers, high-precision decimals, and extreme exponents.

The port therefore normalizes numeric tokens into comparison-safe typed
representations rather than relying solely on `u64`, `u128`, or `f64`.

Covered inputs include:

```text
999999999999999999999999999999
-184467440737095516160000
1.000000000000000000000001
1.25e100
-4.2e-300
```

Equivalent numeric spellings compare equally, while Rust's stable sort
preserves their original input order.

## Unicode handling

Three numeric categories are distinguished:

1. Decimal digits supplied by `grift_unicode`.
2. Non-decimal digit characters such as superscripts and circled digits.
3. Broader numeric characters such as vulgar fractions and Roman numerals.

`unicode_numeric.rs` is generated from the Python project's Unicode tables.
An exhaustive test scans all valid Unicode scalar values and validates:

- 128 non-decimal digit mappings,
- 1,242 broader numeric mappings,
- digit/numeric table agreement,
- finite mapped values.

## Locale architecture

`LocaleProfile` converts platform-style identifiers into explicit profiles.
Current explicit profiles include:

- C/POSIX,
- English (India),
- English (United States),
- Czech (Czechia),
- German (Germany),
- French (France),
- system locale.

ICU4X sort keys provide deterministic alphabetic and numeric collation.
Locale-specific decimal and grouping characters are normalized before numeric
tokenization.

The Czech profile includes the Python issue #140 regression corpus, including
the Czech `Ch` collation position.

## Path architecture

Path sorting is lexical and cross-platform by design:

- `/` and `\` are recognized as separators,
- current-directory components are normalized,
- parent components are preserved,
- root components remain significant,
- up to two plausible filename suffixes are split,
- numeric-looking suffixes remain attached where required by Python behavior.

Dedicated `Path`/`PathBuf` APIs preserve original Rust values in the output.
Because the core engine sorts text, non-UTF-8 paths use `to_string_lossy`.

## Mixed-value architecture

`NaturalValue` provides explicit variants for values that Python accepts
dynamically, including text, bytes, integers, floating-point values, `None`,
and nested sequences.

This produces deterministic cross-type ordering without using `Any` or runtime
downcasting.

Bytes and text remain distinct unless the caller chooses an explicit decoder.
Decoder-enabled APIs make the conversion contract visible and fallible.

## Key caching and performance

Sorting repeatedly compares the same inputs. The optimized implementation
precomputes natural keys and sorts indexes over those cached keys instead of
re-tokenizing during every comparator call.

This change is the main performance foundation behind both the historical
in-process benchmark and the final end-to-end startup/p99/RSS benchmark.

## Error strategy

Rust APIs expose explicit safe alternatives where failure is possible:

| Convenience API | Safe API |
|---|---|
| `order_by_index` | `try_order_by_index` |
| `order_by_index_iter` | `try_order_by_index_iter` |
| decoder-enabled sorting | returns `Result<_, DecodeError>` |
| decoder lookup | returns `Result<Decoder, UnsupportedEncodingError>` |

The convenience index-ordering APIs panic on invalid indexes to parallel
Python's immediate indexing failure. Safe forms preserve the failing index,
its position, and the input length.

## Verification architecture

The project uses independent proof layers rather than one aggregate percentage:

1. **Rust unit tests** validate module-level invariants.
2. **Python-reference integration tests** validate committed observable
   behavior across the Rust API and CLI.
3. **The complete unmodified original suite** runs through a thin
   Python-to-Rust compatibility boundary.
4. **Explicit CLI diffing** compares shared inputs, output bytes, errors, and
   exit codes.
5. **Deterministic differential fuzzing** compares generated inputs against the
   live original Python oracle.
6. **Fresh-clone and Docker verification** prove reproducibility outside the
   development checkout.
7. **Raw latency/RSS benchmarks and generated audits** make performance,
   dependency, LOC, and unsafe claims inspectable.

### Original-suite adapter

```text
Original pytest suite
    │
    ├── Python callback invocation where the public API accepts callbacks
    ├── Python-specific identity and wrapper preservation
    ├── value serialization / response decoding
    └── host-Unicode-version compatibility
    │
    ▼
original-suite-adapter Rust executable
    │
    ▼
Rust parsing, sorting, locale, path, value, and CLI implementation
```

The adapter design deliberately excludes a Python behavioral fallback.
Production Rust builds do not require Python.

### Evidence provenance

Long-running evidence records the exact tested commit, deterministic seed,
binary SHA-256, raw samples, or checksums. The claim-to-proof map is maintained
in `EVIDENCE_INDEX.md`.

## Migration boundaries

The following Python implementation details are deliberately not copied:

- internal transform-factory function signatures,
- exact tuple shapes of Python natural keys,
- pytest fixtures and Hypothesis infrastructure,
- Python's complete codec registry,
- profiling utilities tied to CPython.

Their observable outcomes are covered through typed Rust APIs, deterministic
tests, fuzzing, or explicit documentation.
