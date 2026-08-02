"""Unicode collections supplied by Rust and adapted to the host Python UCD."""

from __future__ import annotations

import struct
import unicodedata

from . import _read_u64, _run_adapter


def _read_codepoints(
    data: bytes,
    offset: int,
) -> tuple[tuple[int, ...], int]:
    count, offset = _read_u64(data, offset)
    values: list[int] = []

    for _ in range(count):
        end = offset + 4

        if end > len(data):
            raise RuntimeError(
                "Rust Unicode-table response ended unexpectedly"
            )

        values.append(
            struct.unpack("<I", data[offset:end])[0]
        )
        offset = end

    return tuple(values), offset


_data = _run_adapter(["unicode-tables"])

# Rust remains the authoritative source of the generated codepoint inventory.
numeric_hex, _offset = _read_codepoints(_data, 0)
_rust_digit_hex, _offset = _read_codepoints(_data, _offset)
_rust_decimal_hex, _offset = _read_codepoints(_data, _offset)

if _offset != len(_data):
    raise RuntimeError(
        "Rust Unicode-table response contained trailing bytes"
    )


# The adapter may be tested with a Python interpreter whose Unicode database
# is older than the Rust Unicode tables. Filter only the Python-visible
# representation through the host interpreter's unicodedata database.
#
# This mirrors upstream natsort's compatibility boundary. It does not perform
# natural sorting, numeric parsing, or tokenization in Python.
numeric_chars = [
    chr(codepoint)
    for codepoint in numeric_hex
    if unicodedata.numeric(chr(codepoint), None) is not None
]

digit_chars = [
    character
    for character in numeric_chars
    if unicodedata.digit(character, None) is not None
]

decimal_chars = [
    character
    for character in numeric_chars
    if unicodedata.decimal(character, None) is not None
]

numeric = "".join(numeric_chars)
digits = "".join(digit_chars)
decimals = "".join(decimal_chars)

digits_no_decimals = "".join(
    character
    for character in digit_chars
    if character not in decimals
)

numeric_no_decimals = "".join(
    character
    for character in numeric_chars
    if character not in decimals
)

__all__ = [
    "decimal_chars",
    "decimals",
    "digit_chars",
    "digits",
    "digits_no_decimals",
    "numeric",
    "numeric_chars",
    "numeric_hex",
    "numeric_no_decimals",
]
