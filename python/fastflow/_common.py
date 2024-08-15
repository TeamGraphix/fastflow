"""Private common functionalities for the fastflow package."""

from __future__ import annotations

from collections.abc import Hashable, Iterable, Mapping
from collections.abc import Set as AbstractSet
from typing import Generic, TypeVar

import networkx as nx

from fastflow.common import Plane, PPlane

# Vertex type
V = TypeVar("V", bound=Hashable)


# Plane-like
P = TypeVar("P", Plane, PPlane)


def check_graph(g: nx.Graph[V], iset: AbstractSet[V], oset: AbstractSet[V]) -> None:
    """Check if `(g, iset, oset)` is a valid open graph for MBQC.

    Raises
    ------
    TypeError
        If input types are incorrect.
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


def check_planelike(vset: AbstractSet[V], oset: AbstractSet[V], plike: Mapping[V, P]) -> None:
    r"""Check if measurement config. is valid.

    Raises
    ------
    TypeError
        If input types are incorrect.
    ValueError
        If plike is not a subset of the vertices, or measurement planes are not specified for all u in V\O.
    """
    if not isinstance(plike, Mapping):
        msg = "Measurement planes must be passed as a mapping."
        raise TypeError(msg)
    if not (plike.keys() <= vset):
        msg = "Cannot find corresponding vertices in the graph."
        raise ValueError(msg)
    if not (vset - oset <= plike.keys()):
        msg = "Measurement planes should be specified for all u in V\\O."
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
        """Encode `v` to the index.

        Returns
        -------
        Index of `v`.

        Raises
        ------
        ValueError
            If `v` is not initially registered.
        """
        ind = self.__v2i.get(v)
        if ind is None:
            msg = f"{v} not found."
            raise ValueError(msg)
        return ind

    def encode_graph(self, g: nx.Graph[V]) -> list[set[int]]:
        """Encode graph.

        Returns
        -------
        Input graph with vertices encoded to indices.
        """
        n = len(g)
        g_: list[set[int]] = [set() for _ in range(n)]
        for u, i in self.__v2i.items():
            for v in g[u]:
                g_[i].add(self.encode(v))
        return g_

    def encode_set(self, vset: AbstractSet[V]) -> set[int]:
        """Encode set.

        Returns
        -------
        Transformed set.
        """
        return {self.encode(v) for v in vset}

    def encode_dictkey(self, mapping: Mapping[V, P]) -> dict[int, P]:
        """Encode dict key.

        Returns
        -------
        Dict with transformed keys.
        """
        return {self.encode(k): v for k, v in mapping.items()}

    def decode(self, i: int) -> V:
        """Decode the index.

        Returns
        -------
        Value corresponding to the index.

        Raises
        ------
        ValueError
            If `i` is out of range.
        """
        v = self.__i2v.get(i)
        if v is None:
            msg = f"{i} not found."
            raise ValueError(msg)
        return v

    def decode_set(self, iset: AbstractSet[int]) -> set[V]:
        """Decode set.

        Returns
        -------
        Transformed set.
        """
        return {self.decode(i) for i in iset}

    def decode_flow(self, f_: Mapping[int, int]) -> dict[V, V]:
        """Decode MBQC flow.

        Returns
        -------
        Transformed flow.
        """
        return {self.decode(i): self.decode(j) for i, j in f_.items()}

    def decode_gflow(self, f_: Mapping[int, AbstractSet[int]]) -> dict[V, set[V]]:
        """Decode MBQC gflow.

        Returns
        -------
        Transformed gflow.
        """
        return {self.decode(i): self.decode_set(si) for i, si in f_.items()}

    def decode_layer(self, layer_: Iterable[int]) -> dict[V, int]:
        """Decode MBQC layer.

        Returns
        -------
        Transformed layer as dict.
        """
        return {self.decode(i): li for i, li in enumerate(layer_)}
