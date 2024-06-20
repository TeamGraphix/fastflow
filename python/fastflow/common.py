"""Common data for fastflow package."""

import warnings
from collections.abc import Collection
from collections.abc import Set as AbstractSet
from typing import NamedTuple


class FlowResult(NamedTuple):
    """MBQC flow."""

    f: dict[int, int]
    layer: list[int]


def check_graph(g: Collection[AbstractSet[int]], iset: AbstractSet[int], oset: AbstractSet[int]) -> None:
    """Check the graph."""
    n = len(g)
    for i, gi in enumerate(g):
        if i in gi:
            msg = "Self-loop detected."
            raise ValueError(msg)
        if all(0 <= gij < n for gij in gi):
            continue
        msg = "Neighboring vertices out of range."
        raise ValueError(msg)
    vset = set(range(n))
    if not iset <= vset:
        msg = "iset must be a subset of the vertices."
        raise ValueError(msg)
    if not oset <= vset:
        msg = "oset must be a subset of the vertices."
        raise ValueError(msg)
    if 0 not in vset:
        msg = "Vertices are assumed to be 0-based."
        warnings.warn(msg, stacklevel=2)
