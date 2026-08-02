"""Thin Python callback bridge for Rust-backed numeric conversion."""

from __future__ import annotations

import struct
from collections.abc import Callable
from typing import TypeVar

from .. import _encode_strings, _run_adapter

_T = TypeVar("_T")


def _nan_bits(value: float) -> int:
    return struct.unpack("<Q", struct.pack("<d", value))[0]


def fast_float(
    x: str,
    key: Callable[[str], _T] = lambda value: value,
    nan: float = float("inf"),
) -> float | _T:
    """Return Rust's float conversion, or apply ``key`` on failure."""
    response = _run_adapter(
        ["fast-float", str(_nan_bits(nan))],
        input_data=_encode_strings([x]),
    )

    if response == b"\x00":
        return key(x)

    if len(response) == 9 and response[0] == 1:
        return struct.unpack("<d", response[1:])[0]

    raise RuntimeError("Rust adapter returned an invalid fast-float response")


def fast_int(
    x: str,
    key: Callable[[str], _T] = lambda value: value,
) -> int | _T:
    """Return Rust's integer conversion, or apply ``key`` on failure."""
    response = _run_adapter(
        ["fast-int"],
        input_data=_encode_strings([x]),
    )

    if response == b"\x00":
        return key(x)

    if len(response) >= 9 and response[0] == 2:
        length = struct.unpack("<Q", response[1:9])[0]
        payload = response[9:]

        if len(payload) != length:
            raise RuntimeError("Rust adapter returned a truncated fast-int response")

        return int(payload.decode("ascii"))

    raise RuntimeError("Rust adapter returned an invalid fast-int response")
