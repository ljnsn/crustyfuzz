use pyo3::prelude::*;
mod common;
pub mod fuzz;
mod indel;
mod lcs_seq;

fn call_processor(processor: &Bound<'_, PyAny>, s: &str) -> Result<String, PyErr> {
    let res = processor.call1((s,))?;
    res.extract::<String>()
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
    let (processed_s1, processed_s2) = match (s1, s2, processor) {
        (Some(s1), Some(s2), Some(proc)) => {
            let processed_s1 = call_processor(proc, s1)?;
            let processed_s2 = call_processor(proc, s2)?;
            (processed_s1, processed_s2)
        }
        (Some(s1), Some(s2), None) => (s1.to_string(), s2.to_string()),
        _ => {
            return Err(PyErr::new::<pyo3::exceptions::PyValueError, _>(
                "Invalid input",
            ))
        }
    };

    Ok(fuzz::ratio(
        Some(&processed_s1),
        Some(&processed_s2),
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
    let (processed_s1, processed_s2) = match (s1, s2, processor) {
        (Some(s1), Some(s2), Some(proc)) => {
            let processed_s1 = call_processor(proc, s1)?;
            let processed_s2 = call_processor(proc, s2)?;
            (processed_s1, processed_s2)
        }
        (Some(s1), Some(s2), None) => (s1.to_string(), s2.to_string()),
        _ => {
            return Err(PyErr::new::<pyo3::exceptions::PyValueError, _>(
                "Invalid input",
            ))
        }
    };

    Ok(fuzz::partial_ratio(
        Some(&processed_s1),
        Some(&processed_s2),
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
