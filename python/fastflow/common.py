"""Common functionalities for the fastflow package."""

from __future__ import annotations

import dataclasses
from collections.abc import Hashable
from typing import Generic, TypeVar

from fastflow._impl import gflow, pflow

Plane = gflow.Plane
PPlane = pflow.PPlane

# Vertex type
V = TypeVar("V", bound=Hashable)


@dataclasses.dataclass(frozen=True)
class FlowResult(Generic[V]):
    """Causal flow [Danos and Kashefi, Phys. Rev. A 74, 052310] of an open graph."""

    f: dict[V, V]  #: Flow function.
    layer: dict[V, int]  #: Layer of each vertex representing the partial order.


@dataclasses.dataclass(frozen=True)
class GFlowResult(Generic[V]):
    """Generalized flow [Browne et al., NJP 9,  250 (2007)] of an open graph."""

    f: dict[V, set[V]]  #: Gflow function.
    layer: dict[V, int]  #: Layer of each vertex representing the partial order.
