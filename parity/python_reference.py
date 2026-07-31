from natsort import natsorted, ns


def show_case(name: str, values: list[str], algorithm: ns = ns.DEFAULT) -> None:
    result = natsorted(values, alg=algorithm)

    print(f"\n{name}")
    print(f"Input:    {values}")
    print(f"Expected: {result}")


def main() -> None:
    show_case(
        "basic_natural_sort",
        ["file10", "file2", "file1"],
    )

    show_case(
        "leading_zeros",
        ["file10", "file002", "file2"],
    )

    show_case(
        "large_integers",
        [
            "file999999999999999999999999",
            "file20",
            "file18446744073709551616",
        ],
    )

    show_case(
        "signed_integers",
        ["value5", "value-2", "value1", "value-10"],
        ns.SIGNED,
    )

    show_case(
        "float_values",
        ["value1.5", "value1.25", "value10.01", "value2.0"],
        ns.FLOAT,
    )

    show_case(
        "scientific_notation",
        ["value1e3", "value2.5e2", "value4.2e-3", "value1", "value10"],
        ns.FLOAT,
    )

    show_case(
        "signed_float_values",
        ["value1.5", "value-2.25", "value-10.5", "value0.25"],
        ns.FLOAT | ns.SIGNED,
    )

    show_case(
        "ignore_case",
        ["File10", "file2", "FILE1"],
        ns.IGNORECASE,
    )


if __name__ == "__main__":
    main()