pub mod damerau_levenshtein;
pub mod hamming;
pub mod indel;
pub mod jaro;
pub mod jaro_winkler;
pub mod lcs_seq;
pub mod levenshtein;
pub mod models;
pub mod osa;
pub mod postfix;
pub mod prefix;

use crate::common::conv_sequences;
use pyo3::prelude::*;
use std::collections::HashMap;
use std::sync::Mutex;
use std::sync::OnceLock;

#[inline]
fn prep_inputs(
    s1: &Bound<'_, PyAny>,
    s2: &Bound<'_, PyAny>,
    processor: Option<&Bound<'_, PyAny>>,
) -> PyResult<(Vec<u32>, Vec<u32>)> {
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
    Ok((s1, s2))
}

pub enum ScorerFlag {
    ResultF64 = 1 << 5,
    ResultI64 = 1 << 6,
    ResultSizeT = 1 << 7,
    Symmetric = 1 << 11,
}

impl std::ops::BitOr for ScorerFlag {
    type Output = u64;
    fn bitor(self, rhs: Self) -> Self::Output {
        self as u64 | rhs as u64
    }
}

pub struct ScorerFlags {
    pub optimal_score: u64,
    pub worst_score: u64,
    pub flags: u64,
}

type ScorerFn = Box<dyn Fn(&HashMap<String, PyObject>) -> ScorerFlags + Send + Sync>;
type ScorerMap = HashMap<String, ScorerFn>;
type GlobalScorerMutex = Mutex<ScorerMap>;

pub static SCORER_METADATA: OnceLock<GlobalScorerMutex> = OnceLock::new();

pub fn get_scorer_flags(
    scorer: &Bound<'_, PyAny>,
    scorer_kwargs: &HashMap<String, PyObject>,
) -> Option<ScorerFlags> {
    let module: String = scorer.getattr("__module__").ok()?.extract().ok()?;
    let name: String = scorer.getattr("__name__").ok()?.extract().ok()?;
    let fqn = format!("{}.{}", module, name);

    SCORER_METADATA
        .get()?
        .lock()
        .unwrap()
        .get(&fqn)
        .map(|f| f(scorer_kwargs))
}

fn get_scorer_flags_distance() -> ScorerFlags {
    ScorerFlags {
        optimal_score: 0,
        worst_score: 2u64.pow(63) - 1,
        flags: ScorerFlag::ResultSizeT | ScorerFlag::Symmetric,
    }
}

fn get_scorer_flags_similarity() -> ScorerFlags {
    ScorerFlags {
        optimal_score: 2u64.pow(63) - 1,
        worst_score: 0,
        flags: ScorerFlag::ResultSizeT | ScorerFlag::Symmetric,
    }
}

fn get_scorer_flags_normalized_distance() -> ScorerFlags {
    ScorerFlags {
        optimal_score: 0,
        worst_score: 1,
        flags: ScorerFlag::ResultF64 | ScorerFlag::Symmetric,
    }
}

fn get_scorer_flags_normalized_similarity() -> ScorerFlags {
    ScorerFlags {
        optimal_score: 1,
        worst_score: 0,
        flags: ScorerFlag::ResultF64 | ScorerFlag::Symmetric,
    }
}

fn get_scorer_flags_fuzz() -> ScorerFlags {
    ScorerFlags {
        optimal_score: 100,
        worst_score: 0,
        flags: ScorerFlag::ResultF64 | ScorerFlag::Symmetric,
    }
}

fn get_scorer_flags_levenshtein_distance(scorer_kwargs: &HashMap<String, PyObject>) -> ScorerFlags {
    Python::with_gil(|py| {
        let mut flags = ScorerFlag::ResultSizeT as u64;
        let weights = scorer_kwargs.get("weights").map(|w| {
            w.extract::<(usize, usize, usize)>(py)
                .expect("Failed to extract weights")
        });
        if weights.is_none() || weights.unwrap().0 == weights.unwrap().1 {
            flags |= ScorerFlag::Symmetric as u64;
        }
        ScorerFlags {
            optimal_score: 0,
            worst_score: 2u64.pow(63) - 1,
            flags,
        }
    })
}

fn get_scorer_flags_levenshtein_similarity(
    scorer_kwargs: &HashMap<String, PyObject>,
) -> ScorerFlags {
    Python::with_gil(|py| {
        let mut flags = ScorerFlag::ResultSizeT as u64;
        let weights = scorer_kwargs.get("weights").map(|w| {
            w.extract::<(usize, usize, usize)>(py)
                .expect("Failed to extract weights")
        });
        if weights.is_none() || weights.unwrap().0 == weights.unwrap().1 {
            flags |= ScorerFlag::Symmetric as u64;
        }
        ScorerFlags {
            optimal_score: 2u64.pow(63) - 1,
            worst_score: 0,
            flags,
        }
    })
}

fn get_scorer_flags_levenshtein_normalized_distance(
    scorer_kwargs: &HashMap<String, PyObject>,
) -> ScorerFlags {
    Python::with_gil(|py| {
        let mut flags = ScorerFlag::ResultF64 as u64;
        let weights = scorer_kwargs.get("weights").map(|w| {
            w.extract::<(usize, usize, usize)>(py)
                .expect("Failed to extract weights")
        });
        if weights.is_none() || weights.unwrap().0 == weights.unwrap().1 {
            flags |= ScorerFlag::Symmetric as u64;
        }
        ScorerFlags {
            optimal_score: 0,
            worst_score: 1,
            flags,
        }
    })
}

fn get_scorer_flags_levenshtein_normalized_similarity(
    scorer_kwargs: &HashMap<String, PyObject>,
) -> ScorerFlags {
    Python::with_gil(|py| {
        let mut flags = ScorerFlag::ResultF64 as u64;
        let weights = scorer_kwargs.get("weights").map(|w| {
            w.extract::<(usize, usize, usize)>(py)
                .expect("Failed to extract weights")
        });
        if weights.is_none() || weights.unwrap().0 == weights.unwrap().1 {
            flags |= ScorerFlag::Symmetric as u64;
        }
        ScorerFlags {
            optimal_score: 1,
            worst_score: 0,
            flags,
        }
    })
}

pub fn setup_scorer_metadata() {
    SCORER_METADATA.get_or_init(|| Mutex::new(HashMap::new()));

    // hamming
    SCORER_METADATA.get().unwrap().lock().unwrap().insert(
        "hamming.distance".to_string(),
        Box::new(|_scorer_kwargs| get_scorer_flags_distance()),
    );
    SCORER_METADATA.get().unwrap().lock().unwrap().insert(
        "hamming.similarity".to_string(),
        Box::new(|_scorer_kwargs| get_scorer_flags_similarity()),
    );
    SCORER_METADATA.get().unwrap().lock().unwrap().insert(
        "hamming.normalized_distance".to_string(),
        Box::new(|_scorer_kwargs| get_scorer_flags_normalized_distance()),
    );
    SCORER_METADATA.get().unwrap().lock().unwrap().insert(
        "hamming.normalized_similarity".to_string(),
        Box::new(|_scorer_kwargs| get_scorer_flags_normalized_similarity()),
    );

    // lcs seq
    SCORER_METADATA.get().unwrap().lock().unwrap().insert(
        "lcs_seq.distance".to_string(),
        Box::new(|_scorer_kwargs| get_scorer_flags_distance()),
    );
    SCORER_METADATA.get().unwrap().lock().unwrap().insert(
        "lcs_seq.similarity".to_string(),
        Box::new(|_scorer_kwargs| get_scorer_flags_similarity()),
    );
    SCORER_METADATA.get().unwrap().lock().unwrap().insert(
        "lcs_seq.normalized_distance".to_string(),
        Box::new(|_scorer_kwargs| get_scorer_flags_normalized_distance()),
    );
    SCORER_METADATA.get().unwrap().lock().unwrap().insert(
        "lcs_seq.normalized_similarity".to_string(),
        Box::new(|_scorer_kwargs| get_scorer_flags_normalized_similarity()),
    );

    // indel
    SCORER_METADATA.get().unwrap().lock().unwrap().insert(
        "indel.distance".to_string(),
        Box::new(|_scorer_kwargs| get_scorer_flags_distance()),
    );
    SCORER_METADATA.get().unwrap().lock().unwrap().insert(
        "indel.similarity".to_string(),
        Box::new(|_scorer_kwargs| get_scorer_flags_similarity()),
    );
    SCORER_METADATA.get().unwrap().lock().unwrap().insert(
        "indel.normalized_distance".to_string(),
        Box::new(|_scorer_kwargs| get_scorer_flags_normalized_distance()),
    );
    SCORER_METADATA.get().unwrap().lock().unwrap().insert(
        "indel.normalized_similarity".to_string(),
        Box::new(|_scorer_kwargs| get_scorer_flags_normalized_similarity()),
    );

    // levenshtein
    SCORER_METADATA.get().unwrap().lock().unwrap().insert(
        "levenshtein.distance".to_string(),
        Box::new(get_scorer_flags_levenshtein_distance),
    );
    SCORER_METADATA.get().unwrap().lock().unwrap().insert(
        "levenshtein.similarity".to_string(),
        Box::new(get_scorer_flags_levenshtein_similarity),
    );
    SCORER_METADATA.get().unwrap().lock().unwrap().insert(
        "levenshtein.normalized_distance".to_string(),
        Box::new(get_scorer_flags_levenshtein_normalized_distance),
    );
    SCORER_METADATA.get().unwrap().lock().unwrap().insert(
        "levenshtein.normalized_similarity".to_string(),
        Box::new(get_scorer_flags_levenshtein_normalized_similarity),
    );

    // damerau-levenshtein
    SCORER_METADATA.get().unwrap().lock().unwrap().insert(
        "damerau_levenshtein.distance".to_string(),
        Box::new(|_scorer_kwargs| get_scorer_flags_distance()),
    );
    SCORER_METADATA.get().unwrap().lock().unwrap().insert(
        "damerau_levenshtein.similarity".to_string(),
        Box::new(|_scorer_kwargs| get_scorer_flags_similarity()),
    );
    SCORER_METADATA.get().unwrap().lock().unwrap().insert(
        "damerau_levenshtein.normalized_distance".to_string(),
        Box::new(|_scorer_kwargs| get_scorer_flags_normalized_distance()),
    );
    SCORER_METADATA.get().unwrap().lock().unwrap().insert(
        "damerau_levenshtein.normalized_similarity".to_string(),
        Box::new(|_scorer_kwargs| get_scorer_flags_normalized_similarity()),
    );

    // jaro
    SCORER_METADATA.get().unwrap().lock().unwrap().insert(
        "jaro.distance".to_string(),
        Box::new(|_scorer_kwargs| get_scorer_flags_normalized_distance()),
    );
    SCORER_METADATA.get().unwrap().lock().unwrap().insert(
        "jaro.similarity".to_string(),
        Box::new(|_scorer_kwargs| get_scorer_flags_normalized_similarity()),
    );
    SCORER_METADATA.get().unwrap().lock().unwrap().insert(
        "jaro.normalized_distance".to_string(),
        Box::new(|_scorer_kwargs| get_scorer_flags_normalized_distance()),
    );
    SCORER_METADATA.get().unwrap().lock().unwrap().insert(
        "jaro.normalized_similarity".to_string(),
        Box::new(|_scorer_kwargs| get_scorer_flags_normalized_similarity()),
    );

    // jaro-winkler
    SCORER_METADATA.get().unwrap().lock().unwrap().insert(
        "jaro_winkler.distance".to_string(),
        Box::new(|_scorer_kwargs| get_scorer_flags_normalized_distance()),
    );
    SCORER_METADATA.get().unwrap().lock().unwrap().insert(
        "jaro_winkler.similarity".to_string(),
        Box::new(|_scorer_kwargs| get_scorer_flags_normalized_similarity()),
    );
    SCORER_METADATA.get().unwrap().lock().unwrap().insert(
        "jaro_winkler.normalized_distance".to_string(),
        Box::new(|_scorer_kwargs| get_scorer_flags_normalized_distance()),
    );
    SCORER_METADATA.get().unwrap().lock().unwrap().insert(
        "jaro_winkler.normalized_similarity".to_string(),
        Box::new(|_scorer_kwargs| get_scorer_flags_normalized_similarity()),
    );

    // OSA
    SCORER_METADATA.get().unwrap().lock().unwrap().insert(
        "osa.distance".to_string(),
        Box::new(|_scorer_kwargs| get_scorer_flags_distance()),
    );
    SCORER_METADATA.get().unwrap().lock().unwrap().insert(
        "osa.similarity".to_string(),
        Box::new(|_scorer_kwargs| get_scorer_flags_similarity()),
    );
    SCORER_METADATA.get().unwrap().lock().unwrap().insert(
        "osa.normalized_distance".to_string(),
        Box::new(|_scorer_kwargs| get_scorer_flags_normalized_distance()),
    );
    SCORER_METADATA.get().unwrap().lock().unwrap().insert(
        "osa.normalized_similarity".to_string(),
        Box::new(|_scorer_kwargs| get_scorer_flags_normalized_similarity()),
    );

    // postfix
    SCORER_METADATA.get().unwrap().lock().unwrap().insert(
        "postfix.distance".to_string(),
        Box::new(|_scorer_kwargs| get_scorer_flags_distance()),
    );
    SCORER_METADATA.get().unwrap().lock().unwrap().insert(
        "postfix.similarity".to_string(),
        Box::new(|_scorer_kwargs| get_scorer_flags_similarity()),
    );
    SCORER_METADATA.get().unwrap().lock().unwrap().insert(
        "postfix.normalized_distance".to_string(),
        Box::new(|_scorer_kwargs| get_scorer_flags_normalized_distance()),
    );
    SCORER_METADATA.get().unwrap().lock().unwrap().insert(
        "postfix.normalized_similarity".to_string(),
        Box::new(|_scorer_kwargs| get_scorer_flags_normalized_similarity()),
    );

    // prefix
    SCORER_METADATA.get().unwrap().lock().unwrap().insert(
        "prefix.distance".to_string(),
        Box::new(|_scorer_kwargs| get_scorer_flags_distance()),
    );
    SCORER_METADATA.get().unwrap().lock().unwrap().insert(
        "prefix.similarity".to_string(),
        Box::new(|_scorer_kwargs| get_scorer_flags_similarity()),
    );
    SCORER_METADATA.get().unwrap().lock().unwrap().insert(
        "prefix.normalized_distance".to_string(),
        Box::new(|_scorer_kwargs| get_scorer_flags_normalized_distance()),
    );
    SCORER_METADATA.get().unwrap().lock().unwrap().insert(
        "prefix.normalized_similarity".to_string(),
        Box::new(|_scorer_kwargs| get_scorer_flags_normalized_similarity()),
    );

    // fuzz
    SCORER_METADATA.get().unwrap().lock().unwrap().insert(
        "fuzz.ratio".to_string(),
        Box::new(|_scorer_kwargs| get_scorer_flags_fuzz()),
    );
    SCORER_METADATA.get().unwrap().lock().unwrap().insert(
        "fuzz.partial_ratio".to_string(),
        Box::new(|_scorer_kwargs| get_scorer_flags_fuzz()),
    );
    SCORER_METADATA.get().unwrap().lock().unwrap().insert(
        "fuzz.token_sort_ratio".to_string(),
        Box::new(|_scorer_kwargs| get_scorer_flags_fuzz()),
    );
    SCORER_METADATA.get().unwrap().lock().unwrap().insert(
        "fuzz.token_set_ratio".to_string(),
        Box::new(|_scorer_kwargs| get_scorer_flags_fuzz()),
    );
    SCORER_METADATA.get().unwrap().lock().unwrap().insert(
        "fuzz.token_ratio".to_string(),
        Box::new(|_scorer_kwargs| get_scorer_flags_fuzz()),
    );
    SCORER_METADATA.get().unwrap().lock().unwrap().insert(
        "fuzz.partial_token_sort_ratio".to_string(),
        Box::new(|_scorer_kwargs| get_scorer_flags_fuzz()),
    );
    SCORER_METADATA.get().unwrap().lock().unwrap().insert(
        "fuzz.partial_token_set_ratio".to_string(),
        Box::new(|_scorer_kwargs| get_scorer_flags_fuzz()),
    );
    SCORER_METADATA.get().unwrap().lock().unwrap().insert(
        "fuzz.partial_token_ratio".to_string(),
        Box::new(|_scorer_kwargs| get_scorer_flags_fuzz()),
    );
    SCORER_METADATA.get().unwrap().lock().unwrap().insert(
        "fuzz.WRatio".to_string(),
        Box::new(|_scorer_kwargs| get_scorer_flags_fuzz()),
    );
    SCORER_METADATA.get().unwrap().lock().unwrap().insert(
        "fuzz.QRatio".to_string(),
        Box::new(|_scorer_kwargs| get_scorer_flags_fuzz()),
    );
}
