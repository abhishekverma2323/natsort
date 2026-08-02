"""Thin compatibility surface backed by the Rust test adapter."""

from __future__ import annotations

import enum
import os
import struct
import subprocess
from collections.abc import Callable, Iterable
from pathlib import Path, PurePath
from typing import Any, TypeVar


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

_T = TypeVar("_T")


def _adapter_binary() -> Path:
    configured = os.environ.get("NATSORT_RUST_ADAPTER_BIN")

    if configured:
        return Path(configured)

    return _DEFAULT_BINARY


def _run_adapter(
    arguments: list[str],
    *,
    input_data: bytes | None = None,
) -> bytes:
    binary = _adapter_binary()

    if not binary.exists():
        raise ImportError(
            "Rust original-suite adapter binary was not found: "
            f"{binary}"
        )

    result = subprocess.run(
        [str(binary), *arguments],
        input=input_data,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        check=False,
    )

    if result.returncode != 0:
        stderr = result.stderr.decode("utf-8", errors="replace")

        raise RuntimeError(
            "Rust original-suite adapter failed:\n"
            f"{stderr}"
        )

    return result.stdout


def _load_flags_from_rust() -> dict[str, int]:
    output = _run_adapter(["flags"])
    text = output.decode("utf-8")
    flags: dict[str, int] = {}

    for line in text.splitlines():
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


def _encode_strings(items: list[str]) -> bytes:
    output = bytearray(struct.pack("<Q", len(items)))

    for item in items:
        encoded = item.encode("utf-8")
        output.extend(struct.pack("<Q", len(encoded)))
        output.extend(encoded)

    return bytes(output)


def _read_u64(data: bytes, offset: int) -> tuple[int, int]:
    end = offset + 8

    if end > len(data):
        raise RuntimeError(
            "Rust adapter response ended unexpectedly"
        )

    return struct.unpack("<Q", data[offset:end])[0], end


def _decode_strings(data: bytes) -> list[str]:
    count, offset = _read_u64(data, 0)
    items: list[str] = []

    for _ in range(count):
        length, offset = _read_u64(data, offset)
        end = offset + length

        if end > len(data):
            raise RuntimeError(
                "Rust adapter string response ended unexpectedly"
            )

        items.append(data[offset:end].decode("utf-8"))
        offset = end

    if offset != len(data):
        raise RuntimeError(
            "Rust adapter response contained trailing bytes"
        )

    return items



_TAG_TEXT = 0
_TAG_BYTES = 1
_TAG_INTEGER = 2
_TAG_FLOAT = 3
_TAG_NONE = 4
_TAG_SEQUENCE = 5


def _encode_sized_value(tag: int, payload: bytes) -> bytes:
    return (
        bytes([tag])
        + struct.pack("<Q", len(payload))
        + payload
    )


def _encode_value(value: Any) -> bytes:
    if value is None:
        return bytes([_TAG_NONE])

    if isinstance(value, os.PathLike):
        value = os.fspath(value)

    if isinstance(value, str):
        return _encode_sized_value(
            _TAG_TEXT,
            value.encode("utf-8"),
        )

    if isinstance(value, (bytes, bytearray, memoryview)):
        return _encode_sized_value(
            _TAG_BYTES,
            bytes(value),
        )

    if isinstance(value, bool):
        value = int(value)

    if isinstance(value, int):
        return _encode_sized_value(
            _TAG_INTEGER,
            str(value).encode("ascii"),
        )

    if isinstance(value, float):
        return bytes([_TAG_FLOAT]) + struct.pack("<d", value)

    if isinstance(value, (list, tuple)):
        output = bytearray([_TAG_SEQUENCE])
        output.extend(struct.pack("<Q", len(value)))

        for item in value:
            output.extend(_encode_value(item))

        return bytes(output)

    raise TypeError(
        "Rust original-suite adapter does not support value type "
        f"{type(value).__name__!r}"
    )


def _encode_values(values: list[Any]) -> bytes:
    output = bytearray(struct.pack("<Q", len(values)))

    for value in values:
        output.extend(_encode_value(value))

    return bytes(output)


def _decode_indexes(data: bytes) -> list[int]:
    count, offset = _read_u64(data, 0)
    indexes: list[int] = []

    for _ in range(count):
        index, offset = _read_u64(data, offset)
        indexes.append(index)

    if offset != len(data):
        raise RuntimeError(
            "Rust adapter index response contained trailing bytes"
        )

    return indexes


def _contains_top_level_bytes(values: list[Any]) -> bool:
    return any(
        isinstance(value, (bytes, bytearray, memoryview))
        for value in values
    )


def _contains_top_level_text(values: list[Any]) -> bool:
    return any(
        isinstance(value, (str, PurePath))
        for value in values
    )


ns = enum.IntEnum(
    "ns",
    _load_flags_from_rust(),
    module=__name__,
)

NSType = ns | int


def as_utf8(value: Any) -> Any:
    """Decode UTF-8 bytes while preserving non-byte values."""
    if isinstance(value, bytes):
        return value.decode("utf-8")

    return value


def natsorted(
    seq: Iterable[_T],
    key: Callable[[_T], Any] | None = None,
    reverse: bool = False,
    alg: int = ns.DEFAULT,
) -> list[_T]:
    """Route original natsort calls to the Rust implementation."""
    items = list(seq)

    if key is None:
        values: list[Any] = items

        if (
            _contains_top_level_bytes(values)
            and _contains_top_level_text(values)
        ):
            raise TypeError(
                "bytes and text cannot be naturally sorted together "
                "without a decoder"
            )
    else:
        values = [key(item) for item in items]

    response = _run_adapter(
        [
            "sort-values",
            str(int(alg)),
            "1" if reverse else "0",
        ],
        input_data=_encode_values(values),
    )

    indexes = _decode_indexes(response)

    for index in indexes:
        if index >= len(items):
            raise RuntimeError(
                "Rust adapter returned an out-of-range index: "
                f"{index}"
            )

    return [items[index] for index in indexes]


globals().update(ns.__members__)

__rust_adapter__ = True

__all__ = [
    "NSType",
    "as_utf8",
    "natsorted",
    "ns",
    *ns.__members__,
]
