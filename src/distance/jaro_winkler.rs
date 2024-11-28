use crate::distance::jaro::similarity as jaro_similarity;
use crate::distance::{conv_sequences, prep_inputs};
use pyo3::prelude::*;
use std::cmp::min;

#[pyfunction]
#[pyo3(
    name = "similarity",
    signature = (s1, s2, *, prefix_weight=0.1, processor=None, score_cutoff=None)
)]
pub fn py_similarity(
    s1: &Bound<'_, PyAny>,
    s2: &Bound<'_, PyAny>,
    prefix_weight: f64,
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
        return Ok(1.0);
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

    let sim = similarity(&s1, &s2, prefix_weight, score_cutoff);

    Ok(sim)
}

pub fn similarity(s1: &[u32], s2: &[u32], prefix_weight: f64, score_cutoff: f64) -> f64 {
    let p_len = s1.len();
    let t_len = s2.len();
    let min_len = min(p_len, t_len);
    let mut prefix = 0;
    let max_prefix = min(min_len, 4);

    for _ in 0..max_prefix {
        if s1[prefix] != s2[prefix] {
            break;
        }
        prefix += 1;
    }

    let mut jaro_score_cutoff = score_cutoff;
    if jaro_score_cutoff > 0.7 {
        let prefix_sim = prefix as f64 * prefix_weight;
        if prefix_sim >= 1.0 {
            jaro_score_cutoff = 0.7;
        } else {
            jaro_score_cutoff = f64::max(0.7, (prefix_sim - jaro_score_cutoff) / (prefix_sim - 1.0))
        }
    }

    let mut sim = jaro_similarity(s1, s2, jaro_score_cutoff);
    if sim > 0.7 {
        sim += prefix as f64 * prefix_weight * (1.0 - sim);
    }

    if sim >= score_cutoff {
        sim
    } else {
        0.0
    }
}

#[pyfunction]
#[pyo3(
    name = "normalized_similarity",
    signature = (s1, s2, *, prefix_weight=0.1, processor=None, score_cutoff=None)
)]
pub fn py_normalized_similarity(
    s1: &Bound<'_, PyAny>,
    s2: &Bound<'_, PyAny>,
    prefix_weight: f64,
    processor: Option<&Bound<'_, PyAny>>,
    score_cutoff: Option<f64>,
) -> PyResult<f64> {
    py_similarity(s1, s2, prefix_weight, processor, score_cutoff)
}

#[pyfunction]
#[pyo3(
    name = "distance",
    signature = (s1, s2, *, prefix_weight=0.1, processor=None, score_cutoff=None)
)]
pub fn py_distance(
    s1: &Bound<'_, PyAny>,
    s2: &Bound<'_, PyAny>,
    prefix_weight: f64,
    processor: Option<&Bound<'_, PyAny>>,
    score_cutoff: Option<f64>,
) -> PyResult<f64> {
    if s1.is_none() || s2.is_none() {
        return Ok(0.0);
    }

    let (s1, s2) = prep_inputs(s1, s2, processor)?;

    if s1.is_empty() && s2.is_empty() {
        return Ok(0.0);
    }

    let cutoff_distance = match score_cutoff {
        Some(cutoff) if cutoff > 1.0 => 0.0,
        Some(cutoff) => 1.0 - cutoff,
        None => 0.0,
    };

    let sim = similarity(&s1, &s2, prefix_weight, cutoff_distance);
    let dist = 1.0 - sim;

    match score_cutoff {
        Some(cutoff) if dist <= cutoff => Ok(dist),
        Some(_) => Ok(1.0),
        None => Ok(dist),
    }
}

#[pyfunction]
#[pyo3(
    name = "normalized_distance",
    signature = (s1, s2, *, prefix_weight=0.1, processor=None, score_cutoff=None)
)]
pub fn py_normalized_distance(
    s1: &Bound<'_, PyAny>,
    s2: &Bound<'_, PyAny>,
    prefix_weight: f64,
    processor: Option<&Bound<'_, PyAny>>,
    score_cutoff: Option<f64>,
) -> PyResult<f64> {
    py_distance(s1, s2, prefix_weight, processor, score_cutoff)
}
