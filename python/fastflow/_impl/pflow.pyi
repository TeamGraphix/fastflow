from fastflow.common import _PPlane

def find(
    g: list[set[int]], iset: set[int], oset: set[int], pplane: dict[int, _PPlane]
) -> tuple[dict[int, set[int]], list[int]] | None: ...
