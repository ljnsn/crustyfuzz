use crustyfuzz::fuzz::*;

fn main() {
    let s1 = "hello";
    let score = ratio(Some(s1), Some("hello"), None, None);
    println!("{:?}", score);
}
