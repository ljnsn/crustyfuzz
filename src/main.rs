use crustyfuzz::fuzz::*;

fn main() {
    let s1 = "this is a test";
    let s2 = "this is a test!";
    let score = ratio(Some(s1), Some(s2), None, None);
    println!("{:?}", score);
}
