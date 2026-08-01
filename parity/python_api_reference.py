from __future__ import annotations

from typing import Any

from natsort import (
    index_natsorted,
    index_realsorted,
    natsorted,
    ns,
    order_by_index,
    realsorted,
)


def show_case(
    name: str,
    input_value: Any,
    expected: Any,
) -> None:
    print(f"\n{name}")
    print(f"Input:    {input_value}")
    print(f"Expected: {expected}")


def main() -> None:
    real_values = [
        "num5.10",
        "num-3",
        "num5.3",
        "num2",
    ]

    show_case(
        "realsorted_basic",
        real_values,
        realsorted(real_values),
    )

    show_case(
        "realsorted_reverse",
        real_values,
        realsorted(real_values, reverse=True),
    )

    basic_values = [
        "num3",
        "num5",
        "num2",
    ]

    show_case(
        "index_natsorted_basic",
        basic_values,
        index_natsorted(basic_values),
    )

    show_case(
        "index_natsorted_reverse",
        basic_values,
        index_natsorted(basic_values, reverse=True),
    )

    show_case(
        "index_realsorted_basic",
        real_values,
        index_realsorted(real_values),
    )

    indexes = index_natsorted(basic_values)

    show_case(
        "order_by_index_primary",
        {
            "values": basic_values,
            "indexes": indexes,
        },
        order_by_index(basic_values, indexes),
    )

    secondary_values = [
        "foo",
        "bar",
        "baz",
    ]

    show_case(
        "order_by_index_secondary",
        {
            "values": secondary_values,
            "indexes": indexes,
        },
        order_by_index(secondary_values, indexes),
    )

    records = [
        {"name": "file10", "id": 10},
        {"name": "file2", "id": 2},
        {"name": "file1", "id": 1},
    ]

    show_case(
        "natsorted_with_key",
        records,
        natsorted(
            records,
            key=lambda record: record["name"],
        ),
    )

    show_case(
        "natsorted_with_key_reverse",
        records,
        natsorted(
            records,
            key=lambda record: record["name"],
            reverse=True,
        ),
    )

    duplicate_records = [
        {"name": "file01", "id": "first"},
        {"name": "file1", "id": "second"},
        {"name": "file001", "id": "third"},
    ]

    show_case(
        "key_sort_stability",
        duplicate_records,
        natsorted(
            duplicate_records,
            key=lambda record: record["name"],
        ),
    )

    show_case(
        "index_equivalent_value_stability",
        [
            "file01",
            "file1",
            "file001",
        ],
        index_natsorted(
            [
                "file01",
                "file1",
                "file001",
            ]
        ),
    )

    presort_values = [
        "a1",
        "a1.45",
        "a01",
        "a1.4500",
    ]

    show_case(
        "index_presort",
        presort_values,
        index_natsorted(
            presort_values,
            alg=ns.FLOAT | ns.PRESORT,
        ),
    )

    show_case(
        "index_presort_reverse",
        presort_values,
        index_natsorted(
            presort_values,
            reverse=True,
            alg=ns.FLOAT | ns.PRESORT,
        ),
    )


if __name__ == "__main__":
    main()