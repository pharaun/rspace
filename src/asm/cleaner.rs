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

    // Inst, rd, imm, csr
    // 3 length
    RegImmCsr(String, ast::Reg, u32, ast::Csr),

    // Inst, rd, rs, csr
    // 3 length
    RegRegCsr(String, ast::Reg, ast::Reg, ast::Csr),

    // Inst, rd, rs1, shamt
    // 3 length
    RegRegShamt(String, ast::Reg, ast::Reg, u32),

    // Inst, rd, rs1, imm
    // 3 length
    RegRegImm(String, ast::Reg, ast::Reg, u32),

    // Inst, rd, rs1, imm
    // 3 length
    RegRegImmStore(String, ast::Reg, ast::Reg, u32),

    // Inst, rd, rs1, imm
    // 3 length
    RegRegILBranch(String, ast::Reg, ast::Reg, CImmLabel),

    // Inst, rd, rs, (imm/label)
    // 3 length
    RegRegIL(String, ast::Reg, ast::Reg, CImmLabel),

    // Inst, rd, (imm/label)
    // 2 length
    RegIL(String, ast::Reg, CImmLabel),

    // Inst, rd, (imm/label)
    // 2 length
    RegILShuffle(String, ast::Reg, CImmLabel),

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
                                opcode::InstType::I => {
                                    match &inst[..] {
                                        "FENCE" | "FENCE.I" | "ECALL" | "EBREAK" => {
                                            Some(CToken::Custom(inst, args))
                                        },
                                        "CSRRWI" | "CSRRSI" | "CSRRCI" => {
                                            if args.len() != 3 {
                                                panic!("I type inst: {:?} arg: {:?}", inst, args);
                                            }
                                            Some(CToken::RegImmCsr(
                                                inst,
                                                extract_reg(args.remove(0)),
                                                extract_imm(args.remove(0)),
                                                extract_csr(args.remove(0))
                                            ))
                                        },
                                        "CSRRW" | "CSRRS" | "CSRRC" => {
                                            if args.len() != 3 {
                                                panic!("I type inst: {:?} arg: {:?}", inst, args);
                                            }
                                            Some(CToken::RegRegCsr(
                                                inst,
                                                extract_reg(args.remove(0)),
                                                extract_reg(args.remove(0)),
                                                extract_csr(args.remove(0))
                                            ))
                                        },
                                        "SLLI" | "SRLI" | "SRAI" => {
                                            if args.len() != 3 {
                                                panic!("I type inst: {:?} arg: {:?}", inst, args);
                                            }
                                            Some(CToken::RegRegShamt(
                                                inst,
                                                extract_reg(args.remove(0)),
                                                extract_reg(args.remove(0)),
                                                extract_imm(args.remove(0))
                                            ))
                                        },
                                        "JALR" => {
                                            if args.len() != 3 {
                                                panic!("I type inst: {:?} arg: {:?}", inst, args);
                                            }
                                            Some(CToken::RegRegIL(
                                                inst,
                                                extract_reg(args.remove(0)),
                                                extract_reg(args.remove(0)),
                                                extract_imm_label(args.remove(0))
                                            ))
                                        },
                                        _ => {
                                            if args.len() != 3 {
                                                panic!("I type inst: {:?} arg: {:?}", inst, args);
                                            }
                                            Some(CToken::RegRegImm(
                                                inst,
                                                extract_reg(args.remove(0)),
                                                extract_reg(args.remove(0)),
                                                extract_imm(args.remove(0))
                                            ))
                                        },
                                    }
                                },
                                opcode::InstType::S => {
                                    if args.len() != 3 {
                                        panic!("S type inst: {:?} arg: {:?}", inst, args);
                                    }
                                    Some(CToken::RegRegImmStore(
                                        inst,
                                        extract_reg(args.remove(0)),
                                        extract_reg(args.remove(0)),
                                        extract_imm(args.remove(0))
                                    ))
                                },
                                opcode::InstType::SB => {
                                    if args.len() != 3 {
                                        panic!("SB type inst: {:?} arg: {:?}", inst, args);
                                    }
                                    Some(CToken::RegRegILBranch(
                                        inst,
                                        extract_reg(args.remove(0)),
                                        extract_reg(args.remove(0)),
                                        extract_imm_label(args.remove(0))
                                    ))
                                },
                                opcode::InstType::U => {
                                    if args.len() != 2 {
                                        panic!("U type inst: {:?} arg: {:?}", inst, args);
                                    }
                                    Some(CToken::RegIL(
                                        inst,
                                        extract_reg(args.remove(0)),
                                        extract_imm_label(args.remove(0))
                                    ))
                                },
                                opcode::InstType::UJ => {
                                    if args.len() != 2 {
                                        panic!("UJ type inst: {:?} arg: {:?}", inst, args);
                                    }
                                    Some(CToken::RegILShuffle(
                                        inst,
                                        extract_reg(args.remove(0)),
                                        extract_imm_label(args.remove(0))
                                    ))
                                },
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

fn extract_imm(arg: parser::Arg) -> u32 {
    match arg {
        parser::Arg::Num(n) => n,
        _ => panic!("Expected a Num, got {:?}", arg),
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

fn extract_imm_label(arg: parser::Arg) -> CImmLabel {
    match arg {
        parser::Arg::Num(n)         => CImmLabel::Imm(n),
        parser::Arg::Label(l, lt)   => CImmLabel::Label(l, lt),
        _ => panic!("Expected a ImmLabel, got {:?}", arg),
    }
}

fn llookup(inst: &str) -> opcode::InstEnc {
    match opcode::lookup(&inst) {
        None    => panic!("Failed to find - {:?}", inst),
        Some(x) => x,
    }
}

pub fn lookup(inst: &CToken) -> opcode::InstEnc {
    match inst {
        CToken::RegRegReg(i, _, _, _)       => llookup(&i),
        CToken::RegImmCsr(i, _, _, _)       => llookup(&i),
        CToken::RegRegCsr(i, _, _, _)       => llookup(&i),
        CToken::RegRegShamt(i, _, _, _)     => llookup(&i),
        CToken::RegRegImm(i, _, _, _)       => llookup(&i),
        CToken::RegRegImmStore(i, _, _, _)  => llookup(&i),
        CToken::RegRegIL(i, _, _, _)        => llookup(&i),
        CToken::RegRegILBranch(i, _, _, _)  => llookup(&i),
        CToken::RegIL(i, _, _)              => llookup(&i),
        CToken::RegILShuffle(i, _, _)       => llookup(&i),
        CToken::Custom(i, _)                => llookup(&i),
        CToken::Label(_, _)                 => panic!("Got a Label, this is bad"),
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

    fn assert_eq(input: &str, expected: Vec<Option<CToken>>) {
        let mut cleaner = Cleaner::new(parser::Parser::new(lexer::Lexer::new(input)));

        for e in expected.iter() {
            let t = &cleaner.next_token();
            println!("expected {:?}, parsed {:?} ", e, t);
            assert_eq!(e, t);
        }
    }

    #[test]
    fn test_labels() {
        let input = "la: 2: // Comments";

        let expected = vec![
            Some(CToken::Label("la".to_string(), parser::LabelType::Global)),
            Some(CToken::Label("2".to_string(), parser::LabelType::Local)),
            None,
        ];

        assert_eq(input, expected);
    }

    #[test]
    fn test_RegRegReg_inst() {
        let input = "add x0 x1 x2";

        let expected = vec![
            Some(CToken::RegRegReg("ADD".to_string(), ast::Reg::X0, ast::Reg::X1, ast::Reg::X2)),
            None,
        ];

        assert_eq(input, expected);
    }

    #[test]
    fn test_Custom_inst() {
        let input = "fence\n fence.i\n ecall\n ebreak";

        let expected = vec![
            Some(CToken::Custom("FENCE".to_string(), Vec::new())),
            Some(CToken::Custom("FENCE.I".to_string(), Vec::new())),
            Some(CToken::Custom("ECALL".to_string(), Vec::new())),
            Some(CToken::Custom("EBREAK".to_string(), Vec::new())),
            None,
        ];

        assert_eq(input, expected);
    }

    #[test]
    fn test_RegImmCsr_inst() {
        let input = "csrrwi x0 33 CYCLE\n csrrsi x1 11 CYCLEH\n csrrci x2 22 TIME";

        let expected = vec![
            Some(CToken::RegImmCsr("CSRRWI".to_string(), ast::Reg::X0, 33, ast::Csr::CYCLE)),
            Some(CToken::RegImmCsr("CSRRSI".to_string(), ast::Reg::X1, 11, ast::Csr::CYCLEH)),
            Some(CToken::RegImmCsr("CSRRCI".to_string(), ast::Reg::X2, 22, ast::Csr::TIME)),
            None,
        ];

        assert_eq(input, expected);
    }

    #[test]
    fn test_RegRegCsr_inst() {
        let input = "csrrw x0 x1 CYCLE\n csrrs x1 x2 CYCLEH\n csrrc x2 x3 TIME";

        let expected = vec![
            Some(CToken::RegRegCsr("CSRRW".to_string(), ast::Reg::X0, ast::Reg::X1, ast::Csr::CYCLE)),
            Some(CToken::RegRegCsr("CSRRS".to_string(), ast::Reg::X1, ast::Reg::X2, ast::Csr::CYCLEH)),
            Some(CToken::RegRegCsr("CSRRC".to_string(), ast::Reg::X2, ast::Reg::X3, ast::Csr::TIME)),
            None,
        ];

        assert_eq(input, expected);
    }

    #[test]
    fn test_RegRegShamt_inst() {
        let input = "slli x0 x1 11\n srli x1 x2 22\n srai x2 x3 33";

        let expected = vec![
            Some(CToken::RegRegShamt("SLLI".to_string(), ast::Reg::X0, ast::Reg::X1, 11)),
            Some(CToken::RegRegShamt("SRLI".to_string(), ast::Reg::X1, ast::Reg::X2, 22)),
            Some(CToken::RegRegShamt("SRAI".to_string(), ast::Reg::X2, ast::Reg::X3, 33)),
            None,
        ];

        assert_eq(input, expected);
    }

    #[test]
    fn test_RegRegIL_inst() {
        let input = "jalr x0 x1 11\n jalr x1 x2 2f\n jalr x2 x3 asdf";

        let expected = vec![
            Some(CToken::RegRegIL(
                "JALR".to_string(),
                ast::Reg::X0,
                ast::Reg::X1,
                CImmLabel::Imm(11)
            )),
            Some(CToken::RegRegIL(
                "JALR".to_string(),
                ast::Reg::X1,
                ast::Reg::X2,
                CImmLabel::Label("2f".to_string(), parser::LabelType::Local)
            )),
            Some(CToken::RegRegIL(
                "JALR".to_string(),
                ast::Reg::X2,
                ast::Reg::X3,
                CImmLabel::Label("asdf".to_string(), parser::LabelType::Global)
            )),
            None,
        ];

        assert_eq(input, expected);
    }

    #[test]
    fn test_RegRegImm_inst() {
        let input = "addi x0 x1 11";

        let expected = vec![
            Some(CToken::RegRegImm("ADDI".to_string(), ast::Reg::X0, ast::Reg::X1, 11)),
            None,
        ];

        assert_eq(input, expected);
    }

    #[test]
    fn test_RegRegImmStore_inst() {
        let input = "sw x0 x1 11";

        let expected = vec![
            Some(CToken::RegRegImmStore("SW".to_string(), ast::Reg::X0, ast::Reg::X1, 11)),
            None,
        ];

        assert_eq(input, expected);
    }

    #[test]
    fn test_RegRegILBranch_inst() {
        let input = "bne x0 x1 11\n bne x1 x2 2f\n bne x2 x3 asdf";

        let expected = vec![
            Some(CToken::RegRegILBranch(
                "BNE".to_string(),
                ast::Reg::X0,
                ast::Reg::X1,
                CImmLabel::Imm(11)
            )),
            Some(CToken::RegRegILBranch(
                "BNE".to_string(),
                ast::Reg::X1,
                ast::Reg::X2,
                CImmLabel::Label("2f".to_string(), parser::LabelType::Local)
            )),
            Some(CToken::RegRegILBranch(
                "BNE".to_string(),
                ast::Reg::X2,
                ast::Reg::X3,
                CImmLabel::Label("asdf".to_string(), parser::LabelType::Global)
            )),
            None,
        ];

        assert_eq(input, expected);
    }

    #[test]
    fn test_RegIL_inst() {
        let input = "lui x0 11\n lui x1 2f\n lui x2 asdf";

        let expected = vec![
            Some(CToken::RegIL(
                "LUI".to_string(),
                ast::Reg::X0,
                CImmLabel::Imm(11)
            )),
            Some(CToken::RegIL(
                "LUI".to_string(),
                ast::Reg::X1,
                CImmLabel::Label("2f".to_string(), parser::LabelType::Local)
            )),
            Some(CToken::RegIL(
                "LUI".to_string(),
                ast::Reg::X2,
                CImmLabel::Label("asdf".to_string(), parser::LabelType::Global)
            )),
            None,
        ];

        assert_eq(input, expected);
    }

    #[test]
    fn test_RegILShuffle_inst() {
        let input = "jal x0 11\n jal x1 2f\n jal x2 asdf";

        let expected = vec![
            Some(CToken::RegILShuffle(
                "JAL".to_string(),
                ast::Reg::X0,
                CImmLabel::Imm(11)
            )),
            Some(CToken::RegILShuffle(
                "JAL".to_string(),
                ast::Reg::X1,
                CImmLabel::Label("2f".to_string(), parser::LabelType::Local)
            )),
            Some(CToken::RegILShuffle(
                "JAL".to_string(),
                ast::Reg::X2,
                CImmLabel::Label("asdf".to_string(), parser::LabelType::Global)
            )),
            None,
        ];

        assert_eq(input, expected);
    }
}