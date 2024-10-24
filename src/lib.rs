use pyo3::prelude::*;
mod common;
mod distance;
mod fuzz;

// A rusty string matching library
#[pymodule]
mod crustyfuzz {
    use super::*;

    #[pymodule(submodule)]
    mod distance {
        #[pymodule_export]
        use crate::distance::models::ScoreAlignment;
    }

    #[pymodule(submodule)]
    mod fuzz {
        #[pymodule_export]
        use crate::fuzz::{partial_ratio, partial_ratio_alignment, ratio};
    }

    #[pymodule(submodule)]
    mod process {}

    #[pymodule(submodule)]
    mod rs_utils {}
}
