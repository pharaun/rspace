mod parser;
//mod labeler;
mod assembler;

pub fn parse_asm(input: &str) -> Vec<u8> {
    let inst = parser::parse_asm_inst(
        input
    ).unwrap().1;

    // TODO: process labeling + symbol table expansions + minimalizing inst size
// labeler::symbol_table_expansion(

    // Output object code
    assembler::generate_object_code(
        inst
    )
}
