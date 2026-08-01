from __future__ import annotations

import argparse
import gc
import json
import statistics
import sys
import time
from pathlib import Path

import natsort
from natsort import natsorted, ns


ALGORITHMS = {
    "default": ns.DEFAULT,
    "float": ns.FLOAT,
    "real": ns.REAL,
    "path": ns.PATH,
    "locale": ns.LOCALE,
}


def load_entries(path: Path) -> list[str]:
    return path.read_text(encoding="utf-8").splitlines()


def benchmark(
    entries: list[str],
    *,
    mode: str,
    warmups: int,
    runs: int,
) -> tuple[list[float], int]:
    algorithm = ALGORITHMS[mode]
    checksum = 0

    for _ in range(warmups):
        result = natsorted(entries, alg=algorithm)
        checksum ^= len(result)

    timings_ms: list[float] = []
    gc_was_enabled = gc.isenabled()
    gc.disable()

    try:
        for _ in range(runs):
            started = time.perf_counter_ns()
            result = natsorted(entries, alg=algorithm)
            elapsed_ns = time.perf_counter_ns() - started

            timings_ms.append(elapsed_ns / 1_000_000.0)

            if result:
                checksum ^= len(result[0])
                checksum ^= len(result[-1]) << 1
            checksum ^= len(result)
    finally:
        if gc_was_enabled:
            gc.enable()

    return timings_ms, checksum


def main() -> None:
    parser = argparse.ArgumentParser(description="Benchmark Python natsort in-process.")
    parser.add_argument("--dataset", type=Path, required=True)
    parser.add_argument("--mode", choices=tuple(ALGORITHMS), required=True)
    parser.add_argument("--warmups", type=int, default=2)
    parser.add_argument("--runs", type=int, default=7)
    args = parser.parse_args()

    if args.warmups < 0:
        parser.error("--warmups cannot be negative")
    if args.runs <= 0:
        parser.error("--runs must be greater than zero")

    entries = load_entries(args.dataset)
    timings_ms, checksum = benchmark(
        entries,
        mode=args.mode,
        warmups=args.warmups,
        runs=args.runs,
    )

    payload = {
        "engine": "python",
        "python_version": sys.version.split()[0],
        "natsort_version": getattr(natsort, "__version__", "unknown"),
        "mode": args.mode,
        "dataset": str(args.dataset),
        "entries": len(entries),
        "warmups": args.warmups,
        "runs": args.runs,
        "times_ms": timings_ms,
        "median_ms": statistics.median(timings_ms),
        "min_ms": min(timings_ms),
        "max_ms": max(timings_ms),
        "checksum": checksum,
    }

    print(json.dumps(payload, ensure_ascii=True, separators=(",", ":")))


if __name__ == "__main__":
    main()