from __future__ import annotations

import networkx as nx
import pytest
from fastflow import _common
from fastflow._common import IndexMap
from fastflow.common import Plane


def test_check_graph_ng_g() -> None:
    with pytest.raises(TypeError):
        _common.check_graph("hoge", set(), set())  # type: ignore[arg-type]

    with pytest.raises(ValueError, match="Graph is empty."):
        _common.check_graph(nx.Graph(), set(), set())

    with pytest.raises(ValueError, match="Self-loop detected."):
        _common.check_graph(nx.Graph([("a", "a"), ("a", "b")]), set(), set())

    with pytest.raises(ValueError, match="iset must be a subset of the nodes."):
        _common.check_graph(nx.Graph([("a", "b")]), {"x"}, set())

    with pytest.raises(ValueError, match="oset must be a subset of the nodes."):
        _common.check_graph(nx.Graph([("a", "b")]), set(), {"x"})


def test_check_graph_ng_set() -> None:
    with pytest.raises(TypeError):
        _common.check_graph(nx.Graph(), "hoge", set())  # type: ignore[arg-type]

    with pytest.raises(TypeError):
        _common.check_graph(nx.Graph(), set(), "hoge")  # type: ignore[arg-type]


def test_check_planelike_ng() -> None:
    with pytest.raises(TypeError):
        _common.check_planelike(set(), set(), "hoge")  # type: ignore[arg-type]

    with pytest.raises(ValueError, match="Cannot find corresponding nodes in the graph."):
        _common.check_planelike({"a"}, set(), {"x": Plane.XY})

    with pytest.raises(ValueError, match=r"Measurement planes should be specified for all u in V\\O."):
        _common.check_planelike({"a", "b"}, {"b"}, {})


@pytest.fixture
def fx_indexmap() -> IndexMap[str]:
    return IndexMap({"a", "b", "c"})


class TestIndexMap:
    @staticmethod
    def test_encode(fx_indexmap: IndexMap[str]) -> None:
        assert {
            fx_indexmap.encode("a"),
            fx_indexmap.encode("b"),
            fx_indexmap.encode("c"),
        } == {0, 1, 2}

        with pytest.raises(ValueError, match="x not found."):
            fx_indexmap.encode("x")

    @staticmethod
    def test_decode(fx_indexmap: IndexMap[str]) -> None:
        assert {
            fx_indexmap.decode(0),
            fx_indexmap.decode(1),
            fx_indexmap.decode(2),
        } == {"a", "b", "c"}

        with pytest.raises(ValueError, match="3 not found."):
            fx_indexmap.decode(3)

    @staticmethod
    def test_encdec(fx_indexmap: IndexMap[str]) -> None:
        assert fx_indexmap.decode(fx_indexmap.encode("a")) == "a"
        assert fx_indexmap.decode(fx_indexmap.encode("b")) == "b"
        assert fx_indexmap.decode(fx_indexmap.encode("c")) == "c"
