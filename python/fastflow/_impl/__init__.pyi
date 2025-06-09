import numpy as np
import numpy.typing as npt
from fastflow._impl.gflow import Plane
from fastflow._impl.pflow import PPlane

class FlowValidationMessage:
    class ExcessiveNonZeroLayer:
        node: int
        layer: int

        def __init__(self, node: int, layer: int) -> None: ...

    class ExcessiveZeroLayer:
        node: int

        def __init__(self, node: int) -> None: ...

    class InvalidFlowCodomain:
        node: int

        def __init__(self, node: int) -> None: ...

    class InvalidFlowDomain:
        node: int

        def __init__(self, node: int) -> None: ...

    class InvalidMeasurementSpec:
        node: int

        def __init__(self, node: int) -> None: ...

    class InconsistentFlowOrder:
        nodes: tuple[int, int]

        def __init__(self, edge: tuple[int, int]) -> None: ...

    class InconsistentFlowPlane:
        node: int
        plane: Plane

        def __init__(self, node: int, plane: Plane) -> None: ...

    class InconsistentFlowPPlane:
        node: int
        pplane: PPlane

        def __init__(self, node: int, pplane: PPlane) -> None: ...

def solve(a: npt.NDArray[np.bool_], b: npt.NDArray[np.bool_]) -> list[npt.NDArray[np.bool_] | None]: ...
