use crate::indel::normalized_similarity;

/**
Calculates the normalized Indel distance.

Parameters
----------
s1 : Option<&str>
    First string to compare.
s2 : Option<&str>
    Second string to compare.
processor: Option<fn(Vec<char>) -> Vec<char>>
    Optional callable that is used to preprocess the strings before
    comparing them. Default is None, which deactivates this behaviour.
score_cutoff : Option<f64>
    Optional argument for a score threshold as a float between 0 and 100.
    For ratio < score_cutoff 0 is returned instead. Default is 0,
    which deactivates this behaviour.

Returns
-------
similarity : f64
    similarity between s1 and s2 as a float between 0 and 100

Examples
--------
>>> fuzz::ratio(Some("this is a test"), Some("this is a test!"), None, None)
96.55171966552734
*/
pub fn ratio(
    s1: Option<&str>,
    s2: Option<&str>,
    processor: Option<fn(Vec<char>) -> Vec<char>>,
    score_cutoff: Option<f64>,
) -> f64 {
    match (s1, s2) {
        (Some(s1), Some(s2)) => {
            if s1.is_empty() || s2.is_empty() {
                return 0.0;
            }

            let mut score_cutoff = score_cutoff;
            if let Some(cutoff) = score_cutoff {
                score_cutoff = Some(cutoff / 100.0);
            }

            let s1_vec: Vec<char> = s1.chars().collect();
            let s2_vec: Vec<char> = s2.chars().collect();

            let score =
                normalized_similarity(Some(&s1_vec), Some(&s2_vec), processor, score_cutoff);
            score * 100.0
        }
        _ => 0.0,
    }
}
