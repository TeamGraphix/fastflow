"""Maximally-delayed Pauli flow algorithm.

This module provides functions to compute and verify maximally-delayed Pauli flow.
See :footcite:t:`Simons2021` for details.
"""

from __future__ import annotations

import warnings
from typing import TYPE_CHECKING

from fastflow import _common
from fastflow._common import IndexMap
from fastflow._impl import pflow as pflow_bind
from fastflow.common import GFlowResult, PPlane, V

if TYPE_CHECKING:
    from collections.abc import Mapping
    from collections.abc import Set as AbstractSet

    import networkx as nx


def find(
    g: nx.Graph[V],
    iset: AbstractSet[V],
    oset: AbstractSet[V],
    pplane: Mapping[V, PPlane] | None = None,
) -> GFlowResult[V] | None:
    r"""Compute Pauli flow.

    If it returns a Pauli flow, it is guaranteed to be maximally-delayed, i.e., the number of layers is minimized.

    Parameters
    ----------
    g : `networkx.Graph`
        Simple graph representing MBQC pattern.
    iset : `collections.abc.Set`
        Input nodes.
    oset : `collections.abc.Set`
        Output nodes.
    pplane : `collections.abc.Mapping`
        Measurement plane or Pauli index for each node in :math:`V \setminus O`.
        Defaults to `PPlane.XY`.

    Returns
    -------
    `GFlowResult` or `None`
        Return the Pauli flow if any, otherwise `None`.

    Notes
    -----
    Use `gflow.find` whenever possible for better performance.
    """
    _common.check_graph(g, iset, oset)
    vset = g.nodes
    if pplane is None:
        pplane = dict.fromkeys(vset - oset, PPlane.XY)
    _common.check_planelike(vset, oset, pplane)
    if all(pp not in {PPlane.X, PPlane.Y, PPlane.Z} for pp in pplane.values()):
        msg = "No Pauli measurement found. Use gflow.find instead."
        warnings.warn(msg, stacklevel=1)
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
    pflow: GFlowResult[V],
    g: nx.Graph[V],
    iset: AbstractSet[V],
    oset: AbstractSet[V],
    pplane: Mapping[V, PPlane] | None = None,
) -> None:
    r"""Verify maximally-delayed Pauli flow.

    Parameters
    ----------
    pflow : `GFlowResult`
        Pauli flow to verify.
    g : `networkx.Graph`
        Simple graph representing MBQC pattern.
    iset : `collections.abc.Set`
        Input nodes.
    oset : `collections.abc.Set`
        Output nodes.
    pplane : `collections.abc.Mapping`
        Measurement plane or Pauli index for each node in :math:`V \setminus O`.
        Defaults to `PPlane.XY`.

    Raises
    ------
    ValueError
        If the graph is invalid or verification fails.
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
    f_ = codec.encode_gflow(pflow.f)
    layer_ = codec.encode_layer(pflow.layer)
    try:
        pflow_bind.verify((f_, layer_), g_, iset_, oset_, pplane_)
    except ValueError as e:
        raise codec.decode_err(e) from None
