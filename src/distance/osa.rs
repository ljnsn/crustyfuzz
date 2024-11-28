use crate::distance::prep_inputs;
use pyo3::prelude::*;
use std::cmp::min;
use std::collections::HashMap;
use std::mem;

fn _osa_distance_hyrroe2003(s1: &[u32], s2: &[u32]) -> usize {
    let len1 = s1.len();
    let len2 = s2.len();

    if len1 == 0 {
        return len2;
    }

    let mut vert_pos = (1_u128 << len1) - 1_u128;
    let mut vert_neg = 0_u128;
    let mut diagonal_zero = 0_u128;
    let mut pattern_match_j_old = 0_u128;
    let mut current_distance = len1;
    let mask = 1_u128 << (len1 - 1);

    let mut block = HashMap::with_capacity(len1);
    let mut position = 1_u128;
    for &ch1 in s1 {
        block
            .entry(ch1)
            .and_modify(|e| *e |= position)
            .or_insert(position);
        position <<= 1;
    }

    for &ch2 in s2 {
        // step 1: computing D0
        let pattern_match_j = *block.get(&ch2).unwrap_or(&0);
        let tr = (((!diagonal_zero) & pattern_match_j) << 1) & pattern_match_j_old;
        diagonal_zero =
            (((pattern_match_j & vert_pos) + vert_pos) ^ vert_pos) | pattern_match_j | vert_neg;
        diagonal_zero |= tr;
        // step 2: computing HP and HN
        let horizontal_pos = vert_neg | !(diagonal_zero | vert_pos);
        let horizontal_neg = diagonal_zero & vert_pos;
        // step 3: computing the value D[m,j]
        current_distance += if (horizontal_pos & mask) == 0 { 0 } else { 1 };
        current_distance -= if (horizontal_neg & mask) == 0 { 0 } else { 1 };
        // step 4: computing VP and VN
        let horizontal_pos = (horizontal_pos << 1) | 1;
        let horizontal_neg = horizontal_neg << 1;
        vert_pos = horizontal_neg | !(diagonal_zero | horizontal_pos);
        vert_neg = horizontal_pos & diagonal_zero;
        pattern_match_j_old = pattern_match_j;
    }

    current_distance
}

fn osa_distance(a: &[u32], b: &[u32]) -> usize {
    // implementation from `strsim` crate, since it can handle arbitrary string lengths
    let b_len = b.len();
    // 0..=b_len behaves like 0..b_len.saturating_add(1) which could be a different size
    // this leads to significantly worse code gen when swapping the vectors below
    let mut prev_two_distances: Vec<usize> = (0..b_len + 1).collect();
    let mut prev_distances: Vec<usize> = (0..b_len + 1).collect();
    let mut curr_distances: Vec<usize> = vec![0; b_len + 1];

    let mut prev_a_char = u32::from(char::MAX);
    let mut prev_b_char = u32::from(char::MAX);

    for (i, a_char) in a.iter().enumerate() {
        curr_distances[0] = i + 1;

        for (j, b_char) in b.iter().enumerate() {
            let cost = usize::from(a_char != b_char);
            curr_distances[j + 1] = min(
                curr_distances[j] + 1,
                min(prev_distances[j + 1] + 1, prev_distances[j] + cost),
            );
            if i > 0
                && j > 0
                && a_char != b_char
                && *a_char == prev_b_char
                && *b_char == prev_a_char
            {
                curr_distances[j + 1] = min(curr_distances[j + 1], prev_two_distances[j - 1] + 1);
            }

            prev_b_char = *b_char;
        }

        mem::swap(&mut prev_two_distances, &mut prev_distances);
        mem::swap(&mut prev_distances, &mut curr_distances);
        prev_a_char = *a_char;
    }

    // access prev_distances instead of curr_distances since we swapped
    // them above. In case a is empty this would still contain the correct value
    // from initializing the last element to b_len
    prev_distances[b_len]
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
    let dist = osa_distance(s1, s2);

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
    fn test_distance() {
        let s1 = "a".repeat(65) + "CA" + &"a".repeat(65);
        let s2 = "b".to_owned() + &"a".repeat(64) + "AC" + &"a".repeat(64) + "b";
        assert_eq!(
            super::distance(
                &s1.chars().map(|c| c as u32).collect::<Vec<_>>(),
                &s2.chars().map(|c| c as u32).collect::<Vec<_>>(),
                None
            ),
            3
        );
    }
}
