# Honest-Number and Safety Audit

Run from a clean repository:

```bash
make audit-final
```

The audit derives claims from the committed repository and existing evidence.
It checks:

- original `natsort/` and `tests/` trees against the pinned source commit;
- Rust library and Python-reference test inventories;
- original-suite, CLI, fuzz, and benchmark evidence;
- physical and nonblank source-line counts;
- generated Unicode data separately from handwritten Rust;
- direct and locked Cargo dependency counts;
- release binary size;
- exact first-party unsafe and foreign-ABI occurrences;
- the unsafe-boundary CI guard.

Outputs:

- `HONEST_NUMBERS.md`
- `parity/evidence/audit/project_metrics.json`
- `parity/evidence/audit/project_metrics.md`
- `parity/evidence/audit/file_inventory.json`
- `parity/evidence/audit/unsafe_matches.txt`
- `parity/evidence/audit/checksums.sha256`

Current headline values:

```text
Original suite:                344 passed
Rust library tests:            381
Rust Python-reference tests:   176
CLI cases:                     20 / 20
Fuzz cases/divergences:        11,727 / 0
Direct Cargo dependencies:     7
Locked Cargo packages:         49
Unsafe blocks / foreign ABI:   1 / 1
Rust release binary:           2,194,280 bytes
```
