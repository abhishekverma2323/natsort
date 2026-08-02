#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(
  cd -- "$(dirname -- "${BASH_SOURCE[0]}")" >/dev/null 2>&1
  pwd
)"
CRATE_ROOT="$(cd -- "$SCRIPT_DIR/.." >/dev/null 2>&1 && pwd)"

cd "$CRATE_ROOT"

actual="$(
  grep -RInE \
    '(^|[^[:alnum:]_])unsafe([^[:alnum:]_]|$)' \
    src tests \
    --include='*.rs' \
    2>/dev/null \
    | sed -E 's#^([^:]+):[0-9]+:#\1:#' \
    || true
)"

expected="$(cat <<'EXPECTED'
src/os_sort.rs:unsafe extern "system" {
src/os_sort.rs:    let result = unsafe { str_cmp_logical_w(left_wide.as_ptr(), right_wide.as_ptr()) };
EXPECTED
)"

if [[ "$actual" != "$expected" ]]; then
  echo "FAIL: First-party unsafe boundary changed."
  echo
  echo "Expected:"
  printf '%s\n' "$expected"
  echo
  echo "Actual:"
  printf '%s\n' "${actual:-<none>}"
  exit 1
fi

if ! grep -Fq '#[cfg(target_os = "windows")]' src/os_sort.rs; then
  echo "FAIL: Windows platform gate was not found."
  exit 1
fi

if ! grep -Fq '#[link(name = "Shlwapi")]' src/os_sort.rs; then
  echo "FAIL: Expected Shlwapi link declaration was not found."
  exit 1
fi

if ! grep -Fq '#[link_name = "StrCmpLogicalW"]' src/os_sort.rs; then
  echo "FAIL: Expected StrCmpLogicalW declaration was not found."
  exit 1
fi

echo "PASS: First-party unsafe usage matches the documented Windows-only boundary."
echo "unsafe_extern_blocks=1"
echo "unsafe_call_sites=1"
