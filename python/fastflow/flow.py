"""Maximally-delayed flow algorithm.

For given undirected graph, input nodes, and output nodes, compute the causal flow having \
the minimum number of layers.
See [Mhalla and Perdrix, Proc. of 35th ICALP, 857 (2008)] for more details.
"""

from __future__ import annotations

from typing import TYPE_CHECKING

from fastflow import _common
from fastflow._common import IndexMap
from fastflow._impl import flow as flow_bind
from fastflow.common import FlowResult, V

if TYPE_CHECKING:
    from collections.abc import Set as AbstractSet

    import networkx as nx


def find(g: nx.Graph[V], iset: AbstractSet[V], oset: AbstractSet[V]) -> FlowResult[V] | None:
    """Compute maximally-delayed causal flow.

    Parameters
    ----------
    g
        Simple graph representing MBQC pattern.
    iset
        Input nodes.
    oset
        Output nodes.

    Returns
    -------
    :
        Return the flow if any, otherwise :py:obj:`None`.
    """
    _common.check_graph(g, iset, oset)
    vset = g.nodes
    codec = IndexMap(vset)
    g_ = codec.encode_graph(g)
    iset_ = codec.encode_set(iset)
    oset_ = codec.encode_set(oset)
    if ret_ := flow_bind.find(g_, iset_, oset_):
        f_, layer_ = ret_
        f = codec.decode_flow(f_)
        layer = codec.decode_layer(layer_)
        return FlowResult(f, layer)
    return None


def verify(flow: FlowResult[V], g: nx.Graph[V], iset: AbstractSet[V], oset: AbstractSet[V]) -> None:
    """Verify maximally-delayed causal flow.

    Parameters
    ----------
    flow
        Flow to verify.
    g
        Simple graph representing MBQC pattern.
    iset
        Input nodes.
    oset
        Output nodes.

    Raises
    ------
    ValueError
        If the graph is invalid or verification fails.
    """
    _common.check_graph(g, iset, oset)
    vset = g.nodes
    codec = IndexMap(vset)
    g_ = codec.encode_graph(g)
    iset_ = codec.encode_set(iset)
    oset_ = codec.encode_set(oset)
    f_ = codec.encode_flow(flow.f)
    layer_ = codec.encode_layer(flow.layer)
    flow_bind.verify((f_, layer_), g_, iset_, oset_)
