# Honest Numbers and Safety Audit

Generated from commit `43719532854ba2f6e926648cb9ae8fdceec13ac2`.

## Behavioral verification

| Evidence | Result |
|---|---:|
| Original test files matching pinned source | 19 / 19 |
| Original suite against Rust | 344 passed |
| Rust library tests | 381 passed/listed |
| Rust Python-parity tests | 176 passed/listed |
| Shared CLI cases | 20 / 20 matched |
| Differential fuzz cases | 11727 completed |
| Differential divergences | 0 |

## Source inventory

| Category | Files | Physical lines | Nonblank lines |
|---|---:|---:|---:|
| Rust production source | 19 | 11127 | 9291 |
| Generated Rust Unicode table | 1 | 457 | 439 |
| Python compatibility boundary | 11 | 1910 | 1474 |
| Judge-facing test/bench/fuzz tooling | 8 | 2215 | 1884 |

Line counts are physical and nonblank line counts, not an
estimated logical-LOC metric. Generated Unicode data is shown
separately rather than hidden inside handwritten Rust totals.

## Original-tree integrity

| Tree | Unchanged | Added | Removed | Modified |
|---|---:|---:|---:|---:|
| `natsort/` | true | 0 | 0 | 0 |
| `tests/` | true | 0 | 0 | 0 |

## Safety and dependencies

| Metric | Value |
|---|---:|
| Explicit `unsafe { ... }` blocks | 1 |
| `unsafe fn` declarations | 0 |
| Foreign ABI declarations | 1 |
| Unsafe guard exit code | 0 |
| Direct Cargo dependencies | 7 |
| Locked Cargo packages | 49 |
| Rust release binary size | 2194280 bytes |

Exact unsafe/FFI matches are recorded in
`parity/evidence/audit/unsafe_matches.txt`.

## Boundary responsibilities

The Python compatibility boundary is validation infrastructure,
not a Python fallback implementation. It is responsible for:

- serializing Python values to the Rust adapter protocol;
- invoking arbitrary Python callback objects when tests require them;
- preserving Python-specific object identity and wrapper types;
- adapting host-Python Unicode-version compatibility;
- launching the Rust adapter and decoding its results.

Sorting, parsing, numeric ordering, algorithm flags, path behavior,
locale-aware decisions, CLI filtering, and range validation are
answered by Rust.

## Known limitations and interpretation

- The compatibility suite uses subprocess IPC and is intentionally
  not presented as production performance.
- CLI error evidence normalizes only executable display names, line
  endings, and wrapping whitespace for error diagnostics.
- Locale and OS-sort behavior can depend on installed locale data and
  operating-system profile; the Docker image pins a reproducible
  Linux validation environment.
- Peak RSS advantage narrows on very large inputs because both
  implementations retain large input/output datasets.
- The generated Unicode numeric table is reported separately so it
  does not inflate handwritten Rust source claims.
