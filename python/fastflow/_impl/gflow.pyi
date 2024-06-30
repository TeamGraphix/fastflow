from fastflow.common import _Plane

def find(
    g: list[set[int]], iset: set[int], oset: set[int], plane: dict[int, _Plane]
) -> tuple[dict[int, set[int]], list[int]] | None: ...
