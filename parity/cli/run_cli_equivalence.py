"""Compare the original Python CLI with the standalone Rust CLI."""

from __future__ import annotations

import argparse
import difflib
import hashlib
import json
import os
import re
import shutil
import subprocess
import sys
from dataclasses import dataclass
from pathlib import Path
from typing import Final

ROOT: Final = Path(__file__).resolve().parents[2]
DEFAULT_OUTPUT: Final = ROOT / "parity" / "evidence" / "cli"


@dataclass(frozen=True)
class Case:
    """One shared command-line behavior case."""

    name: str
    arguments: tuple[str, ...] = ()
    stdin: bytes = b""
    error_case: bool = False


@dataclass(frozen=True)
class Result:
    """Captured process result."""

    return_code: int
    stdout_hex: str
    stderr_hex: str
    stdout_text: str
    stderr_text: str
    diagnostic: str


CASES: Final = (
    Case("positional_default", ("file10", "file2", "file1", "file02")),
    Case("reverse", ("--reverse", "file10", "file2", "file1", "file02")),
    Case(
        "signed_integer",
        ("--sign", "item-2", "item10", "item-10", "item2", "item+3"),
    ),
    Case(
        "float",
        (
            "--number-type",
            "float",
            "v1.10",
            "v1.2",
            "v1.02",
            "v1e3",
            "v2.5",
        ),
    ),
    Case(
        "real",
        ("--number-type", "real", "v-1.5", "v2", "v-10", "v+3", "v0"),
    ),
    Case(
        "float_noexp",
        (
            "--number-type",
            "float",
            "--noexp",
            "v1e3",
            "v2e2",
            "v10",
            "v1e2",
        ),
    ),
    Case(
        "paths",
        (
            "--paths",
            "folder/file10.txt",
            "folder/file2.txt",
            "folder/file1.txt",
            "folder/file1.tar.gz",
        ),
    ),
    Case(
        "inclusive_filter",
        ("--filter", "2", "10", "a1", "a2", "a10", "a11", "plain"),
    ),
    Case(
        "reverse_filter",
        (
            "--reverse-filter",
            "2",
            "10",
            "a1",
            "a2",
            "a10",
            "a11",
            "plain",
        ),
    ),
    Case(
        "repeated_exclude",
        (
            "--exclude",
            "2",
            "--exclude",
            "10",
            "a1",
            "a2",
            "a10",
            "a11",
            "plain",
        ),
    ),
    Case(
        "negative_positional_values",
        ("--number-type", "real", "--", "-3", "2", "-10", "1"),
    ),
    Case(
        "locale_sort",
        ("--locale", "Öl5", "Oase4", "Äpfel10", "äpfel2", "apple2"),
    ),
    Case("stdin_newline", stdin=b"file10\nfile2\nfile1\n"),
    Case("stdin_whitespace", stdin=b" file10 \nfile2\n  file1\n"),
    Case(
        "stdin_zero_terminated",
        ("--zero-terminated",),
        b"file10\0file2\0file1\0",
    ),
    Case("empty_stdin", stdin=b""),
    Case(
        "float_filter",
        (
            "--number-type",
            "float",
            "--filter",
            "1.5",
            "3.5",
            "v1.25",
            "v1.5",
            "v2.75",
            "v4.0",
        ),
    ),
    Case(
        "invalid_number_type",
        ("--number-type", "decimal", "a1"),
        error_case=True,
    ),
    Case(
        "reversed_filter_bounds",
        ("--filter", "10", "2", "a1"),
        error_case=True,
    ),
    Case("unknown_option", ("--definitely-unknown",), error_case=True),
)


def parse_arguments() -> argparse.Namespace:
    """Parse harness arguments."""
    parser = argparse.ArgumentParser()
    parser.add_argument("--python-executable", type=Path, required=True)
    parser.add_argument("--rust-binary", type=Path, required=True)
    parser.add_argument("--output-directory", type=Path, default=DEFAULT_OUTPUT)
    return parser.parse_args()


def run_text(*command: str) -> str:
    """Run a trusted local metadata command."""
    result = subprocess.run(  # noqa: S603
        list(command),
        cwd=ROOT,
        check=True,
        capture_output=True,
        text=True,
    )
    return result.stdout.strip()


def sha256(path: Path) -> str:
    """Return the SHA-256 digest of a file."""
    digest = hashlib.sha256()
    with path.open("rb") as stream:
        for chunk in iter(lambda: stream.read(1024 * 1024), b""):
            digest.update(chunk)
    return digest.hexdigest()


def normalized_text(data: bytes) -> str:
    """Decode UTF-8 output and normalize line endings."""
    return data.decode("utf-8", errors="replace").replace("\r\n", "\n")


def normalize_choice_list(diagnostic: str) -> str:
    """Normalize only quote rendering inside a trailing argparse choice list."""
    match = re.search(r"\(choose from (?P<choices>[^)]*)\)$", diagnostic)
    if match is None:
        return diagnostic

    choices = [
        value.strip().removeprefix("'").removesuffix("'")
        for value in match.group("choices").split(",")
    ]
    normalized = "(choose from " + ", ".join(choices) + ")"
    return diagnostic[: match.start()] + normalized


def final_diagnostic(stderr: bytes) -> str:
    """Normalize stable, non-semantic CLI diagnostic presentation drift."""
    lines = [
        re.sub(r"\s+", " ", line).strip()
        for line in normalized_text(stderr).splitlines()
        if line.strip()
    ]
    if not lines:
        return ""

    diagnostic = re.sub(
        r"^.*?(?:-m natsort|natsort): error:",
        "error:",
        lines[-1],
    )
    return normalize_choice_list(diagnostic)


def execute(command: list[str], case: Case, environment: dict[str, str]) -> Result:
    """Execute one CLI case."""
    completed = subprocess.run(  # noqa: S603
        [*command, *case.arguments],
        cwd=ROOT,
        input=case.stdin,
        capture_output=True,
        check=False,
        env=environment,
    )
    return Result(
        return_code=completed.returncode,
        stdout_hex=completed.stdout.hex(),
        stderr_hex=completed.stderr.hex(),
        stdout_text=normalized_text(completed.stdout),
        stderr_text=normalized_text(completed.stderr),
        diagnostic=final_diagnostic(completed.stderr),
    )


def exact_success_match(python_result: Result, rust_result: Result) -> bool:
    """Compare successful behavior byte-for-byte."""
    return (
        python_result.return_code == rust_result.return_code
        and python_result.stdout_hex == rust_result.stdout_hex
        and python_result.stderr_hex == rust_result.stderr_hex
    )


def semantic_error_match(python_result: Result, rust_result: Result) -> bool:
    """Compare error class, stdout, and normalized final diagnostic."""
    return (
        python_result.return_code == rust_result.return_code
        and python_result.stdout_hex == rust_result.stdout_hex
        and python_result.diagnostic == rust_result.diagnostic
    )


def transcript_entry(case: Case, result: Result, *, normalized_error: bool) -> str:
    """Render one stable transcript entry."""
    stderr_value = (
        result.diagnostic
        if normalized_error and case.error_case
        else result.stderr_text
    )
    return "\n".join(
        (
            f"===== {case.name} =====",
            f"arguments={json.dumps(case.arguments, ensure_ascii=False)}",
            f"stdin_hex={case.stdin.hex()}",
            f"return_code={result.return_code}",
            f"stdout_hex={result.stdout_hex}",
            f"stdout_text={json.dumps(result.stdout_text, ensure_ascii=False)}",
            f"stderr={json.dumps(stderr_value, ensure_ascii=False)}",
            "",
        )
    )


def write_diff(
    expected: str,
    actual: str,
    path: Path,
    *,
    expected_name: str,
    actual_name: str,
) -> None:
    """Write a unified diff."""
    diff = difflib.unified_diff(
        expected.splitlines(keepends=True),
        actual.splitlines(keepends=True),
        fromfile=expected_name,
        tofile=actual_name,
    )
    path.write_text("".join(diff), encoding="utf-8")


def main() -> int:  # noqa: PLR0915
    """Run all shared CLI cases and write judge-facing evidence."""
    arguments = parse_arguments()
    python_executable = arguments.python_executable.resolve()
    rust_binary = arguments.rust_binary.resolve()
    output = arguments.output_directory.resolve()

    implementation_commit = run_text("git", "rev-parse", "HEAD")
    working_tree_before_run = run_text(
        "git",
        "status",
        "--short",
    )

    for required in (python_executable, rust_binary):
        if not required.is_file():
            print(f"ERROR: required executable not found: {required}")  # noqa: T201
            return 2

    if output.exists():
        shutil.rmtree(output)
    (output / "raw" / "python").mkdir(parents=True)
    (output / "raw" / "rust").mkdir(parents=True)

    environment = os.environ.copy()
    environment.update(
        {
            "LANG": "C.UTF-8",
            "LC_ALL": "C.UTF-8",
            "PYTHONIOENCODING": "utf-8",
            "PYTHONUTF8": "1",
            "PYTHONPATH": str(ROOT),
        }
    )

    python_command = [str(python_executable), "-X", "utf8", "-m", "natsort"]
    rust_command = [str(rust_binary)]

    case_manifest = []
    mismatches = []
    python_success_entries = []
    rust_success_entries = []
    python_all_entries = []
    rust_all_entries = []

    for case in CASES:
        python_result = execute(python_command, case, environment)
        rust_result = execute(rust_command, case, environment)
        matched = (
            semantic_error_match(python_result, rust_result)
            if case.error_case
            else exact_success_match(python_result, rust_result)
        )
        if not matched:
            mismatches.append(case.name)

        case_manifest.append(
            {
                "name": case.name,
                "arguments": list(case.arguments),
                "stdin_hex": case.stdin.hex(),
                "comparison": (
                    "exit+stdout+normalized-final-diagnostic"
                    if case.error_case
                    else "exact-exit-stdout-stderr-bytes"
                ),
                "matched": matched,
            }
        )

        for implementation, result in (
            ("python", python_result),
            ("rust", rust_result),
        ):
            directory = output / "raw" / implementation
            (directory / f"{case.name}.stdout").write_bytes(
                bytes.fromhex(result.stdout_hex)
            )
            (directory / f"{case.name}.stderr").write_bytes(
                bytes.fromhex(result.stderr_hex)
            )
            (directory / f"{case.name}.exit").write_text(
                f"{result.return_code}\n",
                encoding="utf-8",
            )

        python_all_entries.append(
            transcript_entry(case, python_result, normalized_error=True)
        )
        rust_all_entries.append(
            transcript_entry(case, rust_result, normalized_error=True)
        )
        if not case.error_case:
            python_success_entries.append(
                transcript_entry(case, python_result, normalized_error=False)
            )
            rust_success_entries.append(
                transcript_entry(case, rust_result, normalized_error=False)
            )

    python_success = "".join(python_success_entries)
    rust_success = "".join(rust_success_entries)
    python_all = "".join(python_all_entries)
    rust_all = "".join(rust_all_entries)

    (output / "cases.json").write_text(
        json.dumps(case_manifest, indent=2, ensure_ascii=False) + "\n",
        encoding="utf-8",
    )
    (output / "python_cli_output.txt").write_text(python_all, encoding="utf-8")
    (output / "rust_cli_output.txt").write_text(rust_all, encoding="utf-8")
    (output / "python_success_exact.txt").write_text(
        python_success,
        encoding="utf-8",
    )
    (output / "rust_success_exact.txt").write_text(
        rust_success,
        encoding="utf-8",
    )

    write_diff(
        python_success,
        rust_success,
        output / "cli_success_output.diff",
        expected_name="python-success-exact",
        actual_name="rust-success-exact",
    )
    write_diff(
        python_all,
        rust_all,
        output / "cli_output.diff",
        expected_name="python-all-normalized",
        actual_name="rust-all-normalized",
    )

    module_location = run_text(
        str(python_executable),
        "-X",
        "utf8",
        "-c",
        "import natsort; print(natsort.__file__)",
    )
    metadata = {
        "implementation_commit": implementation_commit,
        "working_tree_clean_before_run": not working_tree_before_run,
        "working_tree_status_before_run": (
            working_tree_before_run.splitlines() if working_tree_before_run else []
        ),
        "python_executable": str(python_executable),
        "python_version": run_text(str(python_executable), "--version"),
        "python_module": module_location,
        "rust_binary": str(rust_binary),
        "rust_binary_sha256": sha256(rust_binary),
        "rustc": run_text("rustc", "--version"),
        "cargo": run_text("cargo", "--version"),
        "locale": "C.UTF-8",
        "total_cases": len(CASES),
        "success_exact_cases": sum(not case.error_case for case in CASES),
        "error_semantic_cases": sum(case.error_case for case in CASES),
        "matched_cases": len(CASES) - len(mismatches),
        "mismatches": mismatches,
        "comparison_policy": {
            "successful_cases": "Exact return code, stdout bytes, and stderr bytes.",
            "error_cases": (
                "Exact return code and stdout bytes; final diagnostic compared after "
                "normalizing executable display name, line endings, wrapping "
                "whitespace, and quote rendering inside argparse choice lists."
            ),
        },
    }
    (output / "summary.json").write_text(
        json.dumps(metadata, indent=2, ensure_ascii=False) + "\n",
        encoding="utf-8",
    )

    summary_lines = [
        "CLI EQUIVALENCE SUMMARY",
        "",
        f"implementation_commit={implementation_commit}",
        f"python_module={module_location}",
        f"rust_binary_sha256={metadata['rust_binary_sha256']}",
        f"total_cases={metadata['total_cases']}",
        f"success_exact_cases={metadata['success_exact_cases']}",
        f"error_semantic_cases={metadata['error_semantic_cases']}",
        f"matched_cases={metadata['matched_cases']}",
        f"mismatch_count={len(mismatches)}",
        f"mismatches={','.join(mismatches) if mismatches else 'none'}",
        "working_tree_clean_before_run="
        + str(metadata["working_tree_clean_before_run"]).lower(),
        "cli_success_output_diff_empty="
        + str((output / "cli_success_output.diff").stat().st_size == 0).lower(),
        "cli_all_normalized_diff_empty="
        + str((output / "cli_output.diff").stat().st_size == 0).lower(),
    ]
    (output / "summary.txt").write_text(
        "\n".join(summary_lines) + "\n",
        encoding="utf-8",
    )

    print("\n".join(summary_lines))  # noqa: T201
    if mismatches:
        print(  # noqa: T201
            "\nFAIL: inspect parity/evidence/cli/cli_output.diff and raw artifacts."
        )
        return 1
    print("\nPASS: Python and Rust CLI behavior matched on all shared cases.")  # noqa: T201
    return 0


if __name__ == "__main__":
    sys.exit(main())
