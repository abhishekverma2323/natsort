from __future__ import annotations

import locale
from typing import Any

from natsort import (
    humansorted,
    index_humansorted,
    index_natsorted,
    natsort_keygen,
    natsorted,
    ns,
)


LOCALE_CANDIDATES = [
    "",
    "C",
    "en_US.UTF-8",
    "en_US",
    "English_United States.1252",
    "English_India.1252",
    "de_DE.UTF-8",
    "de_DE",
    "German_Germany.1252",
    "fr_FR.UTF-8",
    "fr_FR",
    "French_France.1252",
]


def show(name: str, value: Any) -> None:
    print(f"{name}: {value!r}")


def localized_number(value: float) -> str:
    return locale.format_string(
        "%.2f",
        value,
        grouping=True,
    )


def run_locale_cases(requested_locale: str) -> None:
    try:
        active_locale = locale.setlocale(
            locale.LC_ALL,
            requested_locale,
        )
    except locale.Error as error:
        print(
            f"\nSKIPPED locale={requested_locale!r}: "
            f"{error}"
        )
        return

    print("\n" + "=" * 80)
    show("requested_locale", requested_locale)
    show("active_locale", active_locale)

    locale_data = locale.localeconv()

    show(
        "decimal_point",
        locale_data["decimal_point"],
    )
    show(
        "thousands_sep",
        locale_data["thousands_sep"],
    )
    show(
        "grouping",
        locale_data["grouping"],
    )

    alpha_values = [
        "Apple",
        "apple",
        "Äpfel",
        "äpfel",
        "Banana",
        "banana",
        "Öl",
        "Oase",
        "Zebra",
    ]

    show(
        "default_alpha",
        natsorted(alpha_values),
    )

    show(
        "locale_alpha",
        natsorted(
            alpha_values,
            alg=ns.LOCALEALPHA,
        ),
    )

    show(
        "humansorted_alpha",
        humansorted(alpha_values),
    )

    natural_values = [
        "file10",
        "File2",
        "file1",
        "Äpfel20",
        "Äpfel3",
        "apple11",
        "apple2",
        "Öl5",
        "Oase4",
    ]

    show(
        "default_natural",
        natsorted(natural_values),
    )

    show(
        "locale_natural",
        natsorted(
            natural_values,
            alg=ns.LOCALE,
        ),
    )

    show(
        "humansorted_natural",
        humansorted(natural_values),
    )

    show(
        "index_humansorted_natural",
        index_humansorted(natural_values),
    )

    localized_numbers = [
        localized_number(1234.50),
        localized_number(12.50),
        localized_number(2.75),
        localized_number(1000.25),
        localized_number(10.25),
    ]

    show(
        "localized_numbers_input",
        localized_numbers,
    )

    show(
        "localized_numbers_default",
        natsorted(
            localized_numbers,
            alg=ns.FLOAT,
        ),
    )

    show(
        "localized_numbers_localenum",
        natsorted(
            localized_numbers,
            alg=ns.FLOAT | ns.LOCALENUM,
        ),
    )

    show(
        "localized_numbers_locale",
        natsorted(
            localized_numbers,
            alg=ns.FLOAT | ns.LOCALE,
        ),
    )

    mixed_values = [
        localized_number(1000.50),
        "Apple2",
        localized_number(10.25),
        "apple10",
        localized_number(2.50),
        "Äpfel1",
    ]

    show(
        "mixed_locale_input",
        mixed_values,
    )

    show(
        "mixed_locale_sorted",
        natsorted(
            mixed_values,
            alg=ns.FLOAT | ns.LOCALE,
        ),
    )

    show(
        "mixed_locale_indexes",
        index_natsorted(
            mixed_values,
            alg=ns.FLOAT | ns.LOCALE,
        ),
    )

    locale_key = natsort_keygen(
        alg=ns.FLOAT | ns.LOCALE,
    )

    for value in [
        "Apple2",
        "apple10",
        "Äpfel1",
        localized_number(1234.50),
        localized_number(2.50),
    ]:
        show(
            f"locale_key_{value}",
            locale_key(value),
        )


def main() -> None:
    original_locale = locale.setlocale(
        locale.LC_ALL,
    )

    show("original_locale", original_locale)

    try:
        visited = set()

        for candidate in LOCALE_CANDIDATES:
            try:
                active = locale.setlocale(
                    locale.LC_ALL,
                    candidate,
                )
            except locale.Error:
                active = None

            if active is None or active in visited:
                if active is None:
                    print(
                        f"\nSKIPPED locale={candidate!r}"
                    )
                continue

            visited.add(active)
            run_locale_cases(candidate)
    finally:
        locale.setlocale(
            locale.LC_ALL,
            original_locale,
        )


if __name__ == "__main__":
    main()