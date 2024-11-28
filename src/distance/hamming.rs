use crate::common::error::CrustyError;
use crate::distance::models::{Editop, Editops, Opcodes};
use crate::distance::prep_inputs;
use pyo3::prelude::*;

#[pyfunction]
#[pyo3(
    name = "distance",
    signature = (s1, s2, *, pad=true, processor=None, score_cutoff=None)
)]
pub fn py_distance(
    s1: &Bound<'_, PyAny>,
    s2: &Bound<'_, PyAny>,
    pad: Option<bool>,
    processor: Option<&Bound<'_, PyAny>>,
    score_cutoff: Option<usize>,
) -> PyResult<usize> {
    let pad = pad.unwrap_or(true);
    let (s1, s2) = prep_inputs(s1, s2, processor)?;
    let res = distance(&s1, &s2, pad, score_cutoff);

    match res {
        Ok(res) => Ok(res),
        Err(CrustyError::LengthMismatch) => Err(pyo3::exceptions::PyValueError::new_err(
            "Sequences are not the same length.",
        )),
    }
}

fn distance(
    s1: &[u32],
    s2: &[u32],
    pad: bool,
    score_cutoff: Option<usize>,
) -> std::result::Result<usize, CrustyError> {
    let len1 = s1.len();
    let len2 = s2.len();

    if !pad && len1 != len2 {
        return Err(CrustyError::LengthMismatch);
    }

    let min_len = len1.min(len2);
    let mut distance = len1.max(len2);
    for i in 0..min_len {
        distance -= (s1[i] == s2[i]) as usize;
    }

    match score_cutoff {
        Some(cutoff) if distance <= cutoff => Ok(distance),
        Some(cutoff) => Ok(cutoff + 1),
        None => Ok(distance),
    }
}

#[pyfunction]
#[pyo3(
    name = "similarity",
    signature = (s1, s2, *, pad=true, processor=None, score_cutoff=None)
)]
pub fn py_similarity(
    s1: &Bound<'_, PyAny>,
    s2: &Bound<'_, PyAny>,
    pad: Option<bool>,
    processor: Option<&Bound<'_, PyAny>>,
    score_cutoff: Option<usize>,
) -> PyResult<usize> {
    let pad = pad.unwrap_or(true);
    let (s1, s2) = prep_inputs(s1, s2, processor)?;
    let sim = similarity(&s1, &s2, pad, score_cutoff).map_err(|_| {
        pyo3::exceptions::PyValueError::new_err("Sequences are not the same length.")
    })?;

    Ok(sim)
}

fn similarity(
    s1: &[u32],
    s2: &[u32],
    pad: bool,
    score_cutoff: Option<usize>,
) -> Result<usize, CrustyError> {
    let maximum = s1.len().max(s2.len());
    let dist = distance(s1, s2, pad, score_cutoff)?;
    let sim = maximum - dist;

    match score_cutoff {
        Some(cutoff) if sim >= cutoff => Ok(sim),
        Some(_) => Ok(0),
        None => Ok(sim),
    }
}

#[pyfunction]
#[pyo3(
    name = "normalized_distance",
    signature = (s1, s2, *, pad=true, processor=None, score_cutoff=None)
)]
pub fn py_normalized_distance(
    s1: &Bound<'_, PyAny>,
    s2: &Bound<'_, PyAny>,
    pad: Option<bool>,
    processor: Option<&Bound<'_, PyAny>>,
    score_cutoff: Option<f64>,
) -> PyResult<f64> {
    let pad = pad.unwrap_or(true);
    let (s1, s2) = prep_inputs(s1, s2, processor)?;
    let norm_dist = normalized_distance(&s1, &s2, pad, score_cutoff).map_err(|_| {
        pyo3::exceptions::PyValueError::new_err("Sequences are not the same length.")
    })?;

    Ok(norm_dist)
}

fn normalized_distance(
    s1: &[u32],
    s2: &[u32],
    pad: bool,
    score_cutoff: Option<f64>,
) -> Result<f64, CrustyError> {
    let maximum = s1.len().max(s2.len()) as f64;
    let norm_dist = if maximum == 0.0 {
        0.0
    } else {
        let dist = distance(s1, s2, pad, None)? as f64;
        dist / maximum
    };

    match score_cutoff {
        Some(cutoff) if norm_dist <= cutoff => Ok(norm_dist),
        Some(_) => Ok(1.0),
        None => Ok(norm_dist),
    }
}

#[pyfunction]
#[pyo3(
    name = "normalized_similarity",
    signature = (s1, s2, *, pad=true, processor=None, score_cutoff=None)
)]
pub fn py_normalized_similarity(
    s1: &Bound<'_, PyAny>,
    s2: &Bound<'_, PyAny>,
    pad: Option<bool>,
    processor: Option<&Bound<'_, PyAny>>,
    score_cutoff: Option<f64>,
) -> PyResult<f64> {
    let pad = pad.unwrap_or(true);
    let (s1, s2) = prep_inputs(s1, s2, processor)?;
    let norm_sim = normalized_similarity(&s1, &s2, pad, score_cutoff).map_err(|_| {
        pyo3::exceptions::PyValueError::new_err("Sequences are not the same length.")
    })?;

    Ok(norm_sim)
}

fn normalized_similarity(
    s1: &[u32],
    s2: &[u32],
    pad: bool,
    score_cutoff: Option<f64>,
) -> Result<f64, CrustyError> {
    let norm_dist = normalized_distance(s1, s2, pad, None)?;
    let norm_sim = 1.0 - norm_dist;

    match score_cutoff {
        Some(cutoff) if norm_sim >= cutoff => Ok(norm_sim),
        Some(_) => Ok(0.0),
        None => Ok(norm_sim),
    }
}

#[pyfunction]
#[pyo3(
    name = "editops",
    signature = (s1, s2, *, pad=true, processor=None)
)]
pub fn py_editops(
    s1: &Bound<'_, PyAny>,
    s2: &Bound<'_, PyAny>,
    pad: Option<bool>,
    processor: Option<&Bound<'_, PyAny>>,
) -> PyResult<Editops> {
    let pad = pad.unwrap_or(true);
    let (s1, s2) = prep_inputs(s1, s2, processor)?;
    let editops = editops(&s1, &s2, pad).map_err(|_| {
        pyo3::exceptions::PyValueError::new_err("Sequences are not the same length.")
    })?;

    Ok(editops)
}

fn editops(s1: &[u32], s2: &[u32], pad: bool) -> Result<Editops, CrustyError> {
    if !pad && s1.len() != s2.len() {
        return Err(CrustyError::LengthMismatch);
    }

    let mut ops_vec = Vec::new();
    let min_len = s1.len().min(s2.len());

    for i in 0..min_len {
        if s1[i] != s2[i] {
            ops_vec.push(Editop {
                tag: "replace".to_string(),
                src_pos: i,
                dest_pos: i,
            });
        }
    }

    for i in min_len..s1.len() {
        ops_vec.push(Editop {
            tag: "delete".to_string(),
            src_pos: i,
            dest_pos: s2.len(),
        })
    }

    for i in min_len..s2.len() {
        ops_vec.push(Editop {
            tag: "insert".to_string(),
            src_pos: s1.len(),
            dest_pos: i,
        })
    }

    Ok(Editops::new(s1.len(), s2.len(), ops_vec))
}

#[pyfunction]
#[pyo3(
    name = "opcodes",
    signature = (s1, s2, *, pad=true, processor=None)
)]
pub fn py_opcodes(
    s1: &Bound<'_, PyAny>,
    s2: &Bound<'_, PyAny>,
    pad: Option<bool>,
    processor: Option<&Bound<'_, PyAny>>,
) -> PyResult<Opcodes> {
    let pad = pad.unwrap_or(true);
    let (s1, s2) = prep_inputs(s1, s2, processor)?;
    let opcodes = opcodes(&s1, &s2, pad).map_err(|_| {
        pyo3::exceptions::PyValueError::new_err("Sequences are not the same length.")
    })?;

    Ok(opcodes)
}

fn opcodes(s1: &[u32], s2: &[u32], pad: bool) -> Result<Opcodes, CrustyError> {
    Ok(editops(s1, s2, pad)?.as_opcodes())
}
