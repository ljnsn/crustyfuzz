use crate::distance::{conv_sequences, prep_inputs};
use pyo3::prelude::*;
use std::cmp::min;

fn jaro_calculate_similarity(
    pattern_len: usize,
    text_len: usize,
    common_chars: usize,
    transpositions: usize,
) -> f64 {
    let transpositions = transpositions / 2;
    let mut sim = 0.0;
    sim += common_chars as f64 / pattern_len as f64;
    sim += common_chars as f64 / text_len as f64;
    sim += (common_chars as f64 - transpositions as f64) / common_chars as f64;
    sim / 3.0
}

fn jaro_length_filter(pattern_len: usize, text_len: usize, score_cutoff: f64) -> bool {
    if pattern_len == 0 || text_len == 0 {
        return false;
    }
    let sim = jaro_calculate_similarity(pattern_len, text_len, min(pattern_len, text_len), 0);
    sim >= score_cutoff
}

fn jaro_common_char_filter(
    pattern_len: usize,
    text_len: usize,
    common_chars: usize,
    score_cutoff: f64,
) -> bool {
    if common_chars == 0 {
        return false;
    }
    let sim = jaro_calculate_similarity(pattern_len, text_len, common_chars, 0);
    sim >= score_cutoff
}

fn jaro_bounds<'a>(s1: &'a [u32], s2: &'a [u32]) -> (&'a [u32], &'a [u32], usize) {
    let pattern_len = s1.len();
    let text_len = s2.len();

    let mut s1 = s1;
    let mut s2 = s2;

    // since jaro uses a sliding window some parts of T/P might never be in
    // range and can be removed ahread of time
    let bound;
    if text_len > pattern_len {
        bound = text_len / 2 - 1;
        if text_len > pattern_len + bound {
            s2 = &s2[0..usize::min(pattern_len + bound, s2.len())];
        }
    } else {
        bound = pattern_len / 2 - 1;
        if pattern_len > text_len + bound {
            s1 = &s1[0..usize::min(text_len + bound, s1.len())];
        }
    }

    (s1, s2, bound)
}

#[pyfunction]
#[pyo3(
    name = "similarity",
    signature = (s1, s2, *, processor=None, score_cutoff=None)
)]
pub fn py_similarity(
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

    let sim = similarity(&s1, &s2, score_cutoff);

    Ok(sim)
}

pub fn similarity(s1: &[u32], s2: &[u32], score_cutoff: f64) -> f64 {
    let pattern_len = s1.len();
    let text_len = s2.len();

    // short circuit if score_cutoff can not be reached
    if !jaro_length_filter(pattern_len, text_len, score_cutoff) {
        return 0.0;
    }

    if pattern_len == 1 && text_len == 1 {
        return if s1[0] == s2[0] { 1.0 } else { 0.0 };
    }

    let (s1, s2, bound) = jaro_bounds(s1, s2);

    let mut s1_flags = vec![false; s1.len()];
    let mut s2_flags = vec![false; s2.len()];

    // TODO: use bitparallel implementation
    // looking only within search range, count & flag matched pairs
    let mut common_chars = 0;
    for (i, c1) in s1.iter().enumerate() {
        let start = if i > bound { i - bound } else { 0 };
        let end = if i + bound < text_len {
            i + bound
        } else {
            text_len - 1
        };
        for j in start..=end {
            if c1 == &s2[j] && !s2_flags[j] {
                s1_flags[i] = true;
                s2_flags[j] = true;
                common_chars += 1;
                break;
            }
        }
    }

    // short circuit if score_cutoff can not be reached
    if !jaro_common_char_filter(pattern_len, text_len, common_chars, score_cutoff) {
        return 0.0;
    }

    // TODO: use bitparallel implementation
    // count transpositions
    let mut k = 0;
    let mut trans_count = 0;
    for (i, s1_f) in s1_flags.iter().enumerate() {
        if *s1_f {
            let mut j = k;
            while j < text_len {
                if s2_flags[j] {
                    k = j + 1;
                    if s1[i] != s2[j] {
                        trans_count += 1;
                    }
                    break;
                }
                j += 1;
            }
        }
    }

    jaro_calculate_similarity(pattern_len, text_len, common_chars, trans_count)
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
    py_similarity(s1, s2, processor, score_cutoff)
}

#[pyfunction]
#[pyo3(
    name = "distance",
    signature = (s1, s2, *, processor=None, score_cutoff=None)
)]
pub fn py_distance(
    s1: &Bound<'_, PyAny>,
    s2: &Bound<'_, PyAny>,
    processor: Option<&Bound<'_, PyAny>>,
    score_cutoff: Option<f64>,
) -> PyResult<f64> {
    if s1.is_none() || s2.is_none() {
        return Ok(1.0);
    }

    let (s1, s2) = prep_inputs(s1, s2, processor)?;

    if s1.is_empty() && s2.is_empty() {
        return Ok(0.0);
    }

    let res = distance(&s1, &s2, score_cutoff);

    Ok(res)
}

fn distance(s1: &[u32], s2: &[u32], score_cutoff: Option<f64>) -> f64 {
    let cutoff_distance = match score_cutoff {
        Some(cutoff) if cutoff > 1.0 => 0.0,
        Some(cutoff) => 1.0 - cutoff,
        None => 0.0,
    };

    let sim = similarity(s1, s2, cutoff_distance);
    let dist = 1.0 - sim;

    match score_cutoff {
        Some(cutoff) if dist <= cutoff => dist,
        Some(_) => 1.0,
        None => dist,
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
    py_distance(s1, s2, processor, score_cutoff)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_similarity() {
        let s1 = "00".chars().map(|c| c as u32).collect::<Vec<u32>>();
        let s2 = "00".chars().map(|c| c as u32).collect::<Vec<u32>>();

        let sim = similarity(&s1, &s2, 0.0);
        assert_eq!(sim, 1.0);
    }
}
