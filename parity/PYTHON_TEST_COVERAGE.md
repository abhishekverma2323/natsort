Python Test Inventory → Rust Coverage Map

Scope

The supplied inventory contains 19 Python files and 135 named test_* functions. Parametrization and Hypothesis can expand these into more executed cases, so this is a source-test inventory count, not a pytest runtime case count.

This report maps source tests by observable behavior rather than requiring identical internal helper functions or identical Python tuple representations.

Status legend

Direct — a corresponding Rust unit/parity test exercises the same public behavior.

Semantic — the behavior is covered through an idiomatic Rust API, but not through the same Python internal helper.

Partial — important behavior is covered, but at least one compatibility edge remains.

N/A / Replaced — Python-only test infrastructure or profiling tooling.

File-level mapping

Python source test file

Named tests

Status

Rust coverage assessment

tests/conftest.py

0

N/A

Pytest/Hypothesis locale fixtures and test infrastructure; no runtime feature to port.

tests/profile_natsorted.py

0

Replaced

Python profiling utility is superseded by the committed deterministic Python-vs-Rust benchmark harness.

tests/test_fake_fastnumbers.py

15

Semantic

Numeric parsing, Unicode numerics, NaN handling, and fallback-to-text behavior are covered, but the Python compatibility helper API itself is not exposed in Rust.

tests/test_final_data_transform_factory.py

3

Semantic

Equivalent final-key shaping is exercised through typed NaturalKey, mixed-value, locale, and capital-first tests; no direct factory API exists.

tests/test_input_string_transform_factory.py

7

Direct/Semantic

Casefolding, lowercase-first, dumb/C locale behavior, locale grouping, and localized decimals are covered by text, locale, sort, and parity tests.

tests/test_main.py

18

Mostly direct

CLI parsing, stdin/NUL input, filters, exclusions, ranges, reverse, paths, number modes, errors, and exit codes are covered. Explicit whitespace-preservation cases should still be added.

tests/test_natsorted.py

29

Mostly direct

Core integer/float/real/path/locale/case/NUMAFTER/PRESORT/mixed/nested behavior is heavily covered. Remaining interface gaps include direct path-object input and Python's mixed bytes+str error semantics.

tests/test_natsorted_convenience.py

13

Partial

realsorted, humansorted, index APIs, key APIs, UTF-8/ASCII decoding, and order_by_index are covered. Arbitrary codecs such as latin1 and lazy iter=True behavior are not yet equivalent.

tests/test_natsort_key.py

5

Direct/Semantic

Typed dispatch for text, bytes, numbers, nested sequences, and key functions is covered through NaturalValue and key APIs.

tests/test_natsort_keygen.py

8

Mostly direct

Key generation, algorithms, bytes, nested input, paths, locale, and key stability are covered. Rust returns typed keys rather than Python tuple shapes, so representation parity is intentionally language-specific.

tests/test_ns_enum.py

1

Direct

All documented flag values, aliases, compound aliases, arbitrary integer bits, and SortOptions round trips are covered.

tests/test_os_sorted.py

5

Direct

Windows ordering, Unix/ICU behavior, paths, punctuation corpus, key functions, presort, mixed values, reverse, and indexes are covered.

tests/test_parse_bytes_function.py

1

Direct

Bytes keys, ignore-case, path wrapping, reverse behavior, and decoder paths are covered.

tests/test_parse_number_function.py

2

Direct/Semantic

Direct numbers, None, NaN, infinities, NANLAST, NUMAFTER, path wrapping, and locale equivalence are covered through typed numeric keys.

tests/test_parse_string_function.py

2

Semantic

Tokenizer alternation, transforms, normalization, numeric conversion, and final key behavior are covered; the injectable Python factory-hooks API is not reproduced.

tests/test_regex.py

2

Mostly direct

Six regex families, exact chooser logic, Unicode classes, aliases, nonnumeric flags, and arbitrary raw bits are covered. A full table-driven split-corpus parity test would strengthen proof.

tests/test_string_component_transform_factory.py

1

Direct/Semantic

Integer/float transforms, NANLAST, grouping, locale collation, Unicode casefolding, and normalization are covered.

tests/test_unicode_numbers.py

6

Partial

Generated Unicode numeric data and broad decimal/digit/numeric behavior are covered. Exact exported Python character collections and an exhaustive all-code-point validation are not currently mirrored.

tests/test_utils.py

17

Mostly semantic

Decoding, regex selection, flags, grouping, separators, path splitting, dot paths, extensions, and Unicode behavior are covered. Generic Python-only composition helpers are not public Rust APIs.

Verified evidence already present

Rust unit suite: 357 passing tests

Python-reference parity suite: 165 passing tests

Deterministic differential fuzzing: 1,000 / 1,000 passing cases
- CI differential fuzzing: **200 / 200 passing cases**, seed `20260801`

Cross-platform CI workflow: Linux and Windows jobs for format, all-target tests, Clippy, release binaries, and CLI smoke tests

Same-environment benchmark: Rust is faster across all 15 measured mode/size combinations

Remaining parity work before claiming complete coverage

1. Arbitrary decoder encodings

Python's public decoder(encoding) accepts codecs such as latin1. The current Rust decoder work is centered on UTF-8 and ASCII. Add at least Latin-1 support and define the supported-codec contract explicitly.

Suggested Rust evidence:

decoder_accepts_latin1

latin1_decoder_preserves_non_byte_values

latin1_decoder_sorts_mixed_bytes_and_text

2. Direct path-object input

The Python suite sorts PurePosixPath objects directly. Rust's string APIs currently use AsRef<str>, which does not provide equivalent direct Path/PathBuf input.

Suggested resolution:

add dedicated natsorted_paths / natsorted_paths_with_options, or

add a path-oriented key/sort API that accepts AsRef<Path>

3. Mixed bytes and text default behavior

Python raises TypeError for mixed bytes and str without a decoder. Confirm and document the Rust contract. If strict behavioral parity is required, the mixed-value API should reject this combination unless a decoder is supplied.

4. Lazy order_by_index

Python supports order_by_index(..., iter=True) and returns a generator. Rust currently provides eager collection behavior. Add an iterator-returning equivalent such as order_by_index_iter.

5. Locale breadth and Czech regression

The source inventory includes the Czech cs_CZ regression corpus. Add an explicit Czech/system-locale oracle and parity test rather than relying only on English, German, French, and generic system behavior.

6. CLI whitespace preservation

The Python CLI tests explicitly preserve leading/trailing spaces from stdin and positional arguments. Add direct Rust unit and Python-parity tests for newline and NUL input with whitespace.

7. Full regex split corpus

The chooser and pattern families are covered, but the Python table contains many exact split expectations across all six numeric regex families. Port that table as a table-driven Rust tokenizer/regex parity test.

8. Exhaustive Unicode numeric validation

Add a generated-data integrity test that scans the complete Unicode scalar range and checks every available numeric/digit/decimal value against the Rust data source. Exact Python collection exports are not necessary, but semantic completeness should be machine-verified.

9. Property-based testing in CI

The Python suite uses Hypothesis extensively. The custom differential fuzzer provides strong evidence, but it currently runs manually. Either:

add a bounded deterministic fuzz job to CI, or

introduce proptest for tokenizer, comparison, path, and Unicode invariants.

Recommended completion order

CLI whitespace parity and regex corpus — small, low-risk test additions.

Latin-1 decoder and mixed bytes/text contract.

Path/PathBuf public API and lazy order-by-index iterator.

Czech locale oracle.

Exhaustive Unicode and property tests.

Regenerate this mapping and mark every public behavior as Direct or explicitly documented as language-specific/N/A.

Honest current conclusion

The main sorting engine, algorithms, mixed values, paths, locale modes, OS sorting, CLI, indexing, key generation, decoding basics, fuzzing, benchmarks, and CI have strong evidence. The port is highly complete at the behavioral level, but the gaps above should be closed or explicitly documented before describing it as 100% API-compatible.