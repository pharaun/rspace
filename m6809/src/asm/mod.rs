mod parser;
//mod labeler;
//mod assembler;

pub fn parse_asm(input: &str) {
//pub fn parse_asm(input: &str) -> Vec<u8> {
//    assembler::generate_object_code(
//        labeler::symbol_table_expansion(
    parser::parse_asm_inst(
        input
    );
}
