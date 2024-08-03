"""Maximally-delayed Pauli flow algorithm."""

from __future__ import annotations

import warnings
from typing import TYPE_CHECKING, Mapping

import pydantic
from pydantic import NonNegativeInt, ValidationError

from fastflow import common
from fastflow._impl import pflow
from fastflow.common import GFlowResult, PauliPlane, V, _PPlane

if TYPE_CHECKING:
    from collections.abc import Set as AbstractSet

    import networkx as nx


@pydantic.validate_call(validate_return=True)
def _find_validated(
    g: list[set[NonNegativeInt]],
    iset: set[NonNegativeInt],
    oset: set[NonNegativeInt],
    pplane: dict[NonNegativeInt, _PPlane],
) -> tuple[dict[NonNegativeInt, set[NonNegativeInt]], list[NonNegativeInt]] | None:
    return pflow.find(g, iset, oset, pplane)


def find(
    g: nx.Graph[V], iset: AbstractSet[V], oset: AbstractSet[V], pplane: Mapping[V, PauliPlane] | None = None
) -> GFlowResult[V] | None:
    r"""Compute the maximally-delayed Pauli flow, if any."""
    common.check_graph(g, iset, oset)
    v2i = {v: i for i, v in enumerate(g.nodes)}
    i2v = {i: v for v, i in v2i.items()}
    if pplane is None:
        pplane = dict.fromkeys(v2i.keys() - oset, PauliPlane.XY)
        warnings.warn("pflow.find is inefficient. Use gflow.find instead.", stacklevel=1)
    if pplane.keys() > g.nodes:
        msg = "Keys of pplane must be in g.nodes."
        raise ValueError(msg)
    if pplane.keys() < g.nodes - oset:
        msg = "pplanes should be specified for all u in V\\O."
        raise ValueError(msg)
    n = len(g)
    g_: list[set[int]] = [set() for _ in range(n)]
    for u, i in v2i.items():
        for v in g[u]:
            g_[i].add(v2i[v])
    iset_ = {v2i[v] for v in iset}
    oset_ = {v2i[v] for v in oset}
    pplane_: dict[int, _PPlane] = {v2i[k]: v.value for k, v in pplane.items() if k not in oset}
    if len(pplane_) != len(pplane):
        warnings.warn("Ignoring pplane[v] where v in oset.", stacklevel=1)
    try:
        ret_ = _find_validated(g_, iset_, oset_, pplane_)
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
