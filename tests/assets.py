"""Test assets."""

from __future__ import annotations

import dataclasses

import networkx as nx
from fastflow.common import FlowResult, GFlowResult, Plane


@dataclasses.dataclass(frozen=True)
class FlowTestCase:
    """Test case for flow/gflow."""

    g: nx.Graph[int]
    iset: set[int]
    oset: set[int]
    plane: dict[int, Plane] | None
    flow: FlowResult[int] | None
    gflow: GFlowResult[int] | None


# MEMO: DO NOT modify while testing
#  May be tested in parallel

# 1 - 2
CASE0 = FlowTestCase(
    nx.Graph([(1, 2)]),
    {1, 2},
    {1, 2},
    None,
    FlowResult({}, {1: 0, 2: 0}),
    GFlowResult({}, {1: 0, 2: 0}),
)

# 1 - 2 - 3 - 4 - 5
CASE1 = FlowTestCase(
    nx.Graph([(1, 2), (2, 3), (3, 4), (4, 5)]),
    {1},
    {5},
    None,
    FlowResult({1: 2, 2: 3, 3: 4, 4: 5}, {1: 4, 2: 3, 3: 2, 4: 1, 5: 0}),
    GFlowResult({1: {2}, 2: {3}, 3: {4}, 4: {5}}, {1: 4, 2: 3, 3: 2, 4: 1, 5: 0}),
)


# 1 - 3 - 5
#     |
# 2 - 4 - 6
CASE2 = FlowTestCase(
    nx.Graph([(1, 3), (2, 4), (3, 5), (4, 6)]),
    {1, 2},
    {5, 6},
    None,
    FlowResult({3: 5, 4: 6, 1: 3, 2: 4}, {1: 2, 2: 2, 3: 1, 4: 1, 5: 0, 6: 0}),
    GFlowResult({3: {5}, 4: {6}, 1: {3}, 2: {4}}, {1: 2, 2: 2, 3: 1, 4: 1, 5: 0, 6: 0}),
)

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
CASE3 = FlowTestCase(
    nx.Graph([(1, 4), (1, 6), (2, 4), (2, 5), (2, 6), (3, 5), (3, 6)]),
    {1, 2, 3},
    {4, 5, 6},
    None,
    None,
    GFlowResult({1: {5, 6}, 2: {4, 5, 6}, 3: {4, 6}}, {1: 1, 2: 1, 3: 1, 4: 0, 5: 0, 6: 0}),
)

#   0 - 1
#  /|   |
# 4 |   |
#  \|   |
#   2 - 5 - 3
CASE4 = FlowTestCase(
    nx.Graph([(0, 1), (0, 2), (0, 4), (1, 5), (2, 4), (2, 5), (3, 5)]),
    {0, 1},
    {4, 5},
    {0: Plane.XY, 1: Plane.XY, 2: Plane.ZX, 3: Plane.YZ},
    None,
    GFlowResult({0: {2}, 1: {5}, 2: {2, 4}, 3: {3}}, {0: 2, 1: 2, 2: 1, 3: 1, 4: 0, 5: 0}),
)


# 1 - 3
#  \ /
#   X
#  / \
# 2 - 4
CASE5 = FlowTestCase(
    nx.Graph([(1, 3), (1, 4), (2, 3), (2, 4)]),
    {1, 2},
    {3, 4},
    None,
    None,
    None,
)

CASES = [CASE0, CASE1, CASE2, CASE3, CASE4, CASE5]
