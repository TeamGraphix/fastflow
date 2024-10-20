class Plane:
    XY: Plane
    YZ: Plane
    XZ: Plane

def find(
    g: list[set[int]], iset: set[int], oset: set[int], plane: dict[int, Plane]
) -> tuple[dict[int, set[int]], list[int]] | None: ...
def verify(
    gflow: tuple[dict[int, set[int]], list[int]],
    g: list[set[int]],
    iset: set[int],
    oset: set[int],
    plane: dict[int, Plane],
) -> None: ...
