from __future__ import annotations

import itertools

import numpy as np
import pytest
from fastflow import solver

from tests import utils


def test_1d() -> None:
    a = np.asarray([[1, 1], [0, 1]])
    b = np.asarray([0, 1])
    (x,) = solver.solve(a, b)
    np.testing.assert_array_equal(x, [1, 1])


def test_2d() -> None:
    a = np.asarray([[1, 1], [0, 1]])
    b = np.asarray([[0, 1], [1, 1]])
    x0, x1 = solver.solve(a, b)
    np.testing.assert_array_equal(x0, [1, 1])
    np.testing.assert_array_equal(x1, [0, 1])


def test_no_sol() -> None:
    a = np.asarray([[0, 0], [0, 0]])
    b = np.asarray([1, 1])
    (x,) = solver.solve(a, b)
    assert x is None


def test_baddim_a() -> None:
    a = np.asarray([0, 1])
    b = np.asarray([0, 1])
    with pytest.raises(ValueError, match=r"a must be a 2D array\."):
        solver.solve(a, b)


def test_baddim_b() -> None:
    a = np.asarray([[1, 1], [0, 1]])
    b = np.arange(2).reshape(1, 1, 2)
    with pytest.raises(ValueError, match=r"b must be a 2D array or a 1D vector\."):
        solver.solve(a, b)


def test_inconsistent_rows() -> None:
    a = np.asarray([[1, 1], [0, 1]])
    b = np.asarray([0, 1, 1])
    with pytest.raises(ValueError, match=r"Inconsistent number of rows in a and b\."):
        solver.solve(a, b)


def test_bad_type() -> None:
    a = np.asarray([[1, 1], [0, 1]], dtype=np.bool_)

    b = [1.0, 1.0]
    with pytest.raises(TypeError, match=r".*non-integral.*"):
        solver.solve(a, b)

    b = np.asarray([1, 2])
    with pytest.raises(ValueError, match=r".*not equivalent.*"):
        solver.solve(a, b)


@pytest.mark.parametrize(
    ("m", "n"), [(m, n) for m, n in itertools.product(range(1, 11), repeat=2) if (m + 1) * n <= 12]
)
def test_all(m: int, n: int) -> None:
    for a in utils.iter_bmatrix(m, n):
        b = np.stack(list(utils.iter_bvector(m)), axis=-1)
        x = solver.solve(a, b)
        neqs = len(x)
        assert b.shape == (m, neqs)
        x_ = np.stack([xi for xi in x if xi is not None], axis=-1)
        b_ = np.stack([b[:, i] for i in range(neqs) if x[i] is not None], axis=-1)
        lhs = a.astype(np.int64) @ x_.astype(np.int64)
        lhs = (lhs % 2).astype(np.bool_)
        np.testing.assert_array_equal(lhs, b_)
