"""Maximally-delayed gflow algorithm.

For given undirected graph, input nodes, and output nodes, compute the generalized flow having \
the minimum number of layers.
See Mhalla et al. (2008) for more details.
"""

from __future__ import annotations

import warnings
from typing import TYPE_CHECKING, Mapping

from fastflow import common
from fastflow._impl import gflow
from fastflow.common import GFlowResult, IndexMap, Plane, V, _Plane

if TYPE_CHECKING:
    from collections.abc import Set as AbstractSet

    import networkx as nx


def find(
    g: nx.Graph[V],
    iset: AbstractSet[V],
    oset: AbstractSet[V],
    plane: Mapping[V, Plane] | None = None,
) -> GFlowResult[V] | None:
    r"""Compute the maximally-delayed generalized flow, if any.

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
    plane : `Mapping[V, Plane]` | None, optional
        Measurement planes of each vertex in V\O.
        If `None`, defaults to all `Plane.XY`.

    Returns
    -------
    If a gflow exists, return a `GFlowResult[V]` object.
    Otherwise, return `None`.
    """
    common.check_graph(g, iset, oset)
    vset = g.nodes
    if plane is None:
        plane = dict.fromkeys(vset - oset, Plane.XY)
    common.check_planelike(vset, oset, plane)
    codec = IndexMap(vset)
    g_ = codec.encode_graph(g)
    iset_ = codec.encode_set(iset)
    oset_ = codec.encode_set(oset)
    plane_: dict[int, _Plane] = {codec.v2i[k]: v.value for k, v in plane.items() if k not in oset}
    if len(plane_) != len(plane):
        msg = "Ignoring plane[v] where v in oset."
        warnings.warn(msg, stacklevel=1)
    if ret_ := gflow.find(g_, iset_, oset_, plane_):
        f_, layer_ = ret_
        f = codec.decode_gflow(f_)
        layer = codec.decode_layer(layer_)
        return GFlowResult(f, layer)
    return None
