use num_bigint::BigUint;
use std::collections::HashMap;
use std::fmt::Binary;

/**
Counts zero bits in the least significant `bit_length` bits of a number

# Arguments
* `num` - The number to examine
* `bit_length` - Number of least significant bits to consider (1-64)

# Returns
Number of zero bits in the specified range

# Panics
Panics if `bit_length` is 0 or greater than 64
*/
#[inline]
const fn count_trailing_zeros_in_range(num: u64, bit_length: usize) -> usize {
    assert!(
        bit_length > 0 && bit_length <= 64,
        "bit_length must be between 1 and 64"
    );

    let mask = match bit_length {
        64 => u64::MAX,
        n => (1_u64 << n).wrapping_sub(1),
    };

    bit_length - (num & mask).count_ones() as usize
}

// Counts the number of zeros in a binary string
fn count_zeros_in_binary_string<T: Binary>(s: T, s1: &Vec<u64>) -> usize {
    let binary_string = format!("{:b}", s);
    let start_index = binary_string.len().saturating_sub(s1.len());
    let slice = &binary_string[start_index..];
    slice.chars().filter(|&c| c == '0').count()
}

/**
Calculates the length of the longest common subsequence

Parameters
----------
s1 : &Vec<u64>
    First string to compare.
s2 : &Vec<u64>
    Second string to compare.
score_cutoff : Option<f64>
    Maximum distance between s1 and s2, that is
    considered as a result. If the similarity is smaller than score_cutoff,
    0 is returned instead. Default is None, which deactivates
    this behaviour.

Returns
-------
similarity : f64
    similarity between s1 and s2
*/
pub fn similarity(s1: &Vec<u64>, s2: &Vec<u64>, score_cutoff: Option<f64>) -> f64 {
    if s1.is_empty() {
        return 0.0;
    }

    let mut s = (BigUint::from(1u32) << s1.len()) - BigUint::from(1u32);
    let mut block = HashMap::<u64, u64>::new();
    let mut x = 1;
    for ch1 in s1 {
        *block.entry(*ch1).or_insert(0) |= x;
        x <<= 1;
    }

    for ch2 in s2 {
        let matches = BigUint::from(*block.get(&ch2).unwrap_or(&0));
        let u = &s & &matches;
        s = (&s + &u) | (&s - &u);
    }

    // let s1_s: Vec<_> = s1.iter().map(|v| v.clone()).collect();
    // calculate the equivalent of popcount(~S) in C. This breaks for len(s1) == 0
    let res = count_zeros_in_binary_string(s, s1) as f64;

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

    let mut s = (BigUint::from(1u32) << s1.len()) - BigUint::from(1u32);
    for ch2 in s2 {
        let matches = BigUint::from(*block.get(&ch2).unwrap_or(&0));
        let u = &s & &matches;
        s = (&s + &u) | (&s - &u);
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
    use crate::common::conv_sequences;

    #[test]
    fn test_count_zeros_in_binary_string() {
        let s = 0b1010;
        let s1 = vec![1, 0, 1, 0];
        let result = count_zeros_in_binary_string(s, &s1);

        assert_eq!(result, 2, "Expected 2 zeros in binary string");
    }

    #[test]
    fn test_count_trailing_zeros_in_range() {
        assert_eq!(count_trailing_zeros_in_range(0b1010, 4), 2);
        assert_eq!(count_trailing_zeros_in_range(0, 64), 64);
        assert_eq!(count_trailing_zeros_in_range(u64::MAX, 64), 0);
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
            "Expected similarity of 14.0 for '{}' and '{}', got {}",
            s1, s2, result
        );
    }
}
