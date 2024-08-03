"""Common functionalities for the fastflow package."""

from __future__ import annotations

import dataclasses
from collections.abc import Hashable
from collections.abc import Set as AbstractSet
from enum import Enum
from types import MappingProxyType
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
    if not isinstance(g, nx.Graph):
        msg = "g must be a networkx.Graph."
        raise TypeError(msg)
    if not isinstance(iset, AbstractSet):
        msg = "iset must be a set."
        raise TypeError(msg)
    if not isinstance(oset, AbstractSet):
        msg = "oset must be a set."
        raise TypeError(msg)
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


class IndexMap(Generic[V]):
    """Map between `V` and 0-based indices."""

    __v2i: dict[V, int]
    __i2v: dict[int, V]

    def __init__(self, vset: AbstractSet[V]) -> None:
        """Initialize the map from `vset`."""
        self.__v2i = {v: i for i, v in enumerate(vset)}
        self.__i2v = {i: v for v, i in self.__v2i.items()}

    def encode(self, v: V) -> int:
        """Encode `v` to the index."""
        ind = self.__v2i.get(v)
        if ind is None:
            msg = f"{v} not found."
            raise ValueError(msg)
        return ind

    def encode_graph(self, g: nx.Graph[V]) -> list[set[int]]:
        """Encode graph."""
        n = len(g)
        g_: list[set[int]] = [set() for _ in range(n)]
        for u, i in self.__v2i.items():
            for v in g[u]:
                g_[i].add(self.encode(v))
        return g_

    def encode_set(self, vset: AbstractSet[V]) -> set[int]:
        """Encode set."""
        return {self.encode(v) for v in vset}

    def decode(self, i: int) -> V:
        """Decode the index."""
        v = self.__i2v.get(i)
        if v is None:
            msg = f"{i} not found."
            raise ValueError(msg)
        return v

    def decode_set(self, iset: AbstractSet[int]) -> set[V]:
        """Decode set."""
        return {self.decode(i) for i in iset}

    def decode_flow(self, f_: dict[int, int]) -> dict[V, V]:
        """Decode flow."""
        return {self.decode(i): self.decode(j) for i, j in f_.items()}

    def decode_gflow(self, f_: dict[int, set[int]]) -> dict[V, set[V]]:
        """Decode gflow."""
        return {self.decode(i): self.decode_set(si) for i, si in f_.items()}

    def decode_layer(self, layer_: list[int]) -> dict[V, int]:
        """Decode layer."""
        return {self.decode(i): li for i, li in enumerate(layer_)}

    @property
    def v2i(self) -> MappingProxyType[V, int]:
        """Return the mapping from `V` to the index."""
        return MappingProxyType(self.__v2i)

    @property
    def i2v(self) -> MappingProxyType[int, V]:
        """Return the mapping from the index to `V`."""
        return MappingProxyType(self.__i2v)
