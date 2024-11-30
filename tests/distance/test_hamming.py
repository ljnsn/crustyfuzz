"""Tests for the Hamming distance metric."""

from __future__ import annotations

import pytest

try:
    from rapidfuzz.distance import metrics_cpp as metrics_rf  # type: ignore[attr-defined]
except ImportError:
    metrics_rf = None

from crustyfuzz import utils
from crustyfuzz.distance import Editop, metrics
from tests.distance.common import Hamming


def hamming_editops(s1, s2):
    ops1 = metrics.hamming_editops(s1, s2)

    assert ops1.src_len == len(s1)
    assert ops1.dest_len == len(s2)

    if metrics_rf is not None:
        ops2 = metrics_rf.hamming_editops(s1, s2)
        assert ops2.src_len == len(s1)
        assert ops2.dest_len == len(s2)
        # assert ops1 == ops2
        assert ops1.as_list() == [Editop(*op) for op in ops2.as_list()]
        assert str(ops1) == str(ops2)
        assert repr(ops1) == repr(ops2)

        for op1, op2 in zip(ops1, ops2):
            assert tuple(op1) == tuple(op2)
            assert str(op1) == str(op2)
            assert repr(op1) == repr(op2)

    return ops1


def test_basic():
    assert Hamming.distance("", "") == 0
    assert Hamming.distance("test", "test") == 0
    assert Hamming.distance("aaaa", "bbbb") == 4
    assert Hamming.distance("aaaa", "aaaaa") == 1


def test_disable_padding():
    assert Hamming.distance("", "", pad=False) == 0
    assert Hamming.distance("test", "test", pad=False) == 0
    assert Hamming.distance("aaaa", "bbbb", pad=False) == 4

    with pytest.raises(ValueError, match="Sequences are not the same length."):
        Hamming.distance("aaaa", "aaaaa", catch_exceptions=True, pad=False)

    # todo
    with pytest.raises(ValueError, match="Sequences are not the same length."):
        metrics.hamming_editops("aaaa", "aaaaa", pad=False)


def test_score_cutoff():
    """
    test whether score_cutoff works correctly
    """
    assert Hamming.distance("South Korea", "North Korea") == 2
    assert Hamming.distance("South Korea", "North Korea", score_cutoff=4) == 2
    assert Hamming.distance("South Korea", "North Korea", score_cutoff=3) == 2
    assert Hamming.distance("South Korea", "North Korea", score_cutoff=2) == 2
    assert Hamming.distance("South Korea", "North Korea", score_cutoff=1) == 2
    assert Hamming.distance("South Korea", "North Korea", score_cutoff=0) == 1


def test_Editops():
    """
    basic test for Hamming.editops
    """
    assert hamming_editops("0", "").as_list() == [Editop("delete", 0, 0)]
    assert hamming_editops("", "0").as_list() == [Editop("insert", 0, 0)]

    assert hamming_editops("00", "0").as_list() == [Editop("delete", 1, 1)]
    assert hamming_editops("0", "00").as_list() == [Editop("insert", 1, 1)]

    assert hamming_editops("qabxcd", "abycdf").as_list() == [
        Editop("replace", 0, 0),
        Editop("replace", 1, 1),
        Editop("replace", 2, 2),
        Editop("replace", 3, 3),
        Editop("replace", 4, 4),
        Editop("replace", 5, 5),
    ]

    ops = hamming_editops("aaabaaa", "abbaaabba")
    assert ops.src_len == 7
    assert ops.dest_len == 9


def test_case_insensitive():
    assert (
        Hamming.distance(
            "new york mets",
            "new YORK mets",
            processor=utils.default_process,
        )
        == 0
    )
    assert (
        Hamming.distance(
            "new york mets",
            "new YORK mets",
            processor=utils.default_process,
        )
        == 0
    )
