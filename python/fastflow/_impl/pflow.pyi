class PPlane:
    XY: PPlane
    YZ: PPlane
    ZX: PPlane
    X: PPlane
    Y: PPlane
    Z: PPlane

def find(
    g: list[set[int]], iset: set[int], oset: set[int], pplane: dict[int, PPlane]
) -> tuple[dict[int, set[int]], list[int]] | None: ...
