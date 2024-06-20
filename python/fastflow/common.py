"""Common data for fastflow package."""

from typing import NamedTuple


class FlowResult(NamedTuple):
    """MBQC flow."""

    f: dict[int, int]
    layer: list[int]


def check_graph(g: list[set[int]], iset: set[int], oset: set[int]) -> None:
    """Check the graph."""
    n = len(g)
    for gi in g:
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
