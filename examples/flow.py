"""Example code for finding causal flow."""

# %%

from __future__ import annotations

import networkx as nx
from fastflow import flow

g: nx.Graph[int]

# %%

# 1 - 3 - 5
#     |
# 2 - 4 - 6
g = nx.Graph([(1, 3), (2, 4), (3, 5), (4, 6)])
iset = {1, 2}
oset = {5, 6}

result = flow.find(g, iset, oset)

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

# Not found
result = flow.find(g, iset, oset)

assert result is None
