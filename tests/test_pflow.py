"""Test Pauli flow."""

import pytest
from fastflow import pflow

from tests.assets import CASES, FlowTestCase


@pytest.mark.filterwarnings("ignore:No Pauli measurement found")
@pytest.mark.parametrize("c", CASES)
def test_pflow_graphix(c: FlowTestCase) -> None:
    """Compare the results with the graphix package."""
    result = pflow.find(c.g, c.iset, c.oset, c.pplane)
    assert result == c.pflow
