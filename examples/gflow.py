"""Example code for finding generalized flow."""

# %%

from __future__ import annotations

import networkx as nx
from fastflow import gflow
from fastflow.common import Plane

g: nx.Graph[int]

# %%

#   0 - 1
#  /|   |
# 4 |   |
#  \|   |
#   2 - 5 - 3
g = nx.Graph([(0, 1), (0, 2), (0, 4), (1, 5), (2, 4), (2, 5), (3, 5)])
iset = {0, 1}
oset = {4, 5}
planes = {0: Plane.XY, 1: Plane.XY, 2: Plane.XZ, 3: Plane.YZ}

result = gflow.find(g, iset, oset, planes)

# Found
assert result is not None

# %%

# 1 - 3
#  \ /
#   X
#  / \
# 2 - 4
g = nx.Graph([(1, 3), (1, 4), (2, 3), (2, 4)])
iset = {1, 2}
oset = {3, 4}
# Omitting planes (all Plane.XY)

result = gflow.find(g, iset, oset)

# Not found
assert result is None
