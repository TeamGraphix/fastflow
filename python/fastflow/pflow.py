"""Maximally-delayed Pauli flow algorithm."""

from __future__ import annotations

import warnings
from typing import TYPE_CHECKING, Mapping

from fastflow import common
from fastflow._impl import pflow
from fastflow.common import GFlowResult, IndexMap, PPlane, V

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
    g : `nx.Graph[V]`
        Undirected graph representing MBQC pattern.
        Cannot have self-loops.
    iset : `AbstractSet[V]`
        Input nodes.
        Must be a subset of `g.nodes`.
    oset : `AbstractSet[V]`
        Output nodes.
        Must be a subset of `g.nodes`.
    pplane : `Mapping[V, PPlane] | None`, optional
        Measurement planes or Pauli indices of each vertex in V\O.
        If `None`, defaults to all `PPlane.XY`.

    Returns
    -------
    If a Pauli flow exists, return it as `GFlowResult[V]` object.
    Otherwise, return `None`.

    Notes
    -----
    Do not pass `None` to `pplane`.
    For that case, use `gflow.find` instead.
    """
    common.check_graph(g, iset, oset)
    vset = g.nodes
    if pplane is None:
        pplane = dict.fromkeys(vset - oset, PPlane.XY)
    common.check_planelike(vset, oset, pplane)
    if all(pp not in {PPlane.X, PPlane.Y, PPlane.Z} for pp in pplane.values()):
        msg = "No Pauli measurement found. Use gflow.find instead."
        warnings.warn(msg, stacklevel=1)
    codec = IndexMap(vset)
    g_ = codec.encode_graph(g)
    iset_ = codec.encode_set(iset)
    oset_ = codec.encode_set(oset)
    pplane_ = codec.encode_dictkey(pplane)
    if len(pplane_) != len(pplane):
        msg = "Ignoring pplane[v] where v in oset."
        warnings.warn(msg, stacklevel=1)
    if ret_ := pflow.find(g_, iset_, oset_, pplane_):
        f_, layer_ = ret_
        f = codec.decode_gflow(f_)
        layer = codec.decode_layer(layer_)
        return GFlowResult(f, layer)
    return None
