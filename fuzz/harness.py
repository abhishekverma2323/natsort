"""Run the maintained differential-fuzzing harness."""

from __future__ import annotations

import runpy
from pathlib import Path

TARGET = Path(__file__).resolve().parents[1] / "parity" / "differential_fuzz.py"

runpy.run_path(str(TARGET), run_name="__main__")
