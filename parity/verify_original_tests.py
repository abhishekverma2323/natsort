from __future__ import annotations

import argparse
import hashlib
import subprocess
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
OUTPUT_DIR = ROOT / "parity" / "test_hashes"


def run_text(*arguments: str) -> str:
    result = subprocess.run(
        list(arguments),
        cwd=ROOT,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        check=False,
        text=True,
    )

    if result.returncode != 0:
        raise RuntimeError(
            f"command failed: {' '.join(arguments)}\n{result.stderr}"
        )

    return result.stdout


def run_bytes(*arguments: str) -> bytes:
    result = subprocess.run(
        list(arguments),
        cwd=ROOT,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        check=False,
    )

    if result.returncode != 0:
        raise RuntimeError(
            f"command failed: {' '.join(arguments)}\n"
            + result.stderr.decode("utf-8", errors="replace")
        )

    return result.stdout


def sha256(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def listed_paths(*arguments: str) -> list[str]:
    return sorted(
        line
        for line in run_text(*arguments).splitlines()
        if line
    )


def main() -> None:
    parser = argparse.ArgumentParser(
        description=(
            "Verify that the submission's tracked test blobs match the "
            "pinned source commit."
        )
    )
    parser.add_argument("--source-commit", required=True)
    args = parser.parse_args()

    source_paths = listed_paths(
        "git",
        "ls-tree",
        "-r",
        "--name-only",
        args.source_commit,
        "--",
        "tests",
    )
    submission_paths = listed_paths("git", "ls-files", "--", "tests")

    if not source_paths:
        raise SystemExit("ERROR: no source tests found at pinned commit")

    problems: list[str] = []

    source_set = set(source_paths)
    submission_set = set(submission_paths)

    for relative in sorted(source_set - submission_set):
        problems.append(f"REMOVED  {relative}")

    for relative in sorted(submission_set - source_set):
        problems.append(f"ADDED_TRACKED  {relative}")

    unstaged = listed_paths(
        "git",
        "diff",
        "--name-only",
        "--",
        "tests",
    )
    staged = listed_paths(
        "git",
        "diff",
        "--cached",
        "--name-only",
        "--",
        "tests",
    )
    untracked = listed_paths(
        "git",
        "ls-files",
        "--others",
        "--exclude-standard",
        "--",
        "tests",
    )

    for relative in unstaged:
        problems.append(f"UNSTAGED_CHANGE  {relative}")

    for relative in staged:
        problems.append(f"STAGED_CHANGE  {relative}")

    for relative in untracked:
        problems.append(f"ADDED_UNTRACKED  {relative}")

    source_lines: list[str] = []
    submission_lines: list[str] = []

    for relative in sorted(source_set & submission_set):
        source_data = run_bytes(
            "git",
            "show",
            f"{args.source_commit}:{relative}",
        )
        submission_data = run_bytes(
            "git",
            "show",
            f":{relative}",
        )

        source_digest = sha256(source_data)
        submission_digest = sha256(submission_data)

        source_lines.append(f"{source_digest}  {relative}")
        submission_lines.append(f"{submission_digest}  {relative}")

        if source_digest != submission_digest:
            problems.append(f"MODIFIED_BLOB  {relative}")

    OUTPUT_DIR.mkdir(parents=True, exist_ok=True)

    source_manifest = OUTPUT_DIR / "source.sha256"
    submission_manifest = OUTPUT_DIR / "submission.sha256"
    verification = OUTPUT_DIR / "verification.txt"

    source_manifest.write_text(
        "\n".join(source_lines) + "\n",
        encoding="utf-8",
    )
    submission_manifest.write_text(
        "\n".join(submission_lines) + "\n",
        encoding="utf-8",
    )

    lines = [
        f"source_commit={args.source_commit}",
        "comparison_basis=canonical_git_blob_content",
        "working_tree_line_endings=not_used_for_hash_comparison",
        f"source_test_files={len(source_paths)}",
        f"submission_test_files={len(submission_paths)}",
        f"problems={len(problems)}",
        *problems,
    ]
    verification.write_text(
        "\n".join(lines) + "\n",
        encoding="utf-8",
    )

    if problems:
        print("\n".join(lines))
        raise SystemExit(1)

    print(
        f"PASS: {len(source_paths)} original test files match "
        "the pinned source commit"
    )
    print("Comparison basis: canonical Git blob content")
    print(f"Source manifest: {source_manifest.relative_to(ROOT)}")
    print(
        f"Submission manifest: "
        f"{submission_manifest.relative_to(ROOT)}"
    )
    print(f"Verification: {verification.relative_to(ROOT)}")


if __name__ == "__main__":
    main()
