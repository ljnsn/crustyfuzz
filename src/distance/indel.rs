use crate::common::utils::is_none;
use crate::common::{conv_sequences, Hashable};
use crate::distance::lcs_seq::{block_similarity, similarity};
use std::clone::Clone;
use std::collections::HashMap;

/**
Calculates the minimum number of insertions and deletions
required to change one sequence into the other. This is equivalent to the
Levenshtein distance with a substitution weight of 2.

Parameters
----------
s1 : Sequence[Hashable]
    First string to compare.
s2 : Sequence[Hashable]
    Second string to compare.
processor: callable, optional
    Optional callable that is used to preprocess the strings before
    comparing them. Default is None, which deactivates this behaviour.
score_cutoff : int, optional
    Maximum distance between s1 and s2, that is
    considered as a result. If the distance is bigger than score_cutoff,
    score_cutoff + 1 is returned instead. Default is None, which deactivates
    this behaviour.

Returns
-------
distance : int
    distance between s1 and s2

Examples
--------
Find the Indel distance between two strings:

>>> from rapidfuzz.distance import Indel
>>> Indel.distance("lewenstein", "levenshtein")
3

Setting a maximum distance allows the implementation to select
a more efficient implementation:

>>> Indel.distance("lewenstein", "levenshtein", score_cutoff=1)
2
*/
pub fn distance(s1: &Vec<u64>, s2: &Vec<u64>, score_cutoff: Option<f64>) -> f64 {
    let maximum = (s1.len() + s2.len()) as f64;
    let lcs_sim = similarity(s1, s2, None);
    let dist = maximum - 2.0 * lcs_sim;

    if score_cutoff.is_none() || dist <= score_cutoff.unwrap() {
        dist
    } else {
        score_cutoff.unwrap() + 1.0
    }
}

pub fn block_distance(
    block: &HashMap<u64, u64>,
    s1: &Vec<u64>,
    s2: &Vec<u64>,
    score_cutoff: Option<f64>,
) -> f64 {
    let maximum = (s1.len() + s2.len()) as f64;
    let lcs_sim = block_similarity(block, s1, s2, None);
    let dist = maximum - 2.0 * lcs_sim;

    if score_cutoff.is_none() || dist <= score_cutoff.unwrap() {
        dist
    } else {
        score_cutoff.unwrap() + 1.0
    }
}

/**
Calculates a normalized levenshtein similarity in the range [1, 0].

This is calculated as ``distance / (len1 + len2)``.

Parameters
----------
s1 : Sequence[Hashable]
    First string to compare.
s2 : Sequence[Hashable]
    Second string to compare.
processor: callable, optional
    Optional callable that is used to preprocess the strings before
    comparing them. Default is None, which deactivates this behaviour.
score_cutoff : float, optional
    Optional argument for a score threshold as a float between 0 and 1.0.
    For norm_dist > score_cutoff 1.0 is returned instead. Default is 1.0,
    which deactivates this behaviour.

Returns
-------
norm_dist : float
    normalized distance between s1 and s2 as a float between 0 and 1.0
*/
pub fn normalized_distance(s1: &Vec<u64>, s2: &Vec<u64>, score_cutoff: Option<f64>) -> f64 {
    let maximum = (s1.len() + s2.len()) as f64;
    let dist = distance(s1, s2, None);
    let norm_dist = if maximum == 0.0 { 0.0 } else { dist / maximum };

    if score_cutoff.is_none() || norm_dist <= score_cutoff.unwrap() {
        norm_dist
    } else {
        1.0
    }
}

pub fn block_normalized_distance(
    block: &HashMap<u64, u64>,
    s1: &Vec<u64>,
    s2: &Vec<u64>,
    score_cutoff: Option<f64>,
) -> f64 {
    let maximum = (s1.len() + s2.len()) as f64;
    let dist = block_distance(block, s1, s2, None);
    let norm_dist = if maximum == 0.0 { 0.0 } else { dist / maximum };

    if score_cutoff.is_none() || norm_dist <= score_cutoff.unwrap() {
        norm_dist
    } else {
        1.0
    }
}

/**
Calculates a normalized indel similarity in the range [0, 1].

This is calculated as ``1 - normalized_distance``

Parameters
----------
s1 : Sequence[Hashable]
    First string to compare.
s2 : Sequence[Hashable]
    Second string to compare.
processor: callable, optional
    Optional callable that is used to preprocess the strings before
    comparing them. Default is None, which deactivates this behaviour.
score_cutoff : float, optional
    Optional argument for a score threshold as a float between 0 and 1.0.
    For norm_sim < score_cutoff 0 is returned instead. Default is 0,
    which deactivates this behaviour.

Returns
-------
norm_sim : float
    normalized similarity between s1 and s2 as a float between 0 and 1.0

Examples
--------
Find the normalized Indel similarity between two strings:

>>> from rapidfuzz.distance import Indel
>>> Indel.normalized_similarity("lewenstein", "levenshtein")
0.85714285714285

Setting a score_cutoff allows the implementation to select
a more efficient implementation:

>>> Indel.normalized_similarity("lewenstein", "levenshtein", score_cutoff=0.9)
0.0

When a different processor is used s1 and s2 do not have to be strings

>>> Indel.normalized_similarity(["lewenstein"], ["levenshtein"], processor=lambda s: s[0])
0.8571428571428572
*/
pub fn normalized_similarity<T: Hashable + Clone>(
    s1: Option<&[T]>,
    s2: Option<&[T]>,
    processor: Option<fn(Vec<T>) -> Vec<T>>,
    score_cutoff: Option<f64>,
) -> f64 {
    if is_none(s1) || is_none(s2) {
        return 0.0;
    }

    let s1_mut = s1.unwrap().to_vec();
    let s2_mut = s2.unwrap().to_vec();

    let (processed_s1, processed_s2) = match processor {
        Some(proc) => (proc(s1_mut), proc(s2_mut)),
        None => (s1_mut, s2_mut),
    };

    let (s1_seq, s2_seq) = conv_sequences(&processed_s1, &processed_s2);
    let norm_dist = normalized_distance(&s1_seq, &s2_seq, score_cutoff);
    let norm_sim = 1.0 - norm_dist;

    if score_cutoff.is_none() || norm_sim >= score_cutoff.unwrap() {
        norm_sim
    } else {
        0.0
    }
}

pub fn block_normalized_similarity(
    block: &HashMap<u64, u64>,
    s1: &Vec<u64>,
    s2: &Vec<u64>,
    score_cutoff: Option<f64>,
) -> f64 {
    let norm_dist = block_normalized_distance(block, s1, s2, None);
    let norm_sim = 1.0 - norm_dist;

    if score_cutoff.is_none() || norm_sim >= score_cutoff.unwrap() {
        norm_sim
    } else {
        0.0
    }
}
