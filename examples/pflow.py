"""Example code for finding Pauli flow."""

# %%

from __future__ import annotations

import networkx as nx
from fastflow import pflow
from fastflow.common import PPlane

g: nx.Graph[int]

# %%

# 1   2   3
# | /     |
# 0 - - - 4
g = nx.Graph([(0, 1), (0, 2), (0, 4), (3, 4)])
iset = {0}
oset = {4}
pplanes = {0: PPlane.Z, 1: PPlane.Z, 2: PPlane.Y, 3: PPlane.Y}

result = pflow.find(g, iset, oset, pplanes)

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
# Omitting pplanes (all PPlane.XY)

# NOTE: This results in warning (use gflow.find if pplanes has no Pauli measurements)
result = pflow.find(g, iset, oset)

# Not found
assert result is None
