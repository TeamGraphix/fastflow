"""Test flow."""

from __future__ import annotations

import pytest
from fastflow import flow

from tests.assets import CASES, FlowTestCase


@pytest.mark.parametrize("c", CASES)
def test_flow_graphix(c: FlowTestCase) -> None:
    """Compare the results with the graphix package."""
    result = flow.find(c.g, c.iset, c.oset)
    assert result == c.flow
