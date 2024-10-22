use pyo3::prelude::*;
mod common;
mod distance;
mod fuzz;

// A rusty string matching library
#[pymodule]
mod crustyfuzz {
    use super::*;

    #[pymodule]
    mod distance {}

    #[pymodule]
    mod fuzz {
        #[pymodule_export]
        use crate::fuzz::{partial_ratio, ratio};
    }

    #[pymodule]
    mod process {}

    #[pymodule]
    mod utils {}
}
