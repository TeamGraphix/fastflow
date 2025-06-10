from __future__ import annotations

import subprocess  # noqa: S404
import sys
from pathlib import Path

import pytest

SRC = list(Path("examples").glob("*.py"))


@pytest.mark.parametrize("f", SRC)
def test_examples(f: Path) -> None:
    # MEMO: Possibly insecure!
    subprocess.run(  # noqa: S603
        [sys.executable, str(f)], check=True
    )
