use crate::distance::prep_inputs;
use pyo3::prelude::*;
use std::cmp::min;
use std::collections::HashMap;

fn damerau_levenshtein_distance_zhao(s1: &[u32], s2: &[u32]) -> usize {
    let max_value = usize::max(s1.len(), s2.len()) + 1;
    let mut last_row_id = HashMap::new();
    let size = s2.len() + 2;

    let mut fr = vec![max_value; size];
    let mut r1 = vec![max_value; size];
    let mut r: Vec<usize> = (0..size).collect();
    r[size - 1] = max_value;

    for i in 1..=s1.len() {
        std::mem::swap(&mut r, &mut r1);
        let mut last_col_id: isize = -1;
        let mut last_i2l1 = r[0];
        r[0] = i;
        let mut t = max_value;

        for j in 1..=s2.len() {
            let diag = r1[j - 1] + if s1[i - 1] != s2[j - 1] { 1 } else { 0 };
            let left = r[j - 1] + 1;
            let up = r1[j] + 1;
            let mut temp = min(min(diag, left), up);

            if s1[i - 1] == s2[j - 1] {
                last_col_id = j as isize; // last occurrence of s1_i
                fr[j] = if j >= 2 { r1[j - 2] } else { r1[size - 1] }; // save H_k-1,j-2
                t = last_i2l1; // save H_i-2,l-1
            } else {
                let k = *last_row_id.get(&s2[j - 1]).unwrap_or(&(-1_isize));
                let l = last_col_id;

                if (j as isize - l) == 1 {
                    let transpose = fr[j] + (i as isize - k) as usize;
                    temp = min(temp, transpose);
                } else if (i as isize - k) == 1 {
                    let transpose = t + (j as isize - l) as usize;
                    temp = min(temp, transpose);
                }
            }

            last_i2l1 = r[j];
            r[j] = temp;
        }

        last_row_id.insert(s1[i - 1], i as isize);
    }

    r[s2.len()]
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
    score_cutoff: Option<usize>,
) -> PyResult<usize> {
    let (s1, s2) = prep_inputs(s1, s2, processor)?;
    let res = distance(&s1, &s2, score_cutoff);

    Ok(res)
}

fn distance(s1: &[u32], s2: &[u32], score_cutoff: Option<usize>) -> usize {
    let dist = damerau_levenshtein_distance_zhao(s1, s2);

    match score_cutoff {
        Some(cutoff) if dist <= cutoff => dist,
        Some(cutoff) => cutoff + 1,
        None => dist,
    }
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
    score_cutoff: Option<usize>,
) -> PyResult<usize> {
    let (s1, s2) = prep_inputs(s1, s2, processor)?;
    let sim = similarity(&s1, &s2, score_cutoff);

    Ok(sim)
}

pub fn similarity(s1: &[u32], s2: &[u32], score_cutoff: Option<usize>) -> usize {
    let maximum = usize::max(s1.len(), s2.len());
    let dist = distance(s1, s2, score_cutoff);
    let sim = maximum - dist;

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
    let maximum = usize::max(s1.len(), s2.len()) as f64;
    let norm_dist = if maximum == 0.0 {
        0.0
    } else {
        let dist = distance(s1, s2, None) as f64;
        dist / maximum
    };

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
    let norm_dist = normalized_distance(s1, s2, None);
    let norm_sim = 1.0 - norm_dist;

    match score_cutoff {
        Some(cutoff) if norm_sim >= cutoff => norm_sim,
        Some(_) => 0.0,
        None => norm_sim,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_damerau_levenshtein_distance_zhao_simple() {
        let s1 = "test".chars().map(|c| c as u32).collect::<Vec<_>>();
        let s2 = "text".chars().map(|c| c as u32).collect::<Vec<_>>();
        let dist = damerau_levenshtein_distance_zhao(&s1, &s2);
        assert_eq!(dist, 1);
    }

    #[test]
    fn test_damerau_levenshtein_distance_zhao() {
        let s1 = "kitten".chars().map(|c| c as u32).collect::<Vec<_>>();
        let s2 = "sitting".chars().map(|c| c as u32).collect::<Vec<_>>();
        let dist = damerau_levenshtein_distance_zhao(&s1, &s2);
        assert_eq!(dist, 3);
    }

    #[test]
    fn test_damerau_levenshtein_distance_zhao_empty() {
        let s1 = "".chars().map(|c| c as u32).collect::<Vec<_>>();
        let s2 = "".chars().map(|c| c as u32).collect::<Vec<_>>();
        let dist = damerau_levenshtein_distance_zhao(&s1, &s2);
        assert_eq!(dist, 0);
    }

    #[test]
    fn test_damerau_levenshtein_distance_zhao_empty_1() {
        let s1 = "".chars().map(|c| c as u32).collect::<Vec<_>>();
        let s2 = "sitting".chars().map(|c| c as u32).collect::<Vec<_>>();
        let dist = damerau_levenshtein_distance_zhao(&s1, &s2);
        assert_eq!(dist, 7);
    }

    #[test]
    fn test_damerau_levenshtein_distance_zhao_empty_2() {
        let s1 = "kitten".chars().map(|c| c as u32).collect::<Vec<_>>();
        let s2 = "".chars().map(|c| c as u32).collect::<Vec<_>>();
        let dist = damerau_levenshtein_distance_zhao(&s1, &s2);
        assert_eq!(dist, 6);
    }
}
