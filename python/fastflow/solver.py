"""GF(2) linear solver implemented in Rust."""

from __future__ import annotations

import numpy as np
import numpy.typing as npt

from fastflow._impl import solve as solve_bind


def _arraycheck(x: npt.ArrayLike) -> npt.NDArray[np.bool_]:
    # Cast to array with dtype inferred
    x = np.asarray(x)
    if x.dtype == np.bool_:
        return x
    if not np.issubdtype(x.dtype, np.integer):
        msg = "Need to cast non-integral dtype to boolean."
        raise TypeError(msg)
    xb = x.astype(np.bool_)
    if np.any(x != xb):
        msg = "Casted array is not equivalent to the original."
        raise ValueError(msg)
    return xb


def solve(a: npt.ArrayLike, b: npt.ArrayLike) -> list[npt.NDArray[np.bool_] | None]:
    """Solve the linear equations :math:`Ax = b` over GF(2).

    Parameters
    ----------
    a : `numpy.typing.ArrayLike`
        Coefficient matrix of shape :code:`(rows, cols)`.
    b : `numpy.typing.ArrayLike`
        Right-hand side matrix/vector of shape :code:`(rows, neqs)` or :code:`(rows,)`.

    Returns
    -------
    `list`
        `numpy.ndarray` (if solvable) or `None` (otherwise) for each equation in :code:`b`.

    Notes
    -----
    While this function is deterministic even when :code:`a` is singular, solution picking \
    algorithm is unspecified and subject to change.
    """
    a = _arraycheck(a)
    b = _arraycheck(b)
    if a.ndim != 2:  # noqa: PLR2004
        msg = "a must be a 2D array."
        raise ValueError(msg)
    if b.ndim == 1:
        b = b.reshape(-1, 1)
    if b.ndim != 2:  # noqa: PLR2004
        msg = "b must be a 2D array or a 1D vector."
        raise ValueError(msg)
    rows, _ = a.shape
    _, neqs = b.shape
    if b.shape != (rows, neqs):
        msg = "Inconsistent number of rows in a and b."
        raise ValueError(msg)
    return solve_bind(a, b)
