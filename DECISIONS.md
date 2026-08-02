# Engineering Decision Log — natsort Python to Rust

This document records the important engineering decisions made while
migrating `SethMMorton/natsort` from Python to Rust for Port Mortem 2026.

The migration targets observable sorting behavior, not a line-for-line
translation of Python internals.

Each decision explains:

- the problem or context;
- the Rust design selected;
- alternatives considered;
- compatibility, performance, and safety trade-offs.

## Related documentation

- `README.md`
- `EVIDENCE_INDEX.md`
- `PORT_MORTEM_2026.md`
- `rust-port/ARCHITECTURE.md`
- `rust-port/README.md`
- `parity/PYTHON_TEST_COVERAGE.md`
- `bench/methodology.md`
- `bench/report.md`
- `fuzz/log.txt`
- `HONEST_NUMBERS.md`
- `UNSAFE_AUDIT.md`

---

## Decision 1 — Target observable behavior instead of Python internals

### Context

Python `natsort` uses dynamic objects, tuple-shaped keys, factories,
regular expressions, locale behavior, and Python-specific exceptions.

A direct line-by-line translation would copy implementation details that are
not natural Rust interfaces.

### Decision

Use the original Python implementation as the behavioral oracle while
designing typed and idiomatic Rust modules around observable sorting results.

### Alternatives considered

- Translate Python files and functions one-for-one.
- Reproduce the exact Python tuple-shaped keys.
- Execute or embed Python inside the Rust implementation.

### Rationale

The externally visible sorting order is the compatibility target. Rust
ownership, types, iterators, and error handling should remain idiomatic.

### Impact and trade-off

- Public sorting outcomes are tested against Python.
- Rust does not depend on Python at production runtime.
- Internal Python key representations are not exposed.

### Evidence

- `rust-port/ARCHITECTURE.md`
- `parity/PYTHON_TEST_COVERAGE.md`
- `rust-port/tests/python_parity.rs`

---
## Decision 2 — Keep the Rust port isolated inside the original repository

### Context

The original Python project and its tests are required as the source behavior
inventory and as the behavioral oracle.

### Decision

Keep the Python implementation at the repository root and place the standalone
Rust crate under `rust-port/`.

### Alternatives considered

- Replace the Python project in place.
- Create an unrelated Rust-only repository.
- Mix Rust files into Python package directories.

### Rationale

This preserves project provenance, keeps the language boundary visible, and
allows direct comparison between the original and migrated implementations.

### Impact and trade-off

- The original Python implementation remains available for verification.
- Judges can inspect both implementations side by side.
- The repository contains both Python and Rust build ecosystems.

### Evidence

- `rust-port/Cargo.toml`
- `PORT_MORTEM_2026.md`
- original `natsort/` and `tests/` directories

---

## Decision 3 — Use typed options with Python-compatible flags

### Context

Python exposes many combinable `natsort.ns` bit flags. Rust callers benefit
from explicit fields and builder-style configuration.

### Decision

Provide:

- `AlgorithmFlags` for Python-compatible flag behavior;
- `SortOptions` for explicit Rust configuration;
- conversion between known algorithm bits and typed options.

Rust-specific settings such as `reverse` and `locale_profile` remain explicit
fields rather than Python flag bits.

### Alternatives considered

- Accept only raw integer flags.
- Create a separate function for every option combination.
- Store options in a dynamic map.

### Rationale

The design retains Python compatibility while providing readable and
type-checked Rust configuration.

### Impact and trade-off

- Known Python flags round-trip correctly.
- Unknown bits do not activate unchecked behavior.
- Some Rust-only settings are not representable in the Python bit mask.

### Evidence

- `rust-port/src/algorithm.rs`
- `rust-port/src/options.rs`

---

## Decision 4 — Represent dynamic Python inputs with typed Rust values

### Context

Python can naturally sort strings, bytes, integers, floating-point values,
`None`, NaN, infinities, and nested sequences through dynamic dispatch.

### Decision

Represent mixed inputs with `NaturalValue` and represent generated keys with
typed structures such as `NaturalKey`, `KeyAtom`, and `NumericKey`.

### Alternatives considered

- `std::any::Any`
- runtime downcasting
- converting every value to a string
- serializing arbitrary values before sorting

### Rationale

A typed enum makes supported cross-type behavior explicit, deterministic, and
reviewable.

### Impact and trade-off

- Major mixed and nested Python behaviors receive Rust equivalents.
- No dynamic `Any` or downcast architecture is required.
- Application-specific values must use a supported variant or a key function.

### Evidence

- `rust-port/src/value.rs`
- `rust-port/src/value_api.rs`
- `rust-port/tests/python_parity.rs`

---

## Decision 5 — Preserve numeric precision using normalized comparison parts

### Context

Natural sorting must correctly compare:

- integers larger than `u64` or `u128`;
- high-precision decimal values;
- signed numbers;
- scientific notation;
- extreme exponents;
- equivalent spellings such as `1.5`, `1.50`, and `001.5000`.

Parsing every token into a fixed-width integer or `f64` would lose information.

### Decision

Normalize numeric text into:

- sign;
- significant digit sequence;
- effective decimal position.

Compare these parts directly rather than relying on fixed-width machine
numbers.

### Alternatives considered

- `u64` or `u128`
- `f64`
- lexical comparison of the original spelling
- rounding every number to a fixed decimal precision

### Rationale

This representation supports arbitrary digit length and exact comparison of
decimal and scientific forms without floating-point rounding.

### Impact and trade-off

- Large integers and precise decimals remain correctly ordered.
- Equivalent numeric spellings compare equally.
- The comparison logic is more complex than `parse::<f64>()`.

### Evidence

- `rust-port/src/sort.rs`
- numeric unit tests
- Python parity tests
- differential fuzz numeric modes

---

## Decision 6 — Preserve stable ordering for equivalent numeric values

### Context

Different text values can represent the same numeric value:

- `file1`
- `file01`
- `file001`

Python stable sorting preserves their original order when their keys compare
equal.

### Decision

Treat equivalent normalized values as equal and use stable Rust sorting.

### Alternatives considered

- Break ties using the original string.
- Break ties using the number of leading zeros.
- Use unstable sorting for possible performance gains.

### Rationale

A spelling-based tie-break would change observable Python behavior.

### Impact and trade-off

- Leading-zero and equivalent-decimal order is preserved.
- Equivalent numeric spellings intentionally do not receive an additional
  lexical tie-break.
- Stable sorting may use more memory than unstable sorting.

### Evidence

- stability tests in `rust-port/src/sort.rs`
- Python parity tests for equivalent values

---

## Decision 7 — Cache natural keys before sorting

### Context

Sorting compares the same inputs many times. Tokenizing, normalizing, parsing,
and generating locale keys during every comparison repeats expensive work.

### Decision

Precompute one cached natural key per input and sort values or indexes using
those cached structures.

### Alternatives considered

- Tokenize during every comparator call.
- Cache only numeric tokens.
- Use a global cache shared across sort calls.

### Rationale

Per-sort cached keys remove repeated transformations without introducing
global mutable state or cache invalidation.

### Impact and trade-off

- This optimization is the main performance foundation of the Rust port.
- Cache ownership remains local to each sort.
- Sorting temporarily allocates a key for each input.

### Evidence

- `CachedToken` and `CachedStringKey` in `rust-port/src/sort.rs`
- `parity/BENCHMARK_RESULTS_BEFORE_OPTIMIZATION.md`
- `parity/BENCHMARK_RESULTS.md`

---

## Decision 8 — Separate Unicode decimal, digit, and numeric categories

### Context

Unicode contains numeric characters beyond ASCII `0` through `9`, including:

- decimal digits from multiple scripts;
- superscript and circled digits;
- vulgar fractions;
- Roman numerals;
- broader numeric symbols.

Python `natsort` treats these categories differently depending on numeric mode.

### Decision

Use:

- `grift_unicode` for Unicode decimal digits;
- a generated table for non-decimal digits;
- a generated table for broader Unicode numeric characters.

The generated mappings are derived from the original Python project data.

### Alternatives considered

- Support only ASCII digits.
- Treat all Unicode numerics as plain text.
- Depend only on standard-library character methods.
- Maintain the mappings manually.

### Rationale

The separated categories mirror the source behavior and allow exhaustive
validation.

### Impact and trade-off

- Arabic-Indic, Devanagari, full-width, circled, fraction, and Roman-numeral
  cases are covered.
- Exhaustive tests validate 128 non-decimal digit mappings and 1,242 broader
  numeric mappings.
- The generated source is large and must be regenerated when source Unicode
  data changes.

### Evidence

- `rust-port/src/unicode_numeric.rs`
- `dev/generate_rust_unicode_numeric.py`
- exhaustive Unicode scalar test

---

## Decision 9 — Make Unicode normalization and case behavior explicit

### Context

Canonical equivalence, compatibility characters, case folding,
lowercase-first behavior, and grouped-letter ordering affect natural-sort
keys.

### Decision

Use explicit transformation stages for:

- canonical or compatibility normalization;
- case swapping where required;
- Unicode case folding;
- grouped-letter transformation;
- recomposition before locale collation.

### Alternatives considered

- ASCII-only lowercase conversion.
- Skip Unicode normalization.
- Delegate every text transformation to the operating system locale.

### Rationale

Explicit transformations are deterministic, testable, and portable.

### Impact and trade-off

- Ligatures, full-width characters, sharp-s, Greek sigma variants, and
  canonical equivalents are supported.
- Behavior does not depend only on basic host string functions.
- Transformations allocate normalized strings.

### Evidence

- `rust-port/src/text.rs`
- `unicode-normalization`
- `unicode-casefold`

---

## Decision 10 — Use ICU4X with explicit locale profiles

### Context

Locale-aware sorting cannot be reproduced reliably using basic lexical
comparison. Host locale availability and behavior may also vary across
systems.

### Decision

Use ICU4X collation keys with explicit locale profiles for:

- C/POSIX;
- English (India);
- English (United States);
- Czech (Czechia);
- German (Germany);
- French (France);
- system locale.

Locale decimal and grouping symbols are normalized before numeric
tokenization.

### Alternatives considered

- Host-only `strcoll` behavior.
- English-only locale support.
- One process-global locale.
- Handwritten alphabet ordering tables.

### Rationale

ICU4X provides portable collation while explicit profiles make behavior
visible and testable.

### Impact and trade-off

- Locale alphabetic and numeric sorting are supported.
- Czech `Ch` collation behavior is regression tested.
- Locale keys are cached because collation is more expensive than lexical
  comparison.
- ICU data increases dependency and binary complexity.

### Evidence

- `rust-port/src/locale.rs`
- Czech issue #140 regression tests
- locale benchmark results

---

## Decision 11 — Implement lexical cross-platform path sorting

### Context

Path sorting must recognize separators, current and parent components, roots,
filename stems, and plausible file extensions.

The files do not need to exist on disk.

### Decision

Implement lexical path processing that:

- recognizes both `/` and `\`;
- removes current-directory components;
- preserves parent-directory components;
- preserves roots;
- splits up to two plausible filename suffixes;
- keeps numeric-looking or long suffixes attached where Python behavior
  requires it.

Expose dedicated APIs for Rust `Path` and `PathBuf` values.

### Alternatives considered

- Split only on the host operating system separator.
- Canonicalize every path through the filesystem.
- Treat the complete path as one string.
- Reject native Rust path values.

### Rationale

Lexical sorting remains portable, deterministic, and independent of filesystem
state.

### Impact and trade-off

- Windows and Unix separator styles are handled cross-platform.
- No filesystem access or path canonicalization is required.
- Non-UTF-8 Rust paths use `to_string_lossy`, so some invalid byte sequences
  may become indistinguishable.

### Evidence

- `rust-port/src/path.rs`
- direct path APIs
- path unit tests
- path parity and fuzz tests

---

## Decision 12 — Separate generic path sorting from OS-aware sorting

### Context

Natural path sorting and native operating-system-style sorting are different
behaviors. Windows logical filename ordering differs from Unix and ICU
ordering.

### Decision

Provide a separate OS sorting API with explicit profiles:

- `System`
- `Windows`
- `Unix`

WSL resolves the system profile to Windows-style behavior, while native Linux
uses Unix-style behavior.

### Alternatives considered

- Use one comparator for every platform.
- Select behavior only at compile time.
- Hide platform choice from callers.
- Invoke an external operating-system command.

### Rationale

Explicit profiles make platform behavior testable and allow callers to request
a profile independent of the current host.

### Impact and trade-off

- Windows and Unix ordering have separate implementations.
- Cross-platform tests can exercise emulated profiles.
- Pure-Rust emulation cannot guarantee every undocumented native platform
  detail.

### Evidence

- `rust-port/src/os_sort.rs`
- `OsSortProfile`
- OS sorting parity tests

---

## Decision 13 — Retain one narrow Windows native FFI boundary

### Context

The Windows system profile should preserve native `StrCmpLogicalW` behavior.

Replacing it unconditionally with an approximation could change punctuation,
locale, Unicode, or leading-zero behavior.

### Decision

On Windows, for the system locale profile, call:

`Shlwapi.dll!StrCmpLogicalW`

The complete first-party unsafe surface consists of:

- one `unsafe extern "system"` declaration;
- one unsafe FFI call site.

Both strings are converted to owned, null-terminated UTF-16 buffers that stay
alive during the call.

### Alternatives considered

- Always use the pure-Rust Windows emulation.
- Hide the unsafe call inside another dependency.
- Execute an external Windows or Python process.
- Remove native Windows behavior to claim zero unsafe.

### Rationale

A small, explicit, audited boundary is more honest and behaviorally faithful
than hiding or removing native semantics.

### Impact and trade-off

- Native Windows system-profile behavior is retained.
- The unsafe surface is narrowly scoped and documented.
- CI rejects additional first-party unsafe usage.
- The project does not claim zero unsafe.

### Evidence

- `UNSAFE_AUDIT.md`
- `rust-port/src/os_sort.rs`
- `rust-port/scripts/check_unsafe.sh`
- `.github/workflows/rust-ci.yml`

---

## Decision 14 — Keep Python outside the production Rust artifact

### Context

Python is useful as a behavioral oracle, but a Rust port that embeds or invokes
Python at runtime would not be an independent migration.

### Decision

Use Python only for:

- reference-output generation;
- parity tests;
- differential fuzzing;
- benchmarks.

The Rust library and CLI do not execute, embed, or link to Python.

### Alternatives considered

- PyO3 or CPython embedding.
- Shelling out to Python from Rust.
- Calling Python for difficult Unicode or locale cases.
- Shipping a hybrid executable.

### Rationale

Production behavior must be implemented by Rust itself.

### Impact and trade-off

- The release CLI runs without a Python runtime.
- Oracle and benchmark tooling remain visibly separate under `parity/`.
- Differential testing still requires a Python environment.

### Evidence

- Cargo dependency audit
- standalone release binary evidence
- `parity/differential_fuzz.py`

---

## Decision 15 — Use deterministic differential fuzzing as a separate proof layer

### Context

Handwritten tests may miss combinations involving numeric syntax, Unicode
digits, paths, stdin modes, delimiters, and reverse sorting.

### Decision

Generate randomized but deterministic inputs and compare complete Python and
Rust outputs.

The harness supports:

- fixed case-count runs;
- duration-based runs;
- a public seed;
- six sorting modes;
- positional and stdin input;
- newline and NUL delimiters;
- reverse sorting;
- JSON failure reproduction;
- machine-readable success summaries.

### Alternatives considered

- Nondeterministic fuzzing without a seed.
- Rust-only property tests.
- Compare only generated keys.
- Depend only on handwritten examples.

### Rationale

A deterministic controller provides broad behavioral comparison while keeping
every failure reproducible.

### Impact and trade-off

- CI runs a bounded 200-case differential check.
- A committed survivor run completed 1,523 cases in 65.01989 seconds.
- All six modes were exercised with zero divergences.
- Generated distributions cannot cover every possible input.

### Evidence

- `parity/differential_fuzz.py`
- `evidence/differential_fuzz_65s.json`
- `evidence/differential_fuzz_65s.log`
- `evidence/differential_fuzz_65s.sha256`

---

## Decision 16 — Reuse one CLI engine instead of duplicating logic

### Context

The migration requires both a Rust library and a Python-compatible CLI with:

- positional input;
- stdin;
- NUL-delimited input;
- filters and exclusions;
- numeric ranges;
- whitespace preservation;
- paths;
- reverse and numeric modes;
- platform line endings.

### Decision

Keep command-line parsing and stream behavior in reusable library code
implemented in `cli.rs`.

Keep `src/bin/natsort.rs` as a thin adapter around `run_cli_with_program`.

### Alternatives considered

- Implement everything directly inside `main`.
- Maintain a separate CLI comparator.
- Invoke the Python CLI.
- Support only positional arguments.

### Rationale

A thin binary makes CLI behavior independently testable and ensures the CLI
and library use the same sorting implementation.

### Impact and trade-off

- Advanced CLI behavior can be covered by unit, parity, smoke, and fuzz tests.
- No duplicate sorting implementation exists in the binary.
- The reusable CLI module increases the crate's public surface.

### Evidence

- `rust-port/src/cli.rs`
- `rust-port/src/bin/natsort.rs`
- CLI unit and parity tests

---

## Decision 17 — Make byte decoding explicit and fallible

### Context

Python provides a large codec registry and can sort byte strings using decoder
functions.

Rust byte-to-text conversion must make encoding and failure behavior explicit.

### Decision

Provide built-in decoder support for:

- ASCII;
- UTF-8;
- Latin-1.

Decoder-enabled APIs return typed errors where decoding may fail.

### Alternatives considered

- Treat all byte input as UTF-8.
- Silently use lossy conversion.
- Reproduce Python's complete codec registry.
- Merge bytes and text automatically.

### Rationale

The supported encoding contract remains explicit and portable without turning
the migration into a general-purpose encoding framework.

### Impact and trade-off

- Common Python decoder behavior is represented.
- Invalid ASCII or UTF-8 returns an error.
- Arbitrary Python codec names are intentionally unsupported.

### Evidence

- `rust-port/src/decode.rs`
- decoder-enabled sorting tests
- compatibility documentation

---

## Decision 18 — Pair convenience APIs with safe Result alternatives

### Context

Python indexing fails immediately when an invalid index is used. Rust callers
may prefer concise behavior or recoverable typed errors.

### Decision

Provide paired APIs where appropriate:

| Convenience API | Safe API |
|---|---|
| `order_by_index` | `try_order_by_index` |
| `order_by_index_iter` | `try_order_by_index_iter` |
| decoder lookup | typed `Result` |
| decoder-enabled sorting | typed `Result` |

### Alternatives considered

- Panic in every public API.
- Return `Result` from every operation.
- Ignore invalid indexes.
- Clamp invalid indexes into range.

### Rationale

The paired design retains ergonomic Python-like convenience while providing
safe Rust alternatives.

### Impact and trade-off

- Convenience APIs mirror immediate Python indexing failure.
- Safe forms identify the invalid index, its position, and input length.
- The public API contains paired variants.

### Evidence

- `rust-port/src/api.rs`
- `rust-port/src/decode.rs`
- index and decoder error tests

---

## Decision 19 — Benchmark equivalent in-process sorting work

### Context

Cross-language benchmarks can be distorted by compilation, process startup,
dataset loading, different input data, or reporting only the best run.

### Decision

Use:

- identical deterministic datasets;
- in-memory unsorted input;
- warm-up runs;
- multiple measured runs;
- median timing;
- release-mode Rust builds.

Exclude compilation, process startup, and dataset loading from the timed
sorting region.

### Alternatives considered

- Time full CLI process execution.
- Use different generated inputs.
- Report the fastest single run.
- Include Rust compilation in every measurement.
- Publish only one favorable dataset size.

### Rationale

The benchmark should measure sorting rather than unrelated runtime setup.

### Impact and trade-off

- Both implementations sort identical data.
- Seed, sizes, warmups, runs, and environment are recorded.
- Rust is faster in all 15 committed mode and size combinations.
- Results do not claim cold-start, memory, p95, or p99 performance.

### Evidence

- `parity/run_benchmarks.py`
- `parity/benchmark_results.json`
- `parity/BENCHMARK_RESULTS.md`

---

## Decision 20 — Document compatibility boundaries honestly

### Context

Several Python concepts do not have direct or desirable Rust equivalents:

- arbitrary dynamic objects;
- internal tuple-shaped keys;
- the complete Python codec registry;
- injected transform factories;
- pytest fixtures and Hypothesis infrastructure;
- CPython-specific profiling utilities.

### Decision

Describe behavior as one of:

- direct compatibility;
- semantic compatibility;
- typed Rust equivalent;
- language-specific or not applicable.

Do not claim identical Python internals or universal 100% API identity.

### Alternatives considered

- Claim complete parity based only on passing examples.
- Reproduce every Python internal helper as a Rust public API.
- Hide unsupported behavior.
- Treat internal implementation details as required public compatibility.

### Rationale

Explicit boundaries distinguish deliberate Rust design from missing behavior
and make the submission more credible.

### Impact and trade-off

- Claims remain tied to reproducible evidence.
- Rust interfaces remain idiomatic.
- The project is described as a high-coverage, behaviorally faithful
  migration rather than a byte-for-byte replacement.

### Evidence

- `parity/PYTHON_TEST_COVERAGE.md`
- `rust-port/README.md`
- `PORT_MORTEM_2026.md`

---

## Decision 21 — Reject a Python-side behavioral facade

### Context

An early compatibility approach could satisfy Python tests by recreating
behavior in Python while the Rust port remained incomplete. Passing tests
through that route would not prove a working port.

### Decision

Remove the copied behavioral facade and require the unmodified original suite
to obtain every sorting, parsing, numeric, path, locale, and CLI decision from
the Rust adapter executable.

The Python boundary may only provide transport, callback invocation,
Python-specific representation wrapping, and host-version compatibility.

### Alternatives considered

- Keep a Python reimplementation as a fallback.
- Claim parity from the Rust-only test inventory.
- Rewrite the original tests for a Rust-native interface.

### Rationale

A port is proven only when the original behavioral inventory executes against
the target-language implementation.

### Impact and trade-off

- The original suite takes longer because compatibility calls cross a process
  boundary.
- Hypothesis deadlines are disabled in the compatibility runner because the
  subprocess boundary measures validation overhead rather than algorithmic
  correctness.
- The production Rust library and CLI remain Python-independent.
- A missing or failing Rust adapter cannot be replaced by a Python sorting
  path.

### Evidence

- `parity/original_suite_adapter/`
- `parity/run_original_suite_against_rust.py`
- `parity/evidence/genuine_full_suite_final.txt`
- `rust-port/src/python_compat.rs`
- `rust-port/src/bin/original_suite_adapter.rs`

---

## Decision 22 — Verify original tests with canonical Git blobs

### Context

Raw working-tree bytes can differ across Windows and Linux because Git may
convert CRLF and LF line endings. A byte hash of checked-out files therefore
can report a false modification.

### Decision

Compare the pinned source commit and current submission using canonical Git
blob content. Separately reject any added, removed, staged, unstaged, or
untracked test files.

### Alternatives considered

- Hash working-tree files directly.
- Trust `git status` without a manifest.
- Normalize line endings manually before hashing.

### Rationale

Git blobs are the repository's canonical content representation and remain
stable across checkout policies.

### Impact and trade-off

- All 19 original test files have identical source/submission manifests.
- Line-ending conversion does not weaken the check.
- The verification remains reproducible with one command.

### Evidence

- `parity/verify_original_tests.py`
- `parity/test_hashes/source.sha256`
- `parity/test_hashes/submission.sha256`
- `parity/test_hashes/verification.txt`

---

## Decision 23 — Add an explicit shared-input CLI diff

### Context

Unit tests and successful compilation do not directly show that two command
line programs emit the same bytes, exit codes, and errors on shared inputs.

### Decision

Run the original Python CLI and standalone Rust release CLI on one committed
case matrix with identical arguments, stdin bytes, locale, and working
directory.

Successful cases compare exit code, stdout, and stderr byte-for-byte. Error
cases compare exact exit/stdout and the final diagnostic after normalizing only
the executable display name, line endings, and wrapping whitespace.

### Alternatives considered

- Show a few manually copied examples.
- Compare only sorted output.
- Treat the original Python CLI tests as sufficient evidence.

### Rationale

A committed unified diff is easy for judges to inspect and difficult to
misinterpret.

### Impact and trade-off

- 17 successful cases match exactly.
- 3 error cases match under the documented diagnostic normalization.
- Both committed diff files are empty.
- Raw outputs and exit codes remain available for every case.

### Evidence

- `parity/cli/run_cli_equivalence.py`
- `parity/evidence/cli/summary.json`
- `parity/evidence/cli/cli_success_output.diff`
- `parity/evidence/cli/cli_output.diff`
- `parity/evidence/cli/raw/`

---

## Decision 24 — Measure startup, percentiles, and memory separately

### Context

The historical in-process benchmark measured sorting work after warm-up but
did not answer judge-facing questions about startup, p99 latency, or peak RSS.

### Decision

Retain the in-process optimization benchmark and add a second end-to-end CLI
benchmark that records:

- fresh-process startup;
- p50, p95, and p99 latency;
- median throughput;
- peak RSS;
- binary size;
- raw samples, corpus hashes, and environment metadata.

### Alternatives considered

- Replace historical results with one favorable headline.
- Report mean or best-run latency only.
- Exclude process startup from all published evidence.
- Measure Python and Rust with different corpora.

### Rationale

The two benchmark layers answer different questions. Publishing both prevents
a favorable methodology from hiding cold-start or memory behavior.

### Impact and trade-off

- Rust is 17.1× faster at startup p50.
- Rust is 12.5× faster on the default 1,000-item p50 workload.
- Rust remains 2.2× faster at 50,000 items, where the margin narrows.
- Median RSS reduction narrows from 82.4% at 1,000 items to 10.1% at 50,000.
- Raw samples make unusual variance inspectable.

### Evidence

- `bench/methodology.md`
- `bench/results.json`
- `bench/raw_latency_samples.json`
- `bench/raw_rss_samples.json`
- `bench/environment.json`
- `bench/report.md`

---

## Decision 25 — Generate evidence from clean committed states

### Context

Evidence produced while harness or implementation files are uncommitted cannot
be tied unambiguously to a repository revision.

### Decision

Commit each harness before its final run, record the implementation commit and
binary hash where applicable, and retain raw logs or checksums alongside the
summary.

### Alternatives considered

- Record only screenshots.
- Generate all evidence after documentation edits without provenance.
- Store only rounded headline numbers.

### Rationale

Commit provenance makes every long-running proof independently auditable.

### Impact and trade-off

- CLI, fuzz, benchmark, Docker, and audit artifacts record exact revisions.
- Some evidence files refer to different sequential commits because each
  proof layer was added and committed before the next one.
- Later evidence/documentation commits do not silently rewrite earlier raw
  results.

### Evidence

- `EVIDENCE_INDEX.md`
- `parity/evidence/cli/summary.json`
- `fuzz/results.json`
- `bench/environment.json`
- `parity/evidence/docker_image_metadata.txt`
- `parity/evidence/audit/project_metrics.json`

---

## Verification summary

Current committed verification includes:

- 19 / 19 original test files matching the pinned source commit;
- 344 / 344 original Python tests passing against Rust-backed behavior;
- 381 Rust library tests passing;
- 176 Rust Python-reference parity tests passing;
- 20 / 20 shared CLI cases matching with both diff files empty;
- 11,727 differential fuzz cases with zero divergences;
- successful fresh-clone verification;
- successful Docker build and full verification;
- end-to-end startup, p50/p95/p99, throughput, and peak-RSS evidence;
- one generated honest-number and first-party unsafe audit.

## Accurate submission claim

> This project is a standalone, behaviorally faithful Python-to-Rust migration
> of `natsort`. The complete unmodified original suite passes against
> Rust-backed behavior, and difficult numeric, Unicode, locale, path,
> mixed-value, and CLI behavior is supported by independent tests, exact CLI
> diffs, differential fuzzing, fresh-clone and Docker verification, raw
> benchmarks, and a generated safety audit.

The project does not claim:

- identical Python internals;
- the complete Python codec registry;
- lossless sorting of arbitrary non-UTF-8 paths;
- zero first-party unsafe;
- universal internal API identity;
- benchmark metrics outside the committed methodology.
