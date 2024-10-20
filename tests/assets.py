"""Test assets."""

from __future__ import annotations

import dataclasses

import networkx as nx
from fastflow.common import FlowResult, GFlowResult, Plane, PPlane


@dataclasses.dataclass(frozen=True)
class FlowTestCase:
    g: nx.Graph[int]
    iset: set[int]
    oset: set[int]
    plane: dict[int, Plane] | None
    pplane: dict[int, PPlane] | None
    flow: FlowResult[int] | None
    gflow: GFlowResult[int] | None
    pflow: GFlowResult[int] | None


# MEMO: DO NOT modify while testing
#  May be tested in parallel

# 1 - 2
CASE0 = FlowTestCase(
    nx.Graph([(1, 2)]),
    {1, 2},
    {1, 2},
    None,
    None,
    FlowResult({}, {1: 0, 2: 0}),
    GFlowResult({}, {1: 0, 2: 0}),
    GFlowResult({}, {1: 0, 2: 0}),
)

# 1 - 2 - 3 - 4 - 5
CASE1 = FlowTestCase(
    nx.Graph([(1, 2), (2, 3), (3, 4), (4, 5)]),
    {1},
    {5},
    None,
    None,
    FlowResult({1: 2, 2: 3, 3: 4, 4: 5}, {1: 4, 2: 3, 3: 2, 4: 1, 5: 0}),
    GFlowResult({1: {2}, 2: {3}, 3: {4}, 4: {5}}, {1: 4, 2: 3, 3: 2, 4: 1, 5: 0}),
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
    None,
    FlowResult({3: 5, 4: 6, 1: 3, 2: 4}, {1: 2, 2: 2, 3: 1, 4: 1, 5: 0, 6: 0}),
    GFlowResult({3: {5}, 4: {6}, 1: {3}, 2: {4}}, {1: 2, 2: 2, 3: 1, 4: 1, 5: 0, 6: 0}),
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
    None,
    GFlowResult({1: {5, 6}, 2: {4, 5, 6}, 3: {4, 6}}, {1: 1, 2: 1, 3: 1, 4: 0, 5: 0, 6: 0}),
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
    {0: Plane.XY, 1: Plane.XY, 2: Plane.XZ, 3: Plane.YZ},
    {0: PPlane.XY, 1: PPlane.XY, 2: PPlane.XZ, 3: PPlane.YZ},
    None,
    GFlowResult({0: {2}, 1: {5}, 2: {2, 4}, 3: {3}}, {0: 2, 1: 2, 2: 1, 3: 1, 4: 0, 5: 0}),
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
    None,
    None,
)

#     3
#     |
#     2
#     |
# 0 - 1 - 4
CASE6 = FlowTestCase(
    nx.Graph([(0, 1), (1, 2), (1, 4), (2, 3)]),
    {0},
    {4},
    {0: Plane.XY, 1: Plane.XY, 2: Plane.XY, 3: Plane.XY},
    {0: PPlane.XY, 1: PPlane.X, 2: PPlane.XY, 3: PPlane.X},
    None,
    None,
    GFlowResult({0: {1}, 1: {4}, 2: {3}, 3: {2, 4}}, {0: 1, 1: 1, 2: 0, 3: 1, 4: 0}),
)

# 1   2   3
# | /     |
# 0 - - - 4
CASE7 = FlowTestCase(
    nx.Graph([(0, 1), (0, 2), (0, 4), (3, 4)]),
    {0},
    {4},
    {0: Plane.YZ, 1: Plane.XZ, 2: Plane.XY, 3: Plane.YZ},
    {0: PPlane.Z, 1: PPlane.Z, 2: PPlane.Y, 3: PPlane.Y},
    None,
    None,
    GFlowResult({0: {0}, 1: {1}, 2: {2}, 3: {4}}, {0: 1, 1: 0, 2: 0, 3: 1, 4: 0}),
)

# 0 - 1 -- 3
#    \|   /|
#     |\ / |
#     | /\ |
#     2 -- 4
CASE8 = FlowTestCase(
    nx.Graph([(0, 1), (0, 4), (1, 2), (1, 3), (2, 3), (2, 4), (3, 4)]),
    {0},
    {3, 4},
    {0: Plane.YZ, 1: Plane.XZ, 2: Plane.XY},
    {0: PPlane.Z, 1: PPlane.XZ, 2: PPlane.Y},
    None,
    None,
    GFlowResult({0: {0, 2, 4}, 1: {1, 2}, 2: {4}}, {0: 1, 1: 1, 2: 1, 3: 0, 4: 0}),
)

CASES: tuple[FlowTestCase, ...] = (CASE0, CASE1, CASE2, CASE3, CASE4, CASE5, CASE6, CASE7, CASE8)
