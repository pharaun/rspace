use std::str;


fn main() {
    let test_asm: &'static str = include_str!("test.asm");
    println!("Hello");
}



#[test]
fn comment_test() {
	assert_eq!(";;", ";;");
}
