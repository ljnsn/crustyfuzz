use pyo3::prelude::*;
mod common;
pub mod fuzz;
mod indel;
mod lcs_seq;

fn call_processor(processor: &Bound<'_, PyAny>, s: Option<&str>) -> Result<String, PyErr> {
    let res = processor.call1((s,))?;
    res.extract::<String>()
}

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

#[pyfunction]
#[pyo3(
    signature = (s1, s2, processor=None, score_cutoff=None)
)]
fn ratio(
    s1: Option<&str>,
    s2: Option<&str>,
    processor: Option<&Bound<'_, PyAny>>,
    score_cutoff: Option<f64>,
) -> PyResult<f64> {
    let (processed_s1, processed_s2) = process_inputs(s1, s2, processor)?;

    Ok(fuzz::ratio(
        processed_s1.as_deref(),
        processed_s2.as_deref(),
        None,
        score_cutoff,
    ))
}

#[pyfunction]
#[pyo3(
    signature = (s1, s2, processor=None, score_cutoff=None)
)]
fn partial_ratio(
    s1: Option<&str>,
    s2: Option<&str>,
    processor: Option<&Bound<'_, PyAny>>,
    score_cutoff: Option<f64>,
) -> PyResult<f64> {
    let (processed_s1, processed_s2) = process_inputs(s1, s2, processor)?;

    Ok(fuzz::partial_ratio(
        processed_s1.as_deref(),
        processed_s2.as_deref(),
        None,
        score_cutoff,
    ))
}

#[pymodule]
mod crustyfuzz {
    use super::*;

    #[pymodule]
    mod fuzz {
        #[pymodule_export]
        use super::{partial_ratio, ratio};
    }
}
