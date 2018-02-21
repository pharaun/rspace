extern crate rspace;

use rspace::asm;

fn main() {
    // TODO: ingest asm from file or stdin, emit to file or stdout the binary code
    let binary_code = rspace::asm::parse_asm("add x0 x0 x0");

    // Test number parse
    println!("{:?}", rspace::parse::parse_Number("09213"));
    println!("{:?}", rspace::parse::parse_Number("009213"));
    println!("{:?}", rspace::parse::parse_Number("0xFF"));
    println!("{:?}", rspace::parse::parse_Number("0xff"));
    println!("{:?}", rspace::parse::parse_Number("0x09123"));

    // Test register
    println!("{:?}", rspace::parse::parse_Register("x0"));
    println!("{:?}", rspace::parse::parse_Register("x31"));

    // Test CSR
    println!("{:?}", rspace::parse::parse_Csr("CYCLE"));
    println!("{:?}", rspace::parse::parse_Csr("CYCLEH"));

    // Test Arguments
    println!("{:?}", rspace::parse::parse_Arguments("x0"));
    println!("{:?}", rspace::parse::parse_Arguments("0923"));
    println!("{:?}", rspace::parse::parse_Arguments("0xFF"));
    println!("{:?}", rspace::parse::parse_Arguments("0xff"));

    // Test list of args
    println!("{:?}", rspace::parse::parse_VecArgs(""));
    println!("{:?}", rspace::parse::parse_VecArgs("0xFF"));
    println!("{:?}", rspace::parse::parse_VecArgs("0xFF x0"));
    println!("{:?}", rspace::parse::parse_VecArgs("0xFF x0 0923"));
    println!("{:?}", rspace::parse::parse_VecArgs("0xFF x0 0923 x2"));

    // Test Asm line
    println!("{:?}", rspace::parse::parse_AsmLine("ECALL"));
    println!("{:?}", rspace::parse::parse_AsmLine("CSRRS x0 x1 CYCLE"));
    println!("{:?}", rspace::parse::parse_AsmLine("CSRRS x0 x1 CYCLEH"));
    println!("{:?}", rspace::parse::parse_AsmLine("SFENCE.VM x0"));
    println!("{:?}", rspace::parse::parse_AsmLine("LUI x0 0xFF"));
    println!("{:?}", rspace::parse::parse_AsmLine("LUI x0 0xff"));
    println!("{:?}", rspace::parse::parse_AsmLine("FCVT.W.H x0 x1"));
    println!("{:?}", rspace::parse::parse_AsmLine("FMADD.S x0 x1 x2 x3"));
    println!("{:?}", rspace::parse::parse_AsmLine("csrrci x6 0x3 INSTRET"));

    // Test lookups
    println!("{:?}", rspace::opcode::lookup("ADDI"));
    println!("{:?}", rspace::opcode::lookup("SRA"));
    println!("{:?}", rspace::opcode::lookup("NOP"));

    // Test text labels

    // Test numeric labels

    // test absolute addressing

    // test relative addressing
}
