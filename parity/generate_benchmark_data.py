from __future__ import annotations

import argparse
import hashlib
import json
import random
from pathlib import Path


MODES = ("default", "float", "real", "path", "locale")
DEFAULT_SIZES = (1_000, 10_000, 50_000)

WORDS = (
    "file",
    "File",
    "value",
    "item",
    "alpha",
    "beta",
    "folder",
    "Folder",
    "version",
    "release",
    "archive",
    "sample",
    "report",
    "dataset",
)

LOCALE_WORDS = (
    "apple",
    "Apple",
    "Äpfel",
    "äpfel",
    "Oase",
    "Öl",
    "uber",
    "Über",
    "éclair",
    "Éclair",
    "straße",
    "Strasse",
    "résumé",
    "resume",
)

SEPARATORS = ("", "_", "-", ".", " ", "(", ")")


def integer_token(rng: random.Random) -> str:
    value = rng.randint(0, 10_000_000)
    text = str(value)

    if rng.random() < 0.30:
        text = text.zfill(len(text) + rng.randint(1, 4))

    return text


def float_token(
    rng: random.Random,
    *,
    signed: bool,
    exponent: bool,
) -> str:
    whole = rng.randint(0, 1_000_000)
    fraction = rng.randint(0, 999_999)
    style = rng.randrange(4)

    if style == 0:
        text = f"{whole}.{fraction:06d}"
    elif style == 1:
        text = f".{fraction:06d}"
    elif style == 2:
        text = f"{whole}."
    else:
        text = str(whole)

    if exponent and rng.random() < 0.25:
        power = rng.randint(-12, 12)
        text += rng.choice(("e", "E")) + (f"+{power}" if power >= 0 else str(power))

    if signed and rng.random() < 0.50:
        text = rng.choice(("+", "-")) + text

    return text


def default_entry(rng: random.Random) -> str:
    value = rng.choice(WORDS) + rng.choice(SEPARATORS) + integer_token(rng)

    if rng.random() < 0.45:
        value += rng.choice(SEPARATORS) + rng.choice(WORDS)

    if rng.random() < 0.30:
        value += rng.choice(SEPARATORS) + integer_token(rng)

    return value


def float_entry(rng: random.Random, *, signed: bool) -> str:
    value = (
        rng.choice(WORDS)
        + rng.choice(SEPARATORS)
        + float_token(rng, signed=signed, exponent=True)
    )

    if rng.random() < 0.40:
        value += rng.choice(SEPARATORS) + rng.choice(WORDS)

    return value


def path_entry(rng: random.Random) -> str:
    directory = rng.choice(("Folder", "folder", "Dir", "dir", "archive", ".hidden"))
    directory_number = integer_token(rng)
    file_number = integer_token(rng)
    separator = rng.choice(("/", "\\"))
    extension = rng.choice(("txt", "csv", "json", "log", "tar.gz", "data.bin"))

    if rng.random() < 0.22:
        directory_component = directory
    else:
        directory_component = (
            directory + rng.choice(("", " ", "_", "-")) + directory_number
        )

    value = (
        directory_component
        + separator
        + rng.choice(("file", "File", "item", "data", "report"))
        + file_number
        + "."
        + extension
    )

    if rng.random() < 0.12:
        value = rng.choice(("./", ".\\")) + value

    return value


def locale_entry(rng: random.Random) -> str:
    value = rng.choice(LOCALE_WORDS) + rng.choice(SEPARATORS) + integer_token(rng)

    if rng.random() < 0.35:
        value += rng.choice(SEPARATORS) + rng.choice(LOCALE_WORDS)

    return value


def make_entry(mode: str, rng: random.Random) -> str:
    if mode == "default":
        return default_entry(rng)
    if mode == "float":
        return float_entry(rng, signed=False)
    if mode == "real":
        return float_entry(rng, signed=True)
    if mode == "path":
        return path_entry(rng)
    if mode == "locale":
        return locale_entry(rng)

    raise ValueError(f"unsupported mode: {mode}")


def generate_dataset(
    *,
    mode: str,
    size: int,
    seed: int,
) -> list[str]:
    mode_seed = seed ^ sum((index + 1) * ord(ch) for index, ch in enumerate(mode))
    rng = random.Random(mode_seed + size)
    entries: list[str] = []

    for _ in range(size):
        if entries and rng.random() < 0.08:
            entries.append(rng.choice(entries))
        else:
            entries.append(make_entry(mode, rng))

    rng.shuffle(entries)
    return entries


def write_dataset(path: Path, entries: list[str]) -> str:
    payload = "\n".join(entries) + "\n"
    encoded = payload.encode("utf-8")
    path.write_bytes(encoded)
    return hashlib.sha256(encoded).hexdigest()


def main() -> None:
    parser = argparse.ArgumentParser(
        description="Generate deterministic datasets for Python/Rust natsort benchmarks."
    )
    parser.add_argument("--seed", type=int, default=20260801)
    parser.add_argument(
        "--sizes",
        type=int,
        nargs="+",
        default=list(DEFAULT_SIZES),
    )
    parser.add_argument(
        "--output-dir",
        type=Path,
        default=Path(__file__).resolve().parent / "benchmark_data",
    )
    args = parser.parse_args()

    if any(size <= 0 for size in args.sizes):
        parser.error("all sizes must be greater than zero")

    args.output_dir.mkdir(parents=True, exist_ok=True)
    manifest: dict[str, object] = {
        "seed": args.seed,
        "modes": list(MODES),
        "sizes": args.sizes,
        "datasets": [],
    }

    for mode in MODES:
        for size in args.sizes:
            entries = generate_dataset(mode=mode, size=size, seed=args.seed)
            filename = f"{mode}_{size}.txt"
            path = args.output_dir / filename
            sha256 = write_dataset(path, entries)

            manifest["datasets"].append(
                {
                    "mode": mode,
                    "size": size,
                    "file": filename,
                    "sha256": sha256,
                }
            )
            print(f"generated {filename}: {size} entries")

    manifest_path = args.output_dir / "manifest.json"
    manifest_path.write_text(
        json.dumps(manifest, indent=2, ensure_ascii=False) + "\n",
        encoding="utf-8",
    )
    print(f"manifest: {manifest_path}")


if __name__ == "__main__":
    main()