use crate::common::conv_sequences;
use crate::distance::indel::{block_normalized_similarity, normalized_similarity};
use crate::distance::models::ScoreAlignment;
use pyo3::prelude::*;
use std::collections::{HashMap, HashSet};

// call a python processor function
fn call_processor(processor: &Bound<'_, PyAny>, s: Option<&str>) -> Result<String, PyErr> {
    let res = processor.call1((s,))?;
    res.extract::<String>()
}

// process inputs with a given processor
fn process_inputs(
    s1: Option<&str>,
    s2: Option<&str>,
    processor: Option<&Bound<'_, PyAny>>,
) -> PyResult<(Option<String>, Option<String>)> {
    match processor {
        Some(proc) => {
            let processed_s1 = s1.map(|s| call_processor(proc, Some(s))).transpose()?;
            let processed_s2 = s2.map(|s| call_processor(proc, Some(s))).transpose()?;
            Ok((processed_s1, processed_s2))
        }
        None => Ok((s1.map(ToString::to_string), s2.map(ToString::to_string))),
    }
}

/**
Calculates the normalized Indel distance.

Parameters
----------
s1 : Option<&str>
    First string to compare.
s2 : Option<&str>
    Second string to compare.
processor: Option<fn(Vec<char>) -> Vec<char>>
    Optional callable that is used to preprocess the strings before
    comparing them. Default is None, which deactivates this behaviour.
score_cutoff : Option<f64>
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
#[pyo3(
    signature = (s1, s2, processor=None, score_cutoff=None)
)]
pub fn ratio(
    s1: Option<&str>,
    s2: Option<&str>,
    processor: Option<&Bound<'_, PyAny>>,
    score_cutoff: Option<f64>,
) -> PyResult<f64> {
    let (processed_s1, processed_s2) = process_inputs(s1, s2, processor)?;

    Ok(_ratio(
        processed_s1.as_deref(),
        processed_s2.as_deref(),
        score_cutoff,
    ))
}

fn _ratio(s1: Option<&str>, s2: Option<&str>, score_cutoff: Option<f64>) -> f64 {
    match (s1, s2) {
        (Some(s1), Some(s2)) => {
            let score_cutoff = score_cutoff.map(|cutoff| cutoff / 100.0);

            let s1_vec: Vec<char> = s1.chars().collect();
            let s2_vec: Vec<char> = s2.chars().collect();

            let score = normalized_similarity(Some(&s1_vec), Some(&s2_vec), None, score_cutoff);
            score * 100.0
        }
        _ => 0.0,
    }
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
    signature = (s1, s2, processor=None, score_cutoff=None)
)]
pub fn partial_ratio(
    s1: Option<&str>,
    s2: Option<&str>,
    processor: Option<&Bound<'_, PyAny>>,
    score_cutoff: Option<f64>,
) -> PyResult<f64> {
    let (processed_s1, processed_s2) = process_inputs(s1, s2, processor)?;

    Ok(_partial_ratio(
        processed_s1.as_deref(),
        processed_s2.as_deref(),
        score_cutoff,
    ))
}

fn _partial_ratio(s1: Option<&str>, s2: Option<&str>, score_cutoff: Option<f64>) -> f64 {
    let alignment = _partial_ratio_alignment(s1, s2, score_cutoff);

    match alignment {
        Some(alignment) => alignment.score,
        None => 0.0,
    }
}

/**
implementation of partial_ratio for needles <= 64. assumes s1 is already the
shorter string
*/
fn partial_ratio_short_needle(s1: &Vec<u64>, s2: &Vec<u64>, score_cutoff: f64) -> ScoreAlignment {
    let s1_char_set = s1.iter().cloned().collect::<HashSet<_>>();
    let len1 = s1.len();
    let len2 = s2.len();
    let mut score_cutoff = score_cutoff;

    let mut res = ScoreAlignment {
        score: 0.0,
        src_start: 0,
        src_end: len1,
        dest_start: 0,
        dest_end: len1,
    };

    let mut block = HashMap::<u64, u64>::new();
    let mut x = 1;
    for ch1 in s1 {
        *block.entry(*ch1).or_insert(0) |= x;
        x <<= 1;
    }

    for i in 1..len1 {
        let sustr_last = s2[i - 1];
        if !s1_char_set.contains(&sustr_last) {
            continue;
        }

        let ls_ratio =
            block_normalized_similarity(&block, &s1, &Vec::from(&s2[..i]), Some(score_cutoff));
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

    for i in 0..(len2 - len1) {
        let sustr_last = s2[i + len1 - 1];
        if !s1_char_set.contains(&sustr_last) {
            continue;
        }

        let ls_ratio = block_normalized_similarity(
            &block,
            &s1,
            &Vec::from(&s2[i..i + len1]),
            Some(score_cutoff),
        );
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

    for i in (len2 - len1)..len2 {
        let substr_first = s2[i];
        if !s1_char_set.contains(&substr_first) {
            continue;
        }

        let ls_ratio =
            block_normalized_similarity(&block, &s1, &Vec::from(&s2[i..]), Some(score_cutoff));
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

    res.score = res.score * 100.0;
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
    signature = (s1, s2, processor=None, score_cutoff=None)
)]
pub fn partial_ratio_alignment(
    s1: Option<&str>,
    s2: Option<&str>,
    processor: Option<&Bound<'_, PyAny>>,
    score_cutoff: Option<f64>,
) -> PyResult<Option<ScoreAlignment>> {
    let (processed_s1, processed_s2) = process_inputs(s1, s2, processor)?;

    Ok(_partial_ratio_alignment(
        processed_s1.as_deref(),
        processed_s2.as_deref(),
        score_cutoff,
    ))
}

fn _partial_ratio_alignment(
    s1: Option<&str>,
    s2: Option<&str>,
    score_cutoff: Option<f64>,
) -> Option<ScoreAlignment> {
    if s1.is_none() || s2.is_none() {
        return None;
    }

    let s1 = s1.unwrap();
    let s2 = s2.unwrap();
    let mut score_cutoff = score_cutoff.unwrap_or(0.0);

    if s1.is_empty() || s2.is_empty() {
        return Some(ScoreAlignment {
            score: 100.0,
            src_start: 0,
            src_end: 0,
            dest_start: 0,
            dest_end: 0,
        });
    }

    let s1_vec: Vec<char> = s1.chars().collect();
    let s2_vec: Vec<char> = s2.chars().collect();

    let (s1, s2) = conv_sequences(&s1_vec, &s2_vec);
    let shorter;
    let longer;

    if s1.len() <= s2.len() {
        shorter = &s1;
        longer = &s2;
    } else {
        shorter = &s2;
        longer = &s1;
    }

    let mut res = partial_ratio_short_needle(&shorter, &longer, score_cutoff / 100.0);
    if res.score != 100.0 && s1.len() == s2.len() {
        score_cutoff = f64::max(score_cutoff, res.score);
        let res2 = partial_ratio_short_needle(&longer, &shorter, score_cutoff / 100.0);
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

    if s1.len() <= s2.len() {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ratio() {
        let s1 = "this is a test";
        let s2 = "this is a test!";
        let result = _ratio(Some(s1), Some(s2), None);
        assert!(
            (result - 96.55171966552734).abs() < 1e-5,
            "Expected approximately 96.55171966552734"
        );
    }

    #[test]
    fn test_partial_ratio() {
        let s1 = "this is a test";
        let s2 = "this is a test!";
        let result = _partial_ratio(Some(s1), Some(s2), None);
        assert_eq!(result, 100.0, "Expected 100.0");
    }
}
