mod parser;
mod labeler;
mod assembler;

pub fn parse_asm(input: &str) -> Vec<u8> {
    let inst = parser::parse_asm_inst(
        input
    ).unwrap().1;

    // First pass scan to extract the labels + usage
    let symbol_table = labeler::extract(&inst);

    // TODO:
    //  - A pass to handle symbol table expansion (ie constants being inserted into the instruction stream
    //  - A pass where it iterates 1 or more time to optimize the instruction size & address
    //  - A final pass for actually generating the instruction to pass on to the assembler

    // Output object code
    assembler::generate_object_code(
        inst
    )
}
