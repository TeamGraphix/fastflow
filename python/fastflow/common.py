"""Common functionalities."""

from __future__ import annotations

import dataclasses
from collections.abc import Hashable
from typing import Generic, TypeVar

from fastflow._impl import gflow, pflow

Plane = gflow.Plane
"""Enum-like class for measurement planes."""

PPlane = pflow.PPlane
"""Enum-like class for measurement planes or Pauli indices."""

V = TypeVar("V", bound=Hashable)  #: Node type.


P = TypeVar("P", Plane, PPlane)  #: Measurement plane or Pauli index.


@dataclasses.dataclass(frozen=True)
class FlowResult(Generic[V]):
    """Causal flow of an open graph."""

    f: dict[V, V]  #: Flow function.
    layer: dict[V, int]  #: Layer of each node representing the partial order.


@dataclasses.dataclass(frozen=True)
class GFlowResult(Generic[V]):
    """Generalized flow of an open graph."""

    f: dict[V, set[V]]  #: Gflow function.
    layer: dict[V, int]  #: Layer of each node representing the partial order.
