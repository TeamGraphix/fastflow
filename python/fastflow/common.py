"""Common functionalities for the fastflow package."""

from __future__ import annotations

import dataclasses
from collections.abc import Hashable
from collections.abc import Set as AbstractSet
from enum import Enum
from typing import Generic, Literal, TypeVar

import networkx as nx

# Vertex type
V = TypeVar("V", bound=Hashable)


class Plane(Enum):
    """Measurement planes in MBQC."""

    # DO NOT change the associated values!
    XY = 0
    YZ = 1
    ZX = 2


class PauliPlane(Enum):
    """Measurement planes for Pauli flow."""

    # DO NOT change the associated values!
    XY = 0
    YZ = 1
    ZX = 2
    X = 3
    Y = 4
    Z = 5


_Plane = Literal[0, 1, 2]
_PPlane = Literal[0, 1, 2, 3, 4, 5]


@dataclasses.dataclass(frozen=True)
class FlowResult(Generic[V]):
    """MBQC flow.

    Attributes
    ----------
    f : `dict[V, V]`
        Flow function.
    layer : `dict[V, int]`
        Layer of each vertex representing the partial order.
        (u -> v iff `layer[u] > layer[v]`).
    """

    f: dict[V, V]
    layer: dict[V, int]


@dataclasses.dataclass(frozen=True)
class GFlowResult(Generic[V]):
    """MBQC gflow.

    Attributes
    ----------
    f : `dict[V, set[V]]`
        Gflow function.
    layer : `dict[V, int]`
        Layer of each vertex representing the partial order.
        (u -> v iff `layer[u] > layer[v]`).
    """

    f: dict[V, set[V]]
    layer: dict[V, int]


def check_graph(g: nx.Graph[V], iset: AbstractSet[V], oset: AbstractSet[V]) -> None:
    """Check if g is a valid MBQC graph.

    Raises
    ------
    ValueError
        If the graph is empty, has self-loops, or iset/oset are not subsets of the vertices.
    """
    if len(g) == 0:
        msg = "Graph is empty."
        raise ValueError(msg)
    # BUG: Incorrect annotation
    if nx.number_of_selfloops(g) > 0:  # type: ignore[arg-type]
        msg = "Self-loop detected."
        raise ValueError(msg)
    vset = set(g.nodes)
    if not (iset <= vset):
        msg = "iset must be a subset of the vertices."
        raise ValueError(msg)
    if not (oset <= vset):
        msg = "oset must be a subset of the vertices."
        raise ValueError(msg)
