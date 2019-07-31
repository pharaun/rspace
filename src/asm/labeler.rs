use asm::cleaner;
use asm::ast;
use asm::parser;

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

// TODO: support labeled memory location, for now only branches + jumps (need more support)
// Don't know how to figure out if i should generate a relative or an absolute address, branches are always relative to the inst
// supporting those for now, the jalr/auipc/lui/jal i can't figure out yet
pub fn symbol_table_expansion<'a>(input: cleaner::Cleaner<'a>) -> Vec<AToken> {
    // First pass caches the output from cleaner
    let mut first_pass: Vec<cleaner::CToken> = Vec::new();

    // Symbol table
    let mut position: usize = 0; // Per u32 word
    let mut symbol_table: Vec<((String, parser::LabelType), usize)> = Vec::new();

    // First pass to scan for label and store their positions
    for token in input {
        match token {
            cleaner::CToken::Label(n, lt) => {
                symbol_table.push(((n, lt), position));
            },
            i@_ => {
                first_pass.push(i);

                // Inc inst pointer
                position += 1;
            }
        }
    }

    // Debug label positioning
    println!("{:?}", position);
    println!("{:?}", symbol_table);

    // Convert instructions into AToken via looking up and encoding the labels
    let mut second_pass: Vec<AToken> = Vec::new();

    // Reset position (for relative label use)
    position = 0;

    // Since we're popping, reverse the vec so that we can get instructions in order
    first_pass.reverse();

    // TODO: turn this into a iterator instead?
    while let Some(token) = first_pass.pop() {
        second_pass.push(encode_label(token, &symbol_table, position));
        position += 1;
    }

    second_pass
}

fn encode_label(token: cleaner::CToken, symbol: &Vec<((String, parser::LabelType), usize)>, inst_pos: usize) -> AToken {
    match token {
        cleaner::CToken::Label(_, _)
              => panic!("Should have been filtered out in first pass"),

        // Copy the data bits over
        cleaner::CToken::Data(n)
              => AToken::Data(n),

        // Copy these token over
        cleaner::CToken::RegRegReg(s, rd, rs1, rs2)
              => AToken::RegRegReg(s, rd, rs1, rs2),
        cleaner::CToken::RegImmCsr(s, rd, imm, csr)
              => AToken::RegImmCsr(s, rd, imm, csr),
        cleaner::CToken::RegRegShamt(s, rd, rs1, imm)
              => AToken::RegRegShamt(s, rd, rs1, imm),
        cleaner::CToken::RegRegCsr(s, rd, rs1, csr)
              => AToken::RegRegCsr(s, rd, rs1, csr),
        cleaner::CToken::RegRegImm(s, rd, rs1, imm)
              => AToken::RegRegImm(s, rd, rs1, imm),
        cleaner::CToken::RegRegImmStore(s, rs1, rs2, imm)
              => AToken::RegRegImmStore(s, rs1, rs2, imm),

        // Handle the imm variants of these tokens
        cleaner::CToken::RegRegIL(s, rd, rs1, cleaner::CImmRef::Imm(imm))
              => AToken::RegRegIL(s, rd, rs1, imm),
        cleaner::CToken::RegRegILBranch(s, rs1, rs2, cleaner::CImmRef::Imm(imm))
              => AToken::RegRegILBranch(s, rs1, rs2, imm),
        cleaner::CToken::RegIL(s, rd, cleaner::CImmRef::Imm(imm))
              => AToken::RegIL(s, rd, imm),
        cleaner::CToken::RegILShuffle(s, rd, cleaner::CImmRef::Imm(imm))
              => AToken::RegILShuffle(s, rd, imm),

        // Handle the label variant of these tokens
        cleaner::CToken::RegRegIL(s, rd, rs1, cleaner::CImmRef::AddrRef(l, lt)) => {
            let pos = find_label(&l, lt, symbol, inst_pos);
            let imm = encode_relative_offset(inst_pos, pos);

            // TODO: deal with labels
            AToken::RegRegIL(s, rd, rs1, 999999)
        },
        cleaner::CToken::RegRegILBranch(s, rs1, rs2, cleaner::CImmRef::AddrRef(l, lt)) => {
            let pos = find_label(&l, lt, symbol, inst_pos);
            let imm = encode_relative_offset(inst_pos, pos);

            AToken::RegRegILBranch(s, rs1, rs2, imm)
        },
        cleaner::CToken::RegIL(s, rd, cleaner::CImmRef::AddrRef(l, lt)) => {
            let pos = find_label(&l, lt, symbol, inst_pos);
            let imm = encode_relative_offset(inst_pos, pos);

            AToken::RegIL(s, rd, imm)
        },
        cleaner::CToken::RegILShuffle(s, rd, cleaner::CImmRef::AddrRef(l, lt)) => {
            let pos = find_label(&l, lt, symbol, inst_pos);
            let imm = encode_relative_offset(inst_pos, pos);

            AToken::RegILShuffle(s, rd, imm)
        },
    }
}

fn find_label(name: &String, label_type: parser::AddrRefType, symbol: &Vec<((String, parser::LabelType), usize)>, inst_pos: usize) -> usize {
    // Decode the type of Label it is (is it a word or a numberic label)
    // If word, proceed, but if numberic,
    //      parse the letter after (b or f) for backward or forward numberic ref
    //      find inst pos then proceed backward or forward in search for the numberic ref
    // if word
    //      Assume no duplicate word label (should not happen, integrity check the symbol table)
    //      linear scan till you find the matching word label
    match label_type {
        parser::AddrRefType::Global => {
            // Word label
            for val in symbol.iter() {
                match val {
                    &((ref sl, parser::LabelType::Global), spos) => {
                        if sl == name {
                            return spos
                        }
                    },
                    _ => (),
                }
            }
            panic!("Did not find {} global label in the table", name)
        },
        parser::AddrRefType::LocalForward => {
            // Local Forward, aka numberical
            for val in symbol.iter() {
                match val {
                    &((ref sl, parser::LabelType::Local), spos) => {
                        if (sl == name) & (spos >= inst_pos) {
                            return spos
                        }
                    },
                    _ => (),
                }
            }
            panic!("Did not find {} local forward label in the table", name)
        },
        parser::AddrRefType::LocalBackward => {
            // Local Backward, aka numberical
            for val in symbol.iter().rev() {
                match val {
                    &((ref sl, parser::LabelType::Local), spos) => {
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
    // The address is in byte (hence u32 blocks)
    let inst_addr: i64 = (inst_pos as i64) * 4;
    let label_addr: i64 = (label_pos as i64) * 4;
    let offset = label_addr - inst_addr;
    offset as u32
}
