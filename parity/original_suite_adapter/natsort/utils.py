"""Thin Python representations over Rust compatibility decisions."""

from __future__ import annotations

import os
import re
import struct
from collections.abc import Callable, Iterable, Iterator
from pathlib import PurePath
from typing import Any, TypeAlias

from . import (
    _current_locale_identifier,
    _read_u64,
    _decode_strings,
    _encode_strings,
    _encode_values,
    _run_adapter,
    numeric_regex_chooser,
    ns,
)
from .unicode_numbers import digits_no_decimals, numeric_no_decimals

BytesTransform: TypeAlias = tuple[Any, ...]
BytesTransformer: TypeAlias = Callable[[bytes], BytesTransform]
FinalTransform: TypeAlias = tuple[Any, ...]
FinalTransformer: TypeAlias = Callable[[Iterable[Any], str], FinalTransform]
NumTransform: TypeAlias = tuple[Any, ...]
NumTransformer: TypeAlias = Callable[[Any], NumTransform]
StrParser: TypeAlias = Callable[[str], tuple[Any, ...]]
StrTransformer: TypeAlias = Callable[[Iterable[str]], Iterator[Any]]


class NumericalRegularExpressions:
    """Compiled Python regex objects built from Rust-selected patterns."""

    numeric = numeric_no_decimals
    digits = digits_no_decimals
    exp = r"(?:[eE][-+]?\d+)?"
    float_num = r"(?:\d+\.?\d*|\.\d+)"

    @staticmethod
    def _compile(alg: int) -> re.Pattern[str]:
        return re.compile(f"({numeric_regex_chooser(alg)})", flags=re.UNICODE)

    @classmethod
    def int_sign(cls) -> re.Pattern[str]:
        return cls._compile(int(ns.SIGNED))

    @classmethod
    def int_nosign(cls) -> re.Pattern[str]:
        return cls._compile(int(ns.INT))

    @classmethod
    def float_sign_exp(cls) -> re.Pattern[str]:
        return cls._compile(int(ns.FLOAT | ns.SIGNED))

    @classmethod
    def float_nosign_exp(cls) -> re.Pattern[str]:
        return cls._compile(int(ns.FLOAT))

    @classmethod
    def float_sign_noexp(cls) -> re.Pattern[str]:
        return cls._compile(int(ns.FLOAT | ns.SIGNED | ns.NOEXP))

    @classmethod
    def float_nosign_noexp(cls) -> re.Pattern[str]:
        return cls._compile(int(ns.FLOAT | ns.NOEXP))


def regex_chooser(alg: int) -> re.Pattern[str]:
    return NumericalRegularExpressions._compile(int(alg))


def chain_functions(
    functions: Iterable[Callable[[Any], Any]],
) -> Callable[[Any], Any]:
    functions = tuple(functions)

    def chained(value: Any) -> Any:
        for function in functions:
            value = function(value)
        return value

    return chained


def do_decoding(value: Any, encoding: str) -> Any:
    """Decode bytes through Rust while preserving non-byte identity."""
    if not isinstance(value, bytes):
        return value

    response = _run_adapter(
        ["decode-bytes", encoding],
        input_data=value,
    )
    decoded = _decode_strings(response)

    if len(decoded) != 1:
        raise RuntimeError("Rust decode-bytes returned an invalid response")

    return decoded[0]


def groupletters(value: str) -> str:
    response = _run_adapter(
        ["group-letters"],
        input_data=_encode_strings([value]),
    )
    transformed = _decode_strings(response)

    if len(transformed) != 1:
        raise RuntimeError("Rust group-letters returned an invalid response")

    return transformed[0]


def sep_inserter(
    iterator: Iterator[Any],
    sep: str | bytes,
) -> Iterator[Any]:
    """Yield original objects according to Rust's separator plan."""

    def generate() -> Iterator[Any]:
        items = list(iterator)
        tags = bytes(
            1 if type(item) in (int, float) else 0
            for item in items
        )
        plan = _run_adapter(["separator-plan"], input_data=tags)

        if len(plan) != len(items):
            raise RuntimeError("Rust separator-plan returned an invalid length")

        for insert, item in zip(plan, items):
            if insert:
                yield sep
            yield item

    return generate()


def path_splitter(
    value: str | PurePath,
    *,
    treat_base: bool = True,
) -> Iterator[str]:
    text = os.fspath(value)
    response = _run_adapter(
        [
            "path-components",
            "1" if os.name == "nt" else "0",
            "1" if treat_base else "0",
        ],
        input_data=_encode_strings([text]),
    )
    return iter(_decode_strings(response))


def input_string_transform_factory(
    alg: int,
) -> Callable[[str], str]:
    bits = int(alg)
    plan = _run_adapter(["input-transform-plan", str(bits)])

    if plan == b"\x01":
        return lambda value: value

    if plan != b"\x00":
        raise RuntimeError("Rust input-transform-plan returned invalid data")

    locale_identifier = _current_locale_identifier()

    def transform(value: str) -> str:
        response = _run_adapter(
            ["input-transform", str(bits), locale_identifier],
            input_data=_encode_strings([value]),
        )
        transformed = _decode_strings(response)

        if len(transformed) != 1:
            raise RuntimeError("Rust input-transform returned an invalid response")

        return transformed[0]

    return transform


def final_data_transform_factory(
    alg: int,
    sep: str | bytes,
    pre_sep: str,
) -> FinalTransformer:
    bits = int(alg)
    plan = _run_adapter(["final-transform-plan", str(bits)])

    if len(plan) != 1 or plan[0] not in (0, 1, 2):
        raise RuntimeError("Rust final-transform-plan returned invalid data")

    mode = plan[0]

    if mode == 0:
        return lambda split_value, _original: tuple(split_value)

    locale_identifier = _current_locale_identifier()

    def transform(
        split_value: Iterable[Any],
        original: str,
    ) -> FinalTransform:
        split_tuple = tuple(split_value)

        if not split_tuple:
            return (), ()

        if split_tuple[0] == sep:
            return (pre_sep,), split_tuple

        prefix = original[:1]

        if mode == 2:
            response = _run_adapter(
                ["input-transform", str(1 << 31), locale_identifier],
                input_data=_encode_strings([prefix]),
            )
            values = _decode_strings(response)

            if len(values) != 1:
                raise RuntimeError("Rust prefix swap returned invalid data")

            prefix = values[0]

        return (prefix,), split_tuple

    return transform


def parse_bytes_factory(alg: int) -> BytesTransformer:
    bits = int(alg)

    def transform(value: bytes) -> BytesTransform:
        response = _run_adapter(
            ["parse-bytes", str(bits)],
            input_data=value,
        )

        if not response or response[0] not in (0, 1):
            raise RuntimeError("Rust parse-bytes returned invalid data")

        transformed = response[1:]

        if response[0]:
            return ((transformed,),)

        return (transformed,)

    return transform


def parse_number_or_none_factory(
    alg: int,
    sep: str | bytes,
    pre_sep: str,
) -> NumTransformer:
    bits = int(alg)

    def transform(value: Any) -> NumTransform:
        response = _run_adapter(
            ["parse-number-plan", str(bits)],
            input_data=_encode_values([value]),
        )

        if len(response) != 4:
            raise RuntimeError("Rust parse-number-plan returned invalid data")

        wrapper, kind, positive, suffix = response

        if wrapper not in (0, 1, 2, 3) or kind not in (0, 1):
            raise RuntimeError("Rust parse-number-plan returned invalid markers")

        if kind == 0:
            core: tuple[Any, ...] = (sep, value)
        else:
            if suffix not in (1, 2, 3):
                raise RuntimeError("Rust parse-number-plan returned invalid suffix")

            infinity = float("inf") if positive else float("-inf")
            core = (sep, infinity, str(suffix))

        if wrapper == 0:
            return core
        if wrapper == 1:
            return (core,)
        if wrapper == 2:
            return ((pre_sep,), core)
        return (((pre_sep,), core),)

    return transform


def parse_path_factory(
    string_parser: StrParser,
) -> Callable[[str | PurePath], tuple[Any, ...]]:
    return lambda value: tuple(
        map(string_parser, path_splitter(value))
    )


def _normalize_string_for_parse(
    value: str,
    bits: int,
    *,
    compose: bool,
) -> str:
    response = _run_adapter(
        [
            "normalize-string",
            str(bits),
            "1" if compose else "0",
        ],
        input_data=_encode_strings([value]),
    )
    values = _decode_strings(response)

    if len(values) != 1:
        raise RuntimeError("Rust normalize-string returned invalid data")

    return values[0]


def _decode_component_response(
    data: bytes,
) -> tuple[bool, list[str | int | float]]:
    if not data:
        raise RuntimeError("Rust component response was empty")

    use_locale = bool(data[0])
    count, offset = _read_u64(data, 1)
    output: list[str | int | float] = []

    for _ in range(count):
        if offset >= len(data):
            raise RuntimeError("Rust component response ended unexpectedly")

        tag = data[offset]
        offset += 1

        if tag in (0, 1):
            length, offset = _read_u64(data, offset)
            end = offset + length

            if end > len(data):
                raise RuntimeError("Rust component payload was truncated")

            payload = data[offset:end]
            offset = end

            if tag == 0:
                output.append(payload.decode("utf-8"))
            else:
                output.append(int(payload.decode("ascii")))
        elif tag == 2:
            end = offset + 8

            if end > len(data):
                raise RuntimeError("Rust float component was truncated")

            output.append(struct.unpack("<d", data[offset:end])[0])
            offset = end
        else:
            raise RuntimeError(f"Unknown Rust component tag: {tag}")

    if offset != len(data):
        raise RuntimeError("Rust component response had trailing bytes")

    return use_locale, output


def string_component_transform_factory(
    alg: int,
) -> StrTransformer:
    bits = int(alg)

    def transform(values: Iterable[str]) -> Iterator[Any]:
        items = list(values)
        response = _run_adapter(
            ["transform-components", str(bits)],
            input_data=_encode_strings(items),
        )
        use_locale, converted = _decode_component_response(response)

        if use_locale:
            from .compat.locale import get_strxfrm

            strxfrm = get_strxfrm()
            converted = [
                strxfrm(value) if isinstance(value, str) else value
                for value in converted
            ]

        return iter(converted)

    return transform


def parse_string_factory(
    alg: int,
    sep: str | bytes,
    splitter: Callable[[str], Iterable[str]],
    input_transform: Callable[[str], str],
    component_transform: StrTransformer,
    final_transform: FinalTransformer,
) -> StrParser:
    bits = int(alg)
    plan = _run_adapter(["parse-string-plan", str(bits)])

    if len(plan) != 2 or any(value not in (0, 1) for value in plan):
        raise RuntimeError("Rust parse-string-plan returned invalid data")

    original_after_transform = bool(plan[0])
    compose_for_locale = bool(plan[1])

    def parse(value: str) -> tuple[Any, ...]:
        if isinstance(value, PurePath):
            value = str(value)

        if not isinstance(value, str):
            raise TypeError("parse_string_factory requires string input")

        normalized = _normalize_string_for_parse(
            value,
            bits,
            compose=False,
        )
        transformed = input_transform(normalized)
        original = transformed if original_after_transform else normalized

        prepared = (
            _normalize_string_for_parse(
                transformed,
                bits,
                compose=True,
            )
            if compose_for_locale
            else transformed
        )

        split_values = (part for part in splitter(prepared) if part)
        components = component_transform(split_values)
        separated = sep_inserter(iter(components), sep)
        return final_transform(separated, original)

    return parse


def _classify_value(value: Any) -> int:
    response = _run_adapter(
        ["classify-value"],
        input_data=_encode_values([value]),
    )

    if len(response) != 1 or response[0] not in (0, 1, 2, 3):
        raise RuntimeError("Rust classify-value returned invalid data")

    return response[0]


def natsort_key(
    val: Any,
    key: Callable[[Any], Any] | None,
    string_func: Callable[[Any], tuple[Any, ...]],
    bytes_func: BytesTransformer,
    num_func: NumTransformer,
) -> tuple[Any, ...]:
    if key is not None:
        val = key(val)

    kind = _classify_value(val)

    if kind == 0:
        return string_func(val)

    if kind == 1:
        return bytes_func(val)

    if kind == 2:
        return tuple(
            natsort_key(
                item,
                None,
                string_func,
                bytes_func,
                num_func,
            )
            for item in val
        )

    return num_func(val)

