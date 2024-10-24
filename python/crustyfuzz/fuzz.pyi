from collections.abc import Callable, Hashable, Sequence

def ratio(
    s1: Sequence[Hashable],
    s2: Sequence[Hashable],
    processor: Callable[..., Sequence[Hashable]] | None = None,
    score_cutoff: float | None = 0,
) -> float: ...
def partial_ratio(
    s1: Sequence[Hashable],
    s2: Sequence[Hashable],
    processor: Callable[..., Sequence[Hashable]] | None = None,
    score_cutoff: float | None = 0,
) -> float: ...
