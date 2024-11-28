pub mod error;
pub mod models;
pub mod utils;

use crate::common::error::ConversionError;
use pyo3::prelude::*;
use pyo3::types::{PyBytes, PySequence, PyString};
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

pub type ConversionResult<T> = Result<Option<T>, ConversionError>;
pub type ConvertedSequence = Vec<u32>;

const NEGATIVE_ONE_MARKER: u32 = u32::MAX;

pub fn conv_sequence(s: &Bound<'_, PyAny>) -> ConversionResult<ConvertedSequence> {
    if let Ok(s) = s.downcast::<PyString>() {
        return Ok(Some(
            s.to_cow()
                .map_err(|e| ConversionError::StringExtraction(e.to_string()))?
                .chars()
                .map(|c| c as u32)
                .collect(),
        ));
    }

    if let Ok(bytes) = s.downcast::<PyBytes>() {
        return Ok(Some(bytes.as_bytes().iter().map(|&b| b as u32).collect()));
    }

    if s.is_none() {
        return Ok(None);
    }

    // handle Python array.array objects
    if s.getattr("typecode").is_ok() {
        let lst = s
            .call_method0("tolist")
            .map_err(|e| ConversionError::ArrayConversion(e.to_string()))?;
        return conv_sequence(&lst);
    }

    let seq = s
        .downcast::<PySequence>()
        .map_err(|e| ConversionError::SequenceDowncast(e.to_string()))?;
    let mut result = Vec::new();

    for i in 0..seq
        .len()
        .map_err(|e| ConversionError::SequenceLength(e.to_string()))?
    {
        let elem = seq
            .get_item(i)
            .map_err(|e| ConversionError::SequenceItem(e.to_string()))?;

        if let Ok(s) = elem.downcast::<PyString>() {
            let s = s
                .to_cow()
                .map_err(|e| ConversionError::StringExtraction(e.to_string()))?;
            if s.len() == 1 {
                result.push(s.chars().next().unwrap() as u32);
                continue;
            }
        }

        if let Ok(n) = elem.extract::<i64>() {
            if n == -1 {
                result.push(NEGATIVE_ONE_MARKER);
                continue;
            } else if n > 0 {
                // so that string and ordinal input is interpreted equally
                result.push(n as u32);
                continue;
            }
        }

        let mut hasher = DefaultHasher::new();
        elem.hash()
            .map_err(|e| ConversionError::Hashing(e.to_string()))?
            .hash(&mut hasher);
        // keep only the lower 32 bits
        result.push((hasher.finish() & 0xFFFFFFFF) as u32);
    }

    Ok(Some(result))
}

pub fn conv_sequences(
    s1: &Bound<'_, PyAny>,
    s2: &Bound<'_, PyAny>,
) -> Result<(Option<ConvertedSequence>, Option<ConvertedSequence>), ConversionError> {
    if let (Ok(s1_str), Ok(s2_str)) = (s1.downcast::<PyString>(), s2.downcast::<PyString>()) {
        // if we supported python 3.10 only, we could use `to_str` here, see [1]
        // [1]: https://docs.rs/pyo3/latest/pyo3/types/trait.PyStringMethods.html#required-methods
        return Ok((
            Some(
                s1_str
                    .to_cow()
                    .map_err(|e| ConversionError::StringExtraction(e.to_string()))?
                    .chars()
                    .map(|c| c as u32)
                    .collect(),
            ),
            Some(
                s2_str
                    .to_cow()
                    .map_err(|e| ConversionError::StringExtraction(e.to_string()))?
                    .chars()
                    .map(|c| c as u32)
                    .collect(),
            ),
        ));
    }

    if let (Ok(s1_bytes), Ok(s2_bytes)) = (s1.downcast::<PyBytes>(), s2.downcast::<PyBytes>()) {
        return Ok((
            Some(s1_bytes.as_bytes().iter().map(|&b| b as u32).collect()),
            Some(s2_bytes.as_bytes().iter().map(|&b| b as u32).collect()),
        ));
    }

    Ok((conv_sequence(s1)?, conv_sequence(s2)?))
}

pub fn common_prefix(s1: &[u32], s2: &[u32]) -> usize {
    s1.iter().zip(s2.iter()).take_while(|(a, b)| a == b).count()
}

pub fn common_suffix(s1: &[u32], s2: &[u32]) -> usize {
    s1.iter()
        .rev()
        .zip(s2.iter().rev())
        .take_while(|(a, b)| a == b)
        .count()
}

pub fn common_affix(s1: &[u32], s2: &[u32]) -> (usize, usize) {
    let prefix_len = common_prefix(s1, s2);
    let suffix_len = common_suffix(&s1[prefix_len..], &s2[prefix_len..]);
    (prefix_len, suffix_len)
}

#[cfg(test)]
mod tests {
    use super::*;
    use pyo3::types::{PyBytes, PyList};
    use pyo3::Python;

    #[test]
    fn test_conv_sequence_string() {
        pyo3::prepare_freethreaded_python();
        Python::with_gil(|py| {
            let input = PyString::new_bound(py, "abc");
            let result = conv_sequence(&input).unwrap();
            assert_eq!(result, Some(vec![97, 98, 99]));
        });
    }

    #[test]
    fn test_conv_sequence_bytes() {
        pyo3::prepare_freethreaded_python();
        Python::with_gil(|py| {
            let input = PyBytes::new_bound(py, &[1, 2, 3]);
            let result = conv_sequence(&input).unwrap();
            assert_eq!(result, Some(vec![1, 2, 3]));
        });
    }

    #[test]
    fn test_conv_sequence_none() {
        pyo3::prepare_freethreaded_python();
        Python::with_gil(|py| {
            let input = py.None();
            let result = conv_sequence(&input.into_bound(py)).unwrap();
            assert_eq!(result, None);
        });
    }

    #[test]
    fn test_conv_sequence_list_strings() {
        pyo3::prepare_freethreaded_python();
        Python::with_gil(|py| {
            let list = PyList::new_bound(
                py,
                &[PyString::new_bound(py, "a"), PyString::new_bound(py, "b")],
            );
            let result = conv_sequence(&list).unwrap();
            assert_eq!(result, Some(vec![97, 98]));
        });
    }

    #[test]
    fn test_conv_sequence_with_negative_one() {
        pyo3::prepare_freethreaded_python();

        Python::with_gil(|py| {
            let list = PyList::new_bound(py, [-1i64, 1i64]);
            let result = conv_sequence(&list).unwrap();
            assert_eq!(result, Some(vec![NEGATIVE_ONE_MARKER, 1]));
        });
    }

    #[test]
    fn test_conv_sequences_strings() {
        pyo3::prepare_freethreaded_python();
        Python::with_gil(|py| {
            let s1 = PyString::new_bound(py, "abc");
            let s2 = PyString::new_bound(py, "def");
            let (r1, r2) = conv_sequences(&s1, &s2).unwrap();
            assert_eq!(r1, Some(vec![97, 98, 99]));
            assert_eq!(r2, Some(vec![100, 101, 102]));
        });
    }

    #[test]
    fn test_conv_sequences_bytes() {
        pyo3::prepare_freethreaded_python();
        Python::with_gil(|py| {
            let b1 = PyBytes::new_bound(py, &[1, 2, 3]);
            let b2 = PyBytes::new_bound(py, &[4, 5, 6]);
            let (r1, r2) = conv_sequences(&b1, &b2).unwrap();
            assert_eq!(r1, Some(vec![1, 2, 3]));
            assert_eq!(r2, Some(vec![4, 5, 6]));
        });
    }

    #[test]
    fn test_conv_sequences_mixed() {
        pyo3::prepare_freethreaded_python();

        Python::with_gil(|py| {
            let s1 = PyString::new_bound(py, "abc");
            let list = PyList::new_bound(py, [1i64, -1i64]);
            let (r1, r2) = conv_sequences(&s1, &list).unwrap();
            assert_eq!(r1, Some(vec![97, 98, 99]));
            assert_eq!(r2, Some(vec![1, NEGATIVE_ONE_MARKER]));
        });
    }

    #[test]
    fn test_conv_sequence_array() {
        pyo3::prepare_freethreaded_python();
        Python::with_gil(|py| {
            let array_mod = PyModule::import_bound(py, "array").unwrap();
            let array = array_mod
                .getattr("array")
                .unwrap()
                .call1(("b", (1, 2, 3)))
                .unwrap();
            let result = conv_sequence(&array).unwrap();
            assert_eq!(result, Some(vec![1, 2, 3]));
        });
    }
}
