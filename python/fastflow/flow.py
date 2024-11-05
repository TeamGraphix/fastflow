"""Maximally-delayed flow algorithm.

This module provides functions to compute and verify maximally-delayed causal flow.
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
    """Compute causal flow.

    If it returns a flow, it is guaranteed to be maximally-delayed, i.e., the number of layers is minimized.

    Parameters
    ----------
    g : `networkx.Graph`
        Simple graph representing MBQC pattern.
    iset : `collections.abc.Set`
        Input nodes.
    oset : `collections.abc.Set`
        Output nodes.

    Returns
    -------
    `FlowResult` or `None`
        Return the flow if any, otherwise `None`.
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
    flow : `FlowResult`
        Flow to verify.
    g : `networkx.Graph`
        Simple graph representing MBQC pattern.
    iset : `collections.abc.Set`
        Input nodes.
    oset : `collections.abc.Set`
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
    try:
        flow_bind.verify((f_, layer_), g_, iset_, oset_)
    except ValueError as e:
        raise codec.decode_err(e) from None
