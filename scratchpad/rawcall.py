"""Use the raw binding."""

from fastflow import _impl

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

print(_impl.find(g, iset, oset))
