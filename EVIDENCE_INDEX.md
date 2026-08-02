# Evidence Index

This file is the shortest path from a submission claim to its committed proof.
Every evidence artifact records either an exact commit, a binary hash, a
deterministic seed, checksums, or the command needed to reproduce it.

## Working port

| Claim | Result | Primary artifact | Reproduce |
|---|---:|---|---|
| Original tests unchanged | 19 / 19 canonical Git blobs identical | `parity/test_hashes/verification.txt` | `make verify-tests` |
| Original suite against Rust | 344 / 344 passed | `parity/evidence/genuine_full_suite_final.txt` | `make test-original` |
| Rust library tests | 381 passed | `parity/evidence/make_verify.txt` | `make test-rust` |
| Rust Python-reference tests | 176 passed | `parity/evidence/make_verify.txt` | `make test-rust` |
| Fresh-clone verification | Exit code 0 | `parity/evidence/fresh_clone_verify.txt` | Clone branch, then `make verify` |
| Docker verification | Build 0; verify 0 | `parity/evidence/docker_build.txt`, `docker_verify.txt` | `make docker-verify` |

Source baseline:

```text
b543bdce8771b6e7a7dae0c6745ddf7e80299797
```

The integrity checker compares canonical Git blobs rather than checkout bytes,
so Windows/WSL line-ending conversion cannot produce a false mismatch.

## CLI behavioral equivalence

| Claim | Result | Artifact |
|---|---:|---|
| Shared cases | 20 |
| Successful exact-byte cases | 17 / 17 | `parity/evidence/cli/cli_success_output.diff` |
| Error semantic cases | 3 / 3 | `parity/evidence/cli/cli_output.diff` |
| Total mismatches | 0 | `parity/evidence/cli/summary.json` |
| Unified diff size | 0 bytes | both `.diff` files |

Reproduce:

```bash
make cli-diff
```

Raw stdout, stderr, and exit codes for both implementations are retained under
`parity/evidence/cli/raw/`.

## Differential fuzzing

| Run | Requested | Completed | Divergences |
|---|---:|---:|---:|
| Fixed-count | 5,000 cases | 5,000 | 0 |
| Duration | 120 seconds | 6,727 | 0 |
| Combined | — | **11,727** | **0** |

Seed: `20260802`.

Artifacts:

- `fuzz/log.txt`
- `fuzz/results.json`
- `fuzz/evidence/count_summary.json`
- `fuzz/evidence/duration_summary.json`
- `fuzz/checksums.sha256`

Quick deterministic reproduction:

```bash
make fuzz
```

The final-duration commands are documented in `fuzz/README.md`.

## Performance

| Evidence | Artifact |
|---|---|
| Methodology | `bench/methodology.md` |
| Human-readable table | `bench/report.md` |
| Structured results | `bench/results.json` |
| CSV summary | `bench/results.csv` |
| Raw latency samples | `bench/raw_latency_samples.json` |
| Raw RSS samples | `bench/raw_rss_samples.json` |
| Environment and binary hash | `bench/environment.json` |
| Run transcript | `bench/run.log` |

Reproduce:

```bash
make bench-final
```

Headline results:

- startup p50: **17.1× faster**;
- default 1,000-item p50: **12.5× faster**;
- default 50,000-item p50: **2.2× faster**;
- default 1,000-item median peak RSS: **82.4% lower**;
- default 50,000-item median peak RSS: **10.1% lower**.

The narrower large-input margin is reported explicitly.

## Honest numbers and safety

| Claim | Result | Artifact |
|---|---:|---|
| Original `natsort/` tree | unchanged | `parity/evidence/audit/project_metrics.json` |
| Original `tests/` tree | unchanged | same |
| Rust production source | 19 files / 11,127 physical lines | same |
| Python validation boundary | 11 files / 1,910 physical lines | same |
| Direct Cargo dependencies | 7 | same |
| Locked Cargo packages | 49 | same |
| Rust release binary | 2,194,280 bytes | same |
| Explicit unsafe blocks | 1 | `parity/evidence/audit/unsafe_matches.txt` |
| Foreign ABI declarations | 1 | same |
| Unsafe guard | exit code 0 | same |

Reproduce:

```bash
make audit-final
```

Root reports:

- `HONEST_NUMBERS.md`
- `UNSAFE_AUDIT.md`

## Evidence provenance

Different long-running artifacts intentionally record the exact commit tested
at the time they were generated:

| Evidence | Recorded commit |
|---|---|
| Full Docker verification | `abc23bfd8c8673ac9b736444ee1297ee1ea0a515` |
| Clean CLI equivalence | `be082753a478ee5c262d8c7d57c36243aa9e666b` |
| Final differential fuzzing | `074659134bfaf41ab9bae076a1b1d4343ac7e733` |
| Final performance benchmark | `7e1755400ac0b3f5956e0b20a3ea8a79bbd3efe7` |
| Generated project audit | `43719532854ba2f6e926648cb9ae8fdceec13ac2` |

Later commits in this sequence add evidence and documentation; the original
`natsort/`, original `tests/`, and Rust production implementation remain
audited and unchanged where reported.
