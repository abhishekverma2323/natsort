# Differential Fuzzing

The maintained harness is `parity/differential_fuzz.py`; `fuzz/harness.py`
provides the canonical judge-facing entry point.

## Quick deterministic check

```bash
make fuzz
```

This runs 1,000 generated cases with seed `20260801`.

## Final committed session

```text
Seed: 20260802
Fixed-count run: 5,000 / 5,000 matched
Duration run: 120.022739 seconds
Duration cases: 6,727 / 6,727 matched
Total cases: 11,727
Divergences: 0
```

All six modes were exercised:

```text
default
float
real
signed_int
float_noexp
path
```

Artifacts:

- `fuzz/log.txt`
- `fuzz/results.json`
- `fuzz/evidence/count_summary.json`
- `fuzz/evidence/duration_summary.json`
- `fuzz/evidence/count_run.log`
- `fuzz/evidence/duration_run.log`
- `fuzz/checksums.sha256`

## Reproduce the two final run shapes

```bash
.port-venv/bin/python parity/differential_fuzz.py \
  --cases 5000 \
  --seed 20260802 \
  --python-oracle .port-venv/bin/python \
  --summary-file /tmp/natsort-count-summary.json
```

```bash
.port-venv/bin/python parity/differential_fuzz.py \
  --duration-seconds 120 \
  --seed 20260802 \
  --python-oracle .port-venv/bin/python \
  --summary-file /tmp/natsort-duration-summary.json
```

A divergence writes `parity/differential_fuzz_failure.json` with the seed,
exact input, mode, arguments, Python result, Rust result, return codes, and
stderr needed to reproduce the failure.
