from __future__ import annotations

# DamerauLevenshtein
from crustyfuzz.distance.damerau_levenshtein import (
    distance as damerau_levenshtein_distance,
)
from crustyfuzz.distance.damerau_levenshtein import (
    normalized_distance as damerau_levenshtein_normalized_distance,
)
from crustyfuzz.distance.damerau_levenshtein import (
    normalized_similarity as damerau_levenshtein_normalized_similarity,
)
from crustyfuzz.distance.damerau_levenshtein import (
    similarity as damerau_levenshtein_similarity,
)

# Hamming
from crustyfuzz.distance.hamming import distance as hamming_distance
from crustyfuzz.distance.hamming import editops as hamming_editops
from crustyfuzz.distance.hamming import (
    normalized_distance as hamming_normalized_distance,
)
from crustyfuzz.distance.hamming import (
    normalized_similarity as hamming_normalized_similarity,
)
from crustyfuzz.distance.hamming import opcodes as hamming_opcodes
from crustyfuzz.distance.hamming import similarity as hamming_similarity

# Indel
from crustyfuzz.distance.indel import distance as indel_distance
from crustyfuzz.distance.indel import editops as indel_editops
from crustyfuzz.distance.indel import (
    normalized_distance as indel_normalized_distance,
)
from crustyfuzz.distance.indel import (
    normalized_similarity as indel_normalized_similarity,
)
from crustyfuzz.distance.indel import opcodes as indel_opcodes
from crustyfuzz.distance.indel import similarity as indel_similarity

# Jaro
from crustyfuzz.distance.jaro import distance as jaro_distance
from crustyfuzz.distance.jaro import normalized_distance as jaro_normalized_distance
from crustyfuzz.distance.jaro import (
    normalized_similarity as jaro_normalized_similarity,
)
from crustyfuzz.distance.jaro import similarity as jaro_similarity

# JaroWinkler
from crustyfuzz.distance.jaro_winkler import distance as jaro_winkler_distance
from crustyfuzz.distance.jaro_winkler import (
    normalized_distance as jaro_winkler_normalized_distance,
)
from crustyfuzz.distance.jaro_winkler import (
    normalized_similarity as jaro_winkler_normalized_similarity,
)
from crustyfuzz.distance.jaro_winkler import similarity as jaro_winkler_similarity

# LCSseq
from crustyfuzz.distance.lcs_seq import distance as lcs_seq_distance
from crustyfuzz.distance.lcs_seq import editops as lcs_seq_editops
from crustyfuzz.distance.lcs_seq import (
    normalized_distance as lcs_seq_normalized_distance,
)
from crustyfuzz.distance.lcs_seq import (
    normalized_similarity as lcs_seq_normalized_similarity,
)
from crustyfuzz.distance.lcs_seq import opcodes as lcs_seq_opcodes
from crustyfuzz.distance.lcs_seq import similarity as lcs_seq_similarity

# Levenshtein
from crustyfuzz.distance.levenshtein import distance as levenshtein_distance
from crustyfuzz.distance.levenshtein import editops as levenshtein_editops
from crustyfuzz.distance.levenshtein import (
    normalized_distance as levenshtein_normalized_distance,
)
from crustyfuzz.distance.levenshtein import (
    normalized_similarity as levenshtein_normalized_similarity,
)
from crustyfuzz.distance.levenshtein import opcodes as levenshtein_opcodes
from crustyfuzz.distance.levenshtein import similarity as levenshtein_similarity

# OSA
from crustyfuzz.distance.osa import distance as osa_distance
from crustyfuzz.distance.osa import normalized_distance as osa_normalized_distance
from crustyfuzz.distance.osa import (
    normalized_similarity as osa_normalized_similarity,
)
from crustyfuzz.distance.osa import similarity as osa_similarity

# Postfix
from crustyfuzz.distance.postfix import distance as postfix_distance
from crustyfuzz.distance.postfix import (
    normalized_distance as postfix_normalized_distance,
)
from crustyfuzz.distance.postfix import (
    normalized_similarity as postfix_normalized_similarity,
)
from crustyfuzz.distance.postfix import similarity as postfix_similarity

# Prefix
from crustyfuzz.distance.prefix import distance as prefix_distance
from crustyfuzz.distance.prefix import (
    normalized_distance as prefix_normalized_distance,
)
from crustyfuzz.distance.prefix import (
    normalized_similarity as prefix_normalized_similarity,
)
from crustyfuzz.distance.prefix import similarity as prefix_similarity

__all__ = (
    "damerau_levenshtein_distance",
    "damerau_levenshtein_normalized_distance",
    "damerau_levenshtein_normalized_similarity",
    "damerau_levenshtein_similarity",
    "hamming_distance",
    "hamming_editops",
    "hamming_normalized_distance",
    "hamming_normalized_similarity",
    "hamming_opcodes",
    "hamming_similarity",
    "indel_distance",
    "indel_editops",
    "indel_normalized_distance",
    "indel_normalized_similarity",
    "indel_opcodes",
    "indel_similarity",
    "jaro_distance",
    "jaro_normalized_distance",
    "jaro_normalized_similarity",
    "jaro_similarity",
    "jaro_winkler_distance",
    "jaro_winkler_normalized_distance",
    "jaro_winkler_normalized_similarity",
    "jaro_winkler_similarity",
    "lcs_seq_distance",
    "lcs_seq_editops",
    "lcs_seq_normalized_distance",
    "lcs_seq_normalized_similarity",
    "lcs_seq_opcodes",
    "lcs_seq_similarity",
    "levenshtein_distance",
    "levenshtein_editops",
    "levenshtein_normalized_distance",
    "levenshtein_normalized_similarity",
    "levenshtein_opcodes",
    "levenshtein_similarity",
    "osa_distance",
    "osa_normalized_distance",
    "osa_normalized_similarity",
    "osa_similarity",
    "postfix_distance",
    "postfix_normalized_distance",
    "postfix_normalized_similarity",
    "postfix_similarity",
    "prefix_distance",
    "prefix_normalized_distance",
    "prefix_normalized_similarity",
    "prefix_similarity",
)
