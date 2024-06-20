"""Compute the flow."""

from fastflow import flow

# graph with flow and gflow
# 0 - 2 - 4
#     |
# 1 - 3 - 5
#
# in: 0, 1
# out: 4, 5

g = [{2}, {3}, {0, 3, 4}, {1, 2, 5}, {2}, {3}]
iset = {0, 1}
oset = {4, 5}

print(flow.find(g, iset, oset))
