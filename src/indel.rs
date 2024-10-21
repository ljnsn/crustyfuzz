use crate::common::common::{conv_sequences, Hashable};
use crate::common::utils::utils::is_none;
use crate::lcs_seq::similarity;
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
