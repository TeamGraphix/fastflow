"""Common functionalities for the fastflow package."""

from __future__ import annotations

from collections.abc import Hashable
from collections.abc import Set as AbstractSet
from typing import Generic, NamedTuple, TypeVar

import networkx as nx

# Vertex type
V = TypeVar("V", bound=Hashable)


class FlowResult(NamedTuple, Generic[V]):
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


class GFlowResult(NamedTuple, Generic[V]):
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
        If the graph has self-loops or iset/oset are not subsets of the vertices.
    """
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
