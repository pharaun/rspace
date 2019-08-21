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
    AddrRef(String, ast::AddrRefType),
    Imm(u32),
}

// TODO: implement support for MemRef here onward
#[derive(Debug, PartialEq)]
pub enum CToken {
    Label(String, ast::LabelType),

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

                        // If rem is 0 skip adding padding
                        if self.buffer_idx % 4 != 0 {
                            // 4x u8 in u32
                            let padding = 4 - (self.buffer_idx % 4);
                            self.buffer.push_back(CToken::Padding(padding));
                        }
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
                // There's no more token, check if we still have labels left?
                if self.label_buffer.is_empty() {
                    None
                } else {
                    self.buffer.append(&mut self.label_buffer);
                    self.next_token()
                }
            }
        }
    }
}

// NOTE: jalr rd x0 imm -> one inst subroutine call to any code in the bottom
//       or top 2KiB of memory
//
// TODO: pseudo instructions (here or at a earlier stage, probably earlier)
// MV rd rs1 = addi rd rs1 0
// NOT rd rs1 = xori rd rs1 -1
// SNEZ rd rs2 = sltu rd x0 rs2
// NOP = addi x0 x0 0
// J imm = jal x0 imm
// BGT rs1 rs2 imm = blt rs2 rs1 imm
// BGTU rs1 rs2 imm = bltu rs2 rs1 imm
// BLE rs1 rs2 imm = bge rs2 rs1 imm
// BLEU rs1 rs2 imm = bgeu rs2 rs1 imm
// CSRR rd csr = csrrs rd x0 csr
// CSRW rs1 csr = csrrw x0 rs1 csr
// CSRWI uimm csr = csrrwi x0 uimm csr
// CSRS rs1 csr = csrrs x0 rs1 csr
// CSRC rs1 csr = csrrc x0 rs1 csr
// CSRSI uimm csr = csrrsi x0 uimm csr
// CSRCI uimm csr = csrrci x0 uimm csr
//
// page 139 - table 26.2 - pseudo instructions (several more)
//
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
                        "FENCE" | "ECALL" | "EBREAK" => {
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


fn shift_and_mask(n: &u32, shift: usize) -> u8 {
    ((n >> shift) & 0x00_00_00_FF) as u8
}

// TODO: not sure if the ordering of the data is correct for in memory
fn process_data(dt: ast::DataType, n: Vec<u32>) -> VecDeque<CToken> {
    let mut ret = VecDeque::new();

    match dt {
        ast::DataType::Byte => {
            for num in &n {
                ret.push_back(CToken::ByteData(shift_and_mask(num, 0)));
            }
        },
        ast::DataType::Half => {
            for num in &n {
                ret.push_back(CToken::ByteData(shift_and_mask(num, 0)));
                ret.push_back(CToken::ByteData(shift_and_mask(num, 8)));
            }
        },
        ast::DataType::Word => {
            for num in &n {
                ret.push_back(CToken::ByteData(shift_and_mask(num, 0)));
                ret.push_back(CToken::ByteData(shift_and_mask(num, 8)));
                ret.push_back(CToken::ByteData(shift_and_mask(num, 16)));
                ret.push_back(CToken::ByteData(shift_and_mask(num, 24)));
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
    #![allow(non_snake_case)]

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
            Some(CToken::Label("la".to_string(), ast::LabelType::Global)),
            Some(CToken::Label("2".to_string(), ast::LabelType::Local)),
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
        let input = "csrrwi x0 33 MSTATUS\n csrrsi x1 11 MIE\n csrrci x2 22 MEPC";

        let expected = vec![
            Some(CToken::RegImmCsr("CSRRWI".to_string(), ast::Reg::X0, 33, ast::Csr::MSTATUS)),
            Some(CToken::RegImmCsr("CSRRSI".to_string(), ast::Reg::X1, 11, ast::Csr::MIE)),
            Some(CToken::RegImmCsr("CSRRCI".to_string(), ast::Reg::X2, 22, ast::Csr::MEPC)),
            None,
        ];

        assert_eq(input, expected);
    }

    #[test]
    fn test_RegRegCsr_inst() {
        let input = "csrrw x0 x1 MSTATUS\n csrrs x1 x2 MIE\n csrrc x2 x3 MEPC";

        let expected = vec![
            Some(CToken::RegRegCsr("CSRRW".to_string(), ast::Reg::X0, ast::Reg::X1, ast::Csr::MSTATUS)),
            Some(CToken::RegRegCsr("CSRRS".to_string(), ast::Reg::X1, ast::Reg::X2, ast::Csr::MIE)),
            Some(CToken::RegRegCsr("CSRRC".to_string(), ast::Reg::X2, ast::Reg::X3, ast::Csr::MEPC)),
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
                CImmRef::AddrRef("2".to_string(), ast::AddrRefType::LocalForward)
            )),
            Some(CToken::RegRegIL(
                "JALR".to_string(),
                ast::Reg::X2,
                ast::Reg::X3,
                CImmRef::AddrRef("2".to_string(), ast::AddrRefType::LocalBackward)
            )),
            Some(CToken::RegRegIL(
                "JALR".to_string(),
                ast::Reg::X3,
                ast::Reg::X4,
                CImmRef::AddrRef("asdf".to_string(), ast::AddrRefType::Global)
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
                CImmRef::AddrRef("2".to_string(), ast::AddrRefType::LocalForward)
            )),
            Some(CToken::RegRegImm(
                "ADDI".to_string(),
                ast::Reg::X2,
                ast::Reg::X3,
                CImmRef::AddrRef("2".to_string(), ast::AddrRefType::LocalBackward)
            )),
            Some(CToken::RegRegImm(
                "ADDI".to_string(),
                ast::Reg::X3,
                ast::Reg::X4,
                CImmRef::AddrRef("asdf".to_string(), ast::AddrRefType::Global)
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
                CImmRef::AddrRef("2".to_string(), ast::AddrRefType::LocalForward)
            )),
            Some(CToken::RegRegILBranch(
                "BNE".to_string(),
                ast::Reg::X2,
                ast::Reg::X3,
                CImmRef::AddrRef("2".to_string(), ast::AddrRefType::LocalBackward)
            )),
            Some(CToken::RegRegILBranch(
                "BNE".to_string(),
                ast::Reg::X3,
                ast::Reg::X4,
                CImmRef::AddrRef("asdf".to_string(), ast::AddrRefType::Global)
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
                CImmRef::AddrRef("2".to_string(), ast::AddrRefType::LocalForward)
            )),
            Some(CToken::RegIL(
                "LUI".to_string(),
                ast::Reg::X2,
                CImmRef::AddrRef("2".to_string(), ast::AddrRefType::LocalBackward)
            )),
            Some(CToken::RegIL(
                "LUI".to_string(),
                ast::Reg::X3,
                CImmRef::AddrRef("asdf".to_string(), ast::AddrRefType::Global)
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
                CImmRef::AddrRef("2".to_string(), ast::AddrRefType::LocalForward)
            )),
            Some(CToken::RegILShuffle(
                "JAL".to_string(),
                ast::Reg::X2,
                CImmRef::AddrRef("2".to_string(), ast::AddrRefType::LocalBackward)
            )),
            Some(CToken::RegILShuffle(
                "JAL".to_string(),
                ast::Reg::X3,
                CImmRef::AddrRef("asdf".to_string(), ast::AddrRefType::Global)
            )),
            None,
        ];

        assert_eq(input, expected);
    }

    #[test]
    fn test_basic_padding() {
        let input = ".BYTE 0x10\n add x0 x1 x2\n .HALF 0x1020\n add x0 x1 x2\n .WORD 0x10203040\n add x0 x1 x2";

        let expected = vec![
            Some(CToken::ByteData(0x10)),
            Some(CToken::Padding(3)),
            Some(CToken::RegRegReg("ADD".to_string(), ast::Reg::X0, ast::Reg::X1, ast::Reg::X2)),
            Some(CToken::ByteData(0x20)),
            Some(CToken::ByteData(0x10)),
            Some(CToken::Padding(2)),
            Some(CToken::RegRegReg("ADD".to_string(), ast::Reg::X0, ast::Reg::X1, ast::Reg::X2)),
            Some(CToken::ByteData(0x40)),
            Some(CToken::ByteData(0x30)),
            Some(CToken::ByteData(0x20)),
            Some(CToken::ByteData(0x10)),
            Some(CToken::RegRegReg("ADD".to_string(), ast::Reg::X0, ast::Reg::X1, ast::Reg::X2)),
            None,
        ];

        assert_eq(input, expected);
    }

    #[test]
    fn test_label_dump() {
        let input = ".BYTE 0x10\n la: add x0 x1 x2\n .BYTE 0x10\n la: .BYTE 0x10\n add x0 x1 x2";

        let expected = vec![
            Some(CToken::ByteData(0x10)),
            Some(CToken::Padding(3)),
            Some(CToken::Label("la".to_string(), ast::LabelType::Global)),
            Some(CToken::RegRegReg("ADD".to_string(), ast::Reg::X0, ast::Reg::X1, ast::Reg::X2)),
            Some(CToken::ByteData(0x10)),
            Some(CToken::Label("la".to_string(), ast::LabelType::Global)),
            Some(CToken::ByteData(0x10)),
            Some(CToken::Padding(2)),
            Some(CToken::RegRegReg("ADD".to_string(), ast::Reg::X0, ast::Reg::X1, ast::Reg::X2)),
            None,
        ];

        assert_eq(input, expected);
    }

    #[test]
    fn test_mixed_data_padding() {
        let input = ".BYTE 0x10\n .WORD 0x10203040\n add x0 x1 x2";

        let expected = vec![
            Some(CToken::ByteData(0x10)),
            Some(CToken::ByteData(0x40)),
            Some(CToken::ByteData(0x30)),
            Some(CToken::ByteData(0x20)),
            Some(CToken::ByteData(0x10)),
            Some(CToken::Padding(3)),
            Some(CToken::RegRegReg("ADD".to_string(), ast::Reg::X0, ast::Reg::X1, ast::Reg::X2)),
            None,
        ];

        assert_eq(input, expected);
    }

    #[test]
    fn test_multibyte() {
        let input = ".BYTE 0x10 0x20 0x30 0x40";

        let expected = vec![
            Some(CToken::ByteData(0x10)),
            Some(CToken::ByteData(0x20)),
            Some(CToken::ByteData(0x30)),
            Some(CToken::ByteData(0x40)),
            None,
        ];

        assert_eq(input, expected);
    }
}
