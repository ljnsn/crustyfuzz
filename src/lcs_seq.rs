use std::collections::HashMap;

fn count_zeros_in_binary_string(s: u64, s1: &Vec<u64>) -> usize {
    let binary_string = format!("{:b}", s);
    let start_index = binary_string.len().saturating_sub(s1.len());
    let slice = &binary_string[start_index..];
    slice.chars().filter(|&c| c == '0').count()
}

/**
Calculates the length of the longest common subsequence

Parameters
----------
s1 : Sequence[Hashable]
    First string to compare.
s2 : Sequence[Hashable]
    Second string to compare.
processor: callable, optional
    Optional callable that is used to preprocess the strings before
    comparing them. Default is None, which deactivates this behaviour.
score_cutoff : int, optional
    Maximum distance between s1 and s2, that is
    considered as a result. If the similarity is smaller than score_cutoff,
    0 is returned instead. Default is None, which deactivates
    this behaviour.

Returns
-------
similarity : int
    similarity between s1 and s2
*/
pub fn similarity(s1: &Vec<u64>, s2: &Vec<u64>, score_cutoff: Option<f64>) -> f64 {
    if s1.is_empty() {
        return 0.0;
    }

    let mut s = (1 << s1.len()) - 1;
    let mut block = HashMap::<u64, u64>::new();
    let mut x = 1;
    for ch1 in s1 {
        *block.entry(*ch1).or_insert(0) |= x;
        x <<= 1;
    }

    for ch2 in s2 {
        let matches = block.get(&ch2).unwrap_or(&0);
        let u = s & matches;
        s = (s + u) | (s - u);
    }

    // let s1_s: Vec<_> = s1.iter().map(|v| v.clone()).collect();
    // calculate the equivalent of popcount(~S) in C. This breaks for len(s1) == 0
    let res = count_zeros_in_binary_string(s, s1) as f64;

    dbg!(res);

    if score_cutoff.is_none() || res >= score_cutoff.unwrap() {
        res
    } else {
        score_cutoff.unwrap() + 0.0
    }
}

pub fn block_similarity(
    block: &HashMap<u64, u64>,
    s1: &Vec<u64>,
    s2: &Vec<u64>,
    score_cutoff: Option<f64>,
) -> f64 {
    if s1.is_empty() {
        return 0.0;
    }

    let mut s = (1 << s1.len()) - 1;
    for ch2 in s2 {
        let matches = block.get(&ch2).unwrap_or(&0);
        let u = s & matches;
        s = (s + u) | (s - u);
    }

    let res = count_zeros_in_binary_string(s, s1) as f64;

    if score_cutoff.is_none() || res >= score_cutoff.unwrap() {
        res
    } else {
        0.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::common::common::conv_sequences;

    #[test]
    fn test_count_zeros_in_binary_string() {
        let s = 0b1010;
        let s1 = vec![1, 0, 1, 0];
        let result = count_zeros_in_binary_string(s, &s1);

        assert_eq!(result, 2, "Expected 2 zeros in binary string");
    }

    #[test]
    fn test_similarity() {
        let s1 = "this is a test";
        let s2 = "this is a test!";
        let (seq1, seq2) = conv_sequences(
            &s1.chars().collect::<Vec<_>>(),
            &s2.chars().collect::<Vec<_>>(),
        );

        let result = similarity(&seq1, &seq2, None);

        assert_eq!(
            result, 14.0,
            "Expected similarity of 14.0 for '{}' and '{}'",
            s1, s2
        );
    }
}
