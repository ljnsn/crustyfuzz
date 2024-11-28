"""Tests for the Postfix distance metrics."""

from __future__ import annotations

from crustyfuzz import utils
from tests.distance.common import Postfix


def test_basic():
    assert Postfix.distance("", "") == 0
    assert Postfix.distance("test", "test") == 0
    assert Postfix.distance("aaaa", "bbbb") == 4


def test_score_cutoff():
    """
    test whether score_cutoff works correctly
    """
    assert Postfix.distance("abcd", "eebcd") == 2
    assert Postfix.distance("abcd", "eebcd", score_cutoff=4) == 2
    assert Postfix.distance("abcd", "eebcd", score_cutoff=3) == 2
    assert Postfix.distance("abcd", "eebcd", score_cutoff=2) == 2
    assert Postfix.distance("abcd", "eebcd", score_cutoff=1) == 2
    assert Postfix.distance("abcd", "eebcd", score_cutoff=0) == 1


def testCaseInsensitive():
    assert (
        Postfix.distance(
            "new york mets",
            "new YORK mets",
            processor=utils.default_process,
        )
        == 0
    )
