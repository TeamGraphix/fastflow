"""Example code for GF(2) solver."""

# %%
from __future__ import annotations

import numpy as np
from fastflow import solver

# %%
# Coefficient matrix
a = [
    [1, 1],
    [0, 0],
]

# Two right-hand sides packed as columns
#
# No. 0: [0, 0]
# No. 1: [1, 1]
bm = [
    [0, 1],
    [0, 1],
]

# %%
x0, x1 = solver.solve(a, bm)

# Solution found for bm[:, 0]
assert x0 is not None

# If singular, pick an arbitrary solution
assert np.array_equal(x0, [0, 0])

# No solution for bm[:, 1]
assert x1 is None

# %%

# One right-hand side as a vector
bv = [1, 0]

# %%
(x,) = solver.solve(a, bv)

assert x is not None
assert np.array_equal(x, [1, 0])
