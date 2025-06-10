"""Maximally-delayed gflow algorithm.

This module provides functions to compute and verify maximally-delayed generalized flow.
See :footcite:t:`Mhalla2008` and :footcite:t:`Backens2021` for details.
"""

from __future__ import annotations

from typing import TYPE_CHECKING

from fastflow import _common
from fastflow._common import IndexMap
from fastflow._impl import gflow as gflow_bind
from fastflow.common import GFlowResult, Plane, V

if TYPE_CHECKING:
    from collections.abc import Mapping
    from collections.abc import Set as AbstractSet

    import networkx as nx


def find(
    g: nx.Graph[V],
    iset: AbstractSet[V],
    oset: AbstractSet[V],
    plane: Mapping[V, Plane] | None = None,
) -> GFlowResult[V] | None:
    r"""Compute generalized flow.

    If it returns a gflow, it is guaranteed to be maximally-delayed, i.e., the number of layers is minimized.

    Parameters
    ----------
    g : `networkx.Graph`
        Simple graph representing MBQC pattern.
    iset : `collections.abc.Set`
        Input nodes.
    oset : `collections.abc.Set`
        Output nodes.
    plane : `collections.abc.Mapping`
        Measurement plane for each node in :math:`V \setminus O`.
        Defaults to `Plane.XY`.

    Returns
    -------
    `GFlowResult` or `None`
        Return the gflow if any, otherwise `None`.
    """
    _common.check_graph(g, iset, oset)
    vset = g.nodes
    if plane is None:
        plane = dict.fromkeys(vset - oset, Plane.XY)
    _common.check_planelike(vset, oset, plane)
    codec = IndexMap(vset)
    g_ = codec.encode_graph(g)
    iset_ = codec.encode_set(iset)
    oset_ = codec.encode_set(oset)
    plane_ = codec.encode_dictkey(plane)
    if ret_ := gflow_bind.find(g_, iset_, oset_, plane_):
        f_, layer_ = ret_
        f = codec.decode_gflow(f_)
        layer = codec.decode_layer(layer_)
        return GFlowResult(f, layer)
    return None


def verify(
    gflow: GFlowResult[V],
    g: nx.Graph[V],
    iset: AbstractSet[V],
    oset: AbstractSet[V],
    plane: Mapping[V, Plane] | None = None,
) -> None:
    r"""Verify maximally-delayed generalized flow.

    Parameters
    ----------
    gflow : `GFlowResult`
        Generalized flow to verify.
    g : `networkx.Graph`
        Simple graph representing MBQC pattern.
    iset : `collections.abc.Set`
        Input nodes.
    oset : `collections.abc.Set`
        Output nodes.
    plane : `collections.abc.Mapping`
        Measurement plane for each node in :math:`V \setminus O`.
        Defaults to `Plane.XY`.

    Raises
    ------
    ValueError
        If the graph is invalid or verification fails.
    """
    _common.check_graph(g, iset, oset)
    vset = g.nodes
    if plane is None:
        plane = dict.fromkeys(vset - oset, Plane.XY)
    codec = IndexMap(vset)
    g_ = codec.encode_graph(g)
    iset_ = codec.encode_set(iset)
    oset_ = codec.encode_set(oset)
    plane_ = codec.encode_dictkey(plane)
    f_ = codec.encode_gflow(gflow.f)
    layer_ = codec.encode_layer(gflow.layer)
    codec.ecatch(gflow_bind.verify, (f_, layer_), g_, iset_, oset_, plane_)
