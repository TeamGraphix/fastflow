"""Maximally-delayed gflow algorithm.

For given undirected graph, input nodes, and output nodes, compute the generalized flow having \
the minimum number of layers.
See Mhalla et al. (2008) for more details.
"""

from __future__ import annotations

from typing import TYPE_CHECKING

from fastflow import common
from fastflow._impl import gflow
from fastflow.common import GFlowResult, V

if TYPE_CHECKING:
    from collections.abc import Set as AbstractSet

    import networkx as nx


def find(g: nx.Graph[V], iset: AbstractSet[V], oset: AbstractSet[V]) -> GFlowResult[V] | None:
    """Compute the maximally-delayed generalized flow, if any.

    Parameters
    ----------
    g : `nx.Graph[V]`
        Undirected graph representing MBQC pattern.
        Cannot have self-loops.
    iset : `AbstractSet[V]`
        Input nodes.
        Must be a subset of `g.nodes`.
    oset : `AbstractSet[V]`
        Output nodes.
        Must be a subset of `g.nodes`.

    Returns
    -------
    If a gflow exists, return a `GFlowResult[V]` object.
    Otherwise, return `None`.
    """
    common.check_graph(g, iset, oset)
    v2i = {v: i for i, v in enumerate(g.nodes)}
    i2v = {i: v for v, i in v2i.items()}
    n = len(g)
    g_: list[set[int]] = [set() for _ in range(n)]
    for u, i in v2i.items():
        for v in g[u]:
            g_[i].add(v2i[v])
    iset_ = {v2i[v] for v in iset}
    oset_ = {v2i[v] for v in oset}
    ret_ = gflow.find(g_, iset_, oset_)
    if ret_ is None:
        return None
    f_, layer_ = ret_
    f: dict[V, set[V]] = {}
    for i, si in f_.items():
        si_ = {i2v[j] for j in si}
        f[i2v[i]] = si_
    layer = {i2v[i]: li for i, li in enumerate(layer_)}
    return GFlowResult[V](f, layer)
