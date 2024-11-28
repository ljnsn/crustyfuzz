"""Tests for the Longest Common Subsequence (LCS) distance metric."""

from __future__ import annotations

from crustyfuzz import utils
from crustyfuzz.distance import Editop
from tests.distance.common import LCSseq


def test_basic():
    assert LCSseq.distance("", "") == 0
    assert LCSseq.distance("test", "test") == 0
    assert LCSseq.distance("aaaa", "bbbb") == 4


def test_Editops():
    """
    basic test for LCSseq.editops
    """
    assert LCSseq.editops("0", "").as_list() == [Editop("delete", 0, 0)]
    assert LCSseq.editops("", "0").as_list() == [Editop("insert", 0, 0)]

    assert LCSseq.editops("00", "0").as_list() == [Editop("delete", 1, 1)]
    assert LCSseq.editops("0", "00").as_list() == [Editop("insert", 1, 1)]

    assert LCSseq.editops("qabxcd", "abycdf").as_list() == [
        Editop("delete", 0, 0),
        Editop("insert", 3, 2),
        Editop("delete", 3, 3),
        Editop("insert", 6, 5),
    ]
    assert LCSseq.editops("Lorem ipsum.", "XYZLorem ABC iPsum").as_list() == [
        Editop("insert", 0, 0),
        Editop("insert", 0, 1),
        Editop("insert", 0, 2),
        Editop("insert", 6, 9),
        Editop("insert", 6, 10),
        Editop("insert", 6, 11),
        Editop("insert", 6, 12),
        Editop("insert", 7, 14),
        Editop("delete", 7, 15),
        Editop("delete", 11, 18),
    ]

    ops = LCSseq.editops("aaabaaa", "abbaaabba")
    assert ops.src_len == 7
    assert ops.dest_len == 9


def testCaseInsensitive():
    assert (
        LCSseq.distance(
            "new york mets",
            "new YORK mets",
            processor=utils.default_process,
        )
        == 0
    )
