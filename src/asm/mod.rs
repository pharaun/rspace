use vm::opcode;

use twiddle::Twiddle;

pub mod ast;
pub mod lexer;
pub mod parser;
pub mod cleaner;
pub mod labeler;


// TODO: Code quality improvement
//
// 1. First pass: Clean up the raw parser output, and uniformize into a nice AST with error tracking) (ie file location information
// 2. Second pass: Expand the macros and other relevant bits a needed
// 3. Third pass: Collect up the label and symbols
// 4. Fourth pass: rectify the inst into u32 + attribute memory location for labels
// 5. Fifth pass: Find where to put data, and then rectify reference to data/memory locations.
pub fn parse_asm(input: &str) -> Vec<u32> {
    let parser_iter = cleaner::Cleaner::new(parser::Parser::new(lexer::Lexer::new(input)));
    let mut parser = labeler::symbol_table_expansion(parser_iter);

    // Bytecode output
    let mut bytecode: Vec<u32> = Vec::new();

    parser.reverse(); // Since we pop from end, reverse order
    while let Some(token) = parser.pop() {
        bytecode.push(lut_to_binary(token));
    }

    bytecode
}

fn lut_to_binary(token: labeler::AToken) -> u32 {
    let inst_encode = lookup(&token);
    let mut ret: u32 = 0x0;

    // Opcode
    ret |= inst_encode.opcode;

    // Func3
    ret |= match_and_shift(inst_encode.func3, 12);

    // Func7
    ret |= match_and_shift(inst_encode.func7, 25);

    // 6. i think a good step will be to use the data in the LUT to construct a binary line (u32)
    match token {
        // 31-25, 24-20, 19-15, 14-12, 11-7, 6-0
        // func7,   rs2,   rs1, func3,   rd, opcode
        labeler::AToken::RegRegReg(_, rd, rs1, rs2) => {
            ret |= extract_and_shift(rd,  7);
            ret |= extract_and_shift(rs1, 15);
            ret |= extract_and_shift(rs2, 20);
        },

        labeler::AToken::RegImmCsr(_, rd, imm, csr) => {
            ret |= extract_and_shift(rd,  7);
            ret |= select_and_shift(imm, 5, 0, 15);
            ret |= extract_and_shift(csr, 20);
        },

        labeler::AToken::RegRegShamt(_, rd, rs1, imm) => {
            ret |= extract_and_shift(rd,  7);
            ret |= extract_and_shift(rs1, 15);

            // TODO: deal with imm
            // shamt[4:0]
            ret |= select_and_shift(imm, 4, 0, 20);
            // imm[11:5] - taken care by func7
        },

        labeler::AToken::RegRegCsr(_, rd, rs1, csr) => {
            ret |= extract_and_shift(rd,  7);
            ret |= extract_and_shift(rs1, 15);
            ret |= extract_and_shift(csr, 20);
        },

        labeler::AToken::RegRegIL(_, rd, rs1, imm) => {
            ret |= extract_and_shift(rd,  7);
            ret |= extract_and_shift(rs1, 15);

            // TODO: deal with imm
            // TODO: design a function for dealing with imm (takes a list of range + shift)
            // for extracting bytes and shifting em to relevant spot plus dealing with sign
            // extend as needed
            // imm[11:0]
            ret |= select_and_shift(imm, 11, 0, 20);
        },

        labeler::AToken::RegRegImm(_, rd, rs1, imm) => {
            ret |= extract_and_shift(rd,  7);
            ret |= extract_and_shift(rs1, 15);

            // TODO: to support addi for 'la'
            // TODO: deal with imm
            // TODO: design a function for dealing with imm (takes a list of range + shift)
            // for extracting bytes and shifting em to relevant spot plus dealing with sign
            // extend as needed
            // imm[11:0]
            ret |= select_and_shift(imm, 11, 0, 20);
        },

        // 31-25, 24-20, 19-15, 14-12, 11-7, 6-0
        //   imm,   rs2,   rs1, func3,  imm, opcode
        labeler::AToken::RegRegImmStore(_, rs1, rs2, imm) => {
            ret |= extract_and_shift(rs1, 15);
            ret |= extract_and_shift(rs2, 20);

            // TODO: deal with imm
            // imm[4:0]
            ret |= select_and_shift(imm, 4, 0, 7);
            // imm[11:5]
            ret |= select_and_shift(imm, 11, 5, 25);
        },

        // TODO: objdump shows bne/beq swapped and so on
        // plus all fixed 0x8 jump, so i'm not sure I'm
        // encoding these quite right, but with the re-arranged
        // order it seems to work.... ?
        labeler::AToken::RegRegILBranch(_, rs1, rs2, imm) => {
            ret |= extract_and_shift(rs1, 15);
            ret |= extract_and_shift(rs2, 20);

            // TODO: deal with imm
            // imm[11]
            ret |= select_and_shift(imm, 11, 11, 7);
            // imm[4:1]
            ret |= select_and_shift(imm, 4, 1, 8);
            // imm[10:5]
            ret |= select_and_shift(imm, 10, 5, 25);
            // imm[12]
            ret |= select_and_shift(imm, 12, 12, 31);
        },

        // 31-12, 11-7, 6-0
        //   imm,   rd, opcode
        // TODO: update lui + auipc tests to not use this mips legacy
        //
        // LUI only probably (verify)
        // imm[31:12]
        //
        // Due to mips legacy, GAS takes the bottom 20 bits, not the top 20 as per
        // the spec, but via %hi(0xFF) and %low(0xFF)... macro? we are able to access
        // the upper 20bit (it gets shifted down 12 bits). Let's do what GAS does here
        // so that we can assemble the output of gcc -S.
        //
        // This mismatch what you would expect from the docs:
        //      LUI places the U-immediate value in the top 20 bits of the destination
        //      register rd, filling in the lowest 12 bits with zeros.
        //
        // TODO: Add `li x1 0xff` and `la x1 symbol` which makes this nicer (takes the
        // value and symbol and split it into upper 20 and lower 12 bits and load it)
        // TODO: deal with imm
        //ret |= select_and_shift(imm, 19, 0, 12);
        labeler::AToken::RegIL(_, rd, imm) => {
            ret |= extract_and_shift(rd, 7);

            // TODO: this relative offset doesn't work for LUI, we want to see label, get the value from that and store that
            ret |= select_and_shift(imm, 19, 0, 12);
            //ret |= select_and_shift(imm, 31, 12, 12);
        },

        labeler::AToken::RegILShuffle(_, rd, imm) => {
            ret |= extract_and_shift(rd, 7);

            // TODO: deal with imm
            // imm[19:12]
            ret |= select_and_shift(imm, 19, 12, 0);
            // imm[11]
            ret |= select_and_shift(imm, 11, 11, 20);
            // imm[10:1]
            ret |= select_and_shift(imm, 10, 1, 21);
            // imm[20]
            ret |= select_and_shift(imm, 20, 20, 31);
        },
    }

    // 7. check if its an instruction that needs additional/special processing, if so do the deed
    // 8. emit out the binary code (see if we can't construct a simple example that uses at least 1 of
    //    all instruction and features in gcc, then use that to get the binarycode for that, and then
    //    compare the two to make sure our result is the same)
    // 9. proceed to start work on the virtual machine
    ret
}

fn match_and_shift(byte: Option<u32>, shift: u32) -> u32 {
    match byte {
        Some(x) => x << shift,
        _ => 0x0,
    }
}

fn select_and_shift(imm: u32, hi: usize, lo: usize, shift: usize) -> u32 {
    ((imm & u32::mask(hi..lo)) >> lo) << shift
}

fn extract_and_shift<T: std::convert::Into<u32>>(arg: T, shift: usize) -> u32 {
    let val: u32 = arg.into();
    val << shift
}

fn lookup(inst: &labeler::AToken) -> opcode::InstEnc {
    match inst {
        labeler::AToken::RegRegReg(i, _, _, _)       => llookup(&i),
        labeler::AToken::RegImmCsr(i, _, _, _)       => llookup(&i),
        labeler::AToken::RegRegCsr(i, _, _, _)       => llookup(&i),
        labeler::AToken::RegRegShamt(i, _, _, _)     => llookup(&i),
        labeler::AToken::RegRegImm(i, _, _, _)       => llookup(&i),
        labeler::AToken::RegRegImmStore(i, _, _, _)  => llookup(&i),
        labeler::AToken::RegRegIL(i, _, _, _)        => llookup(&i),
        labeler::AToken::RegRegILBranch(i, _, _, _)  => llookup(&i),
        labeler::AToken::RegIL(i, _, _)              => llookup(&i),
        labeler::AToken::RegILShuffle(i, _, _)       => llookup(&i),
    }
}

fn llookup(inst: &str) -> opcode::InstEnc {
    match opcode::lookup(&inst) {
        None    => panic!("Failed to find - {:?}", inst),
        Some(x) => x,
    }
}
