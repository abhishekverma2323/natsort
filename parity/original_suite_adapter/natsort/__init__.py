"""Thin compatibility surface backed by the Rust test adapter."""

from __future__ import annotations

import enum
import locale as _locale
import os
import platform
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



def _current_locale_identifier() -> str:
    """Return the locale selected by the unchanged Python test fixture."""
    current = _locale.setlocale(_locale.LC_ALL)

    if current in {"C", "POSIX"}:
        return current

    language, _encoding = _locale.getlocale()

    return language or current or "en_US"


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


def decoder(encoding: str) -> Callable[[Any], Any]:
    """Return a bytes decoder while preserving non-byte identity."""

    def decode(value: Any) -> Any:
        if isinstance(value, bytes):
            return value.decode(encoding)

        return value

    return decode


def as_ascii(value: Any) -> Any:
    """Decode ASCII bytes while preserving non-byte values."""
    return decoder("ascii")(value)


def as_utf8(value: Any) -> Any:
    """Decode UTF-8 bytes while preserving non-byte values."""
    return decoder("utf-8")(value)


def _prepare_sort_values(
    items: list[Any],
    key: Callable[[Any], Any] | None,
) -> list[Any]:
    if key is None:
        values = items

        if (
            _contains_top_level_bytes(values)
            and _contains_top_level_text(values)
        ):
            raise TypeError(
                "bytes and text cannot be naturally sorted together "
                "without a decoder"
            )

        return values

    return [key(item) for item in items]



def _effective_algorithm_bits(alg: int) -> int:
    """Apply Python natsort flag interaction rules."""
    bits = int(alg)

    # CAPITALFIRST and UNGROUPLETTERS share bit 512, but this option
    # only affects sorting when LOCALEALPHA is active.
    if not bits & int(ns.LOCALEALPHA):
        bits &= ~int(ns.UNGROUPLETTERS)

    return bits


def _natural_sort_indexes(
    items: list[Any],
    key: Callable[[Any], Any] | None,
    reverse: bool,
    alg: int,
) -> list[int]:
    values = _prepare_sort_values(items, key)

    response = _run_adapter(
        [
            "sort-values",
            str(_effective_algorithm_bits(alg)),
            "1" if reverse else "0",
            _current_locale_identifier(),
        ],
        input_data=_encode_values(values),
    )

    indexes = _decode_indexes(response)

    if any(index >= len(items) for index in indexes):
        raise RuntimeError(
            "Rust adapter returned an out-of-range natural-sort index"
        )

    return indexes


def natsorted(
    seq: Iterable[_T],
    key: Callable[[_T], Any] | None = None,
    reverse: bool = False,
    alg: int = ns.DEFAULT,
) -> list[_T]:
    """Sort through the Rust NaturalValue implementation."""
    items = list(seq)
    indexes = _natural_sort_indexes(items, key, reverse, alg)

    return [items[index] for index in indexes]


def realsorted(
    seq: Iterable[_T],
    key: Callable[[_T], Any] | None = None,
    reverse: bool = False,
    alg: int = ns.DEFAULT,
) -> list[_T]:
    return natsorted(
        seq,
        key=key,
        reverse=reverse,
        alg=int(alg) | int(ns.REAL),
    )


def humansorted(
    seq: Iterable[_T],
    key: Callable[[_T], Any] | None = None,
    reverse: bool = False,
    alg: int = ns.DEFAULT,
) -> list[_T]:
    return natsorted(
        seq,
        key=key,
        reverse=reverse,
        alg=int(alg) | int(ns.LOCALE),
    )


def index_natsorted(
    seq: Iterable[_T],
    key: Callable[[_T], Any] | None = None,
    reverse: bool = False,
    alg: int = ns.DEFAULT,
) -> list[int]:
    items = list(seq)

    return _natural_sort_indexes(items, key, reverse, alg)


def index_realsorted(
    seq: Iterable[_T],
    key: Callable[[_T], Any] | None = None,
    reverse: bool = False,
    alg: int = ns.DEFAULT,
) -> list[int]:
    return index_natsorted(
        seq,
        key=key,
        reverse=reverse,
        alg=int(alg) | int(ns.REAL),
    )


def index_humansorted(
    seq: Iterable[_T],
    key: Callable[[_T], Any] | None = None,
    reverse: bool = False,
    alg: int = ns.DEFAULT,
) -> list[int]:
    return index_natsorted(
        seq,
        key=key,
        reverse=reverse,
        alg=int(alg) | int(ns.LOCALE),
    )


def order_by_index(
    seq: Any,
    index: Iterable[int],
    iter: bool = False,
) -> Any:
    ordered = (seq[position] for position in index)

    return ordered if iter else list(ordered)


def os_sorted(
    seq: Iterable[_T],
    key: Callable[[_T], Any] | None = None,
    reverse: bool = False,
    presort: bool = False,
) -> list[_T]:
    """Route OS sorting through the matching Rust implementation."""
    items = list(seq)

    # Original natsort falls back to natural locale/path sorting on
    # Unix when PyICU is unavailable. Sorting still executes in Rust.
    if platform.system() != "Windows":
        try:
            __import__("icu")
        except ImportError:
            alg = (
                int(ns.LOCALE)
                | int(ns.PATH)
                | int(ns.IGNORECASE)
            )

            if presort:
                alg |= int(ns.PRESORT)

            return natsorted(
                items,
                key=key,
                reverse=reverse,
                alg=alg,
            )

    values = (
        items
        if key is None
        else [key(item) for item in items]
    )

    response = _run_adapter(
        [
            "os-sort-values",
            "1" if reverse else "0",
            "1" if presort else "0",
            "windows"
            if platform.system() == "Windows"
            else "unix",
            _current_locale_identifier(),
        ],
        input_data=_encode_values(values),
    )

    indexes = _decode_indexes(response)

    if any(index >= len(items) for index in indexes):
        raise RuntimeError(
            "Rust adapter returned an out-of-range OS-sort index"
        )

    return [items[index] for index in indexes]


globals().update(ns.__members__)

__rust_adapter__ = True

__version__ = "0.1.0"

__all__ = [
    "NSType",
    "as_ascii",
    "as_utf8",
    "decoder",
    "humansorted",
    "index_humansorted",
    "index_natsorted",
    "index_realsorted",
    "natsorted",
    "ns",
    "order_by_index",
    "os_sorted",
    "realsorted",
    *ns.__members__,
]

def numeric_regex_chooser(alg: int) -> str:
    """Return the numeric regex selected by the Rust implementation."""
    return _run_adapter(["numeric-regex", str(int(alg))]).decode("utf-8")


from . import utils as utils


def natsort_keygen(
    key: Any = None,
    alg: int = ns.DEFAULT,
) -> Any:
    if not isinstance(alg, int):
        raise ValueError(
            "natsort_keygen: 'alg' argument must be from the enum "
            f"'ns', got {alg!s}"
        )

    from .compat import locale as locale_compat
    from .ns_enum import NS_DUMB

    bits = int(alg)

    if bits & int(ns.LOCALEALPHA) and locale_compat.dumb_sort():
        bits |= NS_DUMB

    if bits & int(ns.NUMAFTER):
        sep = (
            locale_compat.null_string_locale_max
            if bits & int(ns.LOCALEALPHA)
            else locale_compat.null_string_max
        )
        pre_sep = locale_compat.null_string_max
    else:
        sep = (
            locale_compat.null_string_locale
            if bits & int(ns.LOCALEALPHA)
            else locale_compat.null_string
        )
        pre_sep = locale_compat.null_string

    regex = utils.regex_chooser(bits)
    input_transform = utils.input_string_transform_factory(bits)
    component_transform = utils.string_component_transform_factory(bits)
    final_transform = utils.final_data_transform_factory(
        bits,
        sep,
        pre_sep,
    )

    string_func = utils.parse_string_factory(
        bits,
        sep,
        regex.split,
        input_transform,
        component_transform,
        final_transform,
    )

    if bits & int(ns.PATH):
        string_func = utils.parse_path_factory(string_func)

    bytes_func = utils.parse_bytes_factory(bits)
    num_func = utils.parse_number_or_none_factory(
        bits,
        sep,
        pre_sep,
    )

    def generated(value: Any) -> tuple[Any, ...]:
        return utils.natsort_key(
            value,
            key,
            string_func,
            bytes_func,
            num_func,
        )

    return generated


_default_natsort_key = natsort_keygen()


def natsort_key(value: Any) -> tuple[Any, ...]:
    return _default_natsort_key(value)

__all__.extend(
    [
        "natsort_key",
        "natsort_keygen",
        "numeric_regex_chooser",
        "utils",
    ]
)
