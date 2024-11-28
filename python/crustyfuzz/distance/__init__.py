"""Distance scorers."""

__all__ = (
    "Editop",
    "Editops",
    "MatchingBlock",
    "Opcode",
    "Opcodes",
    "ScoreAlignment",
)

import sys

from ._initialize import (
    Editop,
    Editops,
    MatchingBlock,
    Opcode,
    Opcodes,
    ScoreAlignment,
    # scorers
    damerau_levenshtein,
    hamming,
    indel,
    jaro,
    jaro_winkler,
    lcs_seq,
    levenshtein,
    osa,
    postfix,
    prefix,
)

sys.modules["crustyfuzz.distance.hamming"] = hamming
sys.modules["crustyfuzz.distance.lcs_seq"] = lcs_seq
sys.modules["crustyfuzz.distance.indel"] = indel
sys.modules["crustyfuzz.distance.levenshtein"] = levenshtein
sys.modules["crustyfuzz.distance.damerau_levenshtein"] = damerau_levenshtein
sys.modules["crustyfuzz.distance.jaro"] = jaro
sys.modules["crustyfuzz.distance.jaro_winkler"] = jaro_winkler
sys.modules["crustyfuzz.distance.osa"] = osa
sys.modules["crustyfuzz.distance.postfix"] = postfix
sys.modules["crustyfuzz.distance.prefix"] = prefix
