from __future__ import annotations

import os
import subprocess
import sys
from pathlib import Path
from typing import Sequence


ROOT = Path(__file__).resolve().parents[1]

ENVIRONMENT = os.environ.copy()
ENVIRONMENT["PYTHONUTF8"] = "1"
ENVIRONMENT["PYTHONIOENCODING"] = "utf-8"


def run_case(
    name: str,
    arguments: Sequence[str],
    stdin: bytes = b"",
) -> None:
    command = [
        sys.executable,
        "-X",
        "utf8",
        "-m",
        "natsort",
        *arguments,
    ]

    result = subprocess.run(
        command,
        input=stdin,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        cwd=ROOT,
        env=ENVIRONMENT,
        check=False,
    )

    print(f"\n===== {name} =====")
    print("arguments:", repr(list(arguments)))
    print("stdin:", repr(stdin))
    print("returncode:", result.returncode)
    print("stdout_bytes:", repr(result.stdout))
    print(
        "stdout_text:",
        repr(
            result.stdout.decode(
                "utf-8",
                errors="backslashreplace",
            )
        ),
    )
    print(
        "stderr_text:",
        repr(
            result.stderr.decode(
                "utf-8",
                errors="backslashreplace",
            )
        ),
    )


def main() -> None:
    run_case(
        "positional_basic",
        ["file10", "file2", "file1"],
    )

    run_case(
        "positional_reverse",
        ["--reverse", "file10", "file2", "file1"],
    )

    run_case(
        "path_sort",
        [
            "--paths",
            "Folder (10)/",
            "Folder (1)/",
            "Folder/",
        ],
    )

    run_case(
        "default_integer",
        [
            "value10.2",
            "value2.5",
            "value1.9",
        ],
    )

    run_case(
        "float_long_name",
        [
            "--number-type",
            "float",
            "value10.2",
            "value2.5",
            "value1.9",
        ],
    )

    run_case(
        "float_short_name",
        [
            "-t",
            "f",
            "value10.2",
            "value2.5",
            "value1.9",
        ],
    )

    run_case(
        "real_long_name",
        [
            "-t",
            "real",
            "--",
            "value-2.5",
            "value1",
            "value-10",
        ],
    )

    run_case(
        "real_short_name",
        [
            "-t",
            "r",
            "--",
            "value-2.5",
            "value1",
            "value-10",
        ],
    )

    run_case(
        "signed_integer",
        [
            "--sign",
            "--",
            "value-2",
            "value1",
            "value-10",
            "value+3",
        ],
    )

    run_case(
        "real_with_nosign",
        [
            "-t",
            "real",
            "--nosign",
            "--",
            "value-2.5",
            "value1",
            "value-10",
        ],
    )

    run_case(
        "float_with_exponent",
        [
            "-t",
            "float",
            "value1e3",
            "value20",
            "value3e1",
        ],
    )

    run_case(
        "float_without_exponent",
        [
            "-t",
            "float",
            "--noexp",
            "value1e3",
            "value20",
            "value3e1",
        ],
    )

    run_case(
        "locale_sort",
        [
            "--locale",
            "Öl5",
            "Oase4",
            "Äpfel10",
            "äpfel2",
            "apple2",
        ],
    )

    run_case(
        "stdin_newline",
        [],
        b"file10\nfile2\nfile1\n",
    )

    run_case(
        "stdin_without_final_newline",
        [],
        b"file10\nfile2\nfile1",
    )

    run_case(
        "stdin_empty",
        [],
        b"",
    )

    run_case(
        "stdin_zero_terminated",
        ["--zero-terminated"],
        b"file10\0file2\0file1\0",
    )

    run_case(
        "stdin_zero_without_final_null",
        ["-z"],
        b"file10\0file2\0file1",
    )

    run_case(
        "filter_integer_range",
        [
            "--filter",
            "2",
            "10",
            "value1",
            "value2",
            "value5",
            "value10",
            "value11",
            "plain",
        ],
    )

    run_case(
        "reverse_filter_integer_range",
        [
            "--reverse-filter",
            "2",
            "10",
            "value1",
            "value2",
            "value5",
            "value10",
            "value11",
            "plain",
        ],
    )

    run_case(
        "filter_multiple_numbers",
        [
            "--filter",
            "2",
            "10",
            "a1b20",
            "a1b5",
            "a20b30",
            "plain",
        ],
    )

    run_case(
        "filter_float_range",
        [
            "-t",
            "float",
            "--filter",
            "1.5",
            "3.0",
            "value1.25",
            "value1.5",
            "value2.75",
            "value3.0",
            "value3.5",
        ],
    )

    run_case(
        "exclude_integer",
        [
            "--exclude",
            "5",
            "value5",
            "value05",
            "value5.0",
            "value15",
            "plain",
        ],
    )

    run_case(
        "exclude_float",
        [
            "-t",
            "float",
            "--exclude",
            "2.5",
            "value2.5",
            "value02.50",
            "value2",
            "value25",
            "plain",
        ],
    )

    run_case(
        "invalid_number_type",
        [
            "--number-type",
            "decimal",
            "value1",
        ],
    )

    run_case(
        "missing_filter_bound",
        [
            "--filter",
            "1",
        ],
    )

    run_case(
        "invalid_filter_bound",
        [
            "--filter",
            "low",
            "high",
            "value1",
        ],
    )

    run_case(
        "filter_reversed_bounds",
        [
            "--filter",
            "10",
            "2",
            "value1",
            "value5",
            "value11",
        ],
    )

    run_case(
        "unknown_option",
        [
            "--unknown-option",
            "value1",
        ],
    )


if __name__ == "__main__":
    main()