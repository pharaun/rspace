extern crate rspace;

fn main() {
    // TODO: ingest asm from file or stdin, emit to file or stdout the binary code
    // Test asm code
    let test_asm = r#"
la:
        addi x0 x0 1
lb:     addi x0 x0 2
        beq x0 x0 2f
1:
        addi x0 x0 4
        beq x0 x0 1b
        jal x0 la
        beq x0 x0 2f
2:      addi x0 x0 5
        beq x0 x0 1b
        jal x0 lb
    "#;

    rspace::asm::parse_asm(test_asm);
}
