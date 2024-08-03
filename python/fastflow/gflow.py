"""Maximally-delayed gflow algorithm.

For given undirected graph, input nodes, and output nodes, compute the generalized flow having \
the minimum number of layers.
See Mhalla et al. (2008) for more details.
"""

from __future__ import annotations

import warnings
from typing import TYPE_CHECKING, Mapping

import pydantic
from pydantic import NonNegativeInt, ValidationError

from fastflow import common
from fastflow._impl import gflow
from fastflow.common import GFlowResult, Plane, V, _Plane

if TYPE_CHECKING:
    from collections.abc import Set as AbstractSet

    import networkx as nx


@pydantic.validate_call(validate_return=True)
def _find_validated(
    g: list[set[NonNegativeInt]],
    iset: set[NonNegativeInt],
    oset: set[NonNegativeInt],
    plane: dict[NonNegativeInt, _Plane],
) -> tuple[dict[NonNegativeInt, set[NonNegativeInt]], list[NonNegativeInt]] | None:
    return gflow.find(g, iset, oset, plane)


def find(
    g: nx.Graph[V], iset: AbstractSet[V], oset: AbstractSet[V], plane: Mapping[V, Plane] | None = None
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
    v2i = {v: i for i, v in enumerate(g.nodes)}
    i2v = {i: v for v, i in v2i.items()}
    if plane is None:
        plane = dict.fromkeys(v2i.keys() - oset, Plane.XY)
    if plane.keys() > g.nodes:
        msg = "Keys of plane must be in g.nodes."
        raise ValueError(msg)
    if plane.keys() < g.nodes - oset:
        msg = "Planes should be specified for all u in V\\O."
        raise ValueError(msg)
    n = len(g)
    g_: list[set[int]] = [set() for _ in range(n)]
    for u, i in v2i.items():
        for v in g[u]:
            g_[i].add(v2i[v])
    iset_ = {v2i[v] for v in iset}
    oset_ = {v2i[v] for v in oset}
    plane_: dict[int, _Plane] = {v2i[k]: v.value for k, v in plane.items() if k not in oset}
    if len(plane_) != len(plane):
        warnings.warn("Ignoring plane[v] where v in oset.", stacklevel=1)
    try:
        ret_ = _find_validated(g_, iset_, oset_, plane_)
    except ValidationError as e:
        msg = "Failed to validate types at bindcall."
        raise ValueError(msg) from e
    if ret_ is None:
        return None
    f_, layer_ = ret_
    f: dict[V, set[V]] = {}
    for i, si in f_.items():
        f[i2v[i]] = {i2v[j] for j in si}
    layer = {i2v[i]: li for i, li in enumerate(layer_)}
    return GFlowResult(f, layer)
