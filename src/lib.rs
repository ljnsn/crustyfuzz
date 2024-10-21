use pyo3::prelude::*;
mod common;
pub mod fuzz;
mod indel;
mod lcs_seq;
use pyo3::types::PyString;

fn call_processor(processor: &Bound<'_, PyString>, s: &str) -> Result<String, PyErr> {
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
    processor: Option<&Bound<'_, PyString>>,
    score_cutoff: Option<f64>,
) -> PyResult<f64> {
    let (processed_s1, processed_s2) = match (s1, s2, processor) {
        (Some(s1), Some(s2), Some(proc)) => {
            let processed_s1 = call_processor(proc, s1)?;
            let processed_s2 = call_processor(proc, s2)?;
            (processed_s1, processed_s2)
        }
        (Some(s1), Some(s2), None) => (s1.to_string(), s2.to_string()),
        _ => return Err(PyErr::new::<pyo3::exceptions::PyValueError, _>("Invalid input")),
    };

    Ok(fuzz::ratio(Some(&processed_s1), Some(&processed_s2), None, score_cutoff))
}

#[pymodule]
fn crustyfuzz(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(ratio, m)?)
}
