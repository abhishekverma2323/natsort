from __future__ import annotations

from typing import Any

from natsort import (
    index_natsorted,
    natsort_keygen,
    natsorted,
    ns,
)


def show_case(
    name: str,
    input_value: Any,
    expected: Any,
) -> None:
    print(f"\n{name}")
    print(f"Input:    {input_value!r}")
    print(f"Expected: {expected!r}")


def main() -> None:
    mixed_basic = [
        "73",
        "5039",
        "Banana",
        "apple",
        "corn",
        "~~~~~~",
    ]

    show_case(
        "numafter_basic",
        mixed_basic,
        natsorted(
            mixed_basic,
            alg=ns.NUMAFTER,
        ),
    )

    show_case(
        "numafter_default_comparison",
        mixed_basic,
        natsorted(mixed_basic),
    )

    mixed_direct = [
        "0",
        1.5,
        "2",
        3,
        "ä",
        "Ä",
        "b",
        "Z",
    ]

    show_case(
        "numafter_mixed_direct_values",
        mixed_direct,
        natsorted(
            mixed_direct,
            alg=ns.NUMAFTER,
        ),
    )

    show_case(
        "numafter_capital_first",
        mixed_direct,
        natsorted(
            mixed_direct,
            alg=ns.NUMAFTER | ns.CAPITALFIRST,
        ),
    )

    show_case(
        "numafter_ignore_case",
        [
            "10",
            "Apple",
            "apple",
            "2",
            "Banana",
            "banana",
        ],
        natsorted(
            [
                "10",
                "Apple",
                "apple",
                "2",
                "Banana",
                "banana",
            ],
            alg=ns.NUMAFTER | ns.IGNORECASE,
        ),
    )

    show_case(
        "numafter_group_letters",
        [
            "10",
            "Apple",
            "apple",
            "2",
            "Banana",
            "banana",
        ],
        natsorted(
            [
                "10",
                "Apple",
                "apple",
                "2",
                "Banana",
                "banana",
            ],
            alg=ns.NUMAFTER | ns.GROUPLETTERS,
        ),
    )

    show_case(
        "numafter_path",
        [
            "10",
            "folder10/file",
            "folder2/file",
            "2",
            "apple",
        ],
        natsorted(
            [
                "10",
                "folder10/file",
                "folder2/file",
                "2",
                "apple",
            ],
            alg=ns.NUMAFTER | ns.PATH,
        ),
    )

    show_case(
        "numafter_signed",
        [
            -10,
            "value-2",
            2,
            "apple",
            "value1",
        ],
        natsorted(
            [
                -10,
                "value-2",
                2,
                "apple",
                "value1",
            ],
            alg=ns.NUMAFTER | ns.SIGNED,
        ),
    )

    show_case(
        "numafter_float",
        [
            1.5,
            "value1.25",
            2,
            "apple",
            "value1.5",
        ],
        natsorted(
            [
                1.5,
                "value1.25",
                2,
                "apple",
                "value1.5",
            ],
            alg=ns.NUMAFTER | ns.FLOAT,
        ),
    )

    show_case(
        "numafter_reverse",
        mixed_basic,
        natsorted(
            mixed_basic,
            reverse=True,
            alg=ns.NUMAFTER,
        ),
    )

    show_case(
        "numafter_indexes",
        mixed_basic,
        index_natsorted(
            mixed_basic,
            alg=ns.NUMAFTER,
        ),
    )

    default_key = natsort_keygen()
    numafter_key = natsort_keygen(
        alg=ns.NUMAFTER
    )

    for value in [
        "apple",
        "73",
        73,
        "file2",
        "~~~~~~",
    ]:
        show_case(
            f"default_key_{value!r}",
            value,
            default_key(value),
        )

        show_case(
            f"numafter_key_{value!r}",
            value,
            numafter_key(value),
        )


if __name__ == "__main__":
    main()