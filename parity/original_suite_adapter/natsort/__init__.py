"""Thin compatibility surface backed by the Rust test adapter."""

from __future__ import annotations

import enum
import os
import subprocess
from pathlib import Path


_ROOT = Path(__file__).resolve().parents[3]
_DEFAULT_BINARY = (
    _ROOT
    / "rust-port"
    / "target"
    / "debug"
    / (
        "original-suite-adapter.exe"
        if os.name == "nt"
        else "original-suite-adapter"
    )
)


def _adapter_binary() -> Path:
    configured = os.environ.get("NATSORT_RUST_ADAPTER_BIN")

    if configured:
        return Path(configured)

    return _DEFAULT_BINARY


def _load_flags_from_rust() -> dict[str, int]:
    binary = _adapter_binary()

    if not binary.exists():
        raise ImportError(
            "Rust original-suite adapter binary was not found: "
            f"{binary}"
        )

    result = subprocess.run(
        [str(binary), "flags"],
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        text=True,
        encoding="utf-8",
        check=False,
    )

    if result.returncode != 0:
        raise ImportError(
            "Rust original-suite adapter failed while loading flags:\n"
            f"{result.stderr}"
        )

    flags: dict[str, int] = {}

    for line in result.stdout.splitlines():
        if not line.strip():
            continue

        try:
            name, raw_value = line.split("\t", 1)
            flags[name] = int(raw_value)
        except ValueError as error:
            raise ImportError(
                f"Invalid Rust flag response: {line!r}"
            ) from error

    if len(flags) != 39:
        raise ImportError(
            "Rust adapter returned an unexpected flag count: "
            f"{len(flags)}"
        )

    return flags


ns = enum.IntEnum(
    "ns",
    _load_flags_from_rust(),
    module=__name__,
)

NSType = ns | int

globals().update(ns.__members__)

__rust_adapter__ = True

__all__ = [
    "NSType",
    "ns",
    *ns.__members__,
]
