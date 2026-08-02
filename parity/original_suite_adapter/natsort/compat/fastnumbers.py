"""Iterator-shaped Python bridge over Rust-backed numeric conversion."""

from __future__ import annotations

from collections.abc import Callable, Iterable, Iterator
from typing import TypeVar

from .fake_fastnumbers import fast_float, fast_int

_T = TypeVar("_T")

__all__ = ["try_float", "try_int"]


def try_float(
    x: Iterable[str],
    *,
    map: bool,
    nan: float = float("inf"),
    on_fail: Callable[[str], _T] = lambda value: value,
) -> Iterator[float | _T]:
    assert map is True
    return (fast_float(value, nan=nan, key=on_fail) for value in x)


def try_int(
    x: Iterable[str],
    *,
    map: bool,
    on_fail: Callable[[str], _T] = lambda value: value,
) -> Iterator[int | _T]:
    assert map is True
    return (fast_int(value, key=on_fail) for value in x)
