# Python ↔ Rust CLI Equivalence

Run:

```bash
make cli-diff
```

The harness executes the original Python CLI and the standalone release Rust
binary on the same arguments, stdin bytes, locale, and working directory.

Successful cases compare the process exit code, stdout bytes, and stderr bytes
exactly. Error cases compare the exact exit code and stdout bytes plus the
final diagnostic after normalizing only executable display names, line
endings, and wrapping whitespace.

Raw stdout, stderr, and exit-code files for both implementations are retained
under `parity/evidence/cli/raw/`.

Judge-facing artifacts:

- `parity/evidence/cli/cases.json`
- `parity/evidence/cli/python_cli_output.txt`
- `parity/evidence/cli/rust_cli_output.txt`
- `parity/evidence/cli/cli_success_output.diff`
- `parity/evidence/cli/cli_output.diff`
- `parity/evidence/cli/summary.json`
- `parity/evidence/cli/summary.txt`

An empty `cli_success_output.diff` proves byte-for-byte equality for successful
cases. An empty `cli_output.diff` proves equality for the complete normalized
transcript, including error behavior.
