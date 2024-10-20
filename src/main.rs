use crustyfuzz::fuzz::*;

fn main() {
    let s1 = "hello";
    let score = ratio(s1.chars(), "hello", None, None);
    println!("{:?}", score);
}
