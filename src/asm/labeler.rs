use crate::asm::ast;
use crate::asm::cleaner;

// TODO: not sure if this is the stage where i want to handle data labels
// (ie copying the value in a label to a instruction) and/or emitting the raw
// u32 unit of data to be embedded into the assembly output

// This handles the symbol table lookup then emits the final ast
// At this stage everything should be fully specified
// (ie. ready to be encoded into bytecode)
#[derive(Debug, PartialEq)]
pub enum AToken {
    RegRegReg(String, ast::Reg, ast::Reg, ast::Reg),
    RegImmCsr(String, ast::Reg, u32, ast::Csr),
    RegRegCsr(String, ast::Reg, ast::Reg, ast::Csr),
    RegRegShamt(String, ast::Reg, ast::Reg, u32),
    RegRegImm(String, ast::Reg, ast::Reg, u32),
    RegRegImmStore(String, ast::Reg, ast::Reg, u32),
    RegRegILBranch(String, ast::Reg, ast::Reg, u32),
    RegRegIL(String, ast::Reg, ast::Reg, u32),
    RegIL(String, ast::Reg, u32),
    RegILShuffle(String, ast::Reg, u32),

    Data(u32), // Raw assembly or data bits
}

// u32 buffer for flushing out a Data(u32)
struct WordBuf {
    buffer: [u8; 4],
    buffer_idx: usize,
}

impl WordBuf {
    pub fn new() -> WordBuf {
        WordBuf {
            buffer: [0x0, 0x0, 0x0, 0x0],
            buffer_idx: 0,
        }
    }

    pub fn write_byte(&mut self, input: u8) -> Option<u32> {
        self.buffer[self.buffer_idx] = input;
        self.buffer_idx += 1;

        if self.buffer_idx == 4 {
            let mut ret = 0x00_00_00_00;
            ret |= (self.buffer[0] as u32) << 0;
            ret |= (self.buffer[1] as u32) << 8;
            ret |= (self.buffer[2] as u32) << 16;
            ret |= (self.buffer[3] as u32) << 24;

            self.buffer = [0x0, 0x0, 0x0, 0x0];
            self.buffer_idx = 0;

            Some(ret)
        } else {
            None
        }
    }

    pub fn is_empty(&self) -> bool {
        self.buffer_idx == 0
    }
}

// TODO: support labeled memory location, for now only branches + jumps (need more support)
// Don't know how to figure out if i should generate a relative or an absolute address, branches are always relative to the inst
// supporting those for now, the jalr/auipc/lui/jal i can't figure out yet
pub fn symbol_table_expansion<'a>(input: cleaner::Cleaner<'a>) -> Vec<AToken> {
    // First pass caches the output from cleaner
    let mut first_pass: Vec<cleaner::CToken> = Vec::new();

    // Symbol table
    let mut position: usize = 0; // Per byte
    let mut symbol_table: Vec<((String, ast::LabelType), usize)> = Vec::new();

    // First pass to scan for label and store their positions
    for token in input {
        match token {
            cleaner::CToken::Label(n, lt) => {
                symbol_table.push(((n, lt), position));
            },
            cleaner::CToken::Padding(p) => {
                first_pass.push(cleaner::CToken::Padding(p));
                position += p;
            },
            cleaner::CToken::ByteData(d) => {
                first_pass.push(cleaner::CToken::ByteData(d));
                position += 1;
            },
            // Instructions here on out
            i@_ => {
                first_pass.push(i);

                // Inc by 4 because each inst is a u32 word
                position += 4;
            }
        }
    }

    #[cfg(feature = "debug")]
    {
        // Debug label positioning
        println!("{:?}", position);
        println!("{:?}", symbol_table);
    }

    // Convert instructions into AToken via looking up and encoding the labels
    let mut second_pass: Vec<AToken> = Vec::new();
    let mut buf = WordBuf::new();

    // Reset position (for relative label use)
    position = 0;

    let first_pass_drain = first_pass.drain(..);
    for token in first_pass_drain {
        match token {
            cleaner::CToken::Padding(p) => {
                for _ in 0..p {
                    match buf.write_byte(0x0) {
                        None    => (),
                        Some(d) => second_pass.push(AToken::Data(d)),
                    }
                }

                position += p;
            },
            cleaner::CToken::ByteData(d) => {
                match buf.write_byte(d) {
                    None     => (),
                    Some(dd) => second_pass.push(AToken::Data(dd)),
                }

                position += 1;
            },

            // Instructions here on out
            _ => {
                if !buf.is_empty() {
                    panic!("Buffer isn't empty, it should be before any instruction");
                }
                second_pass.push(encode_label(token, &symbol_table, position));

                // Inc by 4 because each inst is a u32 word
                position += 4;
            }
        }
    }

    second_pass
}

// TODO: find a good way to handle %hi_lo() since right now i have not ran into it because all of
// my address are small enough to not trip into lui and thus trip the sign-extend of addi, so ...
fn encode_label(token: cleaner::CToken, symbol: &Vec<((String, ast::LabelType), usize)>, inst_pos: usize) -> AToken {
    match token {
        cleaner::CToken::Label(_, _)
              => panic!("Should have been filtered out in first pass"),
        cleaner::CToken::Padding(_)
              => panic!("Should have been filtered out earlier in second pass"),
        cleaner::CToken::ByteData(_)
              => panic!("Should have been filtered out earlier in second pass"),

        // Copy these token over
        cleaner::CToken::RegRegReg(s, rd, rs1, rs2)
              => AToken::RegRegReg(s, rd, rs1, rs2),
        cleaner::CToken::RegImmCsr(s, rd, imm, csr)
              => AToken::RegImmCsr(s, rd, imm, csr),
        cleaner::CToken::RegRegShamt(s, rd, rs1, imm)
              => AToken::RegRegShamt(s, rd, rs1, imm),
        cleaner::CToken::RegRegCsr(s, rd, rs1, csr)
              => AToken::RegRegCsr(s, rd, rs1, csr),
        cleaner::CToken::RegRegImmStore(s, rs1, rs2, imm)
              => AToken::RegRegImmStore(s, rs1, rs2, imm),

        // Handle the imm variants of these tokens
        cleaner::CToken::RegRegImm(s, rd, rs1, cleaner::CImmRef::Imm(imm))
              => AToken::RegRegImm(s, rd, rs1, imm),
        cleaner::CToken::RegRegIL(s, rd, rs1, cleaner::CImmRef::Imm(imm))
              => AToken::RegRegIL(s, rd, rs1, imm),
        cleaner::CToken::RegRegILBranch(s, rs1, rs2, cleaner::CImmRef::Imm(imm))
              => AToken::RegRegILBranch(s, rs1, rs2, imm),
        cleaner::CToken::RegILShuffle(s, rd, cleaner::CImmRef::Imm(imm))
              => AToken::RegILShuffle(s, rd, imm),

        // Since mips legacy shift the imm over 12 bit
        // since it'll only take the lower 20 not the upper
        // 20 inst, and use absolute addressing
        cleaner::CToken::RegIL(s, rd, cleaner::CImmRef::Imm(imm))
        // TODO: apparently tests expects no shift here, yet my code does, figure this out
        //      => AToken::RegIL(s, rd, imm >> 12),
              => AToken::RegIL(s, rd, imm),

        // Handle the label variant of these tokens
        cleaner::CToken::RegRegImm(s, rd, rs1, cleaner::CImmRef::AddrRef(l, lt)) => {
            // TODO: handle relative vs global for now hardcode global position?
            let pos = find_label(&l, lt, symbol, inst_pos);

            // This is for lui (global offset) and auipc (relative offset)
            let imm = match &s[..] {
                "JALR" => encode_relative_offset(inst_pos, pos),
                _      => encode_global_offset(pos),
            };

            AToken::RegRegImm(s, rd, rs1, imm)
        },
        cleaner::CToken::RegRegIL(_s, _rd, _rs1, cleaner::CImmRef::AddrRef(l, lt)) => {
            let pos = find_label(&l, lt, symbol, inst_pos);
            let _imm = encode_relative_offset(inst_pos, pos);

            // TODO: deal with labels
            panic!("Deal with labels");
            //AToken::RegRegIL(s, rd, rs1, 999999)
        },
        cleaner::CToken::RegRegILBranch(s, rs1, rs2, cleaner::CImmRef::AddrRef(l, lt)) => {
            let pos = find_label(&l, lt, symbol, inst_pos);
            let imm = encode_relative_offset(inst_pos, pos);

            AToken::RegRegILBranch(s, rs1, rs2, imm)
        },
        cleaner::CToken::RegIL(s, rd, cleaner::CImmRef::AddrRef(l, lt)) => {
            let pos = find_label(&l, lt, symbol, inst_pos);

            // This is for lui (global offset) and auipc (relative offset)
            let imm = match &s[..] {
                "AUIPC" => encode_relative_offset(inst_pos, pos),
                "LUI"   => encode_global_offset(pos),
                _       => panic!("Unknown inst {}", s),
            };

            // Since mips legacy shift the imm over 12 bit
            // since it'll only take the lower 20 not the upper
            // 20 inst, and use absolute addressing

            AToken::RegIL(s, rd, imm >> 12)
        },
        cleaner::CToken::RegILShuffle(s, rd, cleaner::CImmRef::AddrRef(l, lt)) => {
            let pos = find_label(&l, lt, symbol, inst_pos);
            let imm = encode_relative_offset(inst_pos, pos);

            AToken::RegILShuffle(s, rd, imm)
        },
    }
}

fn find_label(name: &String, label_type: ast::AddrRefType, symbol: &Vec<((String, ast::LabelType), usize)>, inst_pos: usize) -> usize {
    // Decode the type of Label it is (is it a word or a numberic label)
    // If word, proceed, but if numberic,
    //      parse the letter after (b or f) for backward or forward numberic ref
    //      find inst pos then proceed backward or forward in search for the numberic ref
    // if word
    //      Assume no duplicate word label (should not happen, integrity check the symbol table)
    //      linear scan till you find the matching word label
    match label_type {
        ast::AddrRefType::Global => {
            // Word label
            for val in symbol.iter() {
                match val {
                    &((ref sl, ast::LabelType::Global), spos) => {
                        if sl == name {
                            return spos
                        }
                    },
                    _ => (),
                }
            }
            panic!("Did not find {} global label in the table", name)
        },
        ast::AddrRefType::LocalForward => {
            // Local Forward, aka numberical
            for val in symbol.iter() {
                match val {
                    &((ref sl, ast::LabelType::Local), spos) => {
                        if (sl == name) & (spos >= inst_pos) {
                            return spos
                        }
                    },
                    _ => (),
                }
            }
            panic!("Did not find {} local forward label in the table", name)
        },
        ast::AddrRefType::LocalBackward => {
            // Local Backward, aka numberical
            for val in symbol.iter().rev() {
                match val {
                    &((ref sl, ast::LabelType::Local), spos) => {
                        if (sl == name) & (spos <= inst_pos) {
                            return spos
                        }
                    },
                    _ => (),
                }
            }
            panic!("Did not find {} local backward label in the table", name)
        },
    }
}

fn encode_relative_offset(inst_pos: usize, label_pos: usize) -> u32 {
    // TODO: hella truncation
    // This assumes u32 sized instruction words, the pos are integer in block of u32
    // If label pos is earlier than inst_pos its a negative offset
    // and viceverse
    let inst_addr:  i64 = inst_pos as i64;
    let label_addr: i64 = label_pos as i64;
    let offset = label_addr - inst_addr;
    offset as u32
}

fn encode_global_offset(label_pos: usize) -> u32 {
    label_pos as u32
}

#[test]
fn test_wordbuf_one_byte() {
    let mut buf = WordBuf::new();
    let res = buf.write_byte(0x1);

    assert_eq!(buf.buffer[0], 0x1);
    assert_eq!(buf.buffer[1], 0x0);
    assert_eq!(buf.buffer[2], 0x0);
    assert_eq!(buf.buffer[3], 0x0);

    assert_eq!(buf.buffer_idx, 1);

    assert_eq!(buf.is_empty(), false);
    assert_eq!(res, None);
}

#[test]
fn test_wordbuf_four_byte() {
    let mut buf = WordBuf::new();

    let res = buf.write_byte(0x1);
    assert_eq!(res, None);

    let res = buf.write_byte(0x2);
    assert_eq!(res, None);

    let res = buf.write_byte(0x3);
    assert_eq!(res, None);

    let res = buf.write_byte(0x4);
    assert_eq!(res, Some(0x04030201));

    assert_eq!(buf.buffer[0], 0x0);
    assert_eq!(buf.buffer[1], 0x0);
    assert_eq!(buf.buffer[2], 0x0);
    assert_eq!(buf.buffer[3], 0x0);

    assert_eq!(buf.buffer_idx, 0);

    assert_eq!(buf.is_empty(), true);
}

#[test]
fn test_wordbuf_five_byte() {
    let mut buf = WordBuf::new();

    let res = buf.write_byte(0x1);
    assert_eq!(res, None);

    let res = buf.write_byte(0x2);
    assert_eq!(res, None);

    let res = buf.write_byte(0x3);
    assert_eq!(res, None);

    let res = buf.write_byte(0x4);
    assert_eq!(res, Some(0x04030201));

    let res = buf.write_byte(0x5);
    assert_eq!(res, None);
    assert_eq!(buf.buffer_idx, 1);
    assert_eq!(buf.is_empty(), false);
}
