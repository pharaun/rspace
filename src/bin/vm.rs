extern crate rspace;

use std::str;
use rspace::parse;
use rspace::types;
use rspace::opcode;

fn main() {
    let test_asm: &'static str = include_str!("test.asm");
    println!("Hello");

    // Test number parse
    println!("{:?}", rspace::parse::parse_Number("09213"));
    println!("{:?}", rspace::parse::parse_Number("009213"));
    println!("{:?}", rspace::parse::parse_Number("0xFF"));
    println!("{:?}", rspace::parse::parse_Number("0x09123"));

    // Test register
    println!("{:?}", rspace::parse::parse_Register("x0"));
    println!("{:?}", rspace::parse::parse_Register("x31"));

    // Test Arguments
    println!("{:?}", rspace::parse::parse_Arguments("x0"));
    println!("{:?}", rspace::parse::parse_Arguments("0923"));
    println!("{:?}", rspace::parse::parse_Arguments("0xFF"));

    // Test list of args
    println!("{:?}", rspace::parse::parse_VecArgs(""));
    println!("{:?}", rspace::parse::parse_VecArgs("0xFF"));
    println!("{:?}", rspace::parse::parse_VecArgs("0xFF x0"));
    println!("{:?}", rspace::parse::parse_VecArgs("0xFF x0 0923"));
    println!("{:?}", rspace::parse::parse_VecArgs("0xFF x0 0923 x2"));

    // Test Asm line
    println!("{:?}", rspace::parse::parse_AsmLine("ECALL"));
    println!("{:?}", rspace::parse::parse_AsmLine("SFENCE.VM x0"));
    println!("{:?}", rspace::parse::parse_AsmLine("LUI x0 0xFF"));
    println!("{:?}", rspace::parse::parse_AsmLine("FCVT.W.H x0 x1"));
    println!("{:?}", rspace::parse::parse_AsmLine("FMADD.S x0 x1 x2 x3"));

    // Test lookups
    println!("{:?}", rspace::opcode::lookup("ADDI"));
    println!("{:?}", rspace::opcode::lookup("SRA"));
    println!("{:?}", rspace::opcode::lookup("NOP"));
}

#[test]
fn comment_test() {
	assert_eq!(";;", ";;");
}
