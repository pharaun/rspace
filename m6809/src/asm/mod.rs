mod ast;
mod lexer;
mod parser;
//mod cleaner;
//mod labeler;
//mod assembler;

pub fn parse_asm(input: &str) {
//pub fn parse_asm(input: &str) -> Vec<u32> {
//    assembler::generate_object_code(
//        labeler::symbol_table_expansion(
//            cleaner::Cleaner::new(
//                parser::Parser::new(
                    lexer::Lexer::new(
                        input
                    );
//                )
//            )
//        )
//    )
}
