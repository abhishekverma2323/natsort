from __future__ import annotations

import argparse
import json
import os
import platform
import statistics
import subprocess
import sys
from datetime import datetime, timezone
from pathlib import Path
from typing import Any


SCRIPT = Path(__file__).resolve()
ROOT = SCRIPT.parents[1]
PARITY_DIR = SCRIPT.parent
RUST_DIR = ROOT / "rust-port"
DATA_DIR = PARITY_DIR / "benchmark_data"
GENERATOR = PARITY_DIR / "generate_benchmark_data.py"
PYTHON_BENCHMARK = PARITY_DIR / "benchmark_python.py"
RUST_BINARY = RUST_DIR / "target" / "release" / "benchmark"

MODES = ("default", "float", "real", "path", "locale")
DEFAULT_SIZES = (1_000, 10_000, 50_000)


def run_checked(
    command: list[str],
    *,
    cwd: Path | None = None,
) -> subprocess.CompletedProcess[str]:
    result = subprocess.run(
        command,
        cwd=cwd,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        text=True,
        encoding="utf-8",
        check=False,
    )

    if result.returncode != 0:
        raise RuntimeError(
            f"command failed ({result.returncode}): {' '.join(command)}\n"
            f"stdout:\n{result.stdout}\n"
            f"stderr:\n{result.stderr}"
        )

    return result


def windows_path(path: Path) -> str:
    result = run_checked(["wslpath", "-w", str(path.resolve())])
    return result.stdout.strip()


def parse_json_output(result: subprocess.CompletedProcess[str]) -> dict[str, Any]:
    lines = [line for line in result.stdout.splitlines() if line.strip()]

    if not lines:
        raise RuntimeError("benchmark produced no JSON output")

    try:
        payload = json.loads(lines[-1])
    except json.JSONDecodeError as error:
        raise RuntimeError(
            f"unable to parse benchmark output:\n{result.stdout}"
        ) from error

    if not isinstance(payload, dict):
        raise RuntimeError(f"unexpected benchmark payload: {payload!r}")

    return payload


def generate_data(seed: int, sizes: list[int]) -> None:
    command = [
        sys.executable,
        str(GENERATOR),
        "--seed",
        str(seed),
        "--sizes",
        *(str(size) for size in sizes),
    ]
    print("Generating deterministic benchmark datasets...")
    result = run_checked(command, cwd=ROOT)
    print(result.stdout, end="")


def build_rust() -> None:
    print("Building optimized Rust benchmark binary...")
    result = run_checked(
        ["cargo", "build", "--release", "--bin", "benchmark"],
        cwd=RUST_DIR,
    )

    if result.stderr.strip():
        print(result.stderr, end="")


def run_python_case(
    *,
    python_executable: Path,
    dataset: Path,
    mode: str,
    warmups: int,
    runs: int,
) -> dict[str, Any]:
    environment = os.environ.copy()
    environment["PYTHONUTF8"] = "1"
    environment["PYTHONIOENCODING"] = "utf-8"

    windows_python = python_executable.suffix.lower() == ".exe"

    python_script = (
        windows_path(PYTHON_BENCHMARK)
        if windows_python
        else str(PYTHON_BENCHMARK)
    )

    dataset_path = (
        windows_path(dataset)
        if windows_python
        else str(dataset)
    )

    result = subprocess.run(
        [
            str(python_executable),
            "-X",
            "utf8",
            python_script,
            "--dataset",
            dataset_path,
            "--mode",
            mode,
            "--warmups",
            str(warmups),
            "--runs",
            str(runs),
        ],
        cwd=ROOT,
        env=environment,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        text=True,
        encoding="utf-8",
        check=False,
    )

    if result.returncode != 0:
        raise RuntimeError(
            f"Python benchmark failed:\nstdout:\n{result.stdout}\n"
            f"stderr:\n{result.stderr}"
        )

    return parse_json_output(result)


def run_rust_case(
    *,
    dataset: Path,
    mode: str,
    warmups: int,
    runs: int,
) -> dict[str, Any]:
    result = run_checked(
        [
            str(RUST_BINARY),
            "--dataset",
            str(dataset),
            "--mode",
            mode,
            "--warmups",
            str(warmups),
            "--runs",
            str(runs),
        ],
        cwd=RUST_DIR,
    )
    return parse_json_output(result)


def markdown_report(
    *,
    metadata: dict[str, Any],
    rows: list[dict[str, Any]],
) -> str:
    lines = [
        "# Python vs Rust Natural-Sort Benchmark",
        "",
        "The benchmark measures only in-process sorting. Dataset loading, process "
        "startup, and Rust compilation are outside the timed region.",
        "",
        "## Environment",
        "",
        f"- Generated: `{metadata['generated_at_utc']}`",
        f"- Platform: `{metadata['platform']}`",
        f"- Logical CPUs: `{metadata['logical_cpus']}`",
        f"- Seed: `{metadata['seed']}`",
        f"- Warm-up runs: `{metadata['warmups']}`",
        f"- Measured runs: `{metadata['runs']}`",
        "- Rust profile: `release`",
        "",
        "## Results",
        "",
        "| Mode | Entries | Python median (ms) | Rust median (ms) | Speedup |",
        "|---|---:|---:|---:|---:|",
    ]

    for row in rows:
        lines.append(
            "| {mode} | {entries:,} | {python:.3f} | {rust:.3f} | {speedup:.2f}× |".format(
                mode=row["mode"],
                entries=row["entries"],
                python=row["python"]["median_ms"],
                rust=row["rust"]["median_ms"],
                speedup=row["speedup"],
            )
        )

    grouped: dict[str, list[float]] = {mode: [] for mode in MODES}

    for row in rows:
        grouped[row["mode"]].append(row["speedup"])

    lines.extend(
        [
            "",
            "## Median speedup by mode",
            "",
            "| Mode | Median speedup |",
            "|---|---:|",
        ]
    )

    for mode in MODES:
        values = grouped[mode]
        lines.append(f"| {mode} | {statistics.median(values):.2f}× |")

    lines.extend(
        [
            "",
            "## Methodology",
            "",
            "- Identical deterministic UTF-8 datasets are used by both implementations.",
            "- Each implementation performs warm-up runs before measurement.",
            "- Every measured run sorts the same unsorted in-memory input.",
            "- Reported time is the median of the measured runs.",
            "- Python uses the repository virtual environment and `natsorted`.",
            "- Rust uses `natsorted_with_options` compiled with `--release`.",
            "- Locale mode uses each implementation's system locale configuration.",
            "",
        ]
    )

    return "\n".join(lines)


def main() -> None:
    parser = argparse.ArgumentParser(
        description="Run reproducible in-process Python vs Rust benchmarks."
    )
    parser.add_argument("--seed", type=int, default=20260801)
    parser.add_argument(
        "--sizes",
        type=int,
        nargs="+",
        default=list(DEFAULT_SIZES),
    )
    parser.add_argument("--warmups", type=int, default=2)
    parser.add_argument("--runs", type=int, default=7)
    parser.add_argument(
        "--python-executable",
        type=Path,
        default=ROOT / ".venv" / "Scripts" / "python.exe",
    )
    parser.add_argument("--skip-generate", action="store_true")
    args = parser.parse_args()

    if any(size <= 0 for size in args.sizes):
        parser.error("all sizes must be greater than zero")
    if args.warmups < 0:
        parser.error("--warmups cannot be negative")
    if args.runs <= 0:
        parser.error("--runs must be greater than zero")
    if not args.python_executable.exists():
        parser.error(f"Python executable not found: {args.python_executable}")

    if not args.skip_generate:
        generate_data(args.seed, args.sizes)

    build_rust()

    rows: list[dict[str, Any]] = []

    for mode in MODES:
        for size in args.sizes:
            dataset = DATA_DIR / f"{mode}_{size}.txt"

            if not dataset.exists():
                raise FileNotFoundError(f"dataset not found: {dataset}")

            print(f"Benchmarking mode={mode}, entries={size:,}...")

            python_result = run_python_case(
                python_executable=args.python_executable,
                dataset=dataset,
                mode=mode,
                warmups=args.warmups,
                runs=args.runs,
            )
            rust_result = run_rust_case(
                dataset=dataset,
                mode=mode,
                warmups=args.warmups,
                runs=args.runs,
            )

            python_median = float(python_result["median_ms"])
            rust_median = float(rust_result["median_ms"])
            speedup = python_median / rust_median if rust_median > 0 else float("inf")

            row = {
                "mode": mode,
                "entries": size,
                "python": python_result,
                "rust": rust_result,
                "speedup": speedup,
            }
            rows.append(row)

            print(
                f"  Python {python_median:.3f} ms | "
                f"Rust {rust_median:.3f} ms | "
                f"{speedup:.2f}x"
            )

    metadata = {
        "generated_at_utc": datetime.now(timezone.utc).isoformat(),
        "platform": platform.platform(),
        "logical_cpus": os.cpu_count(),
        "seed": args.seed,
        "sizes": args.sizes,
        "warmups": args.warmups,
        "runs": args.runs,
        "modes": list(MODES),
    }

    report = {
        "metadata": metadata,
        "results": rows,
    }

    json_path = PARITY_DIR / "benchmark_results.json"
    markdown_path = PARITY_DIR / "BENCHMARK_RESULTS.md"

    json_path.write_text(
        json.dumps(report, indent=2, ensure_ascii=False) + "\n",
        encoding="utf-8",
    )
    markdown_path.write_text(
        markdown_report(metadata=metadata, rows=rows),
        encoding="utf-8",
    )

    print()
    print(f"JSON report: {json_path}")
    print(f"Markdown report: {markdown_path}")


if __name__ == "__main__":
    main()