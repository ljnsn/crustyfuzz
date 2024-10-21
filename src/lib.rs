mod utils {
    pub fn is_none<T>(s: Option<&[T]>) -> bool {
        match s {
            Some(slice) => slice.is_empty(),
            None => true,
        }
    }
}

mod common {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    pub trait Hashable {
        fn hash_value(&self) -> u64;
    }

    impl Hashable for char {
        fn hash_value(&self) -> u64 {
            *self as u64
        }
    }

    impl Hashable for u8 {
        fn hash_value(&self) -> u64 {
            *self as u64
        }
    }

    impl Hashable for u32 {
        fn hash_value(&self) -> u64 {
            *self as u64
        }
    }

    impl Hashable for u64 {
        fn hash_value(&self) -> u64 {
            *self
        }
    }

    impl Hashable for String {
        fn hash_value(&self) -> u64 {
            let mut hasher = DefaultHasher::new();
            self.hash(&mut hasher);
            hasher.finish()
        }
    }

    impl Hashable for Vec<u8> {
        fn hash_value(&self) -> u64 {
            let mut hasher = DefaultHasher::new();
            self.hash(&mut hasher);
            hasher.finish()
        }
    }

    pub fn conv_sequence<T: Hashable>(s: &[T]) -> Vec<u64> {
        s.iter().map(|elem| elem.hash_value()).collect()
    }

    pub fn conv_sequences<T: Hashable>(s1: &[T], s2: &[T]) -> (Vec<u64>, Vec<u64>) {
        (conv_sequence(s1), conv_sequence(s2))
    }
}

mod lcs_seq {
    use std::collections::HashMap;

    fn count_zeros_in_binary_string(s: u64, s1: &Vec<u64>) -> usize {
        let binary_string = format!("{:b}", s);
        let start_index = binary_string.len().saturating_sub(s1.len());
        let slice = &binary_string[start_index..];
        slice.chars().filter(|&c| c == '0').count()
    }

    pub fn similarity(s1: &Vec<u64>, s2: &Vec<u64>, score_cutoff: Option<f64>) -> f64 {
        if s1.is_empty() {
            return 0.0;
        }

        let mut s = (1 << s1.len()) - 1;
        let mut block = HashMap::<u64, u64>::new();
        let mut x = 1;
        for ch1 in s1 {
            block.get_mut(&ch1).map(|v| *v | x);
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

    #[cfg(test)]
    mod tests {
        use super::*;
        use crate::common::conv_sequences;

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
}

mod indel {
    use crate::common::{conv_sequences, Hashable};
    use crate::lcs_seq::similarity;
    use crate::utils::is_none;
    use std::clone::Clone;

    pub fn distance(s1: &Vec<u64>, s2: &Vec<u64>, score_cutoff: Option<f64>) -> f64 {
        let maximum = (s1.len() + s2.len()) as f64;
        let lcs_sim = similarity(s1, s2, None);
        let dist = maximum - 2.0 * lcs_sim;

        dbg!(dist);

        if score_cutoff.is_none() || dist <= score_cutoff.unwrap() {
            dist
        } else {
            score_cutoff.unwrap() + 1.0
        }
    }

    pub fn normalized_distance(s1: &Vec<u64>, s2: &Vec<u64>, score_cutoff: Option<f64>) -> f64 {
        let maximum = (s1.len() + s2.len()) as f64;
        let dist = distance(s1, s2, None);
        let norm_dist = if maximum == 0.0 { 0.0 } else { dist / maximum };

        dbg!(norm_dist);

        if score_cutoff.is_none() || norm_dist <= score_cutoff.unwrap() {
            norm_dist
        } else {
            1.0
        }
    }

    pub fn normalized_similarity<T: Hashable + Clone>(
        s1: Option<&[T]>,
        s2: Option<&[T]>,
        processor: Option<fn(Vec<T>) -> Vec<T>>,
        score_cutoff: Option<f64>,
    ) -> f64 {
        if is_none(s1) || is_none(s2) {
            return 0.0;
        }

        let s1_mut = s1.unwrap().to_vec();
        let s2_mut = s2.unwrap().to_vec();

        let (processed_s1, processed_s2) = match processor {
            Some(proc) => (proc(s1_mut), proc(s2_mut)),
            None => (s1_mut, s2_mut),
        };

        let (s1_seq, s2_seq) = conv_sequences(&processed_s1, &processed_s2);
        let norm_dist = normalized_distance(&s1_seq, &s2_seq, score_cutoff);
        let norm_sim = 1.0 - norm_dist;

        dbg!(norm_sim);

        if score_cutoff.is_none() || norm_sim >= score_cutoff.unwrap() {
            norm_sim
        } else {
            0.0
        }
    }
}

pub mod fuzz {
    use crate::indel::normalized_similarity;

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
}
