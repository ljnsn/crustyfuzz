use crate::common::{common_affix, conv_sequences};
use crate::distance::indel::distance as indel_distance;
use crate::distance::models::{Editop, Editops, Opcodes};
use num_bigint::BigUint;
use pyo3::prelude::*;
use std::collections::HashMap;

#[derive(FromPyObject)]
pub struct Weights(usize, usize, usize);

fn levenshtein_maximum(s1: &[u32], s2: &[u32], weights: &Weights) -> usize {
    let len1 = s1.len();
    let len2 = s2.len();
    let (insert, delete, replace) = (weights.0, weights.1, weights.2);

    let max_dist = len1 * delete + len2 * insert;

    if len1 >= len2 {
        max_dist.min(len2 * replace + (len1 - len2) * delete)
    } else {
        max_dist.min(len1 * replace + (len2 - len1) * insert)
    }
}

fn uniform_generic(s1: &[u32], s2: &[u32], weights: Weights) -> usize {
    let len1 = s1.len();
    let (insert, delete, replace) = (weights.0, weights.1, weights.2);
    let mut cache = (0..=(len1 * delete)).step_by(delete).collect::<Vec<_>>();

    for &ch2 in s2 {
        let mut temp = cache[0];
        cache[0] += insert;
        for i in 0..len1 {
            let mut x = temp;
            if s1[i] != ch2 {
                x = (usize::min(cache[i] + delete, cache[i + 1] + insert)).min(temp + replace);
            }
            temp = cache[i + 1];
            cache[i + 1] = x;
        }
    }

    cache[cache.len() - 1]
}

// fn uniform_distance(s1: &[u32], s2: &[u32]) -> usize {
//     let len1 = s1.len();
//     let len2 = s2.len();
//
//     if len1 == 0 {
//         return len2;
//     }
//
//     let all_ones = (&BigUint::from(1u32) << len1) - BigUint::from(1u32);
//
//     let mut vertical_positive = all_ones.clone();
//     let mut vertical_negative = BigUint::from(0_u32);
//     let mut current_dist = len1;
//     let mask = &BigUint::from(1_u32) << (len1 - 1);
//
//     let mut block = HashMap::with_capacity(len1);
//     let mut position = BigUint::from(1_u32);
//     for &ch1 in s1 {
//         block
//             .entry(ch1)
//             .and_modify(|e: &mut BigUint| *e |= &position)
//             .or_insert(position.clone());
//         position <<= 1;
//     }
//
//     let b_zero = BigUint::from(0_u32);
//     for &ch2 in s2 {
//         let pattern_match = block.get(&ch2).unwrap_or(&b_zero);
//         let matches = pattern_match;
//         let diagonal_zero = (((matches & &vertical_positive) + &vertical_positive)
//             ^ &vertical_positive)
//             | matches
//             | &vertical_negative;
//         let mut horizontal_positive =
//             &vertical_negative | (&all_ones - (&diagonal_zero | &vertical_positive));
//         let mut horizontal_negative = &diagonal_zero & &vertical_positive;
//         current_dist += (&horizontal_positive & &mask != BigUint::from(0_u32)) as usize;
//         current_dist -= (&horizontal_negative & &mask != BigUint::from(0_u32)) as usize;
//         horizontal_positive = (&horizontal_positive << 1) | BigUint::from(1_u32);
//         horizontal_negative <<= 1;
//         vertical_positive =
//             &horizontal_negative | (&all_ones - (&diagonal_zero | &horizontal_positive));
//         vertical_negative = &horizontal_positive & &diagonal_zero;
//     }
//
//     current_dist
// }

fn uniform_distance(s1: &[u32], s2: &[u32]) -> usize {
    let len1 = s1.len();
    let len2 = s2.len();

    if len1 == 0 {
        return len2;
    }

    let b_zero = BigUint::from(0u32);
    let b_one = BigUint::from(1u32);

    let all_ones = (&b_one << len1) - &b_one;
    let mut vp = all_ones.clone();
    let mut vn = b_zero.clone();
    let mut current_dist = len1;
    let mask = &b_one << (len1 - 1);

    let mut block = HashMap::with_capacity(len1);
    let mut x = b_one.clone();
    for &ch1 in s1 {
        block
            .entry(ch1)
            .and_modify(|e: &mut BigUint| *e |= &x)
            .or_insert(x.clone());
        x <<= 1;
    }

    for &ch2 in s2 {
        // Step 1: Computing D0
        let pm_j = block.get(&ch2).unwrap_or(&b_zero);
        let x = pm_j;
        let d0 = (((x & &vp) + &vp) ^ &vp) | x | &vn;

        // Step 2: Computing HP and HN
        let mut hp = &vn | (&all_ones ^ (&d0 | &vp));
        let mut hn = &d0 & &vp;

        // Step 3: Computing the value D[m,j]
        current_dist += (&hp & &mask != b_zero) as usize;
        current_dist -= (&hn & &mask != b_zero) as usize;

        // Step 4: Computing VP and VN
        hp = (&hp << 1) | &b_one;
        hn <<= 1;
        vp = &hn | (&all_ones ^ (&d0 | &hp));
        vn = &hp & &d0;
    }

    current_dist
}
/**
Calculates the minimum number of insertions, deletions, and substitutions
required to change one sequence into the other according to Levenshtein with custom
costs for insertion, deletion and substitution

Parameters
----------
s1 : Sequence[Hashable]
    First string to compare.
s2 : Sequence[Hashable]
    Second string to compare.
weights : Tuple[int, int, int] or None, optional
    The weights for the three operations in the form
    (insertion, deletion, substitution). Default is (1, 1, 1),
    which gives all three operations a weight of 1.
processor : callable, optional
    Optional callable that is used to preprocess the strings before
    comparing them. Default is None, which deactivates this behaviour.
score_cutoff : int, optional
    Maximum distance between s1 and s2, that is
    considered as a result. If the distance is bigger than score_cutoff,
    score_cutoff + 1 is returned instead. Default is None, which deactivates
    this behaviour.
score_hint : int, optional
    Expected distance between s1 and s2. This is used to select a
    faster implementation. Default is None, which deactivates this behaviour.

Returns
-------
distance : int
    distance between s1 and s2

Raises
------
ValueError
    If unsupported weights are provided a ValueError is thrown

Examples
--------
Find the Levenshtein distance between two strings:

\>>> from rapidfuzz.distance import Levenshtein
\>>> Levenshtein.distance("lewenstein", "levenshtein")
2

Setting a maximum distance allows the implementation to select
a more efficient implementation:

\>>> Levenshtein.distance("lewenstein", "levenshtein", score_cutoff=1)
2

It is possible to select different weights by passing a `weight`
tuple.

\>>> Levenshtein.distance("lewenstein", "levenshtein", weights=(1,1,2))
3
*/
#[pyfunction]
#[pyo3(
    name = "distance",
    signature = (s1, s2, *, weights=Weights(1, 1, 1), processor=None, score_cutoff=None, score_hint=None)
)]
pub fn py_distance(
    s1: &Bound<'_, PyAny>,
    s2: &Bound<'_, PyAny>,
    weights: Option<Weights>,
    processor: Option<&Bound<'_, PyAny>>,
    score_cutoff: Option<usize>,
    score_hint: Option<usize>,
) -> PyResult<usize> {
    // save for later use
    let _ = score_hint;

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

    let dist = distance(&s1, &s2, weights, score_cutoff);

    Ok(dist)
}

pub fn distance(
    s1: &[u32],
    s2: &[u32],
    weights: Option<Weights>,
    score_cutoff: Option<usize>,
) -> usize {
    let dist = match weights {
        None | Some(Weights(1, 1, 1)) => uniform_distance(s1, s2),
        Some(Weights(1, 1, 2)) => {
            if s1.is_empty() {
                return 0;
            }
            indel_distance(s1, s2, None)
        }
        _ => uniform_generic(s1, s2, weights.unwrap()),
    };

    match score_cutoff {
        Some(cutoff) if dist <= cutoff => dist,
        Some(cutoff) => cutoff + 1,
        None => dist,
    }
}

/**
Calculates the levenshtein similarity in the range [max, 0] using custom
costs for insertion, deletion and substitution.

This is calculated as ``max - distance``, where max is the maximal possible
Levenshtein distance given the lengths of the sequences s1/s2 and the weights.

Parameters
----------
s1 : Sequence[Hashable]
    First string to compare.
s2 : Sequence[Hashable]
    Second string to compare.
weights : Tuple[int, int, int] or None, optional
    The weights for the three operations in the form
    (insertion, deletion, substitution). Default is (1, 1, 1),
    which gives all three operations a weight of 1.
processor : callable, optional
    Optional callable that is used to preprocess the strings before
    comparing them. Default is None, which deactivates this behaviour.
score_cutoff : int, optional
    Maximum distance between s1 and s2, that is
    considered as a result. If the similarity is smaller than score_cutoff,
    0 is returned instead. Default is None, which deactivates
    this behaviour.
score_hint : int, optional
    Expected similarity between s1 and s2. This is used to select a
    faster implementation. Default is None, which deactivates this behaviour.

Returns
-------
similarity : int
    similarity between s1 and s2

Raises
------
ValueError
    If unsupported weights are provided a ValueError is thrown
*/
#[pyfunction]
#[pyo3(
    name = "similarity",
    signature = (s1, s2, *, weights=Weights(1, 1, 1), processor=None, score_cutoff=None, score_hint=None)
)]
pub fn py_similarity(
    s1: &Bound<'_, PyAny>,
    s2: &Bound<'_, PyAny>,
    weights: Option<Weights>,
    processor: Option<&Bound<'_, PyAny>>,
    score_cutoff: Option<usize>,
    score_hint: Option<usize>,
) -> PyResult<usize> {
    // save for later use
    let _ = score_hint;

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

    let weights = weights.unwrap_or(Weights(1, 1, 1));
    let sim = similarity(&s1, &s2, weights, score_cutoff);

    Ok(sim)
}

pub fn similarity(s1: &[u32], s2: &[u32], weights: Weights, score_cutoff: Option<usize>) -> usize {
    let maximum = levenshtein_maximum(s1, s2, &weights);
    let dist = distance(s1, s2, Some(weights), None);
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
    signature = (s1, s2, *, weights=Weights(1, 1, 1), processor=None, score_cutoff=None, score_hint=None)
)]
pub fn py_normalized_distance(
    s1: &Bound<'_, PyAny>,
    s2: &Bound<'_, PyAny>,
    weights: Option<Weights>,
    processor: Option<&Bound<'_, PyAny>>,
    score_cutoff: Option<f64>,
    score_hint: Option<usize>,
) -> PyResult<f64> {
    // save for later use
    let _ = score_hint;

    if s1.is_none() || s2.is_none() {
        return Ok(1.0);
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

    let weights = weights.unwrap_or(Weights(1, 1, 1));
    let norm_dist = normalized_distance(&s1, &s2, weights, score_cutoff);

    Ok(norm_dist)
}

pub fn normalized_distance(
    s1: &[u32],
    s2: &[u32],
    weights: Weights,
    score_cutoff: Option<f64>,
) -> f64 {
    let maximum = levenshtein_maximum(s1, s2, &weights);
    let dist = distance(s1, s2, Some(weights), None);
    let norm_dist = match maximum {
        0 => 0.0,
        _ => dist as f64 / maximum as f64,
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
    signature = (s1, s2, *, weights=Weights(1, 1, 1), processor=None, score_cutoff=None, score_hint=None)
)]
pub fn py_normalized_similarity(
    s1: &Bound<'_, PyAny>,
    s2: &Bound<'_, PyAny>,
    weights: Option<Weights>,
    processor: Option<&Bound<'_, PyAny>>,
    score_cutoff: Option<f64>,
    score_hint: Option<usize>,
) -> PyResult<f64> {
    // save for later use
    let _ = score_hint;

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

    let weights = weights.unwrap_or(Weights(1, 1, 1));
    let norm_sim = normalized_similarity(&s1, &s2, weights, score_cutoff);

    Ok(norm_sim)
}

pub fn normalized_similarity(
    s1: &[u32],
    s2: &[u32],
    weights: Weights,
    score_cutoff: Option<f64>,
) -> f64 {
    let norm_dist = normalized_distance(s1, s2, weights, None);
    let norm_sim = 1.0 - norm_dist;

    match score_cutoff {
        Some(cutoff) if norm_sim >= cutoff => norm_sim,
        Some(_) => 0.0,
        None => norm_sim,
    }
}

fn matrix(s1: &[u32], s2: &[u32]) -> (usize, Vec<BigUint>, Vec<BigUint>) {
    let len1 = s1.len();
    let len2 = s2.len();
    if len1 == 0 {
        return (len2, Vec::new(), Vec::new());
    }

    let b_zero = BigUint::from(0u32);
    let b_one = BigUint::from(1u32);

    let all_ones = (&b_one << len1) - &b_one;
    let mut vertical_positive = all_ones.clone();
    let mut vertical_negative = b_zero.clone();
    let mut current_dist = len1;
    let mask = BigUint::from(1_u32) << (len1 - 1);

    let mut block = HashMap::with_capacity(len1);
    let mut position = BigUint::from(1_u32);
    for &ch1 in s1 {
        block
            .entry(ch1)
            .and_modify(|e: &mut BigUint| *e |= &position)
            .or_insert(position.clone());
        position <<= 1;
    }

    let mut matrix_vp = Vec::new();
    let mut matrix_vn = Vec::new();
    for &ch2 in s2 {
        let pattern_match = block.get(&ch2).unwrap_or(&b_zero);
        let matches = pattern_match;
        let diagonal_zero = ((matches & &vertical_positive) + &vertical_positive)
            ^ &vertical_positive
            | matches
            | &vertical_negative;

        let mut horizontal_positive =
            &vertical_negative | (&all_ones ^ (&diagonal_zero | &vertical_positive));
        let mut horizontal_negative = &diagonal_zero & &vertical_positive;

        current_dist += (&horizontal_positive & &mask != b_zero) as usize;
        current_dist -= (&horizontal_negative & &mask != b_zero) as usize;

        horizontal_positive = (horizontal_positive << 1) | &b_one;
        horizontal_negative <<= 1;
        vertical_positive =
            &horizontal_negative | (&all_ones ^ (&diagonal_zero | &horizontal_positive));
        vertical_negative = &horizontal_positive & &diagonal_zero;
        matrix_vp.push(vertical_positive.clone());
        matrix_vn.push(vertical_negative.clone());
    }

    (current_dist, matrix_vp, matrix_vn)
}

/**
Return Editops describing how to turn s1 into s2.

Parameters
----------
s1 : Sequence[Hashable]
    First string to compare.
s2 : Sequence[Hashable]
    Second string to compare.
processor : callable, optional
    Optional callable that is used to preprocess the strings before
    comparing them. Default is None, which deactivates this behaviour.
score_hint : int, optional
    Expected distance between s1 and s2. This is used to select a
    faster implementation. Default is None, which deactivates this behaviour.

Returns
-------
editops : Editops
    edit operations required to turn s1 into s2

Notes
-----
The alignment is calculated using an algorithm of Heikki Hyyrö, which is
described [8]_. It has a time complexity and memory usage of ``O([N/64] * M)``.

References
----------
.. [8] Hyyrö, Heikki. "A Note on Bit-Parallel Alignment Computation."
        Stringology (2004).

Examples
--------
\>>> from rapidfuzz.distance import Levenshtein
\>>> for tag, src_pos, dest_pos in Levenshtein.editops("qabxcd", "abycdf"):
...    print(("%7s s1[%d] s2[%d]" % (tag, src_pos, dest_pos)))
    delete s1[1] s2[0]
replace s1[3] s2[2]
    insert s1[6] s2[5]
*/
#[pyfunction]
#[pyo3(
    name = "editops",
    signature = (s1, s2, *, processor=None, score_hint=None)
)]
pub fn py_editops(
    s1: &Bound<'_, PyAny>,
    s2: &Bound<'_, PyAny>,
    processor: Option<&Bound<'_, PyAny>>,
    score_hint: Option<usize>,
) -> PyResult<Editops> {
    // save for later use
    let _ = score_hint;

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

    Ok(editops(&s1, &s2))
}

pub fn editops(s1: &[u32], s2: &[u32]) -> Editops {
    let (prefix_len, suffix_len) = common_affix(s1, s2);
    let s1 = &s1[prefix_len..s1.len() - suffix_len];
    let s2 = &s2[prefix_len..s2.len() - suffix_len];
    let (dist, matrix_vp, matrix_vn) = matrix(s1, s2);

    // take the length after adjusting
    let len1 = s1.len();
    let len2 = s2.len();
    let src_len = len1 + prefix_len + suffix_len;
    let dest_len = len2 + prefix_len + suffix_len;

    if dist == 0 {
        return Editops::new(src_len, dest_len, Vec::new());
    }

    let mut editop_vec = Vec::with_capacity(dist);
    let mut col = len1;
    let mut row = len2;
    while row != 0 && col != 0 {
        let point_mask = BigUint::from(1_u32) << (col - 1);
        // deletion
        if (&matrix_vp[row - 1] & &point_mask) != BigUint::from(0_u32) {
            col -= 1;
            editop_vec.push(Editop {
                tag: "delete".to_string(),
                src_pos: col + prefix_len,
                dest_pos: row + prefix_len,
            });
        } else {
            row -= 1;

            // insertion
            if (row != 0) && (&matrix_vn[row - 1] & &point_mask) != BigUint::from(0_u32) {
                editop_vec.push(Editop {
                    tag: "insert".to_string(),
                    src_pos: col + prefix_len,
                    dest_pos: row + prefix_len,
                });
            } else {
                col -= 1;

                // replace (matches are not recorded)
                if s1[col] != s2[row] {
                    editop_vec.push(Editop {
                        tag: "replace".to_string(),
                        src_pos: col + prefix_len,
                        dest_pos: row + prefix_len,
                    });
                }
            }
        }
    }

    while col != 0 {
        col -= 1;
        editop_vec.push(Editop {
            tag: "delete".to_string(),
            src_pos: col + prefix_len,
            dest_pos: row + prefix_len,
        });
    }

    while row != 0 {
        row -= 1;
        editop_vec.push(Editop {
            tag: "insert".to_string(),
            src_pos: col + prefix_len,
            dest_pos: row + prefix_len,
        });
    }

    editop_vec.reverse();
    Editops::new(src_len, dest_len, editop_vec)
}

#[pyfunction]
#[pyo3(
    name = "opcodes",
    signature = (s1, s2, *, processor=None, score_hint=None)
)]
pub fn py_opcodes(
    s1: &Bound<'_, PyAny>,
    s2: &Bound<'_, PyAny>,
    processor: Option<&Bound<'_, PyAny>>,
    score_hint: Option<usize>,
) -> PyResult<Opcodes> {
    let editops = py_editops(s1, s2, processor, score_hint)?;
    Ok(editops.as_opcodes())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_distance() {
        let s1 = "00000000000000000000000000000000000000000000000000000000000000000";
        let s2 = "00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000";

        let (seq1, seq2) = (
            &s1.chars().map(|c| c as u32).collect::<Vec<_>>(),
            &s2.chars().map(|c| c as u32).collect::<Vec<_>>(),
        );
        let result = distance(seq1, seq2, None, None);
        assert_eq!(result, 63)
    }

    #[test]
    fn test_editops() {
        let s1 = "00";
        let s2 = "0";
        let (seq1, seq2) = (
            &s1.chars().map(|c| c as u32).collect::<Vec<_>>(),
            &s2.chars().map(|c| c as u32).collect::<Vec<_>>(),
        );
        let result = editops(seq1, seq2);
        assert_eq!(
            result,
            Editops::new(
                2,
                1,
                vec![Editop {
                    tag: "delete".to_string(),
                    src_pos: 1,
                    dest_pos: 1
                }]
            )
        );
    }

    #[test]
    fn test_editops_long() {
        let s1 = "qabxcd";
        let s2 = "abycdf";
        let (seq1, seq2) = (
            &s1.chars().map(|c| c as u32).collect::<Vec<_>>(),
            &s2.chars().map(|c| c as u32).collect::<Vec<_>>(),
        );
        let result = editops(seq1, seq2);
        assert_eq!(
            result,
            Editops::new(
                6,
                6,
                vec![
                    Editop {
                        tag: "delete".to_string(),
                        src_pos: 0,
                        dest_pos: 0
                    },
                    Editop {
                        tag: "replace".to_string(),
                        src_pos: 3,
                        dest_pos: 2
                    },
                    Editop {
                        tag: "insert".to_string(),
                        src_pos: 6,
                        dest_pos: 5
                    }
                ]
            )
        );
    }

    #[test]
    fn test_editops_other() {
        let s1 = r"»CXÎ\U0007b233H×𠿯¬";
        let s2 = r"zÎ\U000fb154£m";
        let (seq1, seq2) = (
            &s1.chars().map(|c| c as u32).collect::<Vec<_>>(),
            &s2.chars().map(|c| c as u32).collect::<Vec<_>>(),
        );
        let result = editops(seq1, seq2);
        dbg!(&result);
        assert_eq!(
            result.as_matching_blocks(),
            result.as_opcodes().as_matching_blocks(),
        );
    }
}
