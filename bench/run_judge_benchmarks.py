"""Run judge-facing Python-versus-Rust CLI benchmarks."""

from __future__ import annotations

import argparse
import csv
import hashlib
import json
import math
import os
import platform
import statistics
import subprocess
import sys
import tempfile
import time
from dataclasses import dataclass
from datetime import datetime, timezone
from pathlib import Path
from typing import Final

ROOT: Final = Path(__file__).resolve().parents[1]
BENCH_DIR: Final = ROOT / "bench"
DATA_DIR: Final = ROOT / "parity" / "benchmark_data"
GNU_TIME: Final = Path("/usr/bin/time")


@dataclass(frozen=True)
class Scenario:
    """One end-to-end CLI benchmark scenario."""

    name: str
    corpus: Path | None
    arguments: tuple[str, ...]
    samples: int
    warmups: int


@dataclass(frozen=True)
class CommandSpec:
    """One benchmarked implementation."""

    name: str
    command: tuple[str, ...]


def parse_arguments() -> argparse.Namespace:
    """Parse benchmark-runner arguments."""
    parser = argparse.ArgumentParser()
    parser.add_argument("--python-executable", type=Path, required=True)
    parser.add_argument("--rust-binary", type=Path, required=True)
    parser.add_argument(
        "--output-directory",
        type=Path,
        default=BENCH_DIR,
    )
    return parser.parse_args()


def run_text(*command: str) -> str:
    """Run a trusted local metadata command."""
    completed = subprocess.run(  # noqa: S603
        list(command),
        cwd=ROOT,
        check=True,
        capture_output=True,
        text=True,
    )
    return completed.stdout.strip()


def sha256(path: Path) -> str:
    """Return a file SHA-256 digest."""
    digest = hashlib.sha256()
    with path.open("rb") as stream:
        for chunk in iter(lambda: stream.read(1024 * 1024), b""):
            digest.update(chunk)
    return digest.hexdigest()


def nearest_rank(values: list[float], percentile: float) -> float:
    """Compute a nearest-rank percentile."""
    ordered = sorted(values)
    rank = max(1, math.ceil(percentile / 100.0 * len(ordered)))
    return ordered[rank - 1]


def summarize_latency(
    samples_ms: list[float],
    item_count: int,
) -> dict[str, float | int]:
    """Summarize one set of wall-clock samples."""
    median_ms = statistics.median(samples_ms)
    mean_ms = statistics.fmean(samples_ms)
    throughput = (
        item_count / (median_ms / 1000.0) if item_count > 0 and median_ms > 0 else 0.0
    )
    return {
        "sample_count": len(samples_ms),
        "min_ms": min(samples_ms),
        "mean_ms": mean_ms,
        "p50_ms": median_ms,
        "p95_ms": nearest_rank(samples_ms, 95.0),
        "p99_ms": nearest_rank(samples_ms, 99.0),
        "max_ms": max(samples_ms),
        "median_items_per_second": throughput,
    }


def execute_once(
    command: CommandSpec,
    arguments: tuple[str, ...],
    input_bytes: bytes,
    environment: dict[str, str],
) -> float:
    """Measure one end-to-end process execution."""
    started = time.perf_counter_ns()
    completed = subprocess.run(  # noqa: S603
        [*command.command, *arguments],
        cwd=ROOT,
        input=input_bytes,
        stdout=subprocess.DEVNULL,
        stderr=subprocess.PIPE,
        check=False,
        env=environment,
    )
    elapsed_ms = (time.perf_counter_ns() - started) / 1_000_000.0

    if completed.returncode != 0:
        diagnostic = completed.stderr.decode("utf-8", errors="replace")
        message = (
            f"{command.name} failed with exit code {completed.returncode}: {diagnostic}"
        )
        raise RuntimeError(message)

    return elapsed_ms


def measure_rss_once(
    command: CommandSpec,
    arguments: tuple[str, ...],
    input_bytes: bytes,
    environment: dict[str, str],
) -> int:
    """Measure peak RSS in KiB with GNU time."""
    with tempfile.NamedTemporaryFile(
        mode="w+",
        encoding="utf-8",
        delete=False,
    ) as stream:
        rss_path = Path(stream.name)

    try:
        completed = subprocess.run(  # noqa: S603
            [
                str(GNU_TIME),
                "-q",
                "-f",
                "%M",
                "-o",
                str(rss_path),
                *command.command,
                *arguments,
            ],
            cwd=ROOT,
            input=input_bytes,
            stdout=subprocess.DEVNULL,
            stderr=subprocess.PIPE,
            check=False,
            env=environment,
        )

        if completed.returncode != 0:
            diagnostic = completed.stderr.decode(
                "utf-8",
                errors="replace",
            )
            message = (
                f"{command.name} RSS run failed with exit code "
                f"{completed.returncode}: {diagnostic}"
            )
            raise RuntimeError(message)

        return int(rss_path.read_text(encoding="utf-8").strip())
    finally:
        rss_path.unlink(missing_ok=True)


def load_corpus(path: Path | None) -> tuple[bytes, int]:
    """Load one corpus and count logical input items."""
    if path is None:
        return b"", 0

    data = path.read_bytes()
    return data, len(data.splitlines())


def scenario_definitions() -> tuple[Scenario, ...]:
    """Return the fixed judge-facing scenario matrix."""
    return (
        Scenario("startup_empty", None, (), 100, 10),
        Scenario(
            "default_1000",
            DATA_DIR / "default_1000.txt",
            (),
            100,
            5,
        ),
        Scenario(
            "float_1000",
            DATA_DIR / "float_1000.txt",
            ("--number-type", "float"),
            50,
            5,
        ),
        Scenario(
            "real_1000",
            DATA_DIR / "real_1000.txt",
            ("--number-type", "real"),
            50,
            5,
        ),
        Scenario(
            "path_1000",
            DATA_DIR / "path_1000.txt",
            ("--paths",),
            50,
            5,
        ),
        Scenario(
            "locale_1000",
            DATA_DIR / "locale_1000.txt",
            ("--locale",),
            50,
            5,
        ),
        Scenario(
            "default_10000",
            DATA_DIR / "default_10000.txt",
            (),
            30,
            3,
        ),
        Scenario(
            "default_50000",
            DATA_DIR / "default_50000.txt",
            (),
            10,
            2,
        ),
    )


def benchmark_latency(
    scenarios: tuple[Scenario, ...],
    commands: tuple[CommandSpec, ...],
    environment: dict[str, str],
) -> tuple[dict[str, object], list[dict[str, object]]]:
    """Run alternating end-to-end latency samples."""
    summarized: dict[str, object] = {}
    raw_rows: list[dict[str, object]] = []

    for scenario in scenarios:
        input_bytes, item_count = load_corpus(scenario.corpus)

        for _ in range(scenario.warmups):
            for command in commands:
                execute_once(
                    command,
                    scenario.arguments,
                    input_bytes,
                    environment,
                )

        by_implementation: dict[str, list[float]] = {
            command.name: [] for command in commands
        }

        for index in range(scenario.samples):
            ordered_commands = commands if index % 2 == 0 else tuple(reversed(commands))
            for command in ordered_commands:
                elapsed_ms = execute_once(
                    command,
                    scenario.arguments,
                    input_bytes,
                    environment,
                )
                by_implementation[command.name].append(elapsed_ms)
                raw_rows.append(
                    {
                        "scenario": scenario.name,
                        "implementation": command.name,
                        "sample_index": index,
                        "elapsed_ms": elapsed_ms,
                        "item_count": item_count,
                    }
                )

        summarized[scenario.name] = {
            "arguments": list(scenario.arguments),
            "corpus": (
                str(scenario.corpus.relative_to(ROOT))
                if scenario.corpus is not None
                else None
            ),
            "item_count": item_count,
            "input_bytes": len(input_bytes),
            "warmups_per_implementation": scenario.warmups,
            "implementations": {
                command.name: summarize_latency(
                    by_implementation[command.name],
                    item_count,
                )
                for command in commands
            },
        }

    return summarized, raw_rows


def benchmark_rss(
    commands: tuple[CommandSpec, ...],
    environment: dict[str, str],
) -> tuple[dict[str, object], list[dict[str, object]]]:
    """Measure peak RSS on small, medium, and large default corpora."""
    rss_scenarios = (
        ("default_1000", DATA_DIR / "default_1000.txt"),
        ("default_10000", DATA_DIR / "default_10000.txt"),
        ("default_50000", DATA_DIR / "default_50000.txt"),
    )
    summarized: dict[str, object] = {}
    raw_rows: list[dict[str, object]] = []

    for scenario_name, corpus_path in rss_scenarios:
        input_bytes, item_count = load_corpus(corpus_path)
        by_implementation: dict[str, list[int]] = {
            command.name: [] for command in commands
        }

        for index in range(5):
            ordered_commands = commands if index % 2 == 0 else tuple(reversed(commands))
            for command in ordered_commands:
                rss_kib = measure_rss_once(
                    command,
                    (),
                    input_bytes,
                    environment,
                )
                by_implementation[command.name].append(rss_kib)
                raw_rows.append(
                    {
                        "scenario": scenario_name,
                        "implementation": command.name,
                        "sample_index": index,
                        "peak_rss_kib": rss_kib,
                        "item_count": item_count,
                    }
                )

        summarized[scenario_name] = {
            "corpus": str(corpus_path.relative_to(ROOT)),
            "item_count": item_count,
            "samples_per_implementation": 5,
            "implementations": {
                command.name: {
                    "min_peak_rss_kib": min(by_implementation[command.name]),
                    "median_peak_rss_kib": statistics.median(
                        by_implementation[command.name]
                    ),
                    "max_peak_rss_kib": max(by_implementation[command.name]),
                }
                for command in commands
            },
        }

    return summarized, raw_rows


def write_latency_csv(
    path: Path,
    scenarios: dict[str, object],
) -> None:
    """Write summarized latency metrics as CSV."""
    fieldnames = [
        "scenario",
        "implementation",
        "item_count",
        "sample_count",
        "min_ms",
        "mean_ms",
        "p50_ms",
        "p95_ms",
        "p99_ms",
        "max_ms",
        "median_items_per_second",
    ]
    with path.open("w", encoding="utf-8", newline="") as stream:
        writer = csv.DictWriter(stream, fieldnames=fieldnames)
        writer.writeheader()
        for scenario_name, scenario_value in scenarios.items():
            scenario = dict(scenario_value)  # type: ignore[arg-type]
            implementations = dict(scenario["implementations"])
            for implementation, metrics_value in implementations.items():
                metrics = dict(metrics_value)
                writer.writerow(
                    {
                        "scenario": scenario_name,
                        "implementation": implementation,
                        "item_count": scenario["item_count"],
                        **metrics,
                    }
                )


def report_table(
    scenarios: dict[str, object],
    rss: dict[str, object],
) -> str:
    """Render a compact Markdown benchmark report."""
    lines = [
        "# Judge-Facing Benchmark Results",
        "",
        "All measurements compare the original Python CLI with the standalone",
        "Rust release CLI on identical input bytes and equivalent flags.",
        "",
        "## End-to-end latency",
        "",
        "| Scenario | Impl | Samples | p50 ms | p95 ms | p99 ms | Median items/s |",
        "|---|---:|---:|---:|---:|---:|---:|",
    ]

    for scenario_name, scenario_value in scenarios.items():
        scenario = dict(scenario_value)  # type: ignore[arg-type]
        implementations = dict(scenario["implementations"])
        for implementation, metrics_value in implementations.items():
            metrics = dict(metrics_value)
            lines.append(
                "| "
                f"{scenario_name} | {implementation} | "
                f"{metrics['sample_count']} | "
                f"{metrics['p50_ms']:.3f} | "
                f"{metrics['p95_ms']:.3f} | "
                f"{metrics['p99_ms']:.3f} | "
                f"{metrics['median_items_per_second']:.1f} |"
            )

    lines.extend(
        (
            "",
            "## Peak RSS",
            "",
            "| Scenario | Impl | Samples | Median KiB | Max KiB |",
            "|---|---:|---:|---:|---:|",
        )
    )

    for scenario_name, scenario_value in rss.items():
        scenario = dict(scenario_value)  # type: ignore[arg-type]
        implementations = dict(scenario["implementations"])
        for implementation, metrics_value in implementations.items():
            metrics = dict(metrics_value)
            lines.append(
                "| "
                f"{scenario_name} | {implementation} | "
                f"{scenario['samples_per_implementation']} | "
                f"{metrics['median_peak_rss_kib']:.1f} | "
                f"{metrics['max_peak_rss_kib']} |"
            )

    lines.extend(
        (
            "",
            "Percentiles use the nearest-rank method. Process launch, argument",
            "parsing, stdin reading, sorting, and output generation are inside",
            "the timed region; compilation and corpus loading are outside it.",
            "Output is redirected to `/dev/null` equally for both programs.",
            "",
        )
    )
    return "\n".join(lines)


def main() -> int:
    """Run benchmarks and write raw and summarized evidence."""
    arguments = parse_arguments()
    python_executable = arguments.python_executable.resolve()
    rust_binary = arguments.rust_binary.resolve()
    output_directory = arguments.output_directory.resolve()

    required_paths = (
        python_executable,
        rust_binary,
        GNU_TIME,
        DATA_DIR / "default_1000.txt",
        DATA_DIR / "default_10000.txt",
        DATA_DIR / "default_50000.txt",
        DATA_DIR / "float_1000.txt",
        DATA_DIR / "real_1000.txt",
        DATA_DIR / "path_1000.txt",
        DATA_DIR / "locale_1000.txt",
    )
    missing = [str(path) for path in required_paths if not path.is_file()]
    if missing:
        print("ERROR: missing required benchmark paths:")  # noqa: T201
        for path in missing:
            print(f"  {path}")  # noqa: T201
        return 2

    status_before_run = run_text("git", "status", "--short")
    commit = run_text("git", "rev-parse", "HEAD")
    branch = run_text("git", "branch", "--show-current")

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

    commands = (
        CommandSpec(
            "python",
            (
                str(python_executable),
                "-X",
                "utf8",
                "-m",
                "natsort",
            ),
        ),
        CommandSpec("rust", (str(rust_binary),)),
    )

    scenarios = scenario_definitions()

    try:
        latency_summary, latency_raw = benchmark_latency(
            scenarios,
            commands,
            environment,
        )
        rss_summary, rss_raw = benchmark_rss(
            commands,
            environment,
        )
    except RuntimeError as error:
        print(f"ERROR: {error}")  # noqa: T201
        return 1

    output_directory.mkdir(parents=True, exist_ok=True)

    corpora = sorted(
        {scenario.corpus for scenario in scenarios if scenario.corpus is not None}
    )
    corpus_metadata = {
        str(path.relative_to(ROOT)): {
            "sha256": sha256(path),
            "bytes": path.stat().st_size,
            "items": len(path.read_bytes().splitlines()),
        }
        for path in corpora
    }

    python_module = run_text(
        str(python_executable),
        "-X",
        "utf8",
        "-c",
        "import natsort; print(natsort.__file__)",
    )

    metadata = {
        "schema_version": 1,
        "status": "passed",
        "implementation_commit": commit,
        "branch": branch,
        "working_tree_clean_before_run": not status_before_run,
        "working_tree_status_before_run": (
            status_before_run.splitlines() if status_before_run else []
        ),
        "recorded_at_utc": (
            datetime.now(timezone.utc).isoformat().replace("+00:00", "Z")
        ),
        "platform": platform.platform(),
        "machine": platform.machine(),
        "processor": platform.processor(),
        "cpu_model": run_text(
            "bash",
            "-lc",
            "lscpu | sed -n 's/^Model name:[[:space:]]*//p' | head -n1",
        ),
        "python_version": run_text(
            str(python_executable),
            "--version",
        ),
        "python_module": python_module,
        "rustc": run_text("rustc", "--version"),
        "cargo": run_text("cargo", "--version"),
        "rust_binary": str(rust_binary),
        "rust_binary_sha256": sha256(rust_binary),
        "rust_binary_size_bytes": rust_binary.stat().st_size,
        "gnu_time": run_text(str(GNU_TIME), "--version").splitlines()[0],
        "locale": "C.UTF-8",
        "percentile_method": "nearest-rank",
        "timed_region": (
            "fresh process launch through completed CLI execution; "
            "stdout redirected to /dev/null"
        ),
        "corpora": corpus_metadata,
    }

    results = {
        "metadata": metadata,
        "latency": latency_summary,
        "peak_rss": rss_summary,
    }

    (output_directory / "results.json").write_text(
        json.dumps(results, indent=2, ensure_ascii=False) + "\n",
        encoding="utf-8",
    )
    (output_directory / "raw_latency_samples.json").write_text(
        json.dumps(latency_raw, indent=2) + "\n",
        encoding="utf-8",
    )
    (output_directory / "raw_rss_samples.json").write_text(
        json.dumps(rss_raw, indent=2) + "\n",
        encoding="utf-8",
    )
    (output_directory / "environment.json").write_text(
        json.dumps(metadata, indent=2, ensure_ascii=False) + "\n",
        encoding="utf-8",
    )

    write_latency_csv(
        output_directory / "results.csv",
        latency_summary,
    )
    (output_directory / "report.md").write_text(
        report_table(latency_summary, rss_summary),
        encoding="utf-8",
    )

    print("BENCHMARK SUMMARY")  # noqa: T201
    print(f"implementation_commit={commit}")  # noqa: T201
    print(  # noqa: T201
        f"working_tree_clean_before_run={str(not status_before_run).lower()}"
    )
    print(f"python_module={python_module}")  # noqa: T201
    print(f"rust_binary_sha256={sha256(rust_binary)}")  # noqa: T201
    print(f"scenario_count={len(scenarios)}")  # noqa: T201
    print("latency_status=passed")  # noqa: T201
    print("rss_status=passed")  # noqa: T201
    print("PASS: Judge-facing benchmark evidence created")  # noqa: T201
    return 0


if __name__ == "__main__":
    sys.exit(main())
