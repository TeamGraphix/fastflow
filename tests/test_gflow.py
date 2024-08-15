"""Test gflow."""

from __future__ import annotations

import networkx as nx
import pytest
from fastflow import gflow
from fastflow.common import Plane

from tests.assets import CASES, FlowTestCase


@pytest.mark.parametrize("c", CASES)
def test_gflow_graphix(c: FlowTestCase) -> None:
    """Compare the results with the graphix package."""
    result = gflow.find(c.g, c.iset, c.oset, c.plane)
    assert result == c.gflow


def test_gflow_redundant() -> None:
    """Specify redundant planes."""
    g = nx.Graph([(0, 1)])
    iset = {0}
    oset = {1}
    planes = {0: Plane.XY, 1: Plane.XY}
    with pytest.warns(UserWarning):
        gflow.find(g, iset, oset, planes)
