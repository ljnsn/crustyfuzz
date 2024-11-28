"""Tests for the Indel distance metric."""

from __future__ import annotations

from crustyfuzz import utils
from crustyfuzz.distance import Editop
from tests.distance.common import Indel


def test_basic():
    assert Indel.distance("", "") == 0
    assert Indel.distance("test", "test") == 0
    assert Indel.distance("aaaa", "bbbb") == 8


def test_issue_196():
    """
    Indel distance did not work correctly for score_cutoff=1
    """
    assert Indel.distance("South Korea", "North Korea") == 4
    assert Indel.distance("South Korea", "North Korea", score_cutoff=4) == 4
    assert Indel.distance("South Korea", "North Korea", score_cutoff=3) == 4
    assert Indel.distance("South Korea", "North Korea", score_cutoff=2) == 3
    assert Indel.distance("South Korea", "North Korea", score_cutoff=1) == 2
    assert Indel.distance("South Korea", "North Korea", score_cutoff=0) == 1


def test_Editops():
    """
    basic test for Indel.editops
    """
    assert Indel.editops("0", "").as_list() == [Editop("delete", 0, 0)]
    assert Indel.editops("", "0").as_list() == [Editop("insert", 0, 0)]

    assert Indel.editops("00", "0").as_list() == [Editop("delete", 1, 1)]
    assert Indel.editops("0", "00").as_list() == [Editop("insert", 1, 1)]

    assert Indel.editops("qabxcd", "abycdf").as_list() == [
        Editop("delete", 0, 0),
        Editop("insert", 3, 2),
        Editop("delete", 3, 3),
        Editop("insert", 6, 5),
    ]
    assert Indel.editops("Lorem ipsum.", "XYZLorem ABC iPsum").as_list() == [
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

    ops = Indel.editops("aaabaaa", "abbaaabba")
    assert ops.src_len == 7
    assert ops.dest_len == 9


def testCaseInsensitive():
    assert (
        Indel.distance(
            "new york mets",
            "new YORK mets",
            processor=utils.default_process,
        )
        == 0
    )
