use std::iter::Peekable;
use asm::lexer;
use asm::parser;

use vm::opcode;

// Use the reg and csr func here for now
use asm::ast;
use std::str::FromStr;

// TODO:
// 1. newtype im
// 3. this is a final step before assembler (it cleans up the stream for the assembler)
//      - prior stage handles macro expansion
//      - assembler handles label lookup

#[derive(Debug, PartialEq)]
pub enum CImmLabel {
    Label(String, parser::LabelType),
    Imm(u32),
}

#[derive(Debug, PartialEq)]
pub enum CToken {
    Label(String, parser::LabelType),

    // Inst, rd, rs1, rs2
    // 3 length
    RegRegReg(String, ast::Reg, ast::Reg, ast::Reg),

    // Inst, rd, rs, (imm/label)
    // 3 length
    RegRegIL(String, ast::Reg, ast::Reg, CImmLabel),

    // Inst, rd, imm, csr
    // 3 length
    RegImmCsr(String, ast::Reg, u32, ast::Csr),

    // Inst, rd, rs, csr
    // 3 length
    RegRegCsr(String, ast::Reg, ast::Reg, ast::Csr),

    // Inst, rd, (imm/label)
    // 2 length
    RegIL(String, ast::Reg, CImmLabel),

    // Custom inst + macros?
    // FENCE, FENCE.I, ECALL, EBREAK
    Custom(String, Vec<parser::Arg>),
}


// Cleaner
pub struct Cleaner<'a> {
    input_iter: Peekable<parser::Parser<'a>>,
}


impl<'a> Cleaner<'a> {
    pub fn new(input: parser::Parser<'a>) -> Cleaner<'a> {
        Cleaner { input_iter: input.peekable() }
    }

    fn discard_token(&mut self) {
        let _ = self.input_iter.next();
    }

    fn read_token(&mut self) -> Option<parser::PToken> {
        self.input_iter.next()
    }

    fn peek_token(&mut self) -> Option<&parser::PToken> {
        self.input_iter.peek()
    }

    pub fn next_token(&mut self) -> Option<CToken> {
        if let Some(t) = self.read_token() {
            match t {
                // 0. Forward labels
                parser::PToken::Label(s, lt) => Some(CToken::Label(s, lt)),
                parser::PToken::Inst(s, args) => {
                    // 1. upper inst
                    // 2. lookup inst (if not found error out)
                    // 3. pick apropos cleaned type (for the assembler) depending on inst+context
                    None
                    //pub enum Arg {
                    //    Num(u32),
                    //    Label(String, LabelType),
                    //    Reg(ast::Reg),
                    //    Csr(String),
                    //}
                },
            }
        } else {
            None
        }
    }
}

impl<'a> Iterator for Cleaner<'a> {
    type Item = CToken;
    fn next(&mut self) -> Option<CToken> {
        self.next_token()
    }
}


#[cfg(test)]
pub mod cleaner_ast {
    use super::*;

    #[test]
    fn test_R_inst() {
        let input = "la: 2: add x0 x1 x2 // Comments";
        let mut cleaner = Cleaner::new(parser::Parser::new(lexer::Lexer::new(input)));

        let expected = vec![
            Some(CToken::Label("la".to_string(), parser::LabelType::Global)),
            Some(CToken::Label("2".to_string(), parser::LabelType::Local)),
            Some(CToken::RegRegReg("ADDI".to_string(), ast::Reg::X0, ast::Reg::X1, ast::Reg::X2)),
            None,
        ];

        // Assert
        for e in expected.iter() {
            let t = &cleaner.next_token();
            println!("expected {:?}, parsed {:?} ", e, t);
            assert_eq!(e, t);
        }
    }
}
