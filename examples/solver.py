"""Example code for GF(2) solver."""

from __future__ import annotations

import numpy as np
from fastflow import solver

# Coefficient matrix
a = [
    [1, 1],
    [0, 0],
]

# Right-hand sides packed as column vectors
#
# No. 0: [0, 0]
# No. 1: [1, 1]
b = [
    [0, 1],
    [0, 1],
]

x0, x1 = solver.solve(a, b)

# If any, solutions are returned as column vectors.
#   sum(x0) is guaranteed to be minimal.
assert x0 is not None
assert np.array_equal(x0, [0, 0])

# Otherwise, None is returned.
assert x1 is None
