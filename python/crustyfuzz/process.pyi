from collections.abc import Hashable, Sequence, Iterable, Mapping, Collection
from typing import Any, Callable, Generator, TypeVar, overload

from crustyfuzz.fuzz import WRatio

_StringType = Sequence[Hashable]
_S1 = TypeVar("_S1")
_S2 = TypeVar("_S2")
_ResultType = int | float

@overload
def extract_one(
    query: _S1,
    choices: Iterable[_S2],
    *,
    scorer: Callable[..., _ResultType] = WRatio,
    processor: Callable[..., _StringType] | None = None,
    score_cutoff: _ResultType | None = None,
    score_hint: _ResultType | None = None,
    scorer_kwargs: dict[str, Any] | None = None,
) -> tuple[_S2, _ResultType, int]: ...
@overload
def extract_one(
    query: _S1,
    choices: Mapping[Any, _S2],
    *,
    scorer: Callable[..., _ResultType] = WRatio,
    processor: Callable[..., _StringType] | None = None,
    score_cutoff: _ResultType | None = None,
    score_hint: _ResultType | None = None,
    scorer_kwargs: dict[str, Any] | None = None,
) -> tuple[_S2, _ResultType, Any]: ...
@overload
def extract(
    query: _S1,
    choices: Collection[_S2],
    *,
    scorer: Callable[..., _ResultType] = WRatio,
    processor: Callable[..., _StringType] | None = None,
    limit: int | None = 5,
    score_cutoff: _ResultType | None = None,
    score_hint: _ResultType | None = None,
    scorer_kwargs: dict[str, Any] | None = None,
) -> list[tuple[_S2, _ResultType, int]]: ...
@overload
def extract(
    query: _S1,
    choices: Mapping[Any, _S2],
    *,
    scorer: Callable[..., _ResultType] = WRatio,
    processor: Callable[..., _StringType] | None = None,
    limit: int | None = 5,
    score_cutoff: _ResultType | None = None,
    score_hint: _ResultType | None = None,
    scorer_kwargs: dict[str, Any] | None = None,
) -> list[tuple[_S2, _ResultType, Any]]: ...
@overload
def extract_iter(
    query: _S1,
    choices: Iterable[_S2],
    *,
    scorer: Callable[..., _ResultType] = WRatio,
    processor: Callable[..., _StringType] | None = None,
    score_cutoff: _ResultType | None = None,
    score_hint: _ResultType | None = None,
    scorer_kwargs: dict[str, Any] | None = None,
) -> Generator[tuple[_S2, _ResultType, int], None, None]: ...
@overload
def extract_iter(
    query: _S1,
    choices: Mapping[Any, _S2],
    *,
    scorer: Callable[..., _ResultType] = WRatio,
    processor: Callable[..., _StringType] | None = None,
    score_cutoff: _ResultType | None = None,
    score_hint: _ResultType | None = None,
    scorer_kwargs: dict[str, Any] | None = None,
) -> Generator[tuple[_S2, _ResultType, Any], None, None]: ...
