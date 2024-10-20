"""Maximally-delayed Pauli flow algorithm."""

from __future__ import annotations

import warnings
from typing import TYPE_CHECKING, Mapping

from fastflow import _common
from fastflow._common import IndexMap, V
from fastflow._impl import pflow as pflow_bind
from fastflow.common import GFlowResult, PPlane

if TYPE_CHECKING:
    from collections.abc import Set as AbstractSet

    import networkx as nx


def find(
    g: nx.Graph[V],
    iset: AbstractSet[V],
    oset: AbstractSet[V],
    pplane: Mapping[V, PPlane] | None = None,
) -> GFlowResult[V] | None:
    r"""Compute the maximally-delayed Pauli flow, if any.

    Parameters
    ----------
    g
        Undirected graph representing MBQC pattern.
        Cannot have self-loops.
    iset
        Input nodes.
        Must be a subset of `g.nodes`.
    oset
        Output nodes.
        Must be a subset of `g.nodes`.
    pplane
        Measurement planes or Pauli indices of each vertex in V\O.
        If `None`, defaults to all `PPlane.XY`.

    Returns
    -------
    :
        If a Pauli flow exists, return it as `GFlowResult[V]` object. Otherwise, return `None`.

    Notes
    -----
    Do not pass `None` to `pplane`.
    For that case, use `gflow.find` instead.
    """
    _common.check_graph(g, iset, oset)
    vset = g.nodes
    if pplane is None:
        pplane = dict.fromkeys(vset - oset, PPlane.XY)
    _common.check_planelike(vset, oset, pplane)
    if all(pp not in {PPlane.X, PPlane.Y, PPlane.Z} for pp in pplane.values()):
        msg = "No Pauli measurement found. Use gflow.find instead."
        warnings.warn(msg, stacklevel=1)
    ignore = pplane.keys() & oset
    if len(ignore) != 0:
        msg = "Ignoring pplane[v] where v in oset."
        warnings.warn(msg, stacklevel=1)
        pplane = {k: v for k, v in pplane.items() if k not in ignore}
    codec = IndexMap(vset)
    g_ = codec.encode_graph(g)
    iset_ = codec.encode_set(iset)
    oset_ = codec.encode_set(oset)
    pplane_ = codec.encode_dictkey(pplane)
    if ret_ := pflow_bind.find(g_, iset_, oset_, pplane_):
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
    pplane: Mapping[V, PPlane] | None = None,
) -> None:
    r"""Verify Pauli flow.

    Parameters
    ----------
    gflow
        Pauli flow to verify.
    g
        Undirected graph representing MBQC pattern.
    iset
        Input nodes.
    oset
        Output nodes.
    pplane
        Measurement planes or Pauli indices of each vertex in V\O.
        If `None`, defaults to all `PPlane.XY`.

    Raises
    ------
    ValueError
        If verification fails.
    """
    _common.check_graph(g, iset, oset)
    vset = g.nodes
    if pplane is None:
        pplane = dict.fromkeys(vset - oset, PPlane.XY)
    codec = IndexMap(vset)
    g_ = codec.encode_graph(g)
    iset_ = codec.encode_set(iset)
    oset_ = codec.encode_set(oset)
    pplane_ = codec.encode_dictkey(pplane)
    f_ = codec.encode_gflow(gflow.f)
    layer_ = codec.encode_layer(gflow.layer)
    pflow_bind.verify((f_, layer_), g_, iset_, oset_, pplane_)
