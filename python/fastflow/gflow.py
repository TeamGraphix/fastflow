"""Maximally-delayed gflow algorithm.

This module provides functions to compute and verify maximally-delayed generalized flow.
"""

from __future__ import annotations

import warnings
from typing import TYPE_CHECKING, Mapping

from fastflow import _common
from fastflow._common import IndexMap
from fastflow._impl import gflow as gflow_bind
from fastflow.common import GFlowResult, Plane, V

if TYPE_CHECKING:
    from collections.abc import Set as AbstractSet

    import networkx as nx


def find(
    g: nx.Graph[V],
    iset: AbstractSet[V],
    oset: AbstractSet[V],
    plane: Mapping[V, Plane] | None = None,
) -> GFlowResult[V] | None:
    r"""Compute maximally-delayed generalized flow.

    Parameters
    ----------
    g
        Simple graph representing MBQC pattern.
    iset
        Input nodes.
    oset
        Output nodes.
    plane
        Measurement plane for each node in :math:`V \setminus O`.
        Defaults to `Plane.XY`.

    Returns
    -------
        Return the gflow if any, otherwise `None`. If found, it is guaranteed to be maximally delayed.
    """
    _common.check_graph(g, iset, oset)
    vset = g.nodes
    if plane is None:
        plane = dict.fromkeys(vset - oset, Plane.XY)
    _common.check_planelike(vset, oset, plane)
    ignore = plane.keys() & oset
    if len(ignore) != 0:
        msg = "Ignoring plane[v] where v in oset."
        warnings.warn(msg, stacklevel=1)
        plane = {k: v for k, v in plane.items() if k not in ignore}
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
    gflow
        Generalized flow to verify.
    g
        Simple graph representing MBQC pattern.
    iset
        Input nodes.
    oset
        Output nodes.
    plane
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
    gflow_bind.verify((f_, layer_), g_, iset_, oset_, plane_)
