from __future__ import annotations

import networkx as nx
import pytest
from fastflow import _common
from fastflow._common import IndexMap
from fastflow._impl import FlowValidationMessage
from fastflow.common import Plane, PPlane
from typing_extensions import Never


def test_check_graph_ng_g() -> None:
    with pytest.raises(TypeError):
        _common.check_graph("hoge", set(), set())  # type: ignore[arg-type]

    with pytest.raises(ValueError, match=r"Graph is empty\."):
        _common.check_graph(nx.Graph(), set(), set())

    with pytest.raises(ValueError, match=r"Self-loop detected\."):
        _common.check_graph(nx.Graph([("a", "a"), ("a", "b")]), set(), set())

    with pytest.raises(ValueError, match=r"iset must be a subset of the nodes\."):
        _common.check_graph(nx.Graph([("a", "b")]), {"x"}, set())

    with pytest.raises(ValueError, match=r"oset must be a subset of the nodes\."):
        _common.check_graph(nx.Graph([("a", "b")]), set(), {"x"})


def test_check_graph_ng_set() -> None:
    with pytest.raises(TypeError):
        _common.check_graph(nx.Graph(), "hoge", set())  # type: ignore[arg-type]

    with pytest.raises(TypeError):
        _common.check_graph(nx.Graph(), set(), "hoge")  # type: ignore[arg-type]


def test_check_planelike_ng() -> None:
    with pytest.raises(TypeError):
        _common.check_planelike(set(), set(), "hoge")  # type: ignore[arg-type]

    with pytest.raises(ValueError, match=r"Cannot find corresponding nodes in the graph\."):
        _common.check_planelike({"a"}, set(), {"x": Plane.XY})

    with pytest.raises(ValueError, match=r"Measurement planes should be specified for all u in V\\O."):
        _common.check_planelike({"a", "b"}, {"b"}, {})


@pytest.fixture
def fx_indexmap() -> IndexMap[str]:
    return IndexMap({"a", "b", "c"})


class TestIndexMap:
    def test_encode(self, fx_indexmap: IndexMap[str]) -> None:
        # Order guaranteed
        assert fx_indexmap.encode("a") == 0
        assert fx_indexmap.encode("b") == 1
        assert fx_indexmap.encode("c") == 2

        with pytest.raises(ValueError, match=r"x not found\."):
            fx_indexmap.encode("x")

    def test_decode(self, fx_indexmap: IndexMap[str]) -> None:
        assert fx_indexmap.decode(0) == "a"
        assert fx_indexmap.decode(1) == "b"
        assert fx_indexmap.decode(2) == "c"

        with pytest.raises(ValueError, match=r"3 not found\."):
            fx_indexmap.decode(3)

    def test_encdec(self) -> None:
        # Hash by id
        a = object()
        b = object()
        c = object()
        codec = IndexMap({a, b, c})
        # Not deterministic but consistent
        assert codec.decode(codec.encode(a)) == a
        assert codec.decode(codec.encode(b)) == b
        assert codec.decode(codec.encode(c)) == c

    @pytest.mark.parametrize(
        "emsg",
        [
            FlowValidationMessage.ExcessiveNonZeroLayer(0, 1),
            FlowValidationMessage.ExcessiveZeroLayer(0),
            FlowValidationMessage.InvalidFlowCodomain(0),
            FlowValidationMessage.InvalidFlowDomain(0),
            FlowValidationMessage.InvalidMeasurementSpec(0),
            FlowValidationMessage.InconsistentFlowOrder((0, 1)),
            FlowValidationMessage.InconsistentFlowPlane(0, Plane.XY),
            FlowValidationMessage.InconsistentFlowPPlane(0, PPlane.XY),
        ],
    )
    def test_decode_err(self, fx_indexmap: IndexMap[str], emsg: ValueError) -> None:
        e_ = fx_indexmap.decode_err(ValueError(emsg))
        assert isinstance(e_, ValueError)

    def test_encode_layer_missing(self, fx_indexmap: IndexMap[str]) -> None:
        with pytest.raises(ValueError, match=r"Layers must be specified for all nodes\."):
            fx_indexmap.encode_layer({"a": 0, "b": 1})

    def test_ecatch(self, fx_indexmap: IndexMap[str]) -> None:
        def dummy_ok(x: int) -> int:
            return x

        def dummy_ng(_: int) -> Never:
            raise ValueError(FlowValidationMessage.ExcessiveZeroLayer(0))

        assert fx_indexmap.ecatch(dummy_ok, 1) == 1
        with pytest.raises(ValueError, match=r"Zero-layer node a outside output nodes\."):
            fx_indexmap.ecatch(dummy_ng, 1)
