pub mod utils;

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

impl Hashable for usize {
    fn hash_value(&self) -> u64 {
        *self as u64
    }
}

impl Hashable for f64 {
    fn hash_value(&self) -> u64 {
        self.to_bits()
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

impl Hashable for Vec<char> {
    fn hash_value(&self) -> u64 {
        let mut hasher = DefaultHasher::new();
        self.iter().for_each(|c| c.hash(&mut hasher));
        hasher.finish()
    }
}

pub fn conv_sequence<T: Hashable>(s: &[T]) -> Vec<u64> {
    s.iter().map(|elem| elem.hash_value()).collect()
}

pub fn conv_sequences<T: Hashable>(s1: &[T], s2: &[T]) -> (Vec<u64>, Vec<u64>) {
    (conv_sequence(s1), conv_sequence(s2))
}
