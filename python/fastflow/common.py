"""Common functionalities."""

from __future__ import annotations

import dataclasses
from collections.abc import Hashable
from typing import Generic, TypeVar

from fastflow._impl import gflow, pflow

Plane = gflow.Plane

PPlane = pflow.PPlane

V = TypeVar("V", bound=Hashable)  #: Node type.


P = TypeVar("P", Plane, PPlane)  #: Measurement plane or Pauli index.


@dataclasses.dataclass(frozen=True)
class FlowResult(Generic[V]):
    r"""Causal flow of an open graph."""

    f: dict[V, V]
    """Flow map as a dictionary, i.e., :math:`f(u)` is stored in :py:obj:`f[u]`."""
    layer: dict[V, int]
    r"""Layer of each node representing the partial order, i.e., :math:`layer(u) > layer(v)` implies :math:`u \prec v`.
    """


@dataclasses.dataclass(frozen=True)
class GFlowResult(Generic[V]):
    r"""Generalized flow of an open graph."""

    f: dict[V, set[V]]
    """Generalized flow map as a dictionary, i.e., :math:`f(u)` is stored in :py:obj:`f[u]`."""
    layer: dict[V, int]
    r"""Layer of each node representing the partial order, i.e., :math:`layer(u) > layer(v)` implies :math:`u \prec v`.
    """
