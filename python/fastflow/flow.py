"""Maximally-delayed flow algorithm.

For given undirected graph, input nodes, and output nodes, compute the causal flow having \
the minimum number of layers.
See Mhalla et al. (2008) for more details.
"""

from __future__ import annotations

from typing import TYPE_CHECKING

import pydantic
from pydantic import NonNegativeInt, ValidationError

from fastflow import common
from fastflow._impl import flow
from fastflow.common import FlowResult, V

if TYPE_CHECKING:
    from collections.abc import Set as AbstractSet

    import networkx as nx


@pydantic.validate_call(validate_return=True)
def _find_validated(
    g: list[set[NonNegativeInt]], iset: set[NonNegativeInt], oset: set[NonNegativeInt]
) -> tuple[dict[NonNegativeInt, NonNegativeInt], list[NonNegativeInt]] | None:
    return flow.find(g, iset, oset)


def find(g: nx.Graph[V], iset: AbstractSet[V], oset: AbstractSet[V]) -> FlowResult[V] | None:
    """Compute the maximally-delayed causal flow, if any.

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
    If a flow exists, return a `FlowResult[V]` object.
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
    try:
        ret_ = _find_validated(g_, iset_, oset_)
    except ValidationError as e:
        msg = "Failed to validate types at bindcall."
        raise ValueError(msg) from e
    if ret_ is None:
        return None
    f_, layer_ = ret_
    f = {i2v[i]: i2v[j] for i, j in f_.items()}
    layer = {i2v[i]: li for i, li in enumerate(layer_)}
    return FlowResult(f, layer)
