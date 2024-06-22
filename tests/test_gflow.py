"""Test gflow."""

import pytest
from fastflow import gflow

from tests.assets import CASES, FlowTestCase


@pytest.mark.parametrize("c", CASES)
def test_gflow_graphix(c: FlowTestCase) -> None:
    """Compare the results with the graphix package."""
    result = gflow.find(c.g, c.iset, c.oset)
    assert result == c.gflow
