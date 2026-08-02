# Differential Fuzzing

The canonical harness entry point is:

```bash
make fuzz
```

`fuzz/harness.py` delegates to the maintained implementation at
`parity/differential_fuzz.py`.

Existing final-duration evidence is currently stored under `evidence/`:

- `evidence/differential_fuzz_65s.json`
- `evidence/differential_fuzz_65s.log`
- `evidence/differential_fuzz_65s.sha256`

A final rerun against the submission commit will be recorded as `fuzz/log.txt`
with its seed, duration, case count, and divergence count.
