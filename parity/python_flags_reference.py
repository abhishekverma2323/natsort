from __future__ import annotations

from typing import Any

from natsort import numeric_regex_chooser, ns


def show(name: str, value: Any) -> None:
    print(f"\n{name}")
    print(repr(value))


def show_flag(name: str) -> None:
    value = getattr(ns, name)

    show(
        f"flag_{name}",
        {
            "repr": repr(value),
            "int": int(value),
        },
    )


def show_regex(name: str, algorithm: Any) -> None:
    try:
        result = numeric_regex_chooser(algorithm)

        show(
            f"regex_{name}",
            {
                "type": type(result).__name__,
                "repr": repr(result),
                "string": str(result),
            },
        )
    except Exception as error:
        show(
            f"regex_error_{name}",
            {
                "type": type(error).__name__,
                "message": str(error),
            },
        )


def main() -> None:
    flag_names = [
        "DEFAULT",
        "INT",
        "UNSIGNED",
        "FLOAT",
        "SIGNED",
        "NOEXP",
        "PATH",
        "LOCALEALPHA",
        "LOCALENUM",
        "LOCALE",
        "IGNORECASE",
        "LOWERCASEFIRST",
        "GROUPLETTERS",
        "UNGROUPLETTERS",
        "NANLAST",
        "COMPATIBILITYNORMALIZE",
        "NUMAFTER",
        "PRESORT",
        "REAL",
    ]

    for flag_name in flag_names:
        if hasattr(ns, flag_name):
            show_flag(flag_name)

    combinations = [
        ("default", ns.DEFAULT),
        ("int", ns.INT),
        ("unsigned", ns.UNSIGNED),
        ("signed_int", ns.INT | ns.SIGNED),
        ("float", ns.FLOAT),
        ("signed_float", ns.FLOAT | ns.SIGNED),
        ("float_noexp", ns.FLOAT | ns.NOEXP),
        (
            "signed_float_noexp",
            ns.FLOAT | ns.SIGNED | ns.NOEXP,
        ),
        ("real", ns.REAL),
        ("path", ns.PATH),
        ("locale", ns.LOCALE),
        (
            "float_locale",
            ns.FLOAT | ns.LOCALE,
        ),
        (
            "signed_float_locale",
            ns.FLOAT | ns.SIGNED | ns.LOCALE,
        ),
        (
            "all_numeric_modifiers",
            ns.FLOAT
            | ns.SIGNED
            | ns.NOEXP
            | ns.LOCALENUM,
        ),
    ]

    for name, algorithm in combinations:
        show_regex(name, algorithm)

    invalid_values = [
        -1,
        999999999,
        "FLOAT",
        None,
        1.5,
    ]

    for index, value in enumerate(invalid_values):
        show_regex(
            f"invalid_{index}_{value!r}",
            value,
        )

    aliases = [
        ("DEFAULT_INT", ns.DEFAULT, ns.INT),
        ("INT_UNSIGNED", ns.INT, ns.UNSIGNED),
        (
            "REAL_SIGNED_FLOAT",
            ns.REAL,
            ns.SIGNED | ns.FLOAT,
        ),
        (
            "LOCALE_COMBINATION",
            ns.LOCALE,
            ns.LOCALEALPHA | ns.LOCALENUM,
        ),
    ]

    for name, left, right in aliases:
        show(
            f"alias_{name}",
            {
                "left": int(left),
                "right": int(right),
                "equal": left == right,
            },
        )


if __name__ == "__main__":
    main()