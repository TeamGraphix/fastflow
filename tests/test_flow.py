from __future__ import annotations

import pytest
from assets import CASES, FlowTestCase
from fastflow import flow


@pytest.mark.parametrize("c", CASES)
def test_flow_graphix(c: FlowTestCase) -> None:
    result = flow.find(c.g, c.iset, c.oset)
    assert result == c.flow
