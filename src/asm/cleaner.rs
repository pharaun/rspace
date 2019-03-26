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
                // TODO: find better way to handle ownership instead of mut the vec to claim ownership
                parser::PToken::Inst(s, mut args) => {
                    // 1. upper inst
                    let inst = s.to_ascii_uppercase();

                    // 2. lookup inst (if not found error out)
                    match opcode::lookup(&inst) {
                        None => {
                            panic!("Failed to find - {:?}", inst);
                        },
                        Some(x) => {
                            // 3. pick apropos cleaned type (for the assembler) depending on inst+context
                            match x.encoding {
                                opcode::InstType::R => {
                                    if args.len() != 3 {
                                        panic!("R type inst: {:?} arg: {:?}", inst, args);
                                    }
                                    Some(CToken::RegRegReg(
                                        inst,
                                        extract_reg(args.remove(0)),
                                        extract_reg(args.remove(0)),
                                        extract_reg(args.remove(0))
                                    ))
                                },
                                opcode::InstType::I => None,
                                opcode::InstType::S => None,
                                opcode::InstType::SB => None,
                                opcode::InstType::U => None,
                                opcode::InstType::UJ => None,
                            }
                        },
                    }
                },
            }
        } else {
            None
        }
    }
}

fn extract_num(arg: parser::Arg) -> u32 {
    match arg {
        parser::Arg::Num(n) => n,
        _ => panic!("Expected a Num, got {:?}", arg),
    }
}

fn extract_label(arg: parser::Arg) -> (String, parser::LabelType) {
    match arg {
        parser::Arg::Label(l, lt) => (l, lt),
        _ => panic!("Expected a Label, got {:?}", arg),
    }
}

fn extract_reg(arg: parser::Arg) -> ast::Reg {
    match arg {
        parser::Arg::Reg(n) => n,
        _ => panic!("Expected a Reg, got {:?}", arg),
    }
}

fn extract_csr(arg: parser::Arg) -> ast::Csr {
    match arg {
        parser::Arg::Csr(n) => n,
        _ => panic!("Expected a Csr, got {:?}", arg),
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
            Some(CToken::RegRegReg("ADD".to_string(), ast::Reg::X0, ast::Reg::X1, ast::Reg::X2)),
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
