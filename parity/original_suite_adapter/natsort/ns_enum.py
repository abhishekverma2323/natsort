"""Rust-backed Python-compatible algorithm flag surface."""

from __future__ import annotations

from . import ns


NS_DUMB = 1 << 31
NSType = ns | int

__all__ = ["NS_DUMB", "NSType", "ns"]
