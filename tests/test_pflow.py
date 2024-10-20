from __future__ import annotations

import networkx as nx
import pytest
from assets import CASES, FlowTestCase
from fastflow import pflow
from fastflow.common import PPlane


@pytest.mark.filterwarnings("ignore:No Pauli measurement found")
@pytest.mark.parametrize("c", CASES)
def test_pflow_graphix(c: FlowTestCase) -> None:
    result = pflow.find(c.g, c.iset, c.oset, c.pplane)
    assert result == c.pflow


def test_pflow_nopauli() -> None:
    g: nx.Graph[int] = nx.Graph([(0, 1)])
    iset = {0}
    oset = {1}
    planes = {0: PPlane.XY}
    with pytest.warns(UserWarning, match=r".*No Pauli measurement found\. Use gflow\.find instead\..*"):
        pflow.find(g, iset, oset, planes)


def test_pflow_redundant() -> None:
    g: nx.Graph[int] = nx.Graph([(0, 1)])
    iset = {0}
    oset = {1}
    planes = {0: PPlane.X, 1: PPlane.Y}
    with pytest.warns(UserWarning, match=r".*Ignoring pplane\[v\] where v in oset\..*"):
        pflow.find(g, iset, oset, planes)
