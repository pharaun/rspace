use std::collections::VecDeque;

use asm::parser;

use vm::opcode;

// Use the reg and csr func here for now
use asm::ast;

// TODO:
// 1. newtype im
// 3. this is a final step before assembler (it cleans up the stream for the assembler)
//      - prior stage handles macro expansion
//      - assembler handles label lookup

// TODO: Find a way to better clarify the type of AddrRef
// 1. relative to the instruction (ie jump back 10 whatever)
// 2. relative to pc (ie auipc)
// 3. absolute addressing
// 4. memref (content of what was at that address at assembly time) (Restricted to only data label?)
#[derive(Debug, PartialEq)]
pub enum CImmRef {
    // TODO: MemRef(String),
    AddrRef(String, parser::AddrRefType),
    Imm(u32),
}

// TODO: implement support for MemRef here onward
#[derive(Debug, PartialEq)]
pub enum CToken {
    Label(String, parser::LabelType),

    // Padding (number of u8 padding bits needed to align to nearest u32 boundary)
    Padding(usize),

    // Data bits (or raw assembly)
    ByteData(u8),

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
    RegRegImm(String, ast::Reg, ast::Reg, CImmRef),

    // Inst, rd, rs1, imm
    // 3 length
    RegRegImmStore(String, ast::Reg, ast::Reg, u32),

    // Inst, rd, rs1, (imm/label)?
    // 3 length
    RegRegILBranch(String, ast::Reg, ast::Reg, CImmRef),

    // Inst, rd, rs, (imm/label)
    // 3 length
    RegRegIL(String, ast::Reg, ast::Reg, CImmRef),

    // Inst, rd, (imm/label)
    // 2 length
    RegIL(String, ast::Reg, CImmRef),

    // Inst, rd, (imm/label)
    // 2 length
    RegILShuffle(String, ast::Reg, CImmRef),
}


// Cleaner
//
// Buffer is needed to allow us to handle padding to ensure that
// instructions are always emitted on a u32 boundary.
//
// When we hit a label or data token, we accumulate them till we
// see an instruction, then we emit a padding and record it into
// the buffer.
//
// Then resume iteration by reading out of the buffer till empty.
pub struct Cleaner<'a> {
    input_iter: parser::Parser<'a>,
    buffer: VecDeque<CToken>,
    buffer_idx: usize,
    label_buffer: VecDeque<CToken>,
}

impl<'a> Cleaner<'a> {
    pub fn new(input: parser::Parser<'a>) -> Cleaner<'a> {
        Cleaner {
            input_iter: input,
            buffer: VecDeque::new(),
            buffer_idx: 0,
            label_buffer: VecDeque::new(),
        }
    }

    fn read_token(&mut self) -> Option<parser::PToken> {
        self.input_iter.next()
    }

    pub fn next_token(&mut self) -> Option<CToken> {
        // 1. remove first element from buffer (pop_front)
        if let Some(t) = self.buffer.pop_front() {
            // 2. if Some(x), return the some x
            Some(t)
        } else {
            // 3. if read_token is inst, pop it and process, and return that
            // 4. if read_token is (label/num) -> accumulate label/num
            if let Some(t) = self.read_token() {
                match t {
                    parser::PToken::Label(s, lt) => {
                        // If label, we accumulate it into the label buffer
                        self.label_buffer.push_back(CToken::Label(s, lt));
                    },
                    parser::PToken::Data(dt, n) => {
                        // We check if there's anything in the label buffer,
                        // if so, we push it now to master buffer, then process data
                        // Append moves data out of label_buffer vs extend (which copies)
                        self.buffer.append(&mut self.label_buffer);

                        // Actually process the data
                        let byte_data = process_data(dt, n);
                        self.buffer_idx += byte_data.len();
                        self.buffer.extend(byte_data);
                    },
                    // 5. in accumulate label/num, accumulate token till hit a inst
                    // 6. if hit inst, record padding, and then store the whole queue in buffer
                    parser::PToken::Inst(inst, mut args) => {
                        // Push a padding token to the buffer, and then append the
                        // label buffer to the master buffer then exit to let the
                        // instruction be handled after we drain the buffer
                        self.buffer.push_back(CToken::Padding(self.buffer_idx % 4)); // 4x u8 in u32
                        self.buffer.append(&mut self.label_buffer);

                        // Reset buffer_idx
                        self.buffer_idx = 0;

                        // Store the inst onto the buffer
                        if let Some(pinst) = process_inst(inst, args) {
                            self.buffer.push_back(pinst);
                        }
                    },
                }

                // 7. Goto 1
                self.next_token()
            } else {
                None
            }
        }
    }
}

fn process_inst(inst: String, mut args: Vec<parser::Arg>) -> Option<CToken> {
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
                            print!("Skipping unsupported instruction: {}", inst);
                            None
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
                                extract_imm_label(args.remove(0))
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
}

fn process_data(dt: parser::DataType, n: Vec<u32>) -> VecDeque<CToken> {
    let mut ret = VecDeque::new();

    match dt {
        parser::DataType::Byte => {
            for num in &n {
                ret.push_back(CToken::ByteData((num & 0x00_00_00_FF) as u8));
            }
        },
        parser::DataType::Half => {
            for num in &n {
                ret.push_back(CToken::ByteData((num & 0x00_00_00_FF) as u8));
                ret.push_back(CToken::ByteData((num & 0x00_00_FF_00) as u8));
            }
        },
        parser::DataType::Word => {
            for num in &n {
                ret.push_back(CToken::ByteData((num & 0x00_00_00_FF) as u8));
                ret.push_back(CToken::ByteData((num & 0x00_00_FF_00) as u8));
                ret.push_back(CToken::ByteData((num & 0x00_FF_00_00) as u8));
                ret.push_back(CToken::ByteData((num & 0xFF_00_00_00) as u8));
            }
        },
    }
    ret
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

fn extract_imm_label(arg: parser::Arg) -> CImmRef {
    match arg {
        parser::Arg::Num(n)           => CImmRef::Imm(n),
        parser::Arg::AddrRef(l, lt)   => CImmRef::AddrRef(l, lt),
        _ => panic!("Expected a ImmLabel, got {:?}", arg),
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
    use asm::lexer;
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
        let input = "jalr x0 x1 11\n jalr x1 x2 2f\n jalr x2 x3 2b\njalr x3 x4 asdf";

        let expected = vec![
            Some(CToken::RegRegIL(
                "JALR".to_string(),
                ast::Reg::X0,
                ast::Reg::X1,
                CImmRef::Imm(11)
            )),
            Some(CToken::RegRegIL(
                "JALR".to_string(),
                ast::Reg::X1,
                ast::Reg::X2,
                CImmRef::AddrRef("2".to_string(), parser::AddrRefType::LocalForward)
            )),
            Some(CToken::RegRegIL(
                "JALR".to_string(),
                ast::Reg::X2,
                ast::Reg::X3,
                CImmRef::AddrRef("2".to_string(), parser::AddrRefType::LocalBackward)
            )),
            Some(CToken::RegRegIL(
                "JALR".to_string(),
                ast::Reg::X3,
                ast::Reg::X4,
                CImmRef::AddrRef("asdf".to_string(), parser::AddrRefType::Global)
            )),
            None,
        ];

        assert_eq(input, expected);
    }

    #[test]
    fn test_RegRegImm_inst() {
        let input = "addi x0 x1 11\n addi x1 x2 2f\n addi x2 x3 2b\n addi x3 x4 asdf";

        let expected = vec![
            Some(CToken::RegRegImm(
                "ADDI".to_string(),
                ast::Reg::X0,
                ast::Reg::X1,
                CImmRef::Imm(11)
            )),
            Some(CToken::RegRegImm(
                "ADDI".to_string(),
                ast::Reg::X1,
                ast::Reg::X2,
                CImmRef::AddrRef("2".to_string(), parser::AddrRefType::LocalForward)
            )),
            Some(CToken::RegRegImm(
                "ADDI".to_string(),
                ast::Reg::X2,
                ast::Reg::X3,
                CImmRef::AddrRef("2".to_string(), parser::AddrRefType::LocalBackward)
            )),
            Some(CToken::RegRegImm(
                "ADDI".to_string(),
                ast::Reg::X3,
                ast::Reg::X4,
                CImmRef::AddrRef("asdf".to_string(), parser::AddrRefType::Global)
            )),
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
        let input = "bne x0 x1 11\n bne x1 x2 2f\n bne x2 x3 2b\n bne x3 x4 asdf";

        let expected = vec![
            Some(CToken::RegRegILBranch(
                "BNE".to_string(),
                ast::Reg::X0,
                ast::Reg::X1,
                CImmRef::Imm(11)
            )),
            Some(CToken::RegRegILBranch(
                "BNE".to_string(),
                ast::Reg::X1,
                ast::Reg::X2,
                CImmRef::AddrRef("2".to_string(), parser::AddrRefType::LocalForward)
            )),
            Some(CToken::RegRegILBranch(
                "BNE".to_string(),
                ast::Reg::X2,
                ast::Reg::X3,
                CImmRef::AddrRef("2".to_string(), parser::AddrRefType::LocalBackward)
            )),
            Some(CToken::RegRegILBranch(
                "BNE".to_string(),
                ast::Reg::X3,
                ast::Reg::X4,
                CImmRef::AddrRef("asdf".to_string(), parser::AddrRefType::Global)
            )),
            None,
        ];

        assert_eq(input, expected);
    }

    #[test]
    fn test_RegIL_inst() {
        let input = "lui x0 11\n lui x1 2f\n lui x2 2b\n lui x3 asdf";

        let expected = vec![
            Some(CToken::RegIL(
                "LUI".to_string(),
                ast::Reg::X0,
                CImmRef::Imm(11)
            )),
            Some(CToken::RegIL(
                "LUI".to_string(),
                ast::Reg::X1,
                CImmRef::AddrRef("2".to_string(), parser::AddrRefType::LocalForward)
            )),
            Some(CToken::RegIL(
                "LUI".to_string(),
                ast::Reg::X2,
                CImmRef::AddrRef("2".to_string(), parser::AddrRefType::LocalBackward)
            )),
            Some(CToken::RegIL(
                "LUI".to_string(),
                ast::Reg::X3,
                CImmRef::AddrRef("asdf".to_string(), parser::AddrRefType::Global)
            )),
            None,
        ];

        assert_eq(input, expected);
    }

    #[test]
    fn test_RegILShuffle_inst() {
        let input = "jal x0 11\n jal x1 2f\n jal x2 2b\n jal x3 asdf";

        let expected = vec![
            Some(CToken::RegILShuffle(
                "JAL".to_string(),
                ast::Reg::X0,
                CImmRef::Imm(11)
            )),
            Some(CToken::RegILShuffle(
                "JAL".to_string(),
                ast::Reg::X1,
                CImmRef::AddrRef("2".to_string(), parser::AddrRefType::LocalForward)
            )),
            Some(CToken::RegILShuffle(
                "JAL".to_string(),
                ast::Reg::X2,
                CImmRef::AddrRef("2".to_string(), parser::AddrRefType::LocalBackward)
            )),
            Some(CToken::RegILShuffle(
                "JAL".to_string(),
                ast::Reg::X3,
                CImmRef::AddrRef("asdf".to_string(), parser::AddrRefType::Global)
            )),
            None,
        ];

        assert_eq(input, expected);
    }
}
