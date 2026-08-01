from __future__ import annotations

import argparse
import json
import os
import random
import shutil
import subprocess
import sys
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

    separator = rng.choice(("/", "\\"))

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


def windows_script_path() -> str:
    result = subprocess.run(
        ["wslpath", "-w", str(SCRIPT)],
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        text=True,
        check=False,
    )

    if result.returncode != 0:
        raise RuntimeError(
            "Unable to convert the fuzz script path to a Windows path:\n"
            + result.stderr
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
        os.name != "nt"
        and resolved_python.suffix.lower() == ".exe"
    )
    script_path = (
        windows_script_path()
        if uses_windows_python_from_wsl
        else str(SCRIPT)
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


def controller() -> None:
    parser = argparse.ArgumentParser(
        description=(
            "Differentially compare Python natsort "
            "with the Rust CLI."
        )
    )
    parser.add_argument(
        "--cases",
        type=int,
        default=1000,
        help="Number of generated cases.",
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

    arguments = parser.parse_args()

    if arguments.cases <= 0:
        parser.error("--cases must be greater than zero")

    rng = random.Random(arguments.seed)

    if FAILURE_FILE.exists():
        FAILURE_FILE.unlink()

    build_rust_binary()

    oracle = start_oracle(arguments.python_oracle)

    mode_counts = {
        mode: 0
        for mode in MODES
    }

    try:
        for case_index in range(1, arguments.cases + 1):
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

            rust_result, used_arguments, rust_stdin = (
                run_rust_case(
                    rng,
                    mode,
                    entries,
                    reverse,
                )
            )

            actual = normalize_output(
                rust_result.stdout
            )

            mode_counts[mode] += 1

            if (
                rust_result.returncode != 0
                or actual != expected
            ):
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
                print(
                    f"Failure saved to: "
                    f"{FAILURE_FILE}"
                )

                raise SystemExit(1)

            if case_index % 100 == 0:
                print(
                    f"Passed {case_index}/"
                    f"{arguments.cases} cases"
                )

    finally:
        if oracle.stdin is not None:
            oracle.stdin.close()

        oracle.terminate()

        try:
            oracle.wait(timeout=5)
        except subprocess.TimeoutExpired:
            oracle.kill()
            oracle.wait()

    print()
    print("DIFFERENTIAL FUZZING PASSED")
    print(f"Seed: {arguments.seed}")
    print(f"Cases: {arguments.cases}")

    for mode in MODES:
        print(
            f"{mode}: {mode_counts[mode]}"
        )


def main() -> None:
    if "--oracle-server" in sys.argv:
        oracle_server()
    else:
        controller()


if __name__ == "__main__":
    main()
