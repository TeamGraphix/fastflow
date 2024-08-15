"""Test Pauli flow."""

from __future__ import annotations

import networkx as nx
import pytest
from fastflow import pflow
from fastflow.common import PPlane

from tests.assets import CASES, FlowTestCase


@pytest.mark.filterwarnings("ignore:No Pauli measurement found")
@pytest.mark.parametrize("c", CASES)
def test_pflow_graphix(c: FlowTestCase) -> None:
    """Compare the results with the graphix package."""
    result = pflow.find(c.g, c.iset, c.oset, c.pplane)
    assert result == c.pflow


def test_pflow_redundant() -> None:
    """Specify redundant pplanes."""
    g = nx.Graph([(0, 1)])
    iset = {0}
    oset = {1}
    planes = {0: PPlane.X, 1: PPlane.Y}
    with pytest.warns(UserWarning):
        pflow.find(g, iset, oset, planes)
