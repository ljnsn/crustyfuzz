mod common;
mod distance;
mod fuzz;
mod process;

use crate::distance::setup_scorer_metadata;
use pyo3::prelude::*;

// A rusty string matching library
#[pymodule]
mod crustyfuzz {
    use super::*;

    #[pymodule_init]
    fn init(_m: &Bound<'_, PyModule>) -> PyResult<()> {
        setup_scorer_metadata();
        Ok(())
    }

    #[pymodule(submodule)]
    mod distance {
        use super::*;

        #[pymodule_export]
        use crate::distance::models::{
            Editop, Editops, MatchingBlock, Opcode, Opcodes, ScoreAlignment,
        };

        #[pymodule(submodule)]
        mod lcs_seq {
            #[pymodule_export]
            use crate::distance::lcs_seq::{
                py_distance, py_editops, py_normalized_distance, py_normalized_similarity,
                py_opcodes, py_similarity,
            };
        }

        #[pymodule(submodule)]
        mod indel {
            #[pymodule_export]
            use crate::distance::indel::{
                py_distance, py_editops, py_normalized_distance, py_normalized_similarity,
                py_opcodes, py_similarity,
            };
        }

        #[pymodule(submodule)]
        mod hamming {
            #[pymodule_export]
            use crate::distance::hamming::{
                py_distance, py_editops, py_normalized_distance, py_normalized_similarity,
                py_opcodes, py_similarity,
            };
        }

        #[pymodule(submodule)]
        mod levenshtein {
            #[pymodule_export]
            use crate::distance::levenshtein::{
                py_distance, py_editops, py_normalized_distance, py_normalized_similarity,
                py_opcodes, py_similarity,
            };
        }

        #[pymodule(submodule)]
        mod damerau_levenshtein {
            #[pymodule_export]
            use crate::distance::damerau_levenshtein::{
                py_distance, py_normalized_distance, py_normalized_similarity, py_similarity,
            };
        }

        #[pymodule(submodule)]
        mod jaro {
            #[pymodule_export]
            use crate::distance::jaro::{
                py_distance, py_normalized_distance, py_normalized_similarity, py_similarity,
            };
        }

        #[pymodule(submodule)]
        mod jaro_winkler {
            #[pymodule_export]
            use crate::distance::jaro_winkler::{
                py_distance, py_normalized_distance, py_normalized_similarity, py_similarity,
            };
        }

        #[pymodule(submodule)]
        mod osa {
            #[pymodule_export]
            use crate::distance::osa::{
                py_distance, py_normalized_distance, py_normalized_similarity, py_similarity,
            };
        }

        #[pymodule(submodule)]
        mod postfix {
            #[pymodule_export]
            use crate::distance::postfix::{
                py_distance, py_normalized_distance, py_normalized_similarity, py_similarity,
            };
        }

        #[pymodule(submodule)]
        mod prefix {
            #[pymodule_export]
            use crate::distance::prefix::{
                py_distance, py_normalized_distance, py_normalized_similarity, py_similarity,
            };
        }
    }

    #[pymodule(submodule)]
    mod fuzz {
        #[pymodule_export]
        use crate::fuzz::{
            py_partial_ratio, py_partial_ratio_alignment, py_partial_token_ratio,
            py_partial_token_set_ratio, py_partial_token_sort_ratio, py_quick_ratio, py_ratio,
            py_token_ratio, py_token_set_ratio, py_token_sort_ratio, py_weighted_ratio,
        };
    }

    #[pymodule(submodule)]
    mod process {
        #[pymodule_export]
        use crate::process::{py_extract, py_extract_iter, py_extract_one};
    }
}
