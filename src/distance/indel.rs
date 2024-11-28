use crate::common::conv_sequences;
use crate::distance::lcs_seq::{
    block_similarity as lcs_seq_block_similarity, py_editops as lcs_seq_py_editops,
    py_opcodes as lcs_seq_py_opcodes, similarity as lcs_seq_similarity,
};
use crate::distance::models::{Editops, Opcodes};
use pyo3::prelude::*;
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

\>>> from rapidfuzz.distance import Indel
\>>> Indel.distance("lewenstein", "levenshtein")
3

Setting a maximum distance allows the implementation to select
a more efficient implementation:

\>>> Indel.distance("lewenstein", "levenshtein", score_cutoff=1)
2
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

    let dist = distance(&s1, &s2, score_cutoff);

    Ok(dist)
}

pub fn distance(s1: &[u32], s2: &[u32], score_cutoff: Option<usize>) -> usize {
    let maximum = s1.len() + s2.len();
    let lcs_sim = if s1.is_empty() {
        0
    } else {
        lcs_seq_similarity(s1, s2, None)
    };
    let dist = maximum - 2 * lcs_sim;

    match score_cutoff {
        Some(cutoff) if dist <= cutoff => dist,
        Some(cutoff) => cutoff + 1,
        None => dist,
    }
}

pub fn block_distance(
    block: &HashMap<u32, u128>,
    s1: &[u32],
    s2: &[u32],
    score_cutoff: Option<f64>,
) -> u32 {
    let maximum = (s1.len() + s2.len()) as u32;
    let lcs_sim = lcs_seq_block_similarity(block, s1, s2, None);
    let dist = maximum - 2 * lcs_sim;

    match score_cutoff {
        Some(cutoff) if dist as f64 <= cutoff => dist,
        Some(cutoff) => cutoff as u32 + 1,
        None => dist,
    }
}

/**
Calculates the Indel similarity in the range [max, 0].

This is calculated as ``(len1 + len2) - distance``.

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
similarity : int
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
    let maximum = s1.len() + s2.len();
    let dist = distance(s1, s2, None);
    let sim = maximum - dist;
    match score_cutoff {
        Some(cutoff) if sim >= cutoff => sim,
        Some(_) => 0,
        None => sim,
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
score_cutoff : float, optional
    Optional argument for a score threshold as a float between 0 and 1.0.
    For norm_dist > score_cutoff 1.0 is returned instead. Default is 1.0,
    which deactivates this behaviour.

Returns
-------
norm_dist : float
    normalized distance between s1 and s2 as a float between 0 and 1.0
*/
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
    let maximum = (s1.len() + s2.len()) as f64;
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

pub fn block_normalized_distance(
    block: &HashMap<u32, u128>,
    s1: &[u32],
    s2: &[u32],
    score_cutoff: Option<f64>,
) -> f64 {
    let maximum = (s1.len() + s2.len()) as f64;
    let norm_dist = if maximum == 0.0 {
        0.0
    } else {
        let dist = block_distance(block, s1, s2, None) as f64;
        dist / maximum
    };

    match score_cutoff {
        Some(cutoff) if norm_dist <= cutoff => norm_dist,
        Some(_) => 1.0,
        None => norm_dist,
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

\>>> from rapidfuzz.distance import Indel
\>>> Indel.normalized_similarity("lewenstein", "levenshtein")
0.85714285714285

Setting a score_cutoff allows the implementation to select
a more efficient implementation:

\>>> Indel.normalized_similarity("lewenstein", "levenshtein", score_cutoff=0.9)
0.0

When a different processor is used s1 and s2 do not have to be strings

\>>> Indel.normalized_similarity(["lewenstein"], ["levenshtein"], processor=lambda s: s[0])
0.8571428571428572
*/
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

    Ok(normalized_similarity(&s1, &s2, score_cutoff))
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

pub fn block_normalized_similarity(
    block: &HashMap<u32, u128>,
    s1: &[u32],
    s2: &[u32],
    score_cutoff: Option<f64>,
) -> f64 {
    let norm_dist = block_normalized_distance(block, s1, s2, None);
    let norm_sim = 1.0 - norm_dist;

    match score_cutoff {
        Some(cutoff) if norm_sim >= cutoff => norm_sim,
        Some(_) => 0.0,
        None => norm_sim,
    }
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
    lcs_seq_py_editops(s1, s2, processor)
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
    lcs_seq_py_opcodes(s1, s2, processor)
}

#[cfg(test)]
mod tests {
    use super::*;
    use pyo3::types::PyString;

    #[test]
    fn test_normalized_similarity() {
        let s1 = "lewenstein".chars().map(|c| c as u32).collect::<Vec<_>>();
        let s2 = "levenshtein".chars().map(|c| c as u32).collect::<Vec<_>>();
        let result = normalized_similarity(&s1, &s2, None);
        assert_eq!(result, 0.8571428571428572);
    }

    #[test]
    fn test_normalized_similarity_with_cutoff() {
        let s1 = "lewenstein".chars().map(|c| c as u32).collect::<Vec<_>>();
        let s2 = "levenshtein".chars().map(|c| c as u32).collect::<Vec<_>>();
        let result = normalized_similarity(&s1, &s2, Some(0.0));
        assert_eq!(result, 0.8571428571428572);
    }

    #[test]
    fn test_normalized_similarity_unordered() {
        let s1 = "new york mets vs atlanta braves"
            .chars()
            .map(|c| c as u32)
            .collect::<Vec<_>>();
        let s2 = "atlanta braves vs new york mets"
            .chars()
            .map(|c| c as u32)
            .collect::<Vec<_>>();
        let result = normalized_similarity(&s1, &s2, None);
        assert!(
            (result - 0.45161290322580).abs() < 1e-5,
            "Expected 0.45161290322580, got {}",
            result
        );
    }

    #[test]
    fn test_normalized_distance_unordered() {
        let s1 = "new york mets vs atlanta braves"
            .chars()
            .map(|c| c as u32)
            .collect::<Vec<_>>();
        let s2 = "atlanta braves vs new york mets"
            .chars()
            .map(|c| c as u32)
            .collect::<Vec<_>>();
        let result = normalized_distance(&s1, &s2, None);
        assert!(
            (result - (1.0 - 0.45161290322580)).abs() < 1e-5,
            "Expected (1.0 - 0.45161290322580), got {}",
            result
        );
    }

    #[test]
    fn test_empty_input() {
        pyo3::prepare_freethreaded_python();
        Python::with_gil(|py| {
            let s1 = PyString::new_bound(py, "");
            let s2 = PyString::new_bound(py, "");
            let result = py_distance(&s1, &s2, None, None).unwrap();
            assert_eq!(result, 0);
        });
    }
}
