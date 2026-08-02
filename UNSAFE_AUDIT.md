# First-Party Rust Unsafe and FFI Audit

## Audit scope

This report audits the first-party Rust implementation under:

- `rust-port/src/`
- `rust-port/tests/`

Audited submission commit:

`15380cdbc000f24fea31bed920dbff186e5ba0b1`

This report does not claim that every transitive third-party dependency is
implemented without unsafe Rust. It reports the unsafe surface directly owned
and maintained by this port.

## Result

The first-party Rust implementation is not zero-unsafe.

The complete identified unsafe surface consists of:

- one Windows-only `unsafe extern "system"` declaration;
- one Windows-only unsafe FFI call;
- zero unsafe functions;
- zero unsafe traits or implementations;
- zero `transmute` uses;
- zero raw allocation ownership conversions;
- zero unchecked indexing operations;
- zero `MaybeUninit` or `ManuallyDrop` uses;
- zero dynamic `Any` or downcast escape hatches;
- zero `.unwrap()` calls.

Both unsafe occurrences are confined to `rust-port/src/os_sort.rs` and are
compiled only on Windows.

## Unsafe boundary

The port calls the Windows Shell Lightweight Utility API:

`StrCmpLogicalW`

The function is provided by:

`Shlwapi.dll`

It is used only when all of the following are true:

1. the Rust target operating system is Windows;
2. the selected OS sorting profile resolves to Windows;
3. the selected locale profile is the system locale.

Other Windows-profile configurations use the safe, pure-Rust emulation path.

Non-Windows builds do not compile or link this FFI declaration.

## Safety contract

Before invoking `StrCmpLogicalW`, each Rust string is:

1. encoded as UTF-16;
2. copied into its own `Vec<u16>`;
3. terminated with a UTF-16 null value;
4. kept alive for the complete duration of the native call.

Only immutable pointers are passed.

The native function does not take ownership of either buffer.

The returned signed integer is converted only into an ordering relative to
zero.

The call site includes an adjacent `SAFETY` comment documenting these
invariants.

## Why the native API is retained

The purpose of the OS-aware sorting API is to preserve platform-observable
ordering behavior.

On Windows, `StrCmpLogicalW` provides the native logical filename comparison
used for Windows-style ordering. Replacing it unconditionally with the
cross-platform emulation could change punctuation, locale, Unicode, or
leading-zero behavior for the system profile.

The migration therefore keeps the native API behind a minimal and explicit
FFI boundary rather than silently weakening Windows behavior to claim a
zero-unsafe result.

## Alternatives considered

### Always use the pure-Rust emulation

Rejected for the Windows system profile because it could reduce fidelity to
native Windows ordering.

The emulation remains available for explicit locale profiles and for
cross-platform compatibility testing.

### Hide the call inside another wrapper dependency

Rejected because moving the same unsafe operation into another crate would
hide rather than eliminate the safety boundary.

### Execute an external Windows or Python sorting process

Rejected because it would weaken reproducibility, performance, and standalone
runtime independence.

## Verification

The audit searched for:

- `unsafe`;
- external ABI declarations;
- raw pointers;
- `transmute`;
- raw ownership conversion;
- unchecked access;
- dynamic `Any`;
- panic-style escape hatches.

At the audited commit:

- Rust formatting passed;
- Clippy passed with warnings denied;
- 377 Rust library/unit tests passed;
- 176 Python-reference parity tests passed;
- the working tree remained clean.

## CI policy

The repository contains `rust-port/scripts/check_unsafe.sh`.

The script permits exactly the documented Windows FFI declaration and call
site. Any additional first-party unsafe occurrence causes CI to fail.

## Honest submission claim

The submission does not claim zero unsafe.

The accurate claim is:

> The Rust port uses one narrowly scoped Windows-only FFI boundary consisting
> of one unsafe extern declaration and one unsafe call site. No general-purpose
> unsafe memory operations or dynamic escape hatches are used.
