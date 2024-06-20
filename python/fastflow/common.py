"""Common data for fastflow package."""

from __future__ import annotations

from collections.abc import Hashable
from typing import Generic, NamedTuple, TypeVar

# Vertex type
V = TypeVar("V", bound=Hashable)


class FlowResult(NamedTuple, Generic[V]):
    """MBQC flow."""

    f: dict[V, V]
    layer: dict[V, int]
