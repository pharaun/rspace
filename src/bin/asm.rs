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

    //parser_test();
}

fn parser_test() {
    // Test number parse
    println!("Numbers:");
    println!("\t{:?}", rspace::asm::parse::parse_Number("09213"));
    println!("\t{:?}", rspace::asm::parse::parse_Number("009213"));
    println!("\t{:?}", rspace::asm::parse::parse_Number("-9213"));
    println!("\t{:?}", rspace::asm::parse::parse_Number("-009213"));
    println!("\t{:?}", rspace::asm::parse::parse_Number("0xFF"));
    println!("\t{:?}", rspace::asm::parse::parse_Number("0xff"));
    println!("\t{:?}", rspace::asm::parse::parse_Number("0x09123"));

    // Test register
    println!();
    println!("Register:");
    println!("\t{:?}", rspace::asm::parse::parse_Register("x0"));
    println!("\t{:?}", rspace::asm::parse::parse_Register("x31"));

    // Test CSR
    println!();
    println!("CSR:");
    println!("\t{:?}", rspace::asm::parse::parse_Arguments("CYCLE"));
    println!("\t{:?}", rspace::asm::parse::parse_Arguments("CYCLEH"));

    // Test Arguments
    println!();
    println!("Argument:");
    println!("\t{:?}", rspace::asm::parse::parse_Arguments("x0"));
    println!("\t{:?}", rspace::asm::parse::parse_Arguments("0923"));
    println!("\t{:?}", rspace::asm::parse::parse_Arguments("0xFF"));
    println!("\t{:?}", rspace::asm::parse::parse_Arguments("0xff"));
    println!("\t{:?}", rspace::asm::parse::parse_Arguments("label"));
    println!("\t{:?}", rspace::asm::parse::parse_Arguments("1b"));
    println!("\t{:?}", rspace::asm::parse::parse_Arguments("32f"));

    // Test list of args
    println!();
    println!("List Argument:");
    println!("\t{:?}", rspace::asm::parse::parse_VecArgs(""));
    println!("\t{:?}", rspace::asm::parse::parse_VecArgs("0xFF"));
    println!("\t{:?}", rspace::asm::parse::parse_VecArgs("0xFF x0"));
    println!("\t{:?}", rspace::asm::parse::parse_VecArgs("0xFF x0 0923"));
    println!("\t{:?}", rspace::asm::parse::parse_VecArgs("0xFF x0 0923 x2"));
    println!("\t{:?}", rspace::asm::parse::parse_VecArgs("0xFF x0 0923 label"));
    println!("\t{:?}", rspace::asm::parse::parse_VecArgs("0xFF x0 0923 1b"));
    println!("\t{:?}", rspace::asm::parse::parse_VecArgs("0xFF x0 0923 32f"));

    // Test Asm line
    println!();
    println!("Asm Line:");
    println!("\t{:?}", rspace::asm::parse::parse_AsmLine("ECALL"));
    println!("\t{:?}", rspace::asm::parse::parse_AsmLine("CSRRS x0 x1 CYCLE"));
    println!("\t{:?}", rspace::asm::parse::parse_AsmLine("CSRRS x0 x1 CYCLEH"));
    println!("\t{:?}", rspace::asm::parse::parse_AsmLine("SFENCE.VM x0"));
    println!("\t{:?}", rspace::asm::parse::parse_AsmLine("LUI x0 0xFF"));
    println!("\t{:?}", rspace::asm::parse::parse_AsmLine("LUI x0 0xff"));
    println!("\t{:?}", rspace::asm::parse::parse_AsmLine("FCVT.W.H x0 x1"));
    println!("\t{:?}", rspace::asm::parse::parse_AsmLine("FMADD.S x0 x1 x2 x3"));
    println!("\t{:?}", rspace::asm::parse::parse_AsmLine("csrrci x6 0x3 INSTRET"));

    // Test lookups
    println!();
    println!("Lookup:");
    println!("\t{:?}", rspace::vm::opcode::lookup("ADDI"));
    println!("\t{:?}", rspace::vm::opcode::lookup("SRA"));
    println!("\t{:?}", rspace::vm::opcode::lookup("NOP"));

    // Test text labels
    println!();
    println!("Text Label:");
    println!("\t{:?}", rspace::asm::parse::parse_AsmLine("label: ECALL"));
    println!("\t{:?}", rspace::asm::parse::parse_AsmLine("label:"));
    println!("\t{:?}", rspace::asm::parse::parse_AsmLine("BEQ x0 x0 label"));
    println!("\t{:?}", rspace::asm::parse::parse_AsmLine("label: BEQ x0 x0 label"));

    // Test numeric labels
    println!();
    println!("Numeric Label:");
    println!("\t{:?}", rspace::asm::parse::parse_AsmLine("1: ECALL"));
    println!("\t{:?}", rspace::asm::parse::parse_AsmLine("2:"));
    println!("\t{:?}", rspace::asm::parse::parse_AsmLine("BEQ x0 x0 3f"));
    println!("\t{:?}", rspace::asm::parse::parse_AsmLine("BEQ x0 x0 2b"));
    println!("\t{:?}", rspace::asm::parse::parse_AsmLine("1: BEQ x0 x0 1b"));

    // test absolute addressing

    // test relative addressing
}
