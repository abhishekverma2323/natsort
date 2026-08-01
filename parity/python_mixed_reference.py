from __future__ import annotations

from typing import Any, Callable

from natsort import (
    as_ascii,
    as_utf8,
    decoder,
    natsort_keygen,
    natsorted,
    ns,
)


def show_case(
    name: str,
    input_value: Any,
    operation: Callable[[], Any],
) -> None:
    print(f"\n{name}")
    print(f"Input:    {input_value!r}")

    try:
        result = operation()
    except Exception as error:
        print(
            "Error:    "
            f"{type(error).__name__}: {error}"
        )
    else:
        print(f"Expected: {result!r}")


def main() -> None:
    mixed_values = ["a2", 3, "a1", 2]

    show_case(
        "mixed_text_and_integers",
        mixed_values,
        lambda: natsorted(mixed_values),
    )

    numeric_strings_and_numbers = ["10", 2, "1", 11]

    show_case(
        "numeric_strings_and_integers",
        numeric_strings_and_numbers,
        lambda: natsorted(numeric_strings_and_numbers),
    )

    signed_mixed_values = [
        "value5",
        -3,
        "value-2",
        1,
        "value1",
    ]

    show_case(
        "signed_mixed_values",
        signed_mixed_values,
        lambda: natsorted(
            signed_mixed_values,
            alg=ns.REAL,
        ),
    )

    float_values = [
        5.10,
        -3.0,
        5.3,
        2,
    ]

    show_case(
        "direct_numeric_values",
        float_values,
        lambda: natsorted(float_values),
    )

    equivalent_numbers = [
        1,
        1.0,
        1e0,
        2,
    ]

    show_case(
        "equivalent_direct_numbers",
        equivalent_numbers,
        lambda: natsorted(equivalent_numbers),
    )

    infinity_values = [
        float("inf"),
        5,
        float("-inf"),
        0,
    ]

    show_case(
        "positive_and_negative_infinity",
        infinity_values,
        lambda: natsorted(infinity_values),
    )

    none_nan_values = [
        3,
        None,
        float("nan"),
        float("-inf"),
        2,
    ]

    show_case(
        "none_and_nan_default",
        none_nan_values,
        lambda: natsorted(none_nan_values),
    )

    none_nan_last_values = [
        3,
        None,
        float("nan"),
        float("inf"),
        2,
    ]

    show_case(
        "none_and_nan_last",
        none_nan_last_values,
        lambda: natsorted(
            none_nan_last_values,
            alg=ns.NANLAST,
        ),
    )

    nested_values = [
        ["a10", "b2"],
        ["a2", "b10"],
        ["a2", "b2"],
    ]

    show_case(
        "nested_string_sequences",
        nested_values,
        lambda: natsorted(nested_values),
    )

    nested_real_values = [
        ["x5.10", "y-2"],
        ["x5.3", "y1"],
        ["x2", "y10"],
    ]

    show_case(
        "nested_real_sequences",
        nested_real_values,
        lambda: natsorted(
            nested_real_values,
            alg=ns.REAL,
        ),
    )

    nested_mixed_values = [
        ["a2", 10],
        ["a2", 2],
        ["a1", 20],
    ]

    show_case(
        "nested_mixed_sequences",
        nested_mixed_values,
        lambda: natsorted(nested_mixed_values),
    )

    deeper_nested_values = [
        [["a2"], ["b10"]],
        [["a2"], ["b2"]],
        [["a1"], ["b20"]],
    ]

    show_case(
        "deeply_nested_sequences",
        deeper_nested_values,
        lambda: natsorted(deeper_nested_values),
    )

    byte_values = [
        b"a10",
        b"a2",
        b"A1",
    ]

    show_case(
        "bytes_default",
        byte_values,
        lambda: natsorted(byte_values),
    )

    byte_ignore_case_values = [
        b"a10",
        b"A2",
        b"a1",
    ]

    show_case(
        "bytes_ignore_case",
        byte_ignore_case_values,
        lambda: natsorted(
            byte_ignore_case_values,
            alg=ns.IGNORECASE,
        ),
    )

    byte_path_values = [
        b"folder10/file",
        b"folder2/file",
        b"folder1/file",
    ]

    show_case(
        "bytes_path_mode",
        byte_path_values,
        lambda: natsorted(
            byte_path_values,
            alg=ns.PATH,
        ),
    )

    decoded_byte_values = [
        b"a10",
        "a2",
        b"a1",
    ]

    show_case(
        "mixed_bytes_and_strings_with_decoder",
        decoded_byte_values,
        lambda: natsorted(
            decoded_byte_values,
            key=decoder("utf8"),
        ),
    )

    show_case(
        "decode_ascii_bytes",
        b"natural10",
        lambda: as_ascii(b"natural10"),
    )

    show_case(
        "decode_ascii_non_bytes",
        "natural10",
        lambda: as_ascii("natural10"),
    )

    show_case(
        "decode_utf8_bytes",
        "café".encode("utf8"),
        lambda: as_utf8("café".encode("utf8")),
    )

    show_case(
        "decode_utf8_non_bytes",
        123,
        lambda: as_utf8(123),
    )

    show_case(
        "invalid_utf8_decoder",
        b"\xff",
        lambda: decoder("utf8")(b"\xff"),
    )

    default_key = natsort_keygen()

    show_case(
        "default_key_string",
        "a-5.034e2",
        lambda: default_key("a-5.034e2"),
    )

    real_key = natsort_keygen(alg=ns.REAL)

    show_case(
        "real_key_string",
        "a-5.034e2",
        lambda: real_key("a-5.034e2"),
    )

    show_case(
        "real_key_direct_number",
        56.7,
        lambda: real_key(56.7),
    )

    show_case(
        "real_key_nested_values",
        ["x5.10", "y-2", 3],
        lambda: real_key(
            ["x5.10", "y-2", 3]
        ),
    )

    ignore_case_key = natsort_keygen(
        alg=ns.IGNORECASE
    )

    show_case(
        "ignore_case_key_string",
        "Straße10",
        lambda: ignore_case_key("Straße10"),
    )

    show_case(
        "ignore_case_key_bytes",
        b"A10",
        lambda: ignore_case_key(b"A10"),
    )

    show_case(
        "default_key_none",
        None,
        lambda: default_key(None),
    )

    show_case(
        "default_key_nan",
        float("nan"),
        lambda: default_key(float("nan")),
    )

    nan_last_key = natsort_keygen(
        alg=ns.NANLAST
    )

    show_case(
        "nan_last_key_nan",
        float("nan"),
        lambda: nan_last_key(float("nan")),
    )

    show_case(
        "nan_last_key_none",
        None,
        lambda: nan_last_key(None),
    )

    show_case(
        "default_key_negative_infinity",
        float("-inf"),
        lambda: default_key(float("-inf")),
    )

    show_case(
        "nan_last_key_positive_infinity",
        float("inf"),
        lambda: nan_last_key(float("inf")),
    )


if __name__ == "__main__":
    main()