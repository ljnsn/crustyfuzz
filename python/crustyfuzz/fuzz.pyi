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
def token_sort_ratio(
    s1: Sequence[Hashable],
    s2: Sequence[Hashable],
    processor: Callable[..., Sequence[Hashable]] | None = None,
    score_cutoff: float | None = 0,
) -> float: ...
def token_set_ratio(
    s1: Sequence[Hashable],
    s2: Sequence[Hashable],
    processor: Callable[..., Sequence[Hashable]] | None = None,
    score_cutoff: float | None = 0,
) -> float: ...
def token_ratio(
    s1: Sequence[Hashable],
    s2: Sequence[Hashable],
    processor: Callable[..., Sequence[Hashable]] | None = None,
    score_cutoff: float | None = 0,
) -> float: ...
def partial_token_sort_ratio(
    s1: Sequence[Hashable],
    s2: Sequence[Hashable],
    processor: Callable[..., Sequence[Hashable]] | None = None,
    score_cutoff: float | None = 0,
) -> float: ...
def partial_token_set_ratio(
    s1: Sequence[Hashable],
    s2: Sequence[Hashable],
    processor: Callable[..., Sequence[Hashable]] | None = None,
    score_cutoff: float | None = 0,
) -> float: ...
def partial_token_ratio(
    s1: Sequence[Hashable],
    s2: Sequence[Hashable],
    processor: Callable[..., Sequence[Hashable]] | None = None,
    score_cutoff: float | None = 0,
) -> float: ...
def weighted_ratio(
    s1: Sequence[Hashable],
    s2: Sequence[Hashable],
    processor: Callable[..., Sequence[Hashable]] | None = None,
    score_cutoff: float | None = 0,
) -> float: ...
def quick_ratio(
    s1: Sequence[Hashable],
    s2: Sequence[Hashable],
    processor: Callable[..., Sequence[Hashable]] | None = None,
    score_cutoff: float | None = 0,
) -> float: ...
def WRatio(
    s1: Sequence[Hashable],
    s2: Sequence[Hashable],
    processor: Callable[..., Sequence[Hashable]] | None = None,
    score_cutoff: float | None = 0,
) -> float: ...
def QRatio(
    s1: Sequence[Hashable],
    s2: Sequence[Hashable],
    processor: Callable[..., Sequence[Hashable]] | None = None,
    score_cutoff: float | None = 0,
) -> float: ...
