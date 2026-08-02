"""Thin Python CLI compatibility surface backed by the Rust sorter."""

from __future__ import annotations

import argparse
import re
import sys
from functools import lru_cache
from typing import Callable, Iterable, Pattern

import natsort

Num = int | float
NumPair = tuple[Num, Num]
NumIter = Iterable[Num]
NumConverter = Callable[[str], Num]


class TypedArgs:
    """Small argparse-compatible namespace used by the unchanged tests."""

    paths: bool
    filter: list[NumPair] | None
    reverse_filter: list[NumPair] | None
    exclude: list[Num]
    reverse: bool
    number_type: str
    signed: bool
    exp: bool
    locale: bool
    zero_terminated: bool
    entries: list[str]

    def __init__(
        self,
        filter: list[NumPair] | None = None,
        reverse_filter: list[NumPair] | None = None,
        exclude: list[Num] | None = None,
        paths: bool = False,
        reverse: bool = False,
        zero_terminated: bool = False,
        entries: list[str] | None = None,
    ) -> None:
        self.filter = filter
        self.reverse_filter = reverse_filter
        self.exclude = [] if exclude is None else exclude
        self.paths = paths
        self.reverse = reverse
        self.number_type = "int"
        self.signed = False
        self.exp = True
        self.locale = False
        self.zero_terminated = zero_terminated
        self.entries = [] if entries is None else entries


@lru_cache(maxsize=4096)
def _rust_pair_order(left: Num, right: Num) -> tuple[int, int]:
    result = natsort.index_natsorted([left, right])
    if result not in ([0, 1], [1, 0]):
        raise RuntimeError(f"Unexpected Rust pair order: {result!r}")
    return result[0], result[1]


def _rust_less(left: Num, right: Num) -> bool:
    # Stable ordering distinguishes equality by checking both directions.
    return (
        _rust_pair_order(left, right) == (0, 1)
        and _rust_pair_order(right, left) == (1, 0)
    )


def _rust_less_equal(left: Num, right: Num) -> bool:
    return _rust_pair_order(left, right) == (0, 1)


def _rust_equal(left: Num, right: Num) -> bool:
    return (
        _rust_pair_order(left, right) == (0, 1)
        and _rust_pair_order(right, left) == (0, 1)
    )


def range_check(low: Num, high: Num) -> NumPair:
    """Validate a range through Rust without losing integer precision."""
    response = natsort._run_adapter(
        ["validate-range"],
        input_data=natsort._encode_values([low, high]),
    )

    if response == bytes([0]):
        raise ValueError("low >= high")

    if response != bytes([1]):
        raise RuntimeError(
            f"Rust validate-range returned invalid data: {response!r}"
        )

    # Preserve the exact original Python values. This matters for integers
    # larger than f64's exactly representable range.
    return low, high


def check_filters(
    filters: Iterable[NumPair] | None,
) -> list[NumPair] | None:
    """Validate all filter ranges."""
    if not filters:
        return None

    try:
        return [range_check(low, high) for low, high in filters]
    except ValueError as error:
        raise ValueError(f"Error in --filter: {error}") from None


def get_entries(args: TypedArgs) -> list[str]:
    """Read positional entries or preserve stdin entries verbatim."""
    if args.entries:
        return args.entries

    separator = "\0" if args.zero_terminated else "\n"
    return sys.stdin.read().rstrip(separator).split(separator)


def keep_entry_range(
    entry: str,
    lows: NumIter,
    highs: NumIter,
    converter: NumConverter,
    regex: Pattern[str],
) -> bool:
    """Apply Python callback extraction; range policy matches the Rust CLI."""
    # Regex matching and converter invocation are Python callback boundaries;
    # every numeric comparison is answered by the Rust-backed index sorter.
    numbers = [converter(value) for value in regex.findall(entry)]
    ranges = tuple(zip(lows, highs))
    return any(
        _rust_less_equal(low, number)
        and _rust_less_equal(number, high)
        for number in numbers
        for low, high in ranges
    )


def keep_entry_value(
    entry: str,
    values: NumIter,
    converter: NumConverter,
    regex: Pattern[str],
) -> bool:
    """Apply Python callback extraction; exclusion policy matches the Rust CLI."""
    # Callback extraction remains in Python; equality is decided by Rust.
    excluded = tuple(values)
    return all(
        not any(
            _rust_equal(converter(value), blocked)
            for blocked in excluded
        )
        for value in regex.findall(entry)
    )


def _algorithm(args: TypedArgs) -> int:
    number_type = args.number_type.lower()

    if number_type in {"float", "f"}:
        algorithm = int(natsort.ns.FLOAT)
    elif number_type in {"real", "r"}:
        algorithm = int(natsort.ns.REAL)
    else:
        algorithm = int(natsort.ns.INT)

    if args.signed:
        algorithm |= int(natsort.ns.SIGNED)
    if not args.exp:
        algorithm |= int(natsort.ns.NOEXP)
    if args.paths:
        algorithm |= int(natsort.ns.PATH)
    if args.locale:
        algorithm |= int(natsort.ns.LOCALE)

    return algorithm


def sort_and_print_entries(
    entries: list[str],
    args: TypedArgs,
) -> None:
    """Filter entries in the host shim and obtain final order from Rust."""
    algorithm = _algorithm(args)

    if args.filter is not None or args.reverse_filter is not None or args.exclude:
        regex = re.compile(
            f"({natsort.numeric_regex_chooser(algorithm)})",
            flags=re.UNICODE,
        )

        if args.filter is not None:
            lows = [pair[0] for pair in args.filter]
            highs = [pair[1] for pair in args.filter]
            entries = [
                entry
                for entry in entries
                if keep_entry_range(entry, lows, highs, float, regex)
            ]

        if args.reverse_filter is not None:
            lows = [pair[0] for pair in args.reverse_filter]
            highs = [pair[1] for pair in args.reverse_filter]
            entries = [
                entry
                for entry in entries
                if not keep_entry_range(entry, lows, highs, float, regex)
            ]

        if args.exclude:
            entries = [
                entry
                for entry in entries
                if keep_entry_value(entry, args.exclude, float, regex)
            ]

    # natsorted is the Rust subprocess-backed public API.
    for entry in natsort.natsorted(
        entries,
        reverse=args.reverse,
        alg=algorithm,
    ):
        print(entry)


def _parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(
        description="Perform a natural sort on entries given on the command-line."
    )
    parser.add_argument(
        "--version",
        action="version",
        version=f"%(prog)s {getattr(natsort, '__version__', 'unknown')}",
    )
    parser.add_argument("-p", "--paths", action="store_true", default=False)
    parser.add_argument(
        "-f",
        "--filter",
        nargs=2,
        type=float,
        metavar=("LOW", "HIGH"),
        action="append",
    )
    parser.add_argument(
        "-F",
        "--reverse-filter",
        nargs=2,
        type=float,
        metavar=("LOW", "HIGH"),
        action="append",
        dest="reverse_filter",
    )
    parser.add_argument(
        "-e",
        "--exclude",
        type=float,
        action="append",
        default=[],
    )
    parser.add_argument("-r", "--reverse", action="store_true", default=False)
    parser.add_argument(
        "-t",
        "--number-type",
        "--number_type",
        dest="number_type",
        choices=("int", "float", "real", "f", "i", "r"),
        default="int",
    )

    sign = parser.add_mutually_exclusive_group()
    sign.add_argument("--nosign", action="store_true", default=False)
    sign.add_argument("-s", "--sign", action="store_true", default=False)

    parser.add_argument("--noexp", action="store_true", default=False)
    parser.add_argument("-l", "--locale", action="store_true", default=False)
    parser.add_argument(
        "-z",
        "--zero-terminated",
        action="store_true",
        default=False,
        dest="zero_terminated",
    )
    parser.add_argument("entries", nargs="*")
    return parser


def main(*arguments: str) -> None:
    """Parse CLI arguments and delegate sorting to the Rust-backed API."""
    parsed = _parser().parse_args(list(arguments) if arguments else None)

    args = TypedArgs(
        filter=parsed.filter,
        reverse_filter=parsed.reverse_filter,
        exclude=parsed.exclude,
        paths=parsed.paths,
        reverse=parsed.reverse,
        zero_terminated=parsed.zero_terminated,
        entries=parsed.entries,
    )
    args.number_type = parsed.number_type
    args.signed = bool(
        parsed.sign
        or (
            not parsed.nosign
            and parsed.number_type in {"real", "r"}
        )
    )
    args.exp = not parsed.noexp
    args.locale = parsed.locale

    args.filter = check_filters(args.filter)
    args.reverse_filter = check_filters(args.reverse_filter)

    entries = get_entries(args)
    sort_and_print_entries(entries, args)


if __name__ == "__main__":
    try:
        main()
    except ValueError as error:
        sys.exit(str(error))
    except KeyboardInterrupt:
        sys.exit(1)
