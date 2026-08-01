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

    show_case(
        "empty_input",
        [],
    )

    show_case(
        "single_item",
        ["file10"],
    )

    show_case(
        "plain_text",
        ["banana", "apple", "cherry"],
    )

    show_case(
        "multiple_numeric_components",
        ["version1.10.2", "version1.2.10", "version1.2.2"],
    )

    show_case(
        "equivalent_leading_zero_values",
        ["file1", "file01", "file001"],
    )

    show_case(
        "signed_zero",
        ["value-0", "value0", "value+0"],
        ns.SIGNED,
    )

    show_case(
        "high_precision_decimals",
        [
            "value1.000000000002",
            "value1.000000000001",
            "value1.1",
        ],
        ns.FLOAT,
    )

    show_case(
        "mixed_scientific_notation",
        ["value1E3", "value2e2", "value5E-1", "value10"],
        ns.FLOAT,
    )

    show_case(
        "punctuation_and_separators",
        ["file-10", "file_2", "file.1", "file-2"],
    )

    show_case(
        "unicode_arabic_indic_digits",
        [
            "file١٠",
            "file٢",
            "file١",
        ],
    )

    show_case(
        "unicode_devanagari_digits",
        [
            "file१०",
            "file२",
            "file१",
        ],
    )

    show_case(
        "unicode_fullwidth_digits",
        [
            "file１０",
            "file２",
            "file１",
        ],
    )

    show_case(
        "mixed_unicode_digit_scripts",
        [
            "file10",
            "file٢",
            "file३",
            "file１",
        ],
    )

    show_case(
        "unicode_signed_integers",
        [
            "value-१०",
            "value२",
            "value-१",
        ],
        ns.SIGNED,
    )

    show_case(
        "unicode_decimal_values",
        [
            "value١.٥",
            "value١.٢٥",
            "value٢.٠",
        ],
        ns.FLOAT,
    )

if __name__ == "__main__":
    main()