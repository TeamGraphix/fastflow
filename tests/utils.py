from __future__ import annotations

from typing import TYPE_CHECKING

import numpy as np
import numpy.typing as npt

if TYPE_CHECKING:
    from collections.abc import Iterator


def iter_bmatrix(rows: int, cols: int) -> Iterator[npt.NDArray[np.bool_]]:
    """Iterate over all binary matrices with given dimensions."""
    assert rows > 0
    assert cols > 0
    size = rows * cols
    bmax = 1 << size
    for b in range(bmax):
        bv = np.fromiter(((b >> i) & 1 for i in range(size)), dtype=np.bool_)
        yield bv.reshape(rows, cols)


def iter_bvector(length: int) -> Iterator[npt.NDArray[np.bool_]]:
    """Iterate over all binary vectors with given length."""
    assert length > 0
    bmax = 1 << length
    for b in range(bmax):
        bv = np.fromiter(((b >> i) & 1 for i in range(length)), dtype=np.bool_)
        yield bv.astype(np.bool_)
