from __future__ import annotations

import argparse
import json
import os
import random
import shutil
import subprocess
import sys
import time
from datetime import datetime, timezone
from pathlib import Path
from typing import Any


SCRIPT = Path(__file__).resolve()
ROOT = SCRIPT.parents[1]
RUST_DIR = ROOT / "rust-port"
RUST_BINARY = (
    RUST_DIR
    / "target"
    / "debug"
    / ("natsort.exe" if os.name == "nt" else "natsort")
)
FAILURE_FILE = ROOT / "parity" / "differential_fuzz_failure.json"

MODES = (
    "default",
    "float",
    "real",
    "signed_int",
    "float_noexp",
    "path",
)

WORDS = (
    "file",
    "File",
    "value",
    "item",
    "alpha",
    "beta",
    "folder",
    "Folder",
    "version",
    "test",
    "apple",
    "Äpfel",
    "Öl",
    "straße",
)

SEPARATORS = ("", "_", "-", ".", " ", "(", ")")

DIGIT_TRANSLATIONS = (
    str.maketrans("0123456789", "０１２３４５６７８９"),
    str.maketrans("0123456789", "٠١٢٣٤٥٦٧٨٩"),
    str.maketrans("0123456789", "०१२३४५६७८९"),
)


def oracle_server() -> None:
    from natsort import natsorted, ns

    algorithms = {
        "default": ns.DEFAULT,
        "float": ns.FLOAT,
        "real": ns.REAL,
        "signed_int": ns.SIGNED,
        "float_noexp": ns.FLOAT | ns.NOEXP,
        "path": ns.PATH,
    }

    for raw_line in sys.stdin:
        raw_line = raw_line.strip()

        if not raw_line:
            continue

        request = json.loads(raw_line)

        mode = request["mode"]
        entries = request["entries"]
        reverse = request["reverse"]

        result = natsorted(
            entries,
            alg=algorithms[mode],
            reverse=reverse,
        )

        sys.stdout.write(
            json.dumps(
                result,
                ensure_ascii=True,
                separators=(",", ":"),
            )
            + "\n"
        )
        sys.stdout.flush()


def maybe_translate_digits(
    value: str,
    rng: random.Random,
) -> str:
    if rng.random() < 0.18:
        return value.translate(rng.choice(DIGIT_TRANSLATIONS))

    return value


def integer_token(
    rng: random.Random,
    *,
    signed: bool,
) -> str:
    number = rng.randint(0, 100_000)
    raw = str(number)

    if rng.random() < 0.35:
        extra_zeroes = rng.randint(1, 4)
        raw = raw.zfill(len(raw) + extra_zeroes)

    raw = maybe_translate_digits(raw, rng)

    if signed and rng.random() < 0.55:
        raw = rng.choice(("+", "-")) + raw

    return raw


def float_token(
    rng: random.Random,
    *,
    signed: bool,
    include_exponent: bool,
) -> str:
    whole = rng.randint(0, 10_000)
    fraction = rng.randint(0, 9_999)

    style = rng.randint(0, 3)

    if style == 0:
        raw = f"{whole}.{fraction:04d}"
    elif style == 1:
        raw = f".{fraction:04d}"
    elif style == 2:
        raw = f"{whole}."
    else:
        raw = str(whole)

    if include_exponent and rng.random() < 0.35:
        exponent = rng.randint(-8, 8)

        if exponent >= 0 and rng.random() < 0.5:
            exponent_text = f"+{exponent}"
        else:
            exponent_text = str(exponent)

        raw += rng.choice(("e", "E")) + exponent_text

    if signed and rng.random() < 0.55:
        raw = rng.choice(("+", "-")) + raw

    return raw


def general_entry(
    rng: random.Random,
    mode: str,
) -> str:
    prefix = rng.choice(WORDS)
    separator = rng.choice(SEPARATORS)

    if mode == "float":
        number = float_token(
            rng,
            signed=False,
            include_exponent=True,
        )
    elif mode == "real":
        number = float_token(
            rng,
            signed=True,
            include_exponent=True,
        )
    elif mode == "signed_int":
        number = integer_token(rng, signed=True)
    elif mode == "float_noexp":
        number = float_token(
            rng,
            signed=False,
            include_exponent=True,
        )
    else:
        number = integer_token(rng, signed=False)

    result = prefix + separator + number

    if rng.random() < 0.45:
        result += rng.choice(SEPARATORS)
        result += rng.choice(WORDS)

    if rng.random() < 0.28:
        result += rng.choice(SEPARATORS)
        result += integer_token(rng, signed=False)

    return result


def path_entry(rng: random.Random) -> str:
    directory = rng.choice(
        (
            "Folder",
            "folder",
            "Dir",
            "dir",
            "Äpfel",
            ".hidden",
        )
    )

    directory_number = integer_token(rng, signed=False)
    file_number = integer_token(rng, signed=False)
    extension = rng.choice(("txt", "csv", "tar.gz", "json", "log"))

    if rng.random() < 0.25:
        directory_component = directory
    else:
        directory_component = (
            directory
            + rng.choice(("", " ", "_", "-"))
            + directory_number
        )

    separator = "/" if os.name != "nt" else rng.choice(("/", "\\"))

    result = (
        directory_component
        + separator
        + rng.choice(("file", "File", "item", "data"))
        + file_number
        + "."
        + extension
    )

    if rng.random() < 0.15:
        result = "." + separator + result

    return result


def plain_entry(rng: random.Random) -> str:
    return rng.choice(WORDS) + rng.choice(
        (
            "",
            "_plain",
            "-text",
            ".data",
            " (copy)",
        )
    )


def generate_entries(
    rng: random.Random,
    mode: str,
) -> list[str]:
    size = rng.randint(2, 30)
    entries: list[str] = []

    for _ in range(size):
        if entries and rng.random() < 0.15:
            entries.append(rng.choice(entries))
            continue

        if rng.random() < 0.12:
            entries.append(plain_entry(rng))
        elif mode == "path":
            entries.append(path_entry(rng))
        else:
            entries.append(general_entry(rng, mode))

    return entries


def rust_arguments(
    mode: str,
    reverse: bool,
) -> list[str]:
    arguments: list[str] = []

    if mode == "float":
        arguments.extend(("-t", "float"))
    elif mode == "real":
        arguments.extend(("-t", "real"))
    elif mode == "signed_int":
        arguments.append("--sign")
    elif mode == "float_noexp":
        arguments.extend(("-t", "float", "--noexp"))
    elif mode == "path":
        arguments.append("--paths")

    if reverse:
        arguments.append("--reverse")

    return arguments


def normalize_output(data: bytes) -> list[str]:
    text = data.decode("utf-8")
    text = text.replace("\r\n", "\n")
    return text.splitlines()


def build_rust_binary() -> None:
    print("Building Rust CLI...")

    result = subprocess.run(
        ["cargo", "build", "--quiet", "--bin", "natsort"],
        cwd=RUST_DIR,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        check=False,
    )

    if result.returncode != 0:
        sys.stderr.write(
            result.stderr.decode(
                "utf-8",
                errors="backslashreplace",
            )
        )
        raise SystemExit(result.returncode)



def default_python_oracle() -> Path:
    candidates = (
        ROOT / ".venv" / "Scripts" / "python.exe",
        ROOT / ".venv" / "bin" / "python",
    )

    for candidate in candidates:
        if candidate.exists():
            return candidate

    return Path(sys.executable)


def resolve_python_executable(value: Path) -> Path:
    if value.exists():
        return value.resolve()

    resolved = shutil.which(str(value))

    if resolved is not None:
        return Path(resolved).resolve()

    raise FileNotFoundError(
        f"Python oracle executable was not found: {value}"
    )


def windows_path(path: Path) -> str:
    result = subprocess.run(
        ["wslpath", "-w", str(path)],
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        text=True,
        check=False,
    )

    if result.returncode != 0:
        raise RuntimeError(
            f"Unable to convert {path} to a Windows path:\n" + result.stderr
        )

    return result.stdout.strip()


def start_oracle(
    python_executable: Path,
) -> subprocess.Popen[str]:
    resolved_python = resolve_python_executable(python_executable)

    environment = os.environ.copy()
    environment["PYTHONUTF8"] = "1"
    environment["PYTHONIOENCODING"] = "utf-8"

    uses_windows_python_from_wsl = (
        os.name != "nt" and resolved_python.suffix.lower() == ".exe"
    )
    script_path = windows_path(SCRIPT) if uses_windows_python_from_wsl else str(SCRIPT)
    root_path = windows_path(ROOT) if uses_windows_python_from_wsl else str(ROOT)
    pythonpath_separator = ";" if uses_windows_python_from_wsl else os.pathsep
    existing_pythonpath = environment.get("PYTHONPATH")

    environment["PYTHONPATH"] = (
        root_path
        if not existing_pythonpath
        else root_path + pythonpath_separator + existing_pythonpath
    )

    return subprocess.Popen(
        [
            str(resolved_python),
            "-X",
            "utf8",
            script_path,
            "--oracle-server",
        ],
        stdin=subprocess.PIPE,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        text=True,
        encoding="utf-8",
        env=environment,
        bufsize=1,
    )


def request_oracle(
    oracle: subprocess.Popen[str],
    request: dict[str, Any],
) -> list[str]:
    if oracle.stdin is None or oracle.stdout is None:
        raise RuntimeError("Oracle pipes are unavailable.")

    oracle.stdin.write(
        json.dumps(
            request,
            ensure_ascii=True,
            separators=(",", ":"),
        )
        + "\n"
    )
    oracle.stdin.flush()

    response = oracle.stdout.readline()

    if not response:
        error_output = ""

        if oracle.stderr is not None:
            error_output = oracle.stderr.read()

        raise RuntimeError(
            "Python oracle exited without returning a result.\n"
            + error_output
        )

    result = json.loads(response)

    if not isinstance(result, list):
        raise RuntimeError(
            f"Unexpected oracle response: {result!r}"
        )

    return result


def run_rust_case(
    rng: random.Random,
    mode: str,
    entries: list[str],
    reverse: bool,
) -> tuple[
    subprocess.CompletedProcess[bytes],
    list[str],
    bytes,
]:
    arguments = rust_arguments(mode, reverse)
    use_stdin = rng.random() < 0.35
    stdin_data = b""

    if use_stdin:
        zero_terminated = rng.random() < 0.30

        if zero_terminated:
            arguments.append("--zero-terminated")
            separator = "\0"
        else:
            separator = "\n"

        input_text = separator.join(entries)

        if rng.random() < 0.5:
            input_text += separator

        stdin_data = input_text.encode("utf-8")
    else:
        arguments.extend(("--", *entries))

    result = subprocess.run(
        [str(RUST_BINARY), *arguments],
        input=stdin_data,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        cwd=RUST_DIR,
        check=False,
    )

    return result, arguments, stdin_data


def save_failure(
    *,
    seed: int,
    case_index: int,
    request: dict[str, Any],
    rust_arguments_used: list[str],
    rust_stdin: bytes,
    expected: list[str],
    actual: list[str],
    returncode: int,
    stderr: bytes,
) -> None:
    payload = {
        "seed": seed,
        "case_index": case_index,
        "request": request,
        "rust_arguments": rust_arguments_used,
        "rust_stdin": repr(rust_stdin),
        "expected": expected,
        "actual": actual,
        "returncode": returncode,
        "stderr": stderr.decode(
            "utf-8",
            errors="backslashreplace",
        ),
    }

    FAILURE_FILE.write_text(
        json.dumps(
            payload,
            ensure_ascii=False,
            indent=2,
        ),
        encoding="utf-8",
    )


def utc_timestamp() -> str:
    return datetime.now(timezone.utc).isoformat().replace("+00:00", "Z")


def write_summary(
    path: Path,
    payload: dict[str, Any],
) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(
        json.dumps(
            payload,
            ensure_ascii=False,
            indent=2,
        )
        + "\n",
        encoding="utf-8",
    )


def controller() -> None:
    parser = argparse.ArgumentParser(
        description=("Differentially compare Python natsort with the Rust CLI.")
    )

    run_limit = parser.add_mutually_exclusive_group()

    run_limit.add_argument(
        "--cases",
        type=int,
        help=(
            "Number of generated cases. Defaults to 1000 when no duration is supplied."
        ),
    )
    run_limit.add_argument(
        "--duration-seconds",
        type=float,
        help=(
            "Run complete differential cases until at least "
            "this many seconds have elapsed."
        ),
    )

    parser.add_argument(
        "--seed",
        type=int,
        default=20260801,
        help="Deterministic random seed.",
    )
    parser.add_argument(
        "--python-oracle",
        type=Path,
        default=default_python_oracle(),
        help=(
            "Python executable used for the reference oracle. "
            "Accepts an absolute path or a command available on PATH."
        ),
    )
    parser.add_argument(
        "--summary-file",
        type=Path,
        help=("Optional JSON file for a machine-readable run summary."),
    )

    arguments = parser.parse_args()

    case_limit = arguments.cases
    duration_target = arguments.duration_seconds

    if case_limit is None and duration_target is None:
        case_limit = 1000

    if case_limit is not None and case_limit <= 0:
        parser.error("--cases must be greater than zero")

    if duration_target is not None and duration_target <= 0:
        parser.error("--duration-seconds must be greater than zero")

    summary_file = arguments.summary_file

    if summary_file is not None and not summary_file.is_absolute():
        summary_file = ROOT / summary_file

    rng = random.Random(arguments.seed)

    if FAILURE_FILE.exists():
        FAILURE_FILE.unlink()

    if summary_file is not None and summary_file.exists():
        summary_file.unlink()

    build_rust_binary()

    resolved_oracle = resolve_python_executable(arguments.python_oracle)
    oracle = start_oracle(resolved_oracle)

    mode_counts = {mode: 0 for mode in MODES}

    run_mode = "duration" if duration_target is not None else "cases"
    case_index = 0
    started_at_utc = utc_timestamp()
    monotonic_start = time.monotonic()

    try:
        while True:
            if case_limit is not None and case_index >= case_limit:
                break

            if (
                duration_target is not None
                and case_index > 0
                and (time.monotonic() - monotonic_start >= duration_target)
            ):
                break

            case_index += 1
            mode = rng.choice(MODES)
            reverse = rng.random() < 0.35
            entries = generate_entries(rng, mode)

            request = {
                "mode": mode,
                "entries": entries,
                "reverse": reverse,
            }

            expected = request_oracle(
                oracle,
                request,
            )

            rust_result, used_arguments, rust_stdin = run_rust_case(
                rng,
                mode,
                entries,
                reverse,
            )

            actual = normalize_output(rust_result.stdout)

            mode_counts[mode] += 1

            if rust_result.returncode != 0 or actual != expected:
                save_failure(
                    seed=arguments.seed,
                    case_index=case_index,
                    request=request,
                    rust_arguments_used=used_arguments,
                    rust_stdin=rust_stdin,
                    expected=expected,
                    actual=actual,
                    returncode=rust_result.returncode,
                    stderr=rust_result.stderr,
                )

                failure_elapsed = time.monotonic() - monotonic_start
                failure_ended_at = utc_timestamp()

                if summary_file is not None:
                    write_summary(
                        summary_file,
                        {
                            "schema_version": 1,
                            "status": "failed",
                            "seed": arguments.seed,
                            "run_mode": run_mode,
                            "requested_cases": case_limit,
                            "requested_duration_seconds": (duration_target),
                            "completed_cases": case_index,
                            "passed_cases": case_index - 1,
                            "failed_case_index": case_index,
                            "failed_mode": mode,
                            "elapsed_seconds": round(
                                failure_elapsed,
                                6,
                            ),
                            "started_at_utc": (started_at_utc),
                            "ended_at_utc": (failure_ended_at),
                            "duration_target_met": False,
                            "zero_divergences": False,
                            "mode_counts": mode_counts,
                            "python_oracle": str(resolved_oracle),
                            "controller_python": (sys.executable),
                            "rust_binary": str(RUST_BINARY),
                            "platform": sys.platform,
                            "command": sys.argv,
                            "failure_file": str(FAILURE_FILE),
                        },
                    )

                print()
                print("DIFFERENTIAL FAILURE")
                print(f"Seed: {arguments.seed}")
                print(f"Case: {case_index}")
                print(f"Mode: {mode}")
                print(f"Reverse: {reverse}")
                print(f"Arguments: {used_arguments!r}")
                print(f"Entries: {entries!r}")
                print(f"Expected: {expected!r}")
                print(f"Actual: {actual!r}")
                print(f"Failure saved to: {FAILURE_FILE}")

                if summary_file is not None:
                    print(f"Summary saved to: {summary_file}")

                raise SystemExit(1)

            if case_index % 100 == 0:
                if case_limit is not None:
                    print(f"Passed {case_index}/{case_limit} cases")
                else:
                    elapsed = time.monotonic() - monotonic_start
                    print(f"Passed {case_index} cases in {elapsed:.2f} seconds")

    finally:
        if oracle.stdin is not None:
            oracle.stdin.close()

        oracle.terminate()

        try:
            oracle.wait(timeout=5)
        except subprocess.TimeoutExpired:
            oracle.kill()
            oracle.wait()

    elapsed_seconds = time.monotonic() - monotonic_start
    ended_at_utc = utc_timestamp()
    duration_target_met = duration_target is None or elapsed_seconds >= duration_target

    summary = {
        "schema_version": 1,
        "status": "passed",
        "seed": arguments.seed,
        "run_mode": run_mode,
        "requested_cases": case_limit,
        "requested_duration_seconds": duration_target,
        "completed_cases": case_index,
        "passed_cases": case_index,
        "elapsed_seconds": round(
            elapsed_seconds,
            6,
        ),
        "started_at_utc": started_at_utc,
        "ended_at_utc": ended_at_utc,
        "duration_target_met": duration_target_met,
        "zero_divergences": True,
        "mode_counts": mode_counts,
        "python_oracle": str(resolved_oracle),
        "controller_python": sys.executable,
        "rust_binary": str(RUST_BINARY),
        "platform": sys.platform,
        "command": sys.argv,
        "failure_file": str(FAILURE_FILE),
    }

    if summary_file is not None:
        write_summary(
            summary_file,
            summary,
        )

    print()
    print("DIFFERENTIAL FUZZING PASSED")
    print(f"Seed: {arguments.seed}")
    print(f"Run mode: {run_mode}")

    if case_limit is not None:
        print(f"Requested cases: {case_limit}")

    if duration_target is not None:
        print(f"Requested duration: {duration_target:.2f} seconds")

    print(f"Elapsed: {elapsed_seconds:.2f} seconds")
    print(f"Cases: {case_index}")
    print("Zero divergences: yes")

    for mode in MODES:
        print(f"{mode}: {mode_counts[mode]}")

    if summary_file is not None:
        print(f"Summary: {summary_file}")


def main() -> None:
    if "--oracle-server" in sys.argv:
        oracle_server()
    else:
        controller()


if __name__ == "__main__":
    main()
