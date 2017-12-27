extern crate rspace;

use std::str;
use rspace::parse;

fn main() {
    let test_asm: &'static str = include_str!("test.asm");
    println!("Hello");
}



#[test]
fn comment_test() {
	assert_eq!(";;", ";;");
}
