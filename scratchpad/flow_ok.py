"""Compute the flow."""

from __future__ import annotations

import networkx as nx
from fastflow import flow

# Flow exists
# 1 - 3 - 5
#     |
# 2 - 4 - 6
#
# in: 1, 2
# out: 5, 6

e: list[tuple[int, int]] = [(1, 3), (3, 5), (2, 4), (4, 6), (3, 4)]
g = nx.Graph(e)
iset = {1, 2}
oset = {5, 6}

print(flow.find(g, iset, oset))
