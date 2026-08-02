# natsort: Python → Rust Port

Port Mortem 2026, Track D.

This repository contains a Rust port of
[SethMMorton/natsort](https://github.com/SethMMorton/natsort) plus a
test-only compatibility adapter that runs the unmodified original Python test
suite against the Rust implementation.

## Current verification status

- Source baseline commit: `b543bdce8771b6e7a7dae0c6745ddf7e80299797`
- Rust-backed original suite: **344 / 344 passed**
- Original test files edited: **0**
- Final evidence: `parity/evidence/genuine_full_suite_final.txt`
- Port source: `rust-port/src/`
- Architecture: `rust-port/ARCHITECTURE.md`
- Engineering decisions: `DECISIONS.md`
- Detailed migration report: `PORT_MORTEM_2026.md`
- Test coverage mapping: `parity/PYTHON_TEST_COVERAGE.md`

## Architecture

```text
Unmodified original pytest suite
        ↓
Thin Python serialization / representation boundary
        ↓
Rust original-suite-adapter binary
        ↓
Rust natsort implementation
```

The Python boundary handles subprocess transport, Python callback invocation,
and Python-specific representation wrapping. Sorting, parsing, numeric
conversion, algorithm selection, Unicode compatibility decisions, and CLI
range validation are answered by Rust.

## One-command workflow

```bash
make build
make verify
```

A containerized verification path is also provided:

```bash
docker build -t natsort-rust-port .
docker run --rm natsort-rust-port
```

The upstream project README remains available as `README.rst`.
