use pyo3::exceptions::PyIndexError;
use pyo3::prelude::*;

#[derive(FromPyObject)]
enum IndexResult {
    #[pyo3(transparent, annotation = "int")]
    Integer(usize),
    #[pyo3(transparent, annotation = "float")]
    Float(f64),
}

#[pyclass(eq, mapping, get_all, module = "crustyfuzz.distance")]
#[derive(PartialEq, Debug)]
pub struct ScoreAlignment {
    pub score: f64,
    pub src_start: usize,
    pub src_end: usize,
    pub dest_start: usize,
    pub dest_end: usize,
}

impl IntoPy<PyObject> for IndexResult {
    fn into_py(self, py: Python<'_>) -> PyObject {
        match self {
            IndexResult::Integer(i) => i.into_py(py),
            IndexResult::Float(f) => f.into_py(py),
        }
    }
}

#[pymethods]
impl ScoreAlignment {
    #[new]
    fn py_new(
        score: f64,
        src_start: usize,
        src_end: usize,
        dest_start: usize,
        dest_end: usize,
    ) -> Self {
        ScoreAlignment {
            score,
            src_start,
            src_end,
            dest_start,
            dest_end,
        }
    }

    fn __len__(&self) -> usize {
        5
    }

    fn __getitem__(&self, idx: isize) -> PyResult<IndexResult> {
        let idx = if idx < 0 { 5 + idx } else { idx };

        match idx {
            0 => Ok(IndexResult::Float(self.score)),
            1 => Ok(IndexResult::Integer(self.src_start)),
            2 => Ok(IndexResult::Integer(self.src_end)),
            3 => Ok(IndexResult::Integer(self.dest_start)),
            4 => Ok(IndexResult::Integer(self.dest_end)),
            _ => Err(PyIndexError::new_err("Opcode index out of range")),
        }
    }

    fn __iter__(slf: PyRef<'_, Self>) -> PyRef<'_, Self> {
        slf
    }
}
