"""Compute the flow."""

from __future__ import annotations

import networkx as nx
from fastflow import flow, gflow

# Flow does not exist
# Gflow exists
#   ______
#  /      |
# 1 - 4   |
#    /    |
#   /     |
#  /      |
# 2 - 5   |
#  \ /    |
#   X    /
#  / \  /
# 3 - 6
#
# in: 1, 2, 3
# out: 4, 5, 6

e: list[tuple[int, int]] = [(1, 4), (1, 6), (2, 4), (2, 5), (2, 6), (3, 5), (3, 6)]
g = nx.Graph(e)
iset = {1, 2, 3}
oset = {4, 5, 6}

print(flow.find(g, iset, oset))
print(gflow.find(g, iset, oset))
