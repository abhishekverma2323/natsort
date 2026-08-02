from __future__ import annotations

import argparse
import importlib
import os
import subprocess
import sys
from pathlib import Path


SCRIPT = Path(__file__).resolve()
ROOT = SCRIPT.parents[1]
SHIM_DIR = ROOT / "parity" / "original_suite_adapter"
RUST_MANIFEST = ROOT / "rust-port" / "Cargo.toml"
RUST_BINARY = (
    ROOT
    / "rust-port"
    / "target"
    / "debug"
    / (
        "original-suite-adapter.exe"
        if os.name == "nt"
        else "original-suite-adapter"
    )
)


def build_adapter() -> None:
    subprocess.run(
        [
            "cargo",
            "build",
            "--quiet",
            "--manifest-path",
            str(RUST_MANIFEST),
            "--bin",
            "original-suite-adapter",
        ],
        cwd=ROOT,
        check=True,
    )


def load_rust_backed_package() -> object:
    os.environ["NATSORT_RUST_ADAPTER_BIN"] = str(RUST_BINARY)

    sys.path.insert(0, str(SHIM_DIR))

    for name in list(sys.modules):
        if name == "natsort" or name.startswith("natsort."):
            del sys.modules[name]

    package = importlib.import_module("natsort")

    if not getattr(package, "__rust_adapter__", False):
        raise RuntimeError(
            "The original Python package was imported instead of "
            "the Rust-backed compatibility shim."
        )

    return package


def main() -> int:
    parser = argparse.ArgumentParser(
        description=(
            "Run unchanged original pytest cases against the Rust adapter."
        )
    )
    parser.add_argument(
        "pytest_arguments",
        nargs=argparse.REMAINDER,
    )
    arguments = parser.parse_args()

    build_adapter()
    package = load_rust_backed_package()

    print(f"Rust-backed package: {package.__file__}")
    print(f"Rust adapter binary: {RUST_BINARY}")

    import pytest

    pytest_arguments = arguments.pytest_arguments

    if pytest_arguments[:1] == ["--"]:
        pytest_arguments = pytest_arguments[1:]

    if not pytest_arguments:
        pytest_arguments = [
            "-q",
            str(ROOT / "tests" / "test_ns_enum.py"),
        ]

    return int(pytest.main(pytest_arguments))


if __name__ == "__main__":
    raise SystemExit(main())
