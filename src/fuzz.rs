use crate::common::conv_sequences;
use crate::common::models::{Token, TokenIterator, TokenSequence};
use crate::distance::indel::{
    block_normalized_similarity as indel_block_normalized_similarity, distance as indel_distance,
    normalized_similarity as indel_normalized_similarity,
    py_normalized_similarity as indel_py_normalized_similarity,
};
use crate::distance::models::ScoreAlignment;
use pyo3::prelude::*;
use std::collections::{HashMap, HashSet};

fn norm_distance(dist: usize, lensum: usize, score_cutoff: f64) -> f64 {
    let score = if lensum != 0 {
        100.0 - 100.0 * dist as f64 / lensum as f64
    } else {
        100.0
    };
    if score < score_cutoff {
        0.0
    } else {
        score
    }
}

fn split_into_tokens(seq: &[u32]) -> TokenIterator {
    TokenIterator::new(seq)
}

fn sort_tokens(seq: &[u32]) -> Vec<u32> {
    if seq.is_empty() {
        return Vec::new();
    }

    let mut tokens: Vec<Token> = split_into_tokens(seq).collect();
    tokens.sort_by_key(|t| Vec::from_iter(t.chars.iter().cloned()));

    TokenSequence::new(tokens).join()
}

/**
Calculates the normalized Indel distance.

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
    Optional argument for a score threshold as a float between 0 and 100.
    For ratio < score_cutoff 0 is returned instead. Default is 0,
    which deactivates this behaviour.

Returns
-------
similarity : f64
    similarity between s1 and s2 as a float between 0 and 100

Examples
--------
>>> fuzz::ratio(Some("this is a test"), Some("this is a test!"), None, None)
96.55171966552734
*/
#[pyfunction]
#[pyo3(name = "ratio", signature = (s1, s2, *, processor=None, score_cutoff=None))]
pub fn py_ratio(
    s1: &Bound<'_, PyAny>,
    s2: &Bound<'_, PyAny>,
    processor: Option<&Bound<'_, PyAny>>,
    score_cutoff: Option<f64>,
) -> PyResult<f64> {
    if s1.is_none() || s2.is_none() {
        return Ok(0.0);
    }

    let score_cutoff = score_cutoff.map(|c| c / 100.0);

    let score = indel_py_normalized_similarity(s1, s2, processor, score_cutoff)?;
    Ok(score * 100.0)
}

/**
Searches for the optimal alignment of the shorter string in the
longer string and returns the fuzz.ratio for this alignment.

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
    Optional argument for a score threshold as a float between 0 and 100.
    For ratio < score_cutoff 0 is returned instead. Default is 0,
    which deactivates this behaviour.

Returns
-------
similarity : float
    similarity between s1 and s2 as a float between 0 and 100

Notes
-----
Depending on the length of the needle (shorter string) different
implementations are used to improve the performance.

short needle (length ≤ 64):
    When using a short needle length the fuzz.ratio is calculated for all
    alignments that could result in an optimal alignment. It is
    guaranteed to find the optimal alignment. For short needles this is very
    fast, since for them fuzz.ratio runs in ``O(N)`` time. This results in a worst
    case performance of ``O(NM)``.

long needle (length > 64):
    For long needles a similar implementation to FuzzyWuzzy is used.
    This implementation only considers alignments which start at one
    of the longest common substrings. This results in a worst case performance
    of ``O(N[N/64]M)``. However usually most of the alignments can be skipped.
    The following Python code shows the concept:

    .. code-block:: python

        blocks = SequenceMatcher(None, needle, longer, False).get_matching_blocks()
        score = 0
        for block in blocks:
            long_start = block[1] - block[0] if (block[1] - block[0]) > 0 else 0
            long_end = long_start + len(shorter)
            long_substr = longer[long_start:long_end]
            score = max(score, fuzz.ratio(needle, long_substr))

    This is a lot faster than checking all possible alignments. However it
    only finds one of the best alignments and not necessarily the optimal one.

Examples
--------
>>> fuzz.partial_ratio("this is a test", "this is a test!")
100.0
*/
#[pyfunction]
#[pyo3(
    name = "partial_ratio",
    signature = (s1, s2, processor=None, score_cutoff=None)
)]
pub fn py_partial_ratio(
    s1: &Bound<'_, PyAny>,
    s2: &Bound<'_, PyAny>,
    processor: Option<&Bound<'_, PyAny>>,
    score_cutoff: Option<f64>,
) -> PyResult<f64> {
    let alignment = py_partial_ratio_alignment(s1, s2, processor, score_cutoff)?;

    match alignment {
        Some(alignment) => Ok(alignment.score),
        None => Ok(0.0),
    }
}

pub fn partial_ratio(s1: &[u32], s2: &[u32], score_cutoff: f64) -> f64 {
    let alignment = partial_ratio_alignment(s1, s2, score_cutoff);
    match alignment {
        Some(alignment) => alignment.score,
        None => 0.0,
    }
}

/**
implementation of partial_ratio for needles <= 64. assumes s1 is already the
shorter string
*/
fn partial_ratio_short_needle(s1: &[u32], s2: &[u32], mut score_cutoff: f64) -> ScoreAlignment {
    if s1.is_empty() {
        return ScoreAlignment {
            score: 0.0,
            src_start: 0,
            src_end: 0,
            dest_start: 0,
            dest_end: 0,
        };
    }

    let len1 = s1.len();
    let len2 = s2.len();
    let mut s1_char_set = HashSet::with_capacity(len1);
    s1_char_set.extend(s1.iter().cloned());

    let mut res = ScoreAlignment {
        score: 0.0,
        src_start: 0,
        src_end: len1,
        dest_start: 0,
        dest_end: len1,
    };

    let shift = 128 - len1;
    let mut block = HashMap::with_capacity(len1);
    let mut x = 1u128 << shift;
    for &ch1 in s1 {
        block.entry(ch1).and_modify(|e| *e |= &x).or_insert(x);
        x <<= 1;
    }

    for i in 1..len1 {
        let substr_last = s2[i - 1];
        if !s1_char_set.contains(&substr_last) {
            continue;
        }

        let ls_ratio = indel_block_normalized_similarity(&block, s1, &s2[..i], Some(score_cutoff));
        if ls_ratio > res.score {
            score_cutoff = ls_ratio;
            res.score = ls_ratio;
            res.dest_start = 0;
            res.dest_end = i;
            if res.score == 1.0 {
                res.score = 100.0;
                return res;
            }
        }
    }

    let window_end = len2 - len1;
    for i in 0..window_end {
        let substr_last = s2[i + len1 - 1];
        if !s1_char_set.contains(&substr_last) {
            continue;
        }

        let ls_ratio =
            indel_block_normalized_similarity(&block, s1, &s2[i..i + len1], Some(score_cutoff));
        if ls_ratio > res.score {
            score_cutoff = ls_ratio;
            res.score = ls_ratio;
            res.dest_start = i;
            res.dest_end = i + len1;
            if res.score == 1.0 {
                res.score = 100.0;
                return res;
            }
        }
    }

    for i in window_end..len2 {
        let substr_first = s2[i];
        if !s1_char_set.contains(&substr_first) {
            continue;
        }

        let ls_ratio = indel_block_normalized_similarity(&block, s1, &s2[i..], Some(score_cutoff));
        if ls_ratio > res.score {
            score_cutoff = ls_ratio;
            res.score = ls_ratio;
            res.dest_start = i;
            res.dest_end = len2;
            if res.score == 1.0 {
                res.score = 100.0;
                return res;
            }
        }
    }

    res.score *= 100.0;
    res
}

/**
Searches for the optimal alignment of the shorter string in the
longer string and returns the fuzz.ratio and the corresponding
alignment.

Parameters
----------
s1 : str | bytes
    First string to compare.
s2 : str | bytes
    Second string to compare.
processor: callable, optional
    Optional callable that is used to preprocess the strings before
    comparing them. Default is None, which deactivates this behaviour.
score_cutoff : float, optional
    Optional argument for a score threshold as a float between 0 and 100.
    For ratio < score_cutoff None is returned instead. Default is 0,
    which deactivates this behaviour.

Returns
-------
alignment : ScoreAlignment, optional
    alignment between s1 and s2 with the score as a float between 0 and 100

Examples
--------
>>> s1 = "a certain string"
>>> s2 = "cetain"
>>> res = fuzz.partial_ratio_alignment(s1, s2)
>>> res
ScoreAlignment(score=83.33333333333334, src_start=2, src_end=8, dest_start=0, dest_end=6)

Using the alignment information it is possible to calculate the same fuzz.ratio

>>> fuzz.ratio(s1[res.src_start:res.src_end], s2[res.dest_start:res.dest_end])
83.33333333333334
*/
#[pyfunction]
#[pyo3(
    name = "partial_ratio_alignment",
    signature = (s1, s2, processor=None, score_cutoff=None)
)]
pub fn py_partial_ratio_alignment(
    s1: &Bound<'_, PyAny>,
    s2: &Bound<'_, PyAny>,
    processor: Option<&Bound<'_, PyAny>>,
    score_cutoff: Option<f64>,
) -> PyResult<Option<ScoreAlignment>> {
    if s1.is_none() || s2.is_none() {
        return Ok(None);
    }

    let (s1, s2) = match processor {
        Some(proc) => (proc.call1((s1,))?, proc.call1((s2,))?),
        None => (s1.to_owned(), s2.to_owned()),
    };

    let score_cutoff = score_cutoff.unwrap_or(0.0);

    if s1.is_empty()? && s2.is_empty()? {
        return Ok(Some(ScoreAlignment {
            score: 100.0,
            src_start: 0,
            src_end: 0,
            dest_start: 0,
            dest_end: 0,
        }));
    }

    let (s1, s2) = match conv_sequences(&s1, &s2) {
        Ok((Some(s1), Some(s2))) => (s1, s2),
        Ok((_, _)) => {
            return Err(pyo3::exceptions::PyValueError::new_err(
                "Cannot handle None",
            ))
        }
        Err(e) => {
            return Err(pyo3::exceptions::PyTypeError::new_err(format!(
                "Failed to convert sequences to u32. {}",
                e
            )))
        }
    };

    Ok(partial_ratio_alignment(&s1, &s2, score_cutoff))
}

fn partial_ratio_alignment(s1: &[u32], s2: &[u32], score_cutoff: f64) -> Option<ScoreAlignment> {
    let mut score_cutoff = score_cutoff;
    let (len1, len2) = (s1.len(), s2.len());

    let (shorter, longer) = if len1 <= len2 { (&s1, &s2) } else { (&s2, &s1) };

    let mut res = partial_ratio_short_needle(shorter, longer, score_cutoff / 100.0);
    if (res.score != 100.0) && (len1 == len2) {
        score_cutoff = f64::max(score_cutoff, res.score);
        let res2 = partial_ratio_short_needle(longer, shorter, score_cutoff / 100.0);
        if res2.score > res.score {
            res = ScoreAlignment {
                score: res2.score,
                src_start: res2.dest_start,
                src_end: res2.dest_end,
                dest_start: res2.src_start,
                dest_end: res2.src_end,
            };
        }
    }

    if res.score < score_cutoff {
        return None;
    }

    if len1 <= len2 {
        return Some(res);
    }

    Some(ScoreAlignment {
        score: res.score,
        src_start: res.dest_start,
        src_end: res.dest_end,
        dest_start: res.src_start,
        dest_end: res.src_end,
    })
}

/**
Sort the words in the strings and calculate the fuzz.ratio between them.

Parameters
----------
s1 : str
    First string to compare.
s2 : str
    Second string to compare.
processor: callable, optional
    Optional callable that is used to preprocess the strings before
    comparing them. Default is None, which deactivates this behaviour.
score_cutoff : float, optional
    Optional argument for a score threshold as a float between 0 and 100.
    For ratio < score_cutoff 0 is returned instead. Default is 0,
    which deactivates this behaviour.

Returns
-------
similarity : float
    similarity between s1 and s2 as a float between 0 and 100

Examples
--------
>>> fuzz.token_sort_ratio("fuzzy wuzzy was a bear", "wuzzy fuzzy was a bear")
100.0
*/
#[pyfunction]
#[pyo3(
    name = "token_sort_ratio",
    signature = (s1, s2, *, processor=None, score_cutoff=None)
)]
pub fn py_token_sort_ratio(
    s1: &Bound<'_, PyAny>,
    s2: &Bound<'_, PyAny>,
    processor: Option<&Bound<'_, PyAny>>,
    score_cutoff: Option<f64>,
) -> PyResult<f64> {
    if s1.is_none() || s2.is_none() {
        return Ok(0.0);
    }

    let (s1, s2) = match processor {
        Some(proc) => (proc.call1((s1,))?, proc.call1((s2,))?),
        None => (s1.to_owned(), s2.to_owned()),
    };

    let (s1, s2) = match conv_sequences(&s1, &s2) {
        Ok((Some(s1), Some(s2))) => (s1, s2),
        Ok((_, _)) => {
            return Err(pyo3::exceptions::PyValueError::new_err(
                "Cannot handle None",
            ))
        }
        Err(e) => {
            return Err(pyo3::exceptions::PyTypeError::new_err(format!(
                "Failed to convert sequences to u32. {}",
                e
            )))
        }
    };

    Ok(token_sort_ratio(&s1, &s2, score_cutoff))
}

pub fn token_sort_ratio(s1: &[u32], s2: &[u32], score_cutoff: Option<f64>) -> f64 {
    let sorted_s1 = sort_tokens(s1);
    let sorted_s2 = sort_tokens(s2);
    let score_cutoff = score_cutoff.map(|c| c / 100.0);

    // equivalent to `ratio`
    let score = indel_normalized_similarity(&sorted_s1, &sorted_s2, score_cutoff);
    score * 100.0
}

/**
Compares the words in the strings based on unique and common words between them
using fuzz.ratio

Parameters
----------
s1 : str
    First string to compare.
s2 : str
    Second string to compare.
processor: callable, optional
    Optional callable that is used to preprocess the strings before
    comparing them. Default is None, which deactivates this behaviour.
score_cutoff : float, optional
    Optional argument for a score threshold as a float between 0 and 100.
    For ratio < score_cutoff 0 is returned instead. Default is 0,
    which deactivates this behaviour.

Returns
-------
similarity : float
    similarity between s1 and s2 as a float between 0 and 100

Notes
-----
.. image:: img/token_set_ratio.svg

Examples
--------
>>> fuzz.token_sort_ratio("fuzzy was a bear", "fuzzy fuzzy was a bear")
83.8709716796875
>>> fuzz.token_set_ratio("fuzzy was a bear", "fuzzy fuzzy was a bear")
100.0
*/
#[pyfunction]
#[pyo3(
    name = "token_set_ratio",
    signature = (s1, s2, *, processor=None, score_cutoff=None)
)]
pub fn py_token_set_ratio(
    s1: &Bound<'_, PyAny>,
    s2: &Bound<'_, PyAny>,
    processor: Option<&Bound<'_, PyAny>>,
    score_cutoff: Option<f64>,
) -> PyResult<f64> {
    if s1.is_none() || s2.is_none() {
        return Ok(0.0);
    }

    let (s1, s2) = match processor {
        Some(proc) => (proc.call1((s1,))?, proc.call1((s2,))?),
        None => (s1.to_owned(), s2.to_owned()),
    };

    let score_cutoff = score_cutoff.unwrap_or(0.0);

    let (s1, s2) = match conv_sequences(&s1, &s2) {
        Ok((Some(s1), Some(s2))) => (s1, s2),
        Ok((_, _)) => {
            return Err(pyo3::exceptions::PyValueError::new_err(
                "Cannot handle None",
            ))
        }
        Err(e) => {
            return Err(pyo3::exceptions::PyTypeError::new_err(format!(
                "Failed to convert sequences to u32. {}",
                e
            )))
        }
    };

    Ok(token_set_ratio(&s1, &s2, score_cutoff))
}

fn token_set_ratio(s1: &[u32], s2: &[u32], score_cutoff: f64) -> f64 {
    let tokens_a: HashSet<Vec<u32>> = split_into_tokens(s1).map(|t| t.chars.to_vec()).collect();
    let tokens_b: HashSet<Vec<u32>> = split_into_tokens(s2).map(|t| t.chars.to_vec()).collect();

    if tokens_a.is_empty() || tokens_b.is_empty() {
        return 0.0;
    }

    // Get intersection and differences
    let intersection: Vec<Vec<u32>> = tokens_a.intersection(&tokens_b).cloned().collect();
    let diff_ab: Vec<Vec<u32>> = tokens_a.difference(&tokens_b).cloned().collect();
    let diff_ba: Vec<Vec<u32>> = tokens_b.difference(&tokens_a).cloned().collect();

    // If intersection exists and one string is subset of other, return 100
    if !intersection.is_empty() && (diff_ab.is_empty() || diff_ba.is_empty()) {
        return 100.0;
    }

    // Create token sequences and join them
    let intersection_tokens = intersection.iter().map(|chars| Token { chars }).collect();
    let diff_ab_tokens = diff_ab.iter().map(|chars| Token { chars }).collect();
    let diff_ba_tokens = diff_ba.iter().map(|chars| Token { chars }).collect();

    let diff_ab_joined = TokenSequence::new(diff_ab_tokens).join();
    let diff_ba_joined = TokenSequence::new(diff_ba_tokens).join();
    let intersection_joined = TokenSequence::new(intersection_tokens).join();

    let ab_len = diff_ab_joined.len();
    let ba_len = diff_ba_joined.len();
    let sect_len = intersection_joined.len();

    let sect_len_not_null = if sect_len != 0 { 1 } else { 0 };
    let sect_ab_len = sect_len + sect_len_not_null + ab_len;
    let sect_ba_len = sect_len + sect_len_not_null + ba_len;

    let mut result = 0.0;
    let cutoff_distance =
        ((sect_ab_len + sect_ba_len) as f64 * (1.0 - score_cutoff / 100.0)).ceil() as usize;
    let dist = indel_distance(&diff_ab_joined, &diff_ba_joined, Some(cutoff_distance));

    if dist <= cutoff_distance {
        result = norm_distance(dist, sect_ab_len + sect_ba_len, score_cutoff);
    }

    // Early exit if no intersection
    if sect_len == 0 {
        return result;
    }

    // Calculate distances for intersection combinations
    let sect_ab_dist = (sect_len != 0) as usize + ab_len;
    let sect_ab_ratio = norm_distance(sect_ab_dist, sect_len + sect_ab_len, score_cutoff);

    let sect_ba_dist = (sect_len != 0) as usize + ba_len;
    let sect_ba_ratio = norm_distance(sect_ba_dist, sect_len + sect_ba_len, score_cutoff);

    result.max(sect_ab_ratio).max(sect_ba_ratio)
}

/**
Helper method that returns the maximum of fuzz.token_set_ratio and fuzz.token_sort_ratio
(faster than manually executing the two functions).

Parameters
----------
s1 : str
    First string to compare.
s2 : str
    Second string to compare.
processor: callable, optional
    Optional callable that is used to preprocess the strings before
    comparing them. Default is None, which deactivates this behaviour.
score_cutoff : float, optional
    Optional argument for a score threshold as a float between 0 and 100.
    For ratio < score_cutoff 0 is returned instead. Default is 0,
    which deactivates this behaviour.

Returns
-------
similarity : float
    similarity between s1 and s2 as a float between 0 and 100
*/
#[pyfunction]
#[pyo3(
    name = "token_ratio",
    signature = (s1, s2, *, processor=None, score_cutoff=None)
)]
pub fn py_token_ratio(
    s1: &Bound<'_, PyAny>,
    s2: &Bound<'_, PyAny>,
    processor: Option<&Bound<'_, PyAny>>,
    score_cutoff: Option<f64>,
) -> PyResult<f64> {
    if s1.is_none() || s2.is_none() {
        return Ok(0.0);
    }

    let (s1, s2) = match processor {
        Some(proc) => (proc.call1((s1,))?, proc.call1((s2,))?),
        None => (s1.to_owned(), s2.to_owned()),
    };

    let token_set_ration = py_token_set_ratio(&s1, &s2, None, score_cutoff)?;
    let token_sort_ratio = py_token_sort_ratio(&s1, &s2, None, score_cutoff)?;
    Ok(token_set_ration.max(token_sort_ratio))
}

/**
Sorts the words in the strings and calculates the fuzz.partial_ratio between
them.

Parameters
----------
s1 : str
    First string to compare.
s2 : str
    Second string to compare.
processor: callable, optional
    Optional callable that is used to preprocess the strings before
    comparing them. Default is None, which deactivates this behaviour.
score_cutoff : float, optional
    Optional argument for a score threshold as a float between 0 and 100.
    For ratio < score_cutoff 0 is returned instead. Default is 0,
    which deactivates this behaviour.

Returns
-------
similarity : float
    similarity between s1 and s2 as a float between 0 and 100
*/
#[pyfunction]
#[pyo3(
    name = "partial_token_sort_ratio",
    signature = (s1, s2, *, processor=None, score_cutoff=None)
)]
pub fn py_partial_token_sort_ratio(
    s1: &Bound<'_, PyAny>,
    s2: &Bound<'_, PyAny>,
    processor: Option<&Bound<'_, PyAny>>,
    score_cutoff: Option<f64>,
) -> PyResult<f64> {
    if s1.is_none() || s2.is_none() {
        return Ok(0.0);
    }

    let (s1, s2) = match processor {
        Some(proc) => (proc.call1((s1,))?, proc.call1((s2,))?),
        None => (s1.to_owned(), s2.to_owned()),
    };

    let score_cutoff = score_cutoff.unwrap_or(0.0);

    if s1.is_empty()? && s2.is_empty()? {
        return Ok(100.0);
    }

    let (s1, s2) = match conv_sequences(&s1, &s2) {
        Ok((Some(s1), Some(s2))) => (s1, s2),
        Ok((_, _)) => {
            return Err(pyo3::exceptions::PyValueError::new_err(
                "Cannot handle None",
            ))
        }
        Err(e) => {
            return Err(pyo3::exceptions::PyTypeError::new_err(format!(
                "Failed to convert sequences to u32. {}",
                e
            )))
        }
    };

    Ok(partial_token_sort_ratio(&s1, &s2, score_cutoff))
}

fn partial_token_sort_ratio(s1: &[u32], s2: &[u32], score_cutoff: f64) -> f64 {
    let sorted_s1 = sort_tokens(s1);
    let sorted_s2 = sort_tokens(s2);

    partial_ratio(&sorted_s1, &sorted_s2, score_cutoff)
}

/**
Compares the words in the strings based on unique and common words between them
using fuzz.partial_ratio

Parameters
----------
s1 : str
    First string to compare.
s2 : str
    Second string to compare.
processor: callable, optional
    Optional callable that is used to preprocess the strings before
    comparing them. Default is None, which deactivates this behaviour.
score_cutoff : float, optional
    Optional argument for a score threshold as a float between 0 and 100.
    For ratio < score_cutoff 0 is returned instead. Default is 0,
    which deactivates this behaviour.

Returns
-------
similarity : float
    similarity between s1 and s2 as a float between 0 and 100
*/
#[pyfunction]
#[pyo3(
    name = "partial_token_set_ratio",
    signature = (s1, s2, *, processor=None, score_cutoff=None)
)]
pub fn py_partial_token_set_ratio(
    s1: &Bound<'_, PyAny>,
    s2: &Bound<'_, PyAny>,
    processor: Option<&Bound<'_, PyAny>>,
    score_cutoff: Option<f64>,
) -> PyResult<f64> {
    if s1.is_none() || s2.is_none() {
        return Ok(0.0);
    }

    let (s1, s2) = match processor {
        Some(proc) => (proc.call1((s1,))?, proc.call1((s2,))?),
        None => (s1.to_owned(), s2.to_owned()),
    };

    let score_cutoff = score_cutoff.unwrap_or(0.0);

    if s1.is_empty()? || s2.is_empty()? {
        return Ok(0.0);
    }

    let (s1, s2) = match conv_sequences(&s1, &s2) {
        Ok((Some(s1), Some(s2))) => (s1, s2),
        Ok((_, _)) => {
            return Err(pyo3::exceptions::PyValueError::new_err(
                "Cannot handle None",
            ))
        }
        Err(e) => {
            return Err(pyo3::exceptions::PyTypeError::new_err(format!(
                "Failed to convert sequences to u32. {}",
                e
            )))
        }
    };

    Ok(partial_token_set_ratio(&s1, &s2, score_cutoff))
}

fn partial_token_set_ratio(s1: &[u32], s2: &[u32], score_cutoff: f64) -> f64 {
    // Convert to tokens and collect into sets
    let tokens_a: HashSet<Vec<u32>> = split_into_tokens(s1).map(|t| t.chars.to_vec()).collect();
    let tokens_b: HashSet<Vec<u32>> = split_into_tokens(s2).map(|t| t.chars.to_vec()).collect();

    if tokens_a.is_empty() || tokens_b.is_empty() {
        return 0.0;
    }

    if tokens_a.intersection(&tokens_b).count() > 0 {
        return 100.0;
    }

    let diff_ab: Vec<Token> = tokens_a
        .difference(&tokens_b)
        .map(|chars| Token { chars })
        .collect();
    let diff_ba: Vec<Token> = tokens_b
        .difference(&tokens_a)
        .map(|chars| Token { chars })
        .collect();

    let diff_ab_joined = TokenSequence::new(diff_ab).join();
    let diff_ba_joined = TokenSequence::new(diff_ba).join();

    partial_ratio(&diff_ab_joined, &diff_ba_joined, score_cutoff)
}

/**
Helper method that returns the maximum of fuzz.partial_token_set_ratio and
fuzz.partial_token_sort_ratio (faster than manually executing the two functions)

Parameters
----------
s1 : str
    First string to compare.
s2 : str
    Second string to compare.
processor: callable, optional
    Optional callable that is used to preprocess the strings before
    comparing them. Default is None, which deactivates this behaviour.
score_cutoff : float, optional
    Optional argument for a score threshold as a float between 0 and 100.
    For ratio < score_cutoff 0 is returned instead. Default is 0,
    which deactivates this behaviour.

Returns
-------
similarity : float
    similarity between s1 and s2 as a float between 0 and 100
*/
#[pyfunction]
#[pyo3(
    name = "partial_token_ratio",
    signature = (s1, s2, *, processor=None, score_cutoff=None)
)]
pub fn py_partial_token_ratio(
    s1: &Bound<'_, PyAny>,
    s2: &Bound<'_, PyAny>,
    processor: Option<&Bound<'_, PyAny>>,
    score_cutoff: Option<f64>,
) -> PyResult<f64> {
    if s1.is_none() || s2.is_none() {
        return Ok(0.0);
    }

    let (s1, s2) = match processor {
        Some(proc) => (proc.call1((s1,))?, proc.call1((s2,))?),
        None => (s1.to_owned(), s2.to_owned()),
    };

    if s1.is_empty()? && s2.is_empty()? {
        return Ok(100.0);
    }

    let score_cutoff = score_cutoff.unwrap_or(0.0);

    let (s1, s2) = match conv_sequences(&s1, &s2) {
        Ok((Some(s1), Some(s2))) => (s1, s2),
        Ok((_, _)) => {
            return Err(pyo3::exceptions::PyValueError::new_err(
                "Cannot handle None",
            ))
        }
        Err(e) => {
            return Err(pyo3::exceptions::PyTypeError::new_err(format!(
                "Failed to convert sequences to u32. {}",
                e
            )))
        }
    };

    Ok(partial_token_ratio(&s1, &s2, score_cutoff))
}

fn partial_token_ratio(s1: &[u32], s2: &[u32], score_cutoff: f64) -> f64 {
    // Get tokens using TokenIterator
    let tokens_a: Vec<Token> = split_into_tokens(s1).collect();
    let tokens_b: Vec<Token> = split_into_tokens(s2).collect();

    // Create sets of token chars for intersection check
    let tokens_a_set: HashSet<Vec<u32>> = tokens_a.iter().map(|t| t.chars.to_vec()).collect();
    let tokens_b_set: HashSet<Vec<u32>> = tokens_b.iter().map(|t| t.chars.to_vec()).collect();

    // Quick return if there's an intersection
    if !tokens_a_set.is_disjoint(&tokens_b_set) {
        return 100.0;
    }

    // Get differences between token sets
    let diff_ab: Vec<Token> = tokens_a
        .iter()
        .filter(|t| !tokens_b_set.contains(t.chars))
        .cloned()
        .collect();
    let diff_ba: Vec<Token> = tokens_b
        .iter()
        .filter(|t| !tokens_a_set.contains(t.chars))
        .cloned()
        .collect();

    // Create joined sequences for comparison
    let tokens_seq_a = TokenSequence::new(tokens_a.clone());
    let tokens_seq_b = TokenSequence::new(tokens_b.clone());
    let joined_a = tokens_seq_a.join();
    let joined_b = tokens_seq_b.join();

    // Calculate initial ratio
    let result = partial_ratio(&joined_a, &joined_b, score_cutoff);

    // If tokens are identical to diffs, return the initial ratio
    if tokens_a.len() == diff_ab.len() && tokens_b.len() == diff_ba.len() {
        return result;
    }

    // Create sorted sequences from diffs
    let diff_seq_a = TokenSequence::new(diff_ab).join();
    let diff_seq_b = TokenSequence::new(diff_ba).join();

    let score_cutoff = score_cutoff.max(result);

    let diff_result = partial_ratio(&diff_seq_a, &diff_seq_b, score_cutoff);
    result.max(diff_result)
}

/**
Calculates a weighted ratio based on the other ratio algorithms

Parameters
----------
s1 : str
    First string to compare.
s2 : str
    Second string to compare.
processor: callable, optional
    Optional callable that is used to preprocess the strings before
    comparing them. Default is None, which deactivates this behaviour.
score_cutoff : float, optional
    Optional argument for a score threshold as a float between 0 and 100.
    For ratio < score_cutoff 0 is returned instead. Default is 0,
    which deactivates this behaviour.

Returns
-------
similarity : float
    similarity between s1 and s2 as a float between 0 and 100
*/
#[pyfunction]
#[pyo3(
    name = "WRatio",  // name is chosen in line with Rapifuzz
    signature = (s1, s2, *, processor=None, score_cutoff=None)
)]
pub fn py_weighted_ratio(
    s1: &Bound<'_, PyAny>,
    s2: &Bound<'_, PyAny>,
    processor: Option<&Bound<'_, PyAny>>,
    score_cutoff: Option<f64>,
) -> PyResult<f64> {
    if s1.is_none() || s2.is_none() {
        return Ok(0.0);
    }

    let (s1, s2) = match processor {
        Some(proc) => (proc.call1((s1,))?, proc.call1((s2,))?),
        None => (s1.to_owned(), s2.to_owned()),
    };

    if s1.is_empty()? || s2.is_empty()? {
        return Ok(0.0);
    }

    let score_cutoff = score_cutoff.unwrap_or(0.0);

    // NOTE: this is not done in RapidFuzz, but otherwise we cannot get len
    // which is required in weighted ratio before using the other scorers
    let (s1, s2) = match conv_sequences(&s1, &s2) {
        Ok((Some(s1), Some(s2))) => (s1, s2),
        Ok((_, _)) => {
            return Err(pyo3::exceptions::PyValueError::new_err(
                "Cannot handle None",
            ))
        }
        Err(e) => {
            return Err(pyo3::exceptions::PyTypeError::new_err(format!(
                "Failed to convert sequences to u32. {}",
                e
            )))
        }
    };

    Ok(weighted_ratio(&s1, &s2, score_cutoff))
}

fn weighted_ratio(s1: &[u32], s2: &[u32], score_cutoff: f64) -> f64 {
    let len1 = s1.len();
    let len2 = s2.len();
    let len_ratio = if len1 > len2 {
        len1 as f64 / len2 as f64
    } else {
        len2 as f64 / len1 as f64
    };

    const UNBASE_SCALE: f64 = 0.95;

    // equivalent to `ratio`
    let ratio = if len1 == 0 {
        1.0
    } else {
        indel_normalized_similarity(s1, s2, Some(score_cutoff / 100.0))
    };
    let end_ratio = ratio * 100.0;

    if len_ratio < 1.5 {
        let score_cutoff = f64::max(score_cutoff, end_ratio) / UNBASE_SCALE;

        // equivalent to `token_ratio`
        let token_set_ration = token_set_ratio(s1, s2, score_cutoff);
        let token_sort_ratio = token_sort_ratio(s1, s2, Some(score_cutoff));
        let token_ratio = token_set_ration.max(token_sort_ratio) * UNBASE_SCALE;

        return f64::max(end_ratio, token_ratio);
    }

    let partial_scale = if len_ratio < 8.0 { 0.9 } else { 0.6 };
    let score_cutoff = f64::max(score_cutoff, end_ratio) / partial_scale;
    let partial_ratio = partial_ratio(s1, s2, score_cutoff) * partial_scale;
    let end_ratio = f64::max(end_ratio, partial_ratio);

    let score_cutoff = f64::max(score_cutoff, end_ratio) / UNBASE_SCALE;
    let partial_token_ratio =
        partial_token_ratio(s1, s2, score_cutoff) * UNBASE_SCALE * partial_scale;

    f64::max(end_ratio, partial_token_ratio)
}

/**
Calculates a quick ratio between two strings using fuzz.ratio.

Since v3.0 this behaves similar to fuzz.ratio with the exception that this
returns 0 when comparing two empty strings

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
    Optional argument for a score threshold as a float between 0 and 100.
    For ratio < score_cutoff 0 is returned instead. Default is 0,
    which deactivates this behaviour.

Returns
-------
similarity : float
    similarity between s1 and s2 as a float between 0 and 100

Examples
--------
>>> fuzz.QRatio("this is a test", "this is a test!")
96.55171966552734
*/
#[pyfunction]
#[pyo3(
    name = "QRatio",  // name is chosen in line with Rapifuzz
    signature = (s1, s2, *, processor=None, score_cutoff=None)
)]
pub fn py_quick_ratio(
    s1: &Bound<'_, PyAny>,
    s2: &Bound<'_, PyAny>,
    processor: Option<&Bound<'_, PyAny>>,
    score_cutoff: Option<f64>,
) -> PyResult<f64> {
    if s1.is_none() || s2.is_none() {
        return Ok(0.0);
    }

    let (s1, s2) = match processor {
        Some(proc) => (proc.call1((s1,))?, proc.call1((s2,))?),
        None => (s1.to_owned(), s2.to_owned()),
    };

    if s1.is_empty()? && s2.is_empty()? {
        return Ok(0.0);
    }

    py_ratio(&s1, &s2, None, score_cutoff)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn str_to_vec(s: &str) -> Vec<u32> {
        s.chars().map(|c| c as u32).collect()
    }

    #[test]
    fn test_ratio() {
        let s1 = str_to_vec("this is a test");
        let s2 = str_to_vec("this is a test!");
        let result = indel_normalized_similarity(&s1, &s2, None) * 100.0;
        assert!(
            (result - 96.55171966552734).abs() < 1e-5,
            "Expected approximately 96.55171966552734"
        );
    }

    #[test]
    fn test_ratio_with_cutoff() {
        let s1 = str_to_vec("this is a test");
        let s2 = str_to_vec("this is a test!");
        let result = indel_normalized_similarity(&s1, &s2, Some(0.0)) * 100.0;
        assert!(
            (result - 96.55171966552734).abs() < 1e-5,
            "Expected approximately 96.55171966552734, got {}",
            result
        );
    }

    #[test]
    fn test_ratio_unordered() {
        let s1 = str_to_vec("new york mets vs atlanta braves");
        let s2 = str_to_vec("atlanta braves vs new york mets");
        let result = indel_normalized_similarity(&s1, &s2, None) * 100.0;
        assert!(
            (result - 45.16129032258065).abs() < 1e-5,
            "Expected 45.16129032258065, got {}",
            result
        );
    }

    #[test]
    fn test_partial_ratio() {
        let s1 = str_to_vec("this is a test");
        let s2 = str_to_vec("this is a test!");
        let result = partial_ratio(&s1, &s2, 0.0);
        assert_eq!(result, 100.0, "Expected 100.0");
    }

    #[test]
    fn test_partial_ratio_issue138() {
        let s1 = str_to_vec(&"a".repeat(65));
        let s2 = str_to_vec(&format!(
            "a{}{}",
            char::from_u32(256).unwrap(),
            "a".repeat(63)
        ));
        let result = partial_ratio(&s1, &s2, 0.0);
        assert!(
            (result - 99.22481).abs() < 1e-5,
            "Expected approximately 99.22481, got {}",
            result
        );
    }

    #[test]
    fn test_partial_ratio_alignment() {
        let str1 = "er merkantilismus förderte handle und verkehr mit teils marktkonformen, teils dirigistischen maßnahmen.";
        let str2 = "ils marktkonformen, teils dirigistischen maßnahmen. an der schwelle zum 19. jahrhundert entstand ein neu";

        let alignment = partial_ratio_alignment(&str_to_vec(str1), &str_to_vec(str2), 0.0);

        dbg!(&alignment);

        assert!(
            (alignment.as_ref().unwrap().score - 66.2337662).abs() < 1e-5,
            "Expected 66.2337662, got {}",
            alignment.unwrap().score
        );
        assert_eq!(alignment.as_ref().unwrap().src_start, 0);
        assert_eq!(alignment.as_ref().unwrap().src_end, 103);
        assert_eq!(alignment.as_ref().unwrap().dest_start, 0);
        assert_eq!(alignment.as_ref().unwrap().dest_end, 51);
    }

    #[test]
    fn test_partial_ratio_short_needle_identical() {
        let s1 = str_to_vec("abcd");
        let s2 = str_to_vec("abcd");
        let result = partial_ratio_short_needle(&s1, &s2, 0.0);
        assert_eq!(result.score, 100.0);
        assert_eq!(result.src_start, 0);
        assert_eq!(result.src_end, 4);
        assert_eq!(result.dest_start, 0);
        assert_eq!(result.dest_end, 4);
    }

    #[test]
    fn test_partial_ratio_short_needle_substring() {
        let s1 = str_to_vec("bcd");
        let s2 = str_to_vec("abcde");
        let result = partial_ratio_short_needle(&s1, &s2, 0.0);
        assert_eq!(result.score, 100.0);
        assert_eq!(result.src_start, 0);
        assert_eq!(result.src_end, 3);
        assert_eq!(result.dest_start, 1);
        assert_eq!(result.dest_end, 4);
    }

    #[test]
    fn test_partial_ratio_short_needle_partial_match() {
        let s1 = str_to_vec("abc");
        let s2 = str_to_vec("bcde");
        let result = partial_ratio_short_needle(&s1, &s2, 0.0);
        assert!((result.score - 80.0).abs() < 1e-10);
        assert_eq!(result.src_start, 0);
        assert_eq!(result.src_end, 3);
        assert_eq!(result.dest_start, 0);
        assert_eq!(result.dest_end, 2);
    }

    #[test]
    fn test_partial_ratio_short_needle_partial_match_score_cutoff() {
        let s1 = str_to_vec("abc");
        let s2 = str_to_vec("bcde");
        let result = partial_ratio_short_needle(&s1, &s2, 0.9);
        assert_eq!(result.score, 0.0);
        assert_eq!(result.src_start, 0);
        assert_eq!(result.src_end, 3);
        assert_eq!(result.dest_start, 0);
        assert_eq!(result.dest_end, 3);
    }

    #[test]
    fn test_partial_ratio_short_needle_no_match() {
        let s1 = str_to_vec("abc");
        let s2 = str_to_vec("def");
        let result = partial_ratio_short_needle(&s1, &s2, 0.0);
        assert_eq!(result.score, 0.0);
    }

    #[test]
    fn test_partial_ratio_short_needle_score_cutoff() {
        let s1 = str_to_vec("abc");
        let s2 = str_to_vec("abcde");
        let result = partial_ratio_short_needle(&s1, &s2, 0.9);
        assert_eq!(result.score, 100.0);
    }

    #[test]
    fn test_partial_ratio_short_needle_empty_s1() {
        let s1 = str_to_vec("");
        let s2 = str_to_vec("abc");
        let result = partial_ratio_short_needle(&s1, &s2, 0.0);
        assert_eq!(result.score, 0.0);
    }

    #[test]
    fn test_partial_ratio_short_needle_certain_string() {
        let s1 = str_to_vec("cetain");
        let s2 = str_to_vec("a certain string");
        let result = partial_ratio_short_needle(&s1, &s2, 0.0);
        dbg!(&result);
        assert!((result.score - 83.33333333333334).abs() < 1e-10);
        assert_eq!(result.src_start, 0);
        assert_eq!(result.src_end, 6);
        assert_eq!(result.dest_start, 2);
        assert_eq!(result.dest_end, 8);
    }

    #[test]
    fn test_token_sort_ratio_empty() {
        let s1 = str_to_vec("");
        let s2 = str_to_vec("");
        let result = token_sort_ratio(&s1, &s2, None);
        assert_eq!(result, 100.0);
    }

    #[test]
    fn test_partial_token_set_ratio_empty() {
        let s1 = str_to_vec("");
        let s2 = str_to_vec("");
        let result = token_set_ratio(&s1, &s2, 0.0);
        assert_eq!(result, 0.0);
    }

    #[test]
    fn test_weighted_ratio() {
        let s1 = str_to_vec("South Korea");
        let s2 = str_to_vec("North Korea");
        let expected = 81.81818181818181;
        let score1 = weighted_ratio(&s1, &s2, 0.0);
        let score2 = weighted_ratio(&s1, &s2, score1 - 0.0001);
        assert_eq!(score1, score2);
        assert_eq!(score1, expected);
    }
}
