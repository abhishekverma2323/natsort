# Honest-Number and Safety Audit

The audit derives exact metrics from the committed repository and existing
judge-facing evidence.

Run from a clean repository:

```bash
make audit-final
```

Outputs:

- `HONEST_NUMBERS.md`
- `parity/evidence/audit/project_metrics.json`
- `parity/evidence/audit/project_metrics.md`
- `parity/evidence/audit/file_inventory.json`
- `parity/evidence/audit/unsafe_matches.txt`
- `parity/evidence/audit/checksums.sha256`

The line-count methodology reports physical and nonblank lines. Generated
Unicode data is separated from handwritten Rust source. The audit also checks
that the original `natsort/` and `tests/` trees still match the pinned source
commit.
