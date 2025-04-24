from fastflow._impl.gflow import Plane
from fastflow._impl.pflow import PPlane

class FlowValidationMessage:
    class ExcessiveNonZeroLayer:
        node: int
        layer: int

    class ExcessiveZeroLayer:
        node: int

    class InvalidFlowCodomain:
        node: int

    class InvalidFlowDomain:
        node: int

    class InvalidMeasurementSpec:
        node: int

    class InconsistentFlowOrder:
        edge: tuple[int, int]

    class InconsistentFlowPlane:
        node: int
        plane: Plane

    class InconsistentFlowPPlane:
        node: int
        pplane: PPlane
