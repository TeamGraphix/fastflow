"""Test gflow."""

from __future__ import annotations

import networkx as nx
import pytest
from assets import CASES, FlowTestCase
from fastflow import gflow
from fastflow.common import Plane


@pytest.mark.parametrize("c", CASES)
def test_gflow_graphix(c: FlowTestCase) -> None:
    """Compare the results with the graphix package."""
    result = gflow.find(c.g, c.iset, c.oset, c.plane)
    assert result == c.gflow


def test_gflow_redundant() -> None:
    """Specify redundant planes."""
    g: nx.Graph[int] = nx.Graph([(0, 1)])
    iset = {0}
    oset = {1}
    planes = {0: Plane.XY, 1: Plane.XY}
    with pytest.warns(UserWarning, match=r".*Ignoring plane\[v\] where v in oset\..*"):
        gflow.find(g, iset, oset, planes)
