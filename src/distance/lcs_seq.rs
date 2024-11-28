use crate::common::{common_affix, conv_sequences};
use crate::distance::models::{Editop, Editops, Opcodes};
use num_bigint::BigUint;
use pyo3::prelude::*;
use std::collections::HashMap;

trait CountZeros {
    fn count_zeros(&self) -> u64;
}

impl CountZeros for BigUint {
    fn count_zeros(&self) -> u64 {
        self.bits() - self.count_ones()
    }
}

/**
Calculates the length of the longest common subsequence

Parameters
----------
s1 : &[u32]
    First string to compare.
s2 : &[u32]
    Second string to compare.
score_cutoff : Option<usize>
    Maximum distance between s1 and s2, that is
    considered as a result. If the similarity is smaller than score_cutoff,
    0 is returned instead. Default is None, which deactivates
    this behaviour.

Returns
-------
similarity : f64
    similarity between s1 and s2
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
    let (s1, s2) = match processor {
        Some(proc) => (proc.call1((s1,))?, proc.call1((s2,))?),
        None => (s1.to_owned(), s2.to_owned()),
    };

    if s1.is_empty()? {
        return Ok(0);
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

    let sim = similarity(&s1, &s2, score_cutoff);

    Ok(sim)
}

pub fn similarity(s1: &[u32], s2: &[u32], score_cutoff: Option<usize>) -> usize {
    let len1 = s1.len();

    let b_one = BigUint::from(1u32);

    let mut state = (&b_one << len1) - &b_one;
    let mut block = HashMap::with_capacity(len1);

    let mut position = b_one.clone();
    for &ch1 in s1 {
        block
            .entry(ch1)
            .and_modify(|e| *e |= &position)
            .or_insert_with(|| position.clone());
        position <<= 1u32;
    }

    for &ch2 in s2 {
        let matches = block
            .get(&ch2)
            .cloned()
            .unwrap_or_else(|| BigUint::from(0u32));
        let update = &state & &matches;
        state = (&state + &update) | (&state - &update);
    }

    let result = state.count_zeros() as usize;

    match score_cutoff {
        Some(cutoff) if result >= cutoff => result,
        Some(_) => 0,
        None => result,
    }
}

pub fn block_similarity(
    block: &HashMap<u32, u128>,
    s1: &[u32],
    s2: &[u32],
    score_cutoff: Option<f64>,
) -> u32 {
    let len1 = s1.len();
    if len1 == 0 {
        return 0;
    }

    let shift = 128 - len1;
    let mut state = ((1u128 << len1) - 1) << shift;

    for &ch2 in s2 {
        let matches = block.get(&ch2).copied().unwrap_or(0);
        let update = state & matches;
        state = state.wrapping_add(update) | state.wrapping_sub(update);
    }

    let res = (state.count_zeros() as usize - shift) as u32;

    match score_cutoff {
        Some(cutoff) if (res as f64) >= cutoff => res,
        Some(_) => 0,
        None => res,
    }
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
    let (s1, s2) = match processor {
        Some(proc) => (proc.call1((s1,))?, proc.call1((s2,))?),
        None => (s1.to_owned(), s2.to_owned()),
    };

    if s1.is_empty()? {
        return Ok(0);
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

    let dist = distance(&s1, &s2, score_cutoff);

    Ok(dist)
}

pub fn distance(s1: &[u32], s2: &[u32], score_cutoff: Option<usize>) -> usize {
    let maximum = s1.len().max(s2.len());
    let sim = similarity(s1, s2, None);
    let dist = maximum - sim;
    match score_cutoff {
        Some(cutoff) if dist <= cutoff => dist,
        Some(cutoff) => cutoff + 1,
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
    if s1.is_none() || s2.is_none() {
        return Ok(1.0);
    }

    let (s1, s2) = match processor {
        Some(proc) => (proc.call1((s1,))?, proc.call1((s2,))?),
        None => (s1.to_owned(), s2.to_owned()),
    };

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

    let norm_dist = normalized_distance(&s1, &s2, score_cutoff);

    Ok(norm_dist)
}

pub fn normalized_distance(s1: &[u32], s2: &[u32], score_cutoff: Option<f64>) -> f64 {
    let maximum = s1.len().max(s2.len()) as f64;
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

    let (s1, s2) = match processor {
        Some(proc) => (proc.call1((s1,))?, proc.call1((s2,))?),
        None => (s1.to_owned(), s2.to_owned()),
    };

    let norm_sim = 1.0 - py_normalized_distance(&s1, &s2, None, None)?;
    match score_cutoff {
        Some(cutoff) if norm_sim >= cutoff => Ok(norm_sim),
        Some(_) => Ok(0.0),
        None => Ok(norm_sim),
    }
}

fn matrix(s1: &[u32], s2: &[u32]) -> (usize, Vec<BigUint>) {
    let mut matrix = Vec::new();
    if s1.is_empty() {
        return (0, matrix);
    }

    let len1 = s1.len();
    let b_one = BigUint::from(1u32);
    let mut state = (&b_one << len1) - &b_one;
    let mut block = HashMap::with_capacity(len1);

    let mut position = b_one;
    for &ch1 in s1 {
        block
            .entry(ch1)
            .and_modify(|e: &mut BigUint| *e |= &position)
            .or_insert(position.clone());
        position <<= 1;
    }

    for &ch2 in s2 {
        let matches = block
            .get(&ch2)
            .cloned()
            .unwrap_or_else(|| BigUint::from(0u32));
        let update = &state & &matches;
        state = (&state + &update) | (&state - &update);
        matrix.push(state.clone());
    }

    let sim = (state >> len1).count_zeros() as usize;
    (sim, matrix)
}

#[pyfunction]
#[pyo3(
    name = "editops",
    signature = (s1, s2, *, processor=None)
)]
pub fn py_editops(
    s1: &Bound<'_, PyAny>,
    s2: &Bound<'_, PyAny>,
    processor: Option<&Bound<'_, PyAny>>,
) -> PyResult<Editops> {
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
    let (sim, matrix) = matrix(s1, s2);

    // take the length after adjusting
    let len1 = s1.len();
    let len2 = s2.len();
    let src_len = len1 + prefix_len + suffix_len;
    let dest_len = len2 + prefix_len + suffix_len;

    let dist = len1 + len2 - 2 * sim;
    if dist == 0 {
        return Editops::new(src_len, dest_len, Vec::new());
    }

    let mut editop_vec = Vec::with_capacity(dist);
    let mut col = len1;
    let mut row = len2;

    while col != 0 && row != 0 {
        let mask = BigUint::from(1_u32) << (col - 1);
        // deletion
        if (&matrix[row - 1] & &mask) != BigUint::from(0_u32) {
            col -= 1;
            editop_vec.push(Editop {
                tag: "delete".to_string(),
                src_pos: col + prefix_len,
                dest_pos: row + prefix_len,
            })
        } else {
            row -= 1;

            // insertion
            if row != 0 && (&matrix[row - 1] & &mask) == BigUint::from(0_u32) {
                editop_vec.push(Editop {
                    tag: "insert".to_string(),
                    src_pos: col + prefix_len,
                    dest_pos: row + prefix_len,
                })
            } else {
                col -= 1;
            }
        }
    }

    while col != 0 {
        col -= 1;
        editop_vec.push(Editop {
            tag: "delete".to_string(),
            src_pos: col + prefix_len,
            dest_pos: row + prefix_len,
        })
    }

    while row != 0 {
        row -= 1;
        editop_vec.push(Editop {
            tag: "insert".to_string(),
            src_pos: col + prefix_len,
            dest_pos: row + prefix_len,
        })
    }

    editop_vec.reverse();
    Editops::new(src_len, dest_len, editop_vec)
}

#[pyfunction]
#[pyo3(
    name = "opcodes",
    signature = (s1, s2, *, processor=None)
)]
pub fn py_opcodes(
    s1: &Bound<'_, PyAny>,
    s2: &Bound<'_, PyAny>,
    processor: Option<&Bound<'_, PyAny>>,
) -> PyResult<Opcodes> {
    py_editops(s1, s2, processor).map(|editops| editops.as_opcodes())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_similarity() {
        let s1 = "this is a test";
        let s2 = "this is a test!";
        let (seq1, seq2) = (
            &s1.chars().map(|c| c as u32).collect::<Vec<_>>(),
            &s2.chars().map(|c| c as u32).collect::<Vec<_>>(),
        );

        let result = similarity(seq1, seq2, None);

        assert_eq!(
            result, 14,
            "Expected similarity of 14 for '{}' and '{}', got {}",
            s1, s2, result
        );
    }

    #[test]
    fn test_similarity_unordered() {
        let s1 = "new york mets vs atlanta braves";
        let s2 = "atlanta braves vs new york mets";
        let (seq1, seq2) = (
            &s1.chars().map(|c| c as u32).collect::<Vec<_>>(),
            &s2.chars().map(|c| c as u32).collect::<Vec<_>>(),
        );
        let result = similarity(seq1, seq2, None);
        assert_eq!(result, 14);
    }

    #[test]
    fn test_block_similarity() {
        let s1 = "this is a test";
        let s2 = "this is a test!";
        let (seq1, seq2) = (
            &s1.chars().map(|c| c as u32).collect::<Vec<_>>(),
            &s2.chars().map(|c| c as u32).collect::<Vec<_>>(),
        );
        let mut block = HashMap::new();
        let shift = 128 - seq1.len();
        let mut x = 1u128 << shift;

        for &ch in seq1 {
            block.entry(ch).and_modify(|e| *e |= &x).or_insert(x);
            x <<= 1;
        }
        let result = block_similarity(&block, seq1, seq2, None);

        assert_eq!(
            result, 14,
            "Expected similarity of 14 for '{}' and '{}', got {}",
            s1, s2, result
        );
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
                        tag: "insert".to_string(),
                        src_pos: 3,
                        dest_pos: 2
                    },
                    Editop {
                        tag: "delete".to_string(),
                        src_pos: 3,
                        dest_pos: 3
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
    fn test_editops_longer() {
        let s1 = r#"\\U0002f232´\\xad𑌓7¡HD眜#\\x1e\\U00068e5fs\\U000671f3;ûñ\\x14ÒDT\\U000ba178Ñ17\\U000cc06aø2\\U000cd6fc\\U00074064\\x9a𣈯i\\x19\\x15\\x9bû\\nSs§\\x06𠧇×%az\\U00019ef90¸¬\\ U000d7ccbv\\x17bï\\U000c4889ñß\\x0c\\x0e\\x04\\x8b¿\\x8d\\x05"#;
        let s2 = "00000000000000000000000000000000000000000000000000000000000000000";
        let (seq1, seq2) = (
            &s1.chars().map(|c| c as u32).collect::<Vec<_>>(),
            &s2.chars().map(|c| c as u32).collect::<Vec<_>>(),
        );
        let result = editops(seq1, seq2);
        assert!(result.len() != 0);
    }

    #[test]
    fn test_similarity_boundary() {
        let s1 = "00000000000000000000000000000000000000000000000000000000000000000";
        let s2 = "00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000";
        let (seq1, seq2) = (
            &s1.chars().map(|c| c as u32).collect::<Vec<_>>(),
            &s2.chars().map(|c| c as u32).collect::<Vec<_>>(),
        );
        let result1 = similarity(seq1, seq2, None);
        let result2 = similarity(seq2, seq1, None);
        assert_eq!(result1, result2);
    }
}
