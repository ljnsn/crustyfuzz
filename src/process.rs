use crate::distance::get_scorer_flags;
use pyo3::prelude::*;
use pyo3::types::{IntoPyDict, PyIterator, PyList, PyMapping, PySequence, PyTuple};
use std::collections::HashMap;

fn get_scorer_bounds(
    scorer: &Bound<'_, PyAny>,
    scorer_kwargs: &HashMap<String, PyObject>,
) -> (usize, usize) {
    get_scorer_flags(scorer, scorer_kwargs).map_or((0, 100), |f| {
        (f.worst_score as usize, f.optimal_score as usize)
    })
}

type ExtractResult<'py> = (Bound<'py, PyAny>, f64, Bound<'py, PyAny>);

#[pyclass]
pub struct ExtractIter {
    inner: Vec<(PyObject, f64, PyObject)>,
    current: usize,
}

impl ExtractIter {
    pub fn new(inner: Option<Vec<(PyObject, f64, PyObject)>>) -> Self {
        Self {
            inner: inner.unwrap_or_default(),
            current: 0,
        }
    }
}

#[pymethods]
impl ExtractIter {
    fn __iter__(slf: PyRef<'_, Self>) -> PyRef<'_, Self> {
        slf
    }

    fn __next__(mut slf: PyRefMut<'_, Self>) -> Option<(PyObject, f64, PyObject)> {
        if slf.current >= slf.inner.len() {
            return None;
        }
        // let result = &slf.inner[slf.current].clone();
        // let result = (item.0.into_py(slf.py()), item.1, item.2.into_py(slf.py()));
        let (first, score, third) = &slf.inner[slf.current];
        let result = (first.clone_ref(slf.py()), *score, third.clone_ref(slf.py()));
        slf.current += 1;
        Some(result)
    }
}

#[pyclass]
struct Container {
    iter: Vec<(PyObject, f64, PyObject)>,
    // iter: Box<dyn Iterator<Item = (PyObject, f64, PyObject)> + Send>,
}

#[pymethods]
impl Container {
    fn __iter__(slf: PyRef<'_, Self>) -> PyResult<Py<ExtractIter>> {
        // let mut iter = ExtractIter::new(None);
        // iter.inner = slf.iter.clone();
        // Py::new(slf.py(), iter)
        let mut new_vec = Vec::with_capacity(slf.iter.len());
        for (first, score, third) in &slf.iter {
            new_vec.push((first.clone_ref(slf.py()), *score, third.clone_ref(slf.py())));
        }
        Py::new(slf.py(), ExtractIter::new(Some(new_vec)))
    }
}

/**
Find the best match in a list of choices

Parameters
----------
query : Sequence[Hashable]
    string we want to find
choices : Iterable[Sequence[Hashable]] | Mapping[Sequence[Hashable]]
    list of all strings the query should be compared with or dict with a mapping
    {<result>: <string to compare>}
scorer : Callable, optional
    Optional callable that is used to calculate the matching score between
    the query and each choice. This can be any of the scorers included in RapidFuzz
    (both scorers that calculate the edit distance or the normalized edit distance), or
    a custom function, which returns a normalized edit distance.
    fuzz.WRatio is used by default.
processor : Callable, optional
    Optional callable that is used to preprocess the strings before
    comparing them. Default is None, which deactivates this behaviour.
score_cutoff : Any, optional
    Optional argument for a score threshold. When an edit distance is used this represents the maximum
    edit distance and matches with a `distance > score_cutoff` are ignored. When a
    normalized edit distance is used this represents the minimal similarity
    and matches with a `similarity < score_cutoff` are ignored. Default is None, which deactivates this behaviour.
score_hint : Any, optional
    Optional argument for an expected score to be passed to the scorer.
    This is used to select a faster implementation. Default is None,
    which deactivates this behaviour.
scorer_kwargs : dict[str, Any], optional
    any other named parameters are passed to the scorer. This can be used to pass
    e.g. weights to `Levenshtein.distance`

Yields
-------
Tuple[Sequence[Hashable], Any, Any]
    Yields similarity between the query and each choice in form of a Tuple with 3 elements.
    The values stored in the tuple depend on the types of the input arguments.

    * The first element is always the current `choice`, which is the value that's compared to the query.

    * The second value represents the similarity calculated by the scorer. This can be:

        * An edit distance (distance is 0 for a perfect match and > 0 for non perfect matches).
        In this case only choices which have a `distance <= score_cutoff` are yielded.
        An example of a scorer with this behavior is `Levenshtein.distance`.
        * A normalized edit distance (similarity is a score between 0 and 100, with 100 being a perfect match).
        In this case only choices which have a `similarity >= score_cutoff` are yielded.
        An example of a scorer with this behavior is `Levenshtein.normalized_similarity`.

        Note, that for all scorers, which are not provided by RapidFuzz, only normalized edit distances are supported.

    * The third parameter depends on the type of the `choices` argument it is:

        * The `index of choice` when choices is a simple iterable like a list
        * The `key of choice` when choices is a mapping like a dict, or a pandas Series
*/
#[allow(clippy::too_many_arguments)]
#[pyfunction]
#[pyo3(
    name = "extract_iter",
    signature = (query, choices, scorer=None, processor=None, score_cutoff=None, score_hint=None, scorer_kwargs=None))
]
pub fn py_extract_iter<'py>(
    _py: Python,
    query: Option<&Bound<'py, PyAny>>,
    choices: &Bound<'py, PyAny>,
    scorer: Option<&Bound<'py, PyAny>>,
    processor: Option<&Bound<'py, PyAny>>,
    score_cutoff: Option<f64>,
    score_hint: Option<f64>,
    scorer_kwargs: Option<HashMap<String, PyObject>>,
) -> PyResult<Py<ExtractIter>> {
    // save for later use
    let _ = score_hint;

    let scorer = match scorer {
        Some(scorer) => scorer.to_owned(),
        None => PyModule::import_bound(_py, "crustyfuzz.fuzz")?.getattr("WRatio")?,
    };

    let mut scorer_kwargs = scorer_kwargs.unwrap_or_default();
    let (worst_score, optimal_score) = get_scorer_bounds(&scorer, &scorer_kwargs);
    scorer_kwargs.insert(
        "score_cutoff".to_string(),
        // TODO: make cutoff type dependent on scorer
        (score_cutoff.map(|c| c as usize)).into_py(_py),
    );
    let scorer_kwargs = scorer_kwargs.into_py_dict_bound(_py);

    let scorer_fn = move |a: &Bound<'_, PyAny>, b: &Bound<'_, PyAny>| -> PyResult<f64> {
        scorer.call((a, b), Some(&scorer_kwargs))?.extract()
    };

    let processor_fn = processor.map(|proc| {
        move |input: &Bound<'_, PyAny>| -> Bound<'_, PyAny> {
            proc.call1((input,)).expect("error calling processor")
        }
    });

    let results = extract_iter(
        query,
        choices,
        scorer_fn,
        processor_fn,
        score_cutoff,
        (worst_score, optimal_score),
    )?;

    let converted_results: Vec<(PyObject, f64, PyObject)> = results
        .into_iter()
        .map(|(choice, score, key)| (choice.into_py(_py), score, key.into_py(_py)))
        .collect();

    Py::new(_py, ExtractIter::new(Some(converted_results)))
}

pub fn extract_iter<'py>(
    query: Option<&Bound<'py, PyAny>>,
    choices: &Bound<'py, PyAny>,
    scorer: impl Fn(&Bound<'py, PyAny>, &Bound<'py, PyAny>) -> PyResult<f64>,
    processor: Option<impl Fn(&Bound<'py, PyAny>) -> Bound<'py, PyAny>>,
    score_cutoff: Option<f64>,
    bounds: (usize, usize),
) -> PyResult<Vec<ExtractResult<'py>>> {
    let (worst_score, optimal_score) = bounds;
    let lowest_score_worst = worst_score < optimal_score;

    if query.is_none() {
        return Ok(Vec::new());
    }

    let query = query.unwrap();
    let score_cutoff = score_cutoff.unwrap_or(worst_score as f64);

    let processed_query = match &processor {
        Some(proc) => proc(query),
        None => query.to_owned(),
    };

    let choices_iter = match choices.downcast::<PyMapping>() {
        Ok(mapping) => mapping.items()?.iter()?,
        Err(_) => {
            let items = choices
                .downcast::<PySequence>()?
                .iter()?
                .enumerate()
                .map(|(i, v)| {
                    PyTuple::new_bound(
                        query.py(),
                        &[i.into_py(query.py()), v.unwrap().into_py(query.py())],
                    )
                })
                .collect::<Vec<_>>();
            // Convert to sequence and get iterator
            PyIterator::from_bound_object(&PyList::new_bound(query.py(), items))?
        }
    };

    // choices_iter
    //     .into_iter()
    //     .filter_map(|item| {
    //         let key_choice = item
    //             .downcast::<PyTuple>()
    //             .expect("error converting to tuple");
    //         let key = key_choice.get_item(0).expect("error getting key");
    //         let choice = key_choice.get_item(1).expect("error getting choice");
    //
    //         if choice.is_none() {
    //             return None;
    //         }
    //
    //         let score = match &processor {
    //             Some(proc) => scorer(&processed_query, &proc(&choice)),
    //             None => scorer(&processed_query, &choice),
    //         };
    //
    //         if (lowest_score_worst && score >= score_cutoff)
    //             || (!lowest_score_worst && score <= score_cutoff)
    //         {
    //             Some((choice, score, key))
    //         } else {
    //             None
    //         }
    //     })
    //     .collect()

    // TODO: process items during iteration instead of collecting to vector first
    let mut results = Vec::new();

    for item in choices_iter {
        let item = item?;
        let key_choice = item.downcast::<PyTuple>()?;
        let key = key_choice.get_item(0)?;
        let choice = key_choice.get_item(1)?;

        if choice.is_none() {
            continue;
        }

        let score = match &processor {
            Some(proc) => scorer(&processed_query, &proc(&choice))?,
            None => scorer(&processed_query, &choice)?,
        };

        if (lowest_score_worst && score >= score_cutoff)
            || (!lowest_score_worst && score <= score_cutoff)
        {
            results.push((choice, score, key));
        }
    }

    Ok(results)
}

/**
Find the best match in a list of choices. When multiple elements have the same similarity,
the first element is returned.

Parameters
----------
query : Sequence[Hashable]
    string we want to find
choices : Iterable[Sequence[Hashable]] | Mapping[Sequence[Hashable]]
    list of all strings the query should be compared with or dict with a mapping
    {<result>: <string to compare>}
scorer : Callable, optional
    Optional callable that is used to calculate the matching score between
    the query and each choice. This can be any of the scorers included in RapidFuzz
    (both scorers that calculate the edit distance or the normalized edit distance), or
    a custom function, which returns a normalized edit distance.
    fuzz.WRatio is used by default.
processor : Callable, optional
    Optional callable that is used to preprocess the strings before
    comparing them. Default is None, which deactivates this behaviour.
score_cutoff : Any, optional
    Optional argument for a score threshold. When an edit distance is used this represents the maximum
    edit distance and matches with a `distance > score_cutoff` are ignored. When a
    normalized edit distance is used this represents the minimal similarity
    and matches with a `similarity < score_cutoff` are ignored. Default is None, which deactivates this behaviour.
score_hint : Any, optional
    Optional argument for an expected score to be passed to the scorer.
    This is used to select a faster implementation. Default is None,
    which deactivates this behaviour.
scorer_kwargs : dict[str, Any], optional
    any other named parameters are passed to the scorer. This can be used to pass
    e.g. weights to `Levenshtein.distance`

Returns
-------
Tuple[Sequence[Hashable], Any, Any]
    Returns the best match in form of a Tuple with 3 elements. The values stored in the
    tuple depend on the types of the input arguments.

    * The first element is always the `choice`, which is the value that's compared to the query.

    * The second value represents the similarity calculated by the scorer. This can be:

        * An edit distance (distance is 0 for a perfect match and > 0 for non perfect matches).
        In this case only choices which have a `distance <= score_cutoff` are returned.
        An example of a scorer with this behavior is `Levenshtein.distance`.
        * A normalized edit distance (similarity is a score between 0 and 100, with 100 being a perfect match).
        In this case only choices which have a `similarity >= score_cutoff` are returned.
        An example of a scorer with this behavior is `Levenshtein.normalized_similarity`.

        Note, that for all scorers, which are not provided by RapidFuzz, only normalized edit distances are supported.

    * The third parameter depends on the type of the `choices` argument it is:

        * The `index of choice` when choices is a simple iterable like a list
        * The `key of choice` when choices is a mapping like a dict, or a pandas Series

None
    When no choice has a `similarity >= score_cutoff`/`distance <= score_cutoff` None is returned

Examples
--------

>>> from rapidfuzz.process import extractOne
>>> from rapidfuzz.distance import Levenshtein
>>> from rapidfuzz.fuzz import ratio

extractOne can be used with normalized edit distances.

>>> extractOne("abcd", ["abce"], scorer=ratio)
("abcd", 75.0, 1)
>>> extractOne("abcd", ["abce"], scorer=Levenshtein.normalized_similarity)
("abcd", 0.75, 1)

extractOne can be used with edit distances as well.

>>> extractOne("abcd", ["abce"], scorer=Levenshtein.distance)
("abce", 1, 0)

additional settings of the scorer can be passed via the scorer_kwargs argument to extractOne

>>> extractOne("abcd", ["abce"], scorer=Levenshtein.distance, scorer_kwargs={"weights":(1,1,2)})
("abcde", 2, 1)

when a mapping is used for the choices the key of the choice is returned instead of the List index

>>> extractOne("abcd", {"key": "abce"}, scorer=ratio)
("abcd", 75.0, "key")

It is possible to specify a processor function which is used to preprocess the strings before comparing them.

>>> extractOne("abcd", ["abcD"], scorer=ratio)
("abcD", 75.0, 0)
>>> extractOne("abcd", ["abcD"], scorer=ratio, processor=utils.default_process)
("abcD", 100.0, 0)
>>> extractOne("abcd", ["abcD"], scorer=ratio, processor=lambda s: s.upper())
("abcD", 100.0, 0)

When only results with a similarity above a certain threshold are relevant, the parameter score_cutoff can be
used to filter out results with a lower similarity. This threshold is used by some of the scorers to exit early,
when they are sure, that the similarity is below the threshold.
For normalized edit distances all results with a similarity below score_cutoff are filtered out

>>> extractOne("abcd", ["abce"], scorer=ratio)
("abce", 75.0, 0)
>>> extractOne("abcd", ["abce"], scorer=ratio, score_cutoff=80)
None

For edit distances all results with an edit distance above the score_cutoff are filtered out

>>> extractOne("abcd", ["abce"], scorer=Levenshtein.distance, scorer_kwargs={"weights":(1,1,2)})
("abce", 2, 0)
>>> extractOne("abcd", ["abce"], scorer=Levenshtein.distance, scorer_kwargs={"weights":(1,1,2)}, score_cutoff=1)
None
*/
#[allow(clippy::too_many_arguments)]
#[pyfunction]
#[pyo3(
    name = "extract_one",
    signature = (query, choices, scorer=None, processor=None, score_cutoff=None, score_hint=None, scorer_kwargs=None))
]
pub fn py_extract_one<'py>(
    _py: Python,
    query: Option<&Bound<'py, PyAny>>,
    choices: &Bound<'py, PyAny>,
    scorer: Option<&Bound<'py, PyAny>>,
    processor: Option<&Bound<'py, PyAny>>,
    score_cutoff: Option<f64>,
    score_hint: Option<f64>,
    scorer_kwargs: Option<HashMap<String, PyObject>>,
) -> PyResult<Option<ExtractResult<'py>>> {
    // save for later use
    let _ = score_hint;

    let scorer = match scorer {
        Some(scorer) => scorer.to_owned(),
        None => PyModule::import_bound(_py, "crustyfuzz.fuzz")?.getattr("WRatio")?,
    };

    let mut scorer_kwargs = scorer_kwargs.unwrap_or_default();
    let (worst_score, optimal_score) = get_scorer_bounds(&scorer, &scorer_kwargs);
    scorer_kwargs.insert(
        "score_cutoff".to_string(),
        // TODO: make cutoff type dependent on scorer
        (score_cutoff.map(|c| c as usize)).into_py(_py),
    );
    let scorer_kwargs = scorer_kwargs.into_py_dict_bound(_py);

    let scorer_fn = move |a: &Bound<'_, PyAny>, b: &Bound<'_, PyAny>| -> PyResult<f64> {
        scorer.call((a, b), Some(&scorer_kwargs))?.extract()
    };

    let processor_fn = processor.map(|proc| {
        move |input: &Bound<'_, PyAny>| -> Bound<'_, PyAny> {
            proc.call1((input,)).expect("error calling processor")
        }
    });

    extract_one(
        query,
        choices,
        scorer_fn,
        processor_fn,
        score_cutoff,
        (worst_score, optimal_score),
    )
}

pub fn extract_one<'py>(
    query: Option<&Bound<'py, PyAny>>,
    choices: &Bound<'py, PyAny>,
    scorer: impl Fn(&Bound<'py, PyAny>, &Bound<'py, PyAny>) -> Result<f64, PyErr>,
    processor: Option<impl Fn(&Bound<'py, PyAny>) -> Bound<'py, PyAny>>,
    score_cutoff: Option<f64>,
    bounds: (usize, usize),
) -> PyResult<Option<ExtractResult<'py>>> {
    let (worst_score, optimal_score) = bounds;
    let lowest_score_worst = worst_score < optimal_score;

    if query.is_none() {
        return Ok(None);
    }

    let query = query.unwrap();
    let score_cutoff = score_cutoff.unwrap_or(worst_score as f64);

    let processed_query = match &processor {
        Some(proc) => proc(query),
        None => query.to_owned(),
    };

    let mut result: Option<(Bound<'py, PyAny>, f64, Bound<'py, PyAny>)> = None;

    // let choices_iter = match choices.downcast::<PyMapping>() {
    //     Ok(mapping) => mapping
    //         .items()
    //         .expect("error getting items")
    //         .iter()
    //         .expect("error getting iter"),
    //     Err(_) => choices
    //         .downcast::<PySequence>()
    //         .expect("error converting to sequence")
    //         .iter()
    //         .expect("error getting iter")
    //         .enumerate(),
    // };

    let choices_iter = match choices.downcast::<PyMapping>() {
        Ok(mapping) => mapping.items()?.iter()?,
        Err(_) => {
            let items = choices
                .downcast::<PySequence>()?
                .iter()?
                .enumerate()
                .map(|(i, v)| {
                    PyTuple::new_bound(
                        query.py(),
                        &[i.into_py(query.py()), v.unwrap().into_py(query.py())],
                    )
                })
                .collect::<Vec<_>>();
            // Convert to sequence and get iterator
            PyIterator::from_bound_object(&PyList::new_bound(query.py(), items))?
        }
    };

    let mut current_score_cutoff = score_cutoff;
    for item in choices_iter {
        let item = item?;
        let key_choice = item.downcast::<PyTuple>()?;
        let key = key_choice.get_item(0)?;
        let choice = key_choice.get_item(1)?;

        if choice.is_none() {
            continue;
        }

        let score = match &processor {
            Some(proc) => scorer(&processed_query, &proc(&choice))?,
            None => scorer(&processed_query, &choice)?,
        };

        let should_update = if lowest_score_worst {
            score >= current_score_cutoff
                && (result.is_none() || score > result.as_ref().unwrap().1)
        } else {
            score <= current_score_cutoff
                && (result.is_none() || score < result.as_ref().unwrap().1)
        };

        if should_update {
            current_score_cutoff = score;
            result = Some((choice, score, key));
        }

        if score == optimal_score as f64 {
            break;
        }
    }

    Ok(result)
}

/**
Find the best matches in a list of choices. The list is sorted by the similarity.
When multiple choices have the same similarity, they are sorted by their index

Parameters
----------
query : Sequence[Hashable]
    string we want to find
choices : Collection[Sequence[Hashable]] | Mapping[Sequence[Hashable]]
    list of all strings the query should be compared with or dict with a mapping
    {<result>: <string to compare>}
scorer : Callable, optional
    Optional callable that is used to calculate the matching score between
    the query and each choice. This can be any of the scorers included in RapidFuzz
    (both scorers that calculate the edit distance or the normalized edit distance), or
    a custom function, which returns a normalized edit distance.
    fuzz.WRatio is used by default.
processor : Callable, optional
    Optional callable that is used to preprocess the strings before
    comparing them. Default is None, which deactivates this behaviour.
limit : int, optional
    maximum amount of results to return. None can be passed to disable this behavior.
    Default is 5.
score_cutoff : Any, optional
    Optional argument for a score threshold. When an edit distance is used this represents the maximum
    edit distance and matches with a `distance > score_cutoff` are ignored. When a
    normalized edit distance is used this represents the minimal similarity
    and matches with a `similarity < score_cutoff` are ignored. Default is None, which deactivates this behaviour.
score_hint : Any, optional
    Optional argument for an expected score to be passed to the scorer.
    This is used to select a faster implementation. Default is None,
    which deactivates this behaviour.
scorer_kwargs : dict[str, Any], optional
    any other named parameters are passed to the scorer. This can be used to pass
    e.g. weights to `Levenshtein.distance`

Returns
-------
List[Tuple[Sequence[Hashable], Any, Any]]
    The return type is always a List of Tuples with 3 elements. However the values stored in the
    tuple depend on the types of the input arguments.

    * The first element is always the `choice`, which is the value that's compared to the query.

    * The second value represents the similarity calculated by the scorer. This can be:

        * An edit distance (distance is 0 for a perfect match and > 0 for non perfect matches).
        In this case only choices which have a `distance <= score_cutoff` are returned.
        An example of a scorer with this behavior is `Levenshtein.distance`.
        * A normalized edit distance (similarity is a score between 0 and 100, with 100 being a perfect match).
        In this case only choices which have a `similarity >= score_cutoff` are returned.
        An example of a scorer with this behavior is `Levenshtein.normalized_similarity`.

        Note, that for all scorers, which are not provided by RapidFuzz, only normalized edit distances are supported.

    * The third parameter depends on the type of the `choices` argument it is:

        * The `index of choice` when choices is a simple iterable like a list
        * The `key of choice` when choices is a mapping like a dict, or a pandas Series

    The list is sorted by similarity or distance depending on the scorer used. The first element in the list
    has the `highest similarity`/`smallest distance`.
*/
#[allow(clippy::too_many_arguments)]
#[pyfunction]
#[pyo3(
    name = "extract",
    signature = (query, choices, *, scorer=None, processor=None, limit=5, score_cutoff=None, score_hint=None, scorer_kwargs=None))
]
pub fn py_extract<'py>(
    _py: Python,
    query: Option<&Bound<'py, PyAny>>,
    choices: &Bound<'py, PyAny>,
    scorer: Option<&Bound<'py, PyAny>>,
    processor: Option<&Bound<'py, PyAny>>,
    limit: Option<usize>,
    score_cutoff: Option<f64>,
    score_hint: Option<f64>,
    scorer_kwargs: Option<HashMap<String, PyObject>>,
) -> PyResult<Vec<ExtractResult<'py>>> {
    // save for later use
    let _ = score_hint;

    let scorer = match scorer {
        Some(scorer) => scorer.to_owned(),
        None => PyModule::import_bound(_py, "crustyfuzz.fuzz")?.getattr("WRatio")?,
    };

    let mut scorer_kwargs = scorer_kwargs.unwrap_or_default();
    let (worst_score, optimal_score) = get_scorer_bounds(&scorer, &scorer_kwargs);
    scorer_kwargs.insert(
        "score_cutoff".to_string(),
        // TODO: make cutoff type dependent on scorer
        (score_cutoff.map(|c| c as usize)).into_py(_py),
    );
    let scorer_kwargs = scorer_kwargs.into_py_dict_bound(_py);

    let scorer_fn = move |a: &Bound<'_, PyAny>, b: &Bound<'_, PyAny>| -> PyResult<f64> {
        scorer.call((a, b), Some(&scorer_kwargs))?.extract()
    };

    let processor_fn = processor.map(|proc| {
        move |input: &Bound<'_, PyAny>| -> Bound<'_, PyAny> {
            proc.call1((input,)).expect("error calling processor")
        }
    });

    extract(
        query,
        choices,
        scorer_fn,
        processor_fn,
        limit,
        score_cutoff,
        (worst_score, optimal_score),
    )
}

pub fn extract<'py>(
    query: Option<&Bound<'py, PyAny>>,
    choices: &Bound<'py, PyAny>,
    scorer: impl Fn(&Bound<'py, PyAny>, &Bound<'py, PyAny>) -> PyResult<f64>,
    processor: Option<impl Fn(&Bound<'py, PyAny>) -> Bound<'py, PyAny>>,
    limit: Option<usize>,
    score_cutoff: Option<f64>,
    bounds: (usize, usize),
) -> PyResult<Vec<ExtractResult<'py>>> {
    let (worst_score, optimal_score) = bounds;
    let lowest_score_worst = worst_score < optimal_score;

    if limit == Some(1) {
        return Ok(extract_one(
            query,
            choices,
            scorer,
            processor,
            score_cutoff,
            (worst_score, optimal_score),
        )?
        .map_or_else(Vec::new, |res| vec![res]));
    }

    let mut results = extract_iter(
        query,
        choices,
        scorer,
        processor,
        score_cutoff,
        (worst_score, optimal_score),
    )?;

    results.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());
    if lowest_score_worst {
        results.reverse();
    }

    match limit {
        Some(limit) => {
            if lowest_score_worst {
                Ok(results.into_iter().take(limit).collect())
            } else {
                Ok(results.into_iter().rev().take(limit).collect())
            }
        }
        None => Ok(results),
    }
}
