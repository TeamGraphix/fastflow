"""Maximally-delayed flow algorithm.

For given undirected graph, input nodes, and output nodes, compute the causal flow having \
the minimum number of layers.
See Mhalla et al. (2008) for more details.
"""

from __future__ import annotations

from typing import TYPE_CHECKING

from fastflow import common
from fastflow._impl import flow
from fastflow.common import FlowResult, IndexMap, V

if TYPE_CHECKING:
    from collections.abc import Set as AbstractSet

    import networkx as nx


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
    vset = g.nodes
    codec = IndexMap(vset)
    g_ = codec.encode_graph(g)
    iset_ = codec.encode_set(iset)
    oset_ = codec.encode_set(oset)
    if ret_ := flow.find(g_, iset_, oset_):
        f_, layer_ = ret_
        f = codec.decode_flow(f_)
        layer = codec.decode_layer(layer_)
        return FlowResult(f, layer)
    return None
