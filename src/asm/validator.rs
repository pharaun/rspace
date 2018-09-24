use std::iter::Peekable;
use asm::lexer;
use asm::parser;

use vm::opcode;

// Use the reg and csr func here for now
use asm::ast;
use std::str::FromStr;


//#[derive(Debug, PartialEq)]
//pub enum Arg {
//    Num(u32),
//    Label(String, LabelType),
//    Reg(ast::Reg),
//    Csr(String),
//}
#[derive(Debug, PartialEq)]
pub enum VToken {
    Label(String, parser::LabelType),

    // Inst, rd, rs1, rs2
    // 3 length
    RInst(String, ast::Reg, ast::Reg, ast::Reg),

    // FENCE, FENCE.I, ECALL, EBREAK
    IInstCustom(String, Vec<parser::Arg>),

    // Inst, rd, imm, csr
    // 3 length
    IInstImmCSR(String, ast::Reg, u32, String),

    // Inst, rd, rs, csr
    // 3 length
    IInstRegCSR(String, ast::Reg, ast::Reg, String),

    // Inst, rd, rs, imm
    // 3 length
    IInstRegImm(String, ast::Reg, ast::Reg, u32),

    // Inst, rd, rs, label
    // 3 length
    IInstRegLabel(String, ast::Reg, ast::Reg, (String, parser::LabelType)),

    // Inst, rd, rs, imm
    // 3 length
    SInst(String, ast::Reg, ast::Reg, u32),

    // Inst rd, rs, imm
    // 3 length
    // Subtype of S
    SBInstImm(String, ast::Reg, ast::Reg, u32),

    // Inst rd, rs, imm
    // 3 length
    // Subtype of S
    SBInstLabel(String, ast::Reg, ast::Reg, (String, parser::LabelType)),

    // Inst, rd, imm
    // 2 length
    UInstImm(String, ast::Reg, u32),

    // Inst, rd, label
    // 2 length
    UInstLabel(String, ast::Reg, (String, parser::LabelType)),

    // Inst, rd, imm
    // 2 length
    // Subtype of U
    UJInstImm(String, ast::Reg, u32),

    // Inst, rd, label
    // 2 length
    // Subtype of U
    UJInstLabel(String, ast::Reg, (String, parser::LabelType)),
}


// Validator
pub struct Validator<'a> {
    input_iter: Peekable<parser::Parser<'a>>,
}


impl<'a> Validator<'a> {
    pub fn new(input: parser::Parser<'a>) -> Validator<'a> {
        Validator { input_iter: input.peekable() }
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

    pub fn next_token(&mut self) -> Option<VToken> {
        if let Some(t) = self.read_token() {
            // Steps:
            // 0. forward labels
            // 1. upper the inst
            // 2. lookup the inst, if not found (macros/not supported, skip for now)
            // 3. validate the inst
            None
        } else {
            None
        }
    }
}

impl<'a> Iterator for Validator<'a> {
    type Item = VToken;
    fn next(&mut self) -> Option<VToken> {
        self.next_token()
    }
}


#[cfg(test)]
pub mod validator_ast {
    use super::*;

    #[test]
    fn test_R_inst() {
        let input = "la: 2: add x0 x1 x2 // Comments";
        let mut validator = Validator::new(parser::Parser::new(lexer::Lexer::new(input)));

        let expected = vec![
            Some(VToken::Label("la".to_string(), parser::LabelType::Global)),
            Some(VToken::Label("2".to_string(), parser::LabelType::Local)),
            Some(VToken::RInst("ADDI".to_string(), ast::Reg::X0, ast::Reg::X1, ast::Reg::X2)),
            None,
        ];

        // Assert
        for e in expected.iter() {
            let t = &validator.next_token();
            println!("expected {:?}, parsed {:?} ", e, t);
            assert_eq!(e, t);
        }
    }
}
