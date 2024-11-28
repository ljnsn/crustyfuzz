use crate::common::models::{IndexResult, StrOrInt};
use pyo3::exceptions::{PyIndexError, PyValueError};
use pyo3::prelude::*;
use pyo3::types::PyType;

/**
Tuple like object describing the position of the compared strings in
src and dest.

It indicates that the score has been calculated between
src[src_start:src_end] and dest[dest_start:dest_end]
*/
#[pyclass(eq, mapping, get_all, module = "crustyfuzz.distance")]
#[derive(PartialEq, Debug)]
pub struct ScoreAlignment {
    pub score: f64,
    pub src_start: usize,
    pub src_end: usize,
    pub dest_start: usize,
    pub dest_end: usize,
}

impl std::fmt::Display for ScoreAlignment {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "ScoreAlignment(score={}, src_start={}, src_end={}, dest_start={}, dest_end={})",
            self.score, self.src_start, self.src_end, self.dest_start, self.dest_end
        )
    }
}

#[pyclass]
struct AlignmentIter {
    inner: std::vec::IntoIter<IndexResult>,
}

#[pymethods]
impl AlignmentIter {
    fn __iter__(slf: PyRef<'_, Self>) -> PyRef<'_, Self> {
        slf
    }

    fn __next__(mut slf: PyRefMut<'_, Self>) -> Option<IndexResult> {
        slf.inner.next()
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

    fn __str__(&self) -> String {
        self.to_string()
    }

    fn __repr__(&self) -> String {
        self.to_string()
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

    fn __iter__(slf: PyRef<'_, Self>) -> PyResult<Py<AlignmentIter>> {
        let iter = AlignmentIter {
            inner: vec![
                IndexResult::Float(slf.score),
                IndexResult::Integer(slf.src_start),
                IndexResult::Integer(slf.src_end),
                IndexResult::Integer(slf.dest_start),
                IndexResult::Integer(slf.dest_end),
            ]
            .into_iter(),
        };
        Py::new(slf.py(), iter)
    }
}

#[pyclass(eq, mapping, get_all, module = "crustyfuzz.distance")]
#[derive(PartialEq, Debug)]
pub struct MatchingBlock {
    pub a: usize,
    pub b: usize,
    pub size: usize,
}

impl std::fmt::Display for MatchingBlock {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "MatchingBlock(a={}, b={}, size={})",
            self.a, self.b, self.size
        )
    }
}

// Triple describing matching subsequences.
#[pymethods]
impl MatchingBlock {
    #[new]
    fn py_new(a: usize, b: usize, size: usize) -> Self {
        MatchingBlock { a, b, size }
    }

    fn __str__(&self) -> String {
        self.to_string()
    }

    fn __repr__(&self) -> String {
        self.to_string()
    }

    fn __len__(&self) -> usize {
        3
    }

    fn __getitem__(&self, idx: isize) -> PyResult<IndexResult> {
        let idx = if idx < 0 { 3 + idx } else { idx };
        match idx {
            0 => Ok(IndexResult::Integer(self.a)),
            1 => Ok(IndexResult::Integer(self.b)),
            2 => Ok(IndexResult::Integer(self.size)),
            _ => Err(PyIndexError::new_err("MatchingBlock index out of range")),
        }
    }

    fn __iter__(slf: PyRef<'_, Self>) -> PyRef<'_, Self> {
        slf
    }
}

#[pyclass]
struct EditopIter {
    inner: std::vec::IntoIter<StrOrInt>,
}

#[pymethods]
impl EditopIter {
    fn __iter__(slf: PyRef<'_, Self>) -> PyRef<'_, Self> {
        slf
    }

    fn __next__(mut slf: PyRefMut<'_, Self>) -> Option<StrOrInt> {
        slf.inner.next()
    }
}

/**
Tuple like object describing an edit operation.
It is in the form (tag, src_pos, dest_pos)

The tags are strings, with these meanings:

+-----------+---------------------------------------------------+
| tag       | explanation                                       |
+===========+===================================================+
| 'replace' | src[src_pos] should be replaced by dest[dest_pos] |
+-----------+---------------------------------------------------+
| 'delete'  | src[src_pos] should be deleted                    |
+-----------+---------------------------------------------------+
| 'insert'  | dest[dest_pos] should be inserted at src[src_pos] |
+-----------+---------------------------------------------------+
*/
#[pyclass(eq, mapping, get_all, module = "crustyfuzz.distance")]
#[derive(PartialEq, Clone, Debug)]
pub struct Editop {
    pub tag: String,
    pub src_pos: usize,
    pub dest_pos: usize,
}

impl std::fmt::Display for Editop {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "Editop(tag='{}', src_pos={}, dest_pos={})",
            self.tag, self.src_pos, self.dest_pos
        )
    }
}

#[pymethods]
impl Editop {
    #[new]
    fn py_new(tag: String, src_pos: usize, dest_pos: usize) -> Self {
        Editop {
            tag,
            src_pos,
            dest_pos,
        }
    }

    fn __str__(&self) -> String {
        self.to_string()
    }

    fn __repr__(&self) -> String {
        self.to_string()
    }

    fn __len__(&self) -> usize {
        3
    }

    fn __getitem__(&self, idx: isize) -> PyResult<IndexResult> {
        let idx = if idx < 0 { 3 + idx } else { idx };
        match idx {
            0 => Ok(IndexResult::String(self.tag.clone())),
            1 => Ok(IndexResult::Integer(self.src_pos)),
            2 => Ok(IndexResult::Integer(self.dest_pos)),
            _ => Err(PyIndexError::new_err("Editop index out of range")),
        }
    }

    fn __iter__(slf: PyRef<'_, Self>) -> PyResult<Py<EditopIter>> {
        let iter = EditopIter {
            inner: vec![
                StrOrInt::Str(slf.tag.clone()),
                StrOrInt::Int(slf.src_pos),
                StrOrInt::Int(slf.dest_pos),
            ]
            .into_iter(),
        };

        Py::new(slf.py(), iter)
    }
}

#[pyclass]
struct EditopsIter {
    inner: std::vec::IntoIter<Editop>,
}

#[pymethods]
impl EditopsIter {
    fn __iter__(slf: PyRef<'_, Self>) -> PyRef<'_, Self> {
        slf
    }

    fn __next__(mut slf: PyRefMut<'_, Self>) -> Option<Editop> {
        slf.inner.next()
    }
}

// List like object of Editops describing how to turn s1 into s2.
#[pyclass(eq, mapping, get_all, module = "crustyfuzz.distance")]
#[derive(Clone, Debug, PartialEq)]
pub struct Editops {
    src_len: usize,
    dest_len: usize,
    editops: Vec<Editop>,
}

impl std::fmt::Display for Editops {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let editops_str = self
            .editops
            .iter()
            .map(|x| x.to_string())
            .collect::<Vec<String>>()
            .join(", ");
        write!(
            f,
            "Editops([{}], src_len={}, dest_len={})",
            editops_str, self.src_len, self.dest_len
        )
    }
}

impl IntoIterator for Editops {
    type Item = Editop;
    type IntoIter = std::vec::IntoIter<Editop>;

    fn into_iter(self) -> Self::IntoIter {
        self.editops.into_iter()
    }
}

impl<'a> IntoIterator for &'a Editops {
    type Item = &'a Editop;
    type IntoIter = std::slice::Iter<'a, Editop>;

    fn into_iter(self) -> Self::IntoIter {
        self.editops.iter()
    }
}

impl Editops {
    pub fn new(src_len: usize, dest_len: usize, editops: Vec<Editop>) -> Self {
        Editops {
            src_len,
            dest_len,
            editops,
        }
    }

    pub fn len(&self) -> usize {
        self.editops.len()
    }
}

#[pymethods]
impl Editops {
    #[new]
    fn py_new(src_len: usize, dest_len: usize, editops: Vec<Editop>) -> Self {
        Editops {
            src_len,
            dest_len,
            editops,
        }
    }

    fn __len__(&self) -> usize {
        self.len()
    }

    fn __delitem__(&mut self, index: usize) {
        // TODO: make this work with slices
        self.editops.remove(index);
    }

    fn __getitem__(&self, index: usize) -> PyResult<Editop> {
        // TODO: make this work with slices
        if index >= self.len() {
            return Err(PyIndexError::new_err("Editop index out of range"));
        }
        Ok(self.editops[index].clone())
    }

    fn __iter__(slf: PyRef<'_, Self>) -> PyResult<Py<EditopsIter>> {
        let iter = EditopsIter {
            inner: slf.editops.clone().into_iter(),
        };

        Py::new(slf.py(), iter)
    }

    fn __str__(&self) -> String {
        self.to_string()
    }

    fn __repr__(&self) -> String {
        self.to_string()
    }

    /**
    Create Editops from Opcodes.

    Parameters
    ----------
    opcodes : Opcodes
        opcodes to convert to editops

    Returns
    -------
    editops : Editops
        Opcodes converted to Editops
    */
    #[classmethod]
    fn from_opcodes(_cls: &Bound<'_, PyType>, opcodes: Opcodes) -> Editops {
        opcodes.as_editops()
    }

    // Convert to Opcodes.
    pub fn as_opcodes(&self) -> Opcodes {
        let mut x = Opcodes {
            src_len: self.src_len,
            dest_len: self.dest_len,
            opcodes: Vec::new(),
        };

        let mut blocks = vec![];
        let mut src_pos = 0;
        let mut dest_pos = 0;
        let mut i = 0;

        while i < self.len() {
            if src_pos < self.editops[i].src_pos || dest_pos < self.editops[i].dest_pos {
                blocks.push(Opcode {
                    tag: "equal".to_string(),
                    src_start: src_pos,
                    src_end: self.editops[i].src_pos,
                    dest_start: dest_pos,
                    dest_end: self.editops[i].dest_pos,
                });
                src_pos = self.editops[i].src_pos;
                dest_pos = self.editops[i].dest_pos;
            }

            let src_begin = src_pos;
            let dest_begin = dest_pos;
            let tag = self.editops[i].tag.clone();

            while i < self.editops.len()
                && self.editops[i].tag == tag
                && src_pos == self.editops[i].src_pos
                && dest_pos == self.editops[i].dest_pos
            {
                match tag.as_str() {
                    "replace" => {
                        src_pos += 1;
                        dest_pos += 1;
                    }
                    "delete" => {
                        src_pos += 1;
                    }
                    "insert" => {
                        dest_pos += 1;
                    }
                    _ => {
                        panic!("Invalid tag: {}", tag);
                    }
                }

                i += 1;
            }

            blocks.push(Opcode {
                tag,
                src_start: src_begin,
                src_end: src_pos,
                dest_start: dest_begin,
                dest_end: dest_pos,
            });
        }

        if src_pos < self.src_len || dest_pos < self.dest_len {
            blocks.push(Opcode {
                tag: "equal".to_string(),
                src_start: src_pos,
                src_end: self.src_len,
                dest_start: dest_pos,
                dest_end: self.dest_len,
            });
        }

        x.opcodes = blocks;
        x
    }

    // Convert to matching blocks.
    pub fn as_matching_blocks(&self) -> Vec<MatchingBlock> {
        let mut blocks = vec![];
        let mut src_pos = 0;
        let mut dest_pos = 0;

        for op in self {
            if src_pos < op.src_pos || dest_pos < op.dest_pos {
                dbg!(op, src_pos, dest_pos);
                let length = usize::min(
                    if src_pos < op.src_pos {
                        op.src_pos - src_pos
                    } else {
                        0
                    },
                    if dest_pos < op.dest_pos {
                        op.dest_pos - dest_pos
                    } else {
                        0
                    },
                );
                if length > 0 {
                    blocks.push(MatchingBlock {
                        a: src_pos,
                        b: dest_pos,
                        size: length,
                    });
                }
                src_pos = op.src_pos;
                dest_pos = op.dest_pos;
            }

            match op.tag.as_str() {
                "replace" => {
                    src_pos += 1;
                    dest_pos += 1;
                }
                "delete" => {
                    src_pos += 1;
                }
                "insert" => {
                    dest_pos += 1;
                }
                _ => {
                    panic!("Invalid tag: {}", op.tag);
                }
            }
        }

        if src_pos < self.src_len || dest_pos < self.dest_len {
            let length = (self.src_len - src_pos).min(self.dest_len - dest_pos);
            if length > 0 {
                blocks.push(MatchingBlock {
                    a: src_pos,
                    b: dest_pos,
                    size: length,
                });
            }
        }

        blocks.push(MatchingBlock {
            a: self.src_len,
            b: self.dest_len,
            size: 0,
        });

        blocks
    }

    /**
    Convert Editops to a list of tuples.

    This is the equivalent of ``[x for x in editops]``
    */
    fn as_list(&self) -> Vec<Editop> {
        self.editops.clone()
    }

    // Copy the Editops.
    fn copy(&self) -> Self {
        self.clone()
    }

    fn inverse(&self) -> Self {
        let mut blocks = vec![];
        for op in self {
            let tag = match op.tag.as_str() {
                "replace" => "replace",
                "delete" => "insert",
                "insert" => "delete",
                _ => {
                    panic!("Invalid tag: {}", op.tag);
                }
            };
            blocks.push(Editop {
                tag: tag.to_string(),
                src_pos: op.dest_pos,
                dest_pos: op.src_pos,
            });
        }

        Editops {
            src_len: self.dest_len,
            dest_len: self.src_len,
            editops: blocks,
        }
    }

    // Remove a subsequence from the editops.
    fn remove_subsequence(&self, subsequence: Editops) -> PyResult<Editops> {
        let mut result = self.editops.clone();

        if subsequence.len() > self.len() {
            return Err(PyValueError::new_err(
                "Subsequence is longer than the original editops",
            ));
        }

        // offset to correct removed edit operation
        let mut offset = 0;
        let mut op_pos = 0;
        let mut result_pos = 0;

        for sop in subsequence {
            while op_pos != self.len() && sop != self.editops[op_pos] {
                result[result_pos] = self.editops[op_pos].clone();
                result[result_pos].src_pos += offset;
                result_pos += 1;
                op_pos += 1;
            }

            // element of subsequence not part of the sequence
            if op_pos == self.len() {
                return Err(PyValueError::new_err("Subsequence not found in editops"));
            }

            if sop.tag == "insert" {
                offset += 1;
            } else if sop.tag == "delete" {
                offset -= 1;
            }

            op_pos += 1;
        }

        while op_pos != self.len() {
            result[result_pos] = self.editops[op_pos].clone();
            result[result_pos].src_pos += offset;
            result_pos += 1;
            op_pos += 1;
        }

        Ok(Editops {
            src_len: self.src_len,
            dest_len: self.dest_len,
            editops: result[..result_pos].to_vec(),
        })
    }

    fn apply(&self, source_string: &str, destination_string: &str) -> String {
        let mut res_str = String::new();
        let mut src_pos = 0;

        for op in self {
            // matches between last and current editop
            while src_pos < op.src_pos {
                res_str.push_str(
                    &source_string
                        .chars()
                        .skip(src_pos)
                        .take(1)
                        .collect::<String>(),
                );
                src_pos += 1;
            }

            match op.tag.as_str() {
                "replace" => {
                    res_str.push_str(
                        &destination_string
                            .chars()
                            .skip(op.dest_pos)
                            .take(1)
                            .collect::<String>(),
                    );
                    src_pos += 1;
                }
                "insert" => {
                    res_str.push_str(
                        &destination_string
                            .chars()
                            .skip(op.dest_pos)
                            .take(1)
                            .collect::<String>(),
                    );
                }
                "delete" => {
                    src_pos += 1;
                }
                _ => {
                    panic!("Invalid tag: {}", op.tag);
                }
            }
        }

        // matches after the last editop
        while src_pos < source_string.len() {
            res_str.push_str(
                &source_string
                    .chars()
                    .skip(src_pos)
                    .take(1)
                    .collect::<String>(),
            );
            src_pos += 1;
        }

        res_str
    }
}

/**
Tuple like object describing an edit operation.
It is in the form (tag, src_start, src_end, dest_start, dest_end)

The tags are strings, with these meanings:

+-----------+-----------------------------------------------------+
| tag       | explanation                                         |
+===========+=====================================================+
| 'replace' | src[src_start:src_end] should be                    |
|           | replaced by dest[dest_start:dest_end]               |
+-----------+-----------------------------------------------------+
| 'delete'  | src[src_start:src_end] should be deleted.           |
|           | Note that dest_start==dest_end in this case.        |
+-----------+-----------------------------------------------------+
| 'insert'  | dest[dest_start:dest_end] should be inserted        |
|           | at src[src_start:src_start].                        |
|           | Note that src_start==src_end in this case.          |
+-----------+-----------------------------------------------------+
| 'equal'   | src[src_start:src_end] == dest[dest_start:dest_end] |
+-----------+-----------------------------------------------------+

Note
----
Opcode is compatible with the tuples returned by difflib's SequenceMatcher to make them
interoperable
*/
#[pyclass(eq, mapping, get_all, module = "crustyfuzz.distance")]
#[derive(Clone, Debug, PartialEq)]
pub struct Opcode {
    pub tag: String,
    pub src_start: usize,
    pub src_end: usize,
    pub dest_start: usize,
    pub dest_end: usize,
}

impl std::fmt::Display for Opcode {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "Opcode(tag={}, src_start={}, src_end={}, dest_start={}, dest_end={})",
            self.tag, self.src_start, self.src_end, self.dest_start, self.dest_end
        )
    }
}

#[pymethods]
impl Opcode {
    #[new]
    fn py_new(
        tag: String,
        src_start: usize,
        src_end: usize,
        dest_start: usize,
        dest_end: usize,
    ) -> Self {
        Opcode {
            tag,
            src_start,
            src_end,
            dest_start,
            dest_end,
        }
    }

    fn __len__(&self) -> usize {
        5
    }

    fn __getitem__(slf: PyRef<'_, Self>, idx: isize) -> PyResult<IndexResult> {
        let idx = if idx < 0 { 5 + idx } else { idx };
        match idx {
            0 => Ok(IndexResult::String(slf.tag.clone())),
            1 => Ok(IndexResult::Integer(slf.src_start)),
            2 => Ok(IndexResult::Integer(slf.src_end)),
            3 => Ok(IndexResult::Integer(slf.dest_start)),
            4 => Ok(IndexResult::Integer(slf.dest_end)),
            _ => Err(PyIndexError::new_err("Opcode index out of range")),
        }
    }

    fn __repr__(&self) -> String {
        self.to_string()
    }

    fn __iter__(slf: PyRef<'_, Self>) -> PyRef<'_, Self> {
        slf
    }
}

/**
List like object of Opcodes describing how to turn s1 into s2.
The first Opcode has src_start == dest_start == 0, and remaining tuples
have src_start == the src_end from the tuple preceding it,
and likewise for dest_start == the previous dest_end.
*/
#[pyclass(eq, mapping, get_all, module = "crustyfuzz.distance")]
#[derive(Clone, Debug, PartialEq)]
pub struct Opcodes {
    src_len: usize,
    dest_len: usize,
    opcodes: Vec<Opcode>,
}

impl std::fmt::Display for Opcodes {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "Opcodes(src_len={}, dest_len={}, opcodes={:?})",
            self.src_len, self.dest_len, self.opcodes
        )
    }
}

impl IntoIterator for Opcodes {
    type Item = Opcode;
    type IntoIter = std::vec::IntoIter<Opcode>;
    fn into_iter(self) -> Self::IntoIter {
        self.opcodes.into_iter()
    }
}

impl<'a> IntoIterator for &'a Opcodes {
    type Item = &'a Opcode;
    type IntoIter = std::slice::Iter<'a, Opcode>;
    fn into_iter(self) -> Self::IntoIter {
        self.opcodes.iter()
    }
}

#[pymethods]
impl Opcodes {
    #[new]
    fn py_new(src_len: usize, dest_len: usize, opcodes: Vec<Opcode>) -> Self {
        Opcodes {
            src_len,
            dest_len,
            opcodes,
        }
    }

    fn __len__(&self) -> usize {
        self.opcodes.len()
    }

    fn __getitem__(&self, index: usize) -> PyResult<Opcode> {
        if index >= self.opcodes.len() {
            return Err(PyIndexError::new_err("Opcode index out of range"));
        }
        Ok(self.opcodes[index].clone())
    }

    fn __iter__(slf: PyRef<'_, Self>) -> PyRef<'_, Self> {
        slf
    }

    fn __repr__(&self) -> String {
        self.to_string()
    }

    #[classmethod]
    fn from_editops(_cls: &Bound<'_, PyType>, editops: Editops) -> Opcodes {
        editops.as_opcodes()
    }

    fn as_editops(&self) -> Editops {
        let mut blocks = vec![];
        for op in self {
            match op.tag.as_str() {
                "replace" => {
                    for j in 0..(op.src_end - op.src_start) {
                        blocks.push(Editop {
                            tag: "replace".to_string(),
                            src_pos: op.src_start + j,
                            dest_pos: op.dest_start + j,
                        });
                    }
                }
                "insert" => {
                    for j in 0..(op.dest_end - op.dest_start) {
                        blocks.push(Editop {
                            tag: "insert".to_string(),
                            src_pos: op.src_start,
                            dest_pos: op.dest_start + j,
                        });
                    }
                }
                "delete" => {
                    for j in 0..(op.src_end - op.src_start) {
                        blocks.push(Editop {
                            tag: "delete".to_string(),
                            src_pos: op.src_start + j,
                            dest_pos: op.dest_start,
                        });
                    }
                }
                "equal" => {}
                _ => {
                    panic!("Invalid tag: {}", op.tag);
                }
            }
        }

        Editops {
            src_len: self.src_len,
            dest_len: self.dest_len,
            editops: blocks,
        }
    }

    // Convert to matching blocks.
    pub fn as_matching_blocks(&self) -> Vec<MatchingBlock> {
        let mut blocks = vec![];
        for op in self {
            if op.tag == "equal" {
                let length = (op.src_end - op.src_start).min(op.dest_end - op.dest_start);
                if length > 0 {
                    blocks.push(MatchingBlock {
                        a: op.src_start,
                        b: op.dest_start,
                        size: length,
                    });
                }
            }
        }
        blocks.push(MatchingBlock {
            a: self.src_len,
            b: self.dest_len,
            size: 0,
        });
        blocks
    }

    fn as_list(&self) -> Vec<Opcode> {
        self.opcodes.clone()
    }

    fn copy(&self) -> Self {
        self.clone()
    }

    /**
    Invert Opcodes, so it describes how to transform the destination string to
    the source string.

    Returns
    -------
    opcodes : Opcodes
        inverted Opcodes

    Examples
    --------
    \>>> from rapidfuzz.distance import Levenshtein
    \>>> Levenshtein.opcodes('spam', 'park')
    [Opcode(tag=delete, src_start=0, src_end=1, dest_start=0, dest_end=0),
        Opcode(tag=equal, src_start=1, src_end=3, dest_start=0, dest_end=2),
        Opcode(tag=replace, src_start=3, src_end=4, dest_start=2, dest_end=3),
        Opcode(tag=insert, src_start=4, src_end=4, dest_start=3, dest_end=4)]

    \>>> Levenshtein.opcodes('spam', 'park').inverse()
    [Opcode(tag=insert, src_start=0, src_end=0, dest_start=0, dest_end=1),
        Opcode(tag=equal, src_start=0, src_end=2, dest_start=1, dest_end=3),
        Opcode(tag=replace, src_start=2, src_end=3, dest_start=3, dest_end=4),
        Opcode(tag=delete, src_start=3, src_end=4, dest_start=4, dest_end=4)]
    */
    fn inverse(&self) -> Opcodes {
        let mut blocks = vec![];
        for op in self {
            let tag = match op.tag.as_str() {
                "replace" => "replace",
                "delete" => "insert",
                "insert" => "delete",
                _ => {
                    panic!("Invalid tag: {}", op.tag);
                }
            };
            blocks.push(Opcode {
                tag: tag.to_string(),
                src_start: op.dest_start,
                src_end: op.dest_end,
                dest_start: op.src_start,
                dest_end: op.src_end,
            });
        }
        Opcodes {
            src_len: self.dest_len,
            dest_len: self.src_len,
            opcodes: blocks,
        }
    }

    /**
    Apply opcodes to source_string.

    Parameters
    ----------
    source_string : str | bytes
        string to apply opcodes to
    destination_string : str | bytes
        string to use for replacements / insertions into source_string

    Returns
    -------
    mod_string : str
        modified source_string
    */
    fn apply(&self, source_string: &str, destination_string: &str) -> String {
        let mut res_str = String::new();

        for op in self.opcodes.iter() {
            match op.tag.as_str() {
                "equal" => {
                    res_str.push_str(
                        &source_string
                            .chars()
                            .skip(op.src_start)
                            .take(op.src_end - op.src_start)
                            .collect::<String>(),
                    );
                }
                "replace" | "insert" => {
                    res_str.push_str(
                        &destination_string
                            .chars()
                            .skip(op.dest_start)
                            .take(op.dest_end - op.dest_start)
                            .collect::<String>(),
                    );
                }
                _ => {}
            }
        }

        res_str
    }
}
