use crate::distance::prep_inputs;
use pyo3::prelude::*;
use std::iter::zip;

/**
Calculates the Prefix distance between two strings.

Parameters
----------
s1 : Sequence[Hashable]
    First string to compare.
s2 : Sequence[Hashable]
    Second string to compare.
processor: callable, optional
    Optional callable that is used to preprocess the strings before
    comparing them. Default is None, which deactivates this behaviour.
score_cutoff : int or None, optional
    Maximum distance between s1 and s2, that is
    considered as a result. If the distance is bigger than score_cutoff,
    score_cutoff + 1 is returned instead. Default is None, which deactivates
    this behaviour.

Returns
-------
distance : int
    distance between s1 and s2
*/
#[pyfunction]
#[pyo3(
    name = "distance",
    signature = (s1, s2, *, processor=None, score_cutoff=None)
)]
pub fn py_distance(
    s1: &Bound<'_, PyAny>,
    s2: &Bound<'_, PyAny>,
    processor: Option<&Bound<'_, PyAny>>,
    score_cutoff: Option<usize>,
) -> PyResult<usize> {
    let (s1, s2) = prep_inputs(s1, s2, processor)?;
    let res = distance(&s1, &s2, score_cutoff);

    Ok(res)
}

fn distance(s1: &[u32], s2: &[u32], score_cutoff: Option<usize>) -> usize {
    let maximum = usize::max(s1.len(), s2.len());
    let sim = similarity(s1, s2, None);
    let dist = maximum - sim;

    match score_cutoff {
        Some(cutoff) if dist <= cutoff => dist,
        Some(cutoff) => cutoff + 1,
        None => dist,
    }
}

/**
Calculates the prefix similarity between two strings.

This is calculated as ``len1 - distance``.

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
    considered as a result. If the similarity is smaller than score_cutoff,
    0 is returned instead. Default is None, which deactivates
    this behaviour.

Returns
-------
distance : int
    distance between s1 and s2
*/
#[pyfunction]
#[pyo3(
    name = "similarity",
    signature = (s1, s2, *, processor=None, score_cutoff=None)
)]
pub fn py_similarity(
    s1: &Bound<'_, PyAny>,
    s2: &Bound<'_, PyAny>,
    processor: Option<&Bound<'_, PyAny>>,
    score_cutoff: Option<usize>,
) -> PyResult<usize> {
    let (s1, s2) = prep_inputs(s1, s2, processor)?;
    let sim = similarity(&s1, &s2, score_cutoff);

    Ok(sim)
}

pub fn similarity(s1: &[u32], s2: &[u32], score_cutoff: Option<usize>) -> usize {
    let mut sim = 0;
    for (a, b) in zip(s1, s2) {
        if a != b {
            break;
        }
        sim += 1;
    }

    match score_cutoff {
        Some(cutoff) if sim >= cutoff => sim,
        Some(_) => 0,
        None => sim,
    }
}

#[pyfunction]
#[pyo3(
    name = "normalized_distance",
    signature = (s1, s2, *, processor=None, score_cutoff=None)
)]
pub fn py_normalized_distance(
    s1: &Bound<'_, PyAny>,
    s2: &Bound<'_, PyAny>,
    processor: Option<&Bound<'_, PyAny>>,
    score_cutoff: Option<f64>,
) -> PyResult<f64> {
    if s1.is_none() || s2.is_none() {
        return Ok(1.0);
    }

    let (s1, s2) = prep_inputs(s1, s2, processor)?;
    let norm_dist = normalized_distance(&s1, &s2, score_cutoff);

    Ok(norm_dist)
}

pub fn normalized_distance(s1: &[u32], s2: &[u32], score_cutoff: Option<f64>) -> f64 {
    let norm_sim = normalized_similarity(s1, s2, None);
    let norm_dist = 1.0 - norm_sim;

    match score_cutoff {
        Some(cutoff) if norm_dist <= cutoff => norm_dist,
        Some(_) => 1.0,
        None => norm_dist,
    }
}

#[pyfunction]
#[pyo3(
    name = "normalized_similarity",
    signature = (s1, s2, *, processor=None, score_cutoff=None)
)]
pub fn py_normalized_similarity(
    s1: &Bound<'_, PyAny>,
    s2: &Bound<'_, PyAny>,
    processor: Option<&Bound<'_, PyAny>>,
    score_cutoff: Option<f64>,
) -> PyResult<f64> {
    if s1.is_none() || s2.is_none() {
        return Ok(0.0);
    }

    let (s1, s2) = prep_inputs(s1, s2, processor)?;
    let norm_sim = normalized_similarity(&s1, &s2, score_cutoff);

    Ok(norm_sim)
}

pub fn normalized_similarity(s1: &[u32], s2: &[u32], score_cutoff: Option<f64>) -> f64 {
    let maxinum = usize::max(s1.len(), s2.len()) as f64;
    let sim = similarity(s1, s2, None) as f64;
    let norm_sim = if maxinum == 0.0 { 1.0 } else { sim / maxinum };

    match score_cutoff {
        Some(cutoff) if norm_sim >= cutoff => norm_sim,
        Some(_) => 0.0,
        None => norm_sim,
    }
}
