# First-Party Rust Unsafe and FFI Audit

## Scope and generated evidence

This report covers first-party Rust code under `rust-port/src/` and
`rust-port/tests/`. It does not claim that every transitive dependency is
implemented without unsafe Rust.

The reproducible audit was generated from commit:

```text
43719532854ba2f6e926648cb9ae8fdceec13ac2
```

Machine-readable evidence:

- `parity/evidence/audit/project_metrics.json`
- `parity/evidence/audit/unsafe_matches.txt`
- `parity/evidence/audit/checksums.sha256`
- `HONEST_NUMBERS.md`

## Result

The first-party implementation is **not zero-unsafe**.

Exact owned unsafe surface:

```text
Explicit unsafe blocks:       1
Unsafe functions:             0
Foreign ABI declarations:     1
Unsafe guard exit code:       0
```

Exact matches:

```text
rust-port/src/os_sort.rs:407: unsafe extern "system"
rust-port/src/os_sort.rs:419: StrCmpLogicalW call inside one unsafe block
```

No first-party use was found for:

- `transmute`;
- unchecked indexing;
- raw allocation ownership conversion;
- `MaybeUninit` or `ManuallyDrop`;
- dynamic `Any` / downcast escape hatches;
- general-purpose unsafe functions.

## Windows-only FFI boundary

The port calls the Windows Shell Lightweight Utility function:

```text
StrCmpLogicalW
Shlwapi.dll
```

This path is compiled only on Windows and used only when the selected OS-sort
profile resolves to the native Windows system profile. Explicit locale
profiles and non-Windows targets use the safe Rust implementation.

## Safety contract

Before the native call, each Rust string is:

1. encoded as UTF-16;
2. copied into an independently owned `Vec<u16>`;
3. terminated with a UTF-16 null;
4. kept alive for the entire native call.

Only immutable pointers are passed. The native function does not take
ownership. Its signed result is converted only into an ordering relative to
zero. The call site has an adjacent `SAFETY` comment recording these
invariants.

## Why the native call is retained

OS-aware sorting exists to preserve platform-observable behavior. On Windows,
`StrCmpLogicalW` provides native logical filename ordering. Replacing it
unconditionally with the cross-platform emulation could alter punctuation,
Unicode, locale, or leading-zero behavior.

Keeping one visible, documented boundary is more honest than:

- silently weakening Windows fidelity;
- hiding the same call inside a wrapper crate;
- invoking an external Python or Windows process.

## CI enforcement

`rust-port/scripts/check_unsafe.sh` permits exactly the documented declaration
and call site. Any additional first-party `unsafe` occurrence fails CI.

The generated audit also records:

```text
Rust library tests:             381
Rust Python-reference tests:    176
Original suite against Rust:    344
CLI shared cases:               20 / 20
Differential fuzz divergences:  0 / 11,727
```

## Accurate submission claim

> The Rust port uses one narrowly scoped Windows-only FFI boundary consisting
> of one foreign declaration and one unsafe call site. It uses no
> general-purpose first-party unsafe memory operations.
