"""Maximally-delayed flow algorithm."""

from __future__ import annotations

from fastflow import common
from fastflow._impl import flow
from fastflow.common import FlowResult


def find(g: list[set[int]], iset: set[int], oset: set[int]) -> FlowResult | None:
    common.check_graph(g, iset, oset)
    ret = flow.find(g, iset, oset)
    if ret is None:
        return None
    return FlowResult(*ret)
