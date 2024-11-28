use pyo3::prelude::*;

/// A token represents a contiguous sequence of non-whitespace characters
#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct Token<'a> {
    pub chars: &'a [u32],
}

/// An iterator that yields tokens from a character sequence
pub struct TokenIterator<'a> {
    chars: &'a [u32],
    pos: usize,
}

impl<'a> TokenIterator<'a> {
    pub fn new(chars: &'a [u32]) -> Self {
        TokenIterator { chars, pos: 0 }
    }
}

impl<'a> Iterator for TokenIterator<'a> {
    type Item = Token<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        // Skip whitespace
        while self.pos < self.chars.len() {
            if !char::from_u32(self.chars[self.pos])
                .expect("invalid char")
                .is_whitespace()
            {
                break;
            }
            self.pos += 1;
        }

        if self.pos >= self.chars.len() {
            return None;
        }

        let start = self.pos;

        // Find end of token
        while self.pos < self.chars.len() {
            if char::from_u32(self.chars[self.pos])
                .expect("invalid char")
                .is_whitespace()
            {
                break;
            }
            self.pos += 1;
        }

        Some(Token {
            chars: &self.chars[start..self.pos],
        })
    }
}

/// A sequence of tokens that can be joined back into a string
pub struct TokenSequence<'a> {
    tokens: Vec<Token<'a>>,
}

impl<'a> TokenSequence<'a> {
    pub fn new(tokens: Vec<Token<'a>>) -> Self {
        TokenSequence { tokens }
    }

    pub fn join(&self) -> Vec<u32> {
        if self.tokens.is_empty() {
            return Vec::new();
        }

        // Calculate spaces needed between tokens (tokens.len() - 1)
        let spaces_needed = self.tokens.len() - 1;

        // Calculate total capacity needed
        let total_len = self.tokens.iter().map(|t| t.chars.len()).sum::<usize>() + spaces_needed;

        let mut result = Vec::with_capacity(total_len);

        let mut tokens = self.tokens.clone();
        tokens.sort();

        for (i, token) in tokens.iter().enumerate() {
            if i > 0 {
                result.push(' ' as u32);
            }
            result.extend_from_slice(token.chars);
        }
        result
    }
}

#[derive(FromPyObject)]
pub enum IndexResult {
    #[pyo3(transparent, annotation = "int")]
    Integer(usize),
    #[pyo3(transparent, annotation = "float")]
    Float(f64),
    #[pyo3(transparent, annotation = "str")]
    String(String),
}

impl IntoPy<PyObject> for IndexResult {
    fn into_py(self, py: Python<'_>) -> PyObject {
        match self {
            IndexResult::Integer(i) => i.into_py(py),
            IndexResult::Float(f) => f.into_py(py),
            IndexResult::String(s) => s.into_py(py),
        }
    }
}

#[derive(Clone, FromPyObject)]
pub enum StrOrInt {
    #[pyo3(transparent, annotation = "str")]
    Str(String),
    #[pyo3(transparent, annotation = "int")]
    Int(usize),
}

impl IntoPy<PyObject> for StrOrInt {
    fn into_py(self, py: Python<'_>) -> PyObject {
        match self {
            StrOrInt::Str(i) => i.into_py(py),
            StrOrInt::Int(s) => s.into_py(py),
        }
    }
}
