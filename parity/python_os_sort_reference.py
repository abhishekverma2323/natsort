from __future__ import annotations

import locale
import platform
from typing import Any

from natsort import (
    os_sort_key,
    os_sort_keygen,
    os_sorted,
)


def show(name: str, value: Any) -> None:
    print(f"\n{name}")
    print(repr(value))


def main() -> None:
    show("platform", platform.platform())
    show("system", platform.system())
    show("locale", locale.setlocale(locale.LC_ALL, ""))

    basic = [
        "file10",
        "file2",
        "File3",
        "file1",
        "file_0",
    ]

    show("basic_input", basic)
    show("basic_sorted", os_sorted(basic))
    show(
        "basic_reverse",
        os_sorted(basic, reverse=True),
    )

    equal_numbers = [
        "a1",
        "a01",
        "a001",
    ]

    show(
        "equivalent_default",
        os_sorted(equal_numbers),
    )

    show(
        "equivalent_presort",
        os_sorted(equal_numbers, presort=True),
    )

    key_values = [
        "foo0",
        "foo2",
        "goo1",
    ]

    show(
        "key_sort",
        os_sorted(
            key_values,
            key=lambda value: value.replace("g", "f"),
        ),
    )

    path_values = [
        "Folder10/file2.txt",
        "Folder2/file10.txt",
        "Folder2/file2.txt",
        "folder1/file20.txt",
    ]

    show("path_sort", os_sorted(path_values))

    unicode_values = [
        "Äpfel10",
        "apple2",
        "Apple10",
        "äpfel2",
        "Öl5",
        "Oase4",
    ]

    show("unicode_sort", os_sorted(unicode_values))

    mixed_numbers = [
        "10",
        2,
        "1",
        11,
        5.5,
        None,
        float("nan"),
    ]

    show("mixed_sort", os_sorted(mixed_numbers))

    punctuation = [
        "11111",
        "aaaaa",
        "foo0",
        "foo_0",
        "foo1",
        "foo2",
        "foo4",
        "foo10",
        "Foo3",
        "!",
        "#",
        "$",
        "%",
        "&",
        "'",
        "(",
        ")",
        "+",
        "+11111",
        "+aaaaa",
        ",",
        "-",
        ";",
        "=",
        "@",
        "[",
        "]",
        "^",
        "_",
        "`",
        "{",
        "}",
        "~",
        "§",
        "°",
        "´",
        "µ",
        "€",
    ]

    show(
        "punctuation_sort",
        os_sorted(punctuation),
    )

    generated_key = os_sort_keygen()

    for value in [
        "file10",
        "file2",
        "File3",
        "foo_0",
        "10",
        10,
        None,
        float("nan"),
    ]:
        try:
            show(
                f"os_sort_key_{value!r}",
                os_sort_key(value),
            )
        except Exception as error:
            show(
                f"os_sort_key_error_{value!r}",
                (
                    type(error).__name__,
                    str(error),
                ),
            )

        try:
            show(
                f"os_sort_keygen_{value!r}",
                generated_key(value),
            )
        except Exception as error:
            show(
                f"os_sort_keygen_error_{value!r}",
                (
                    type(error).__name__,
                    str(error),
                ),
            )


if __name__ == "__main__":
    main()