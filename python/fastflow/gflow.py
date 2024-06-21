"""Maximally-delayed gflow algorithm."""

from __future__ import annotations

from typing import TYPE_CHECKING

import networkx as nx

from fastflow._impl import gflow
from fastflow.common import GFlowResult, V

if TYPE_CHECKING:
    from collections.abc import Set as AbstractSet


def find(g: nx.Graph[V], iset: AbstractSet[V], oset: AbstractSet[V]) -> GFlowResult[V] | None:
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
