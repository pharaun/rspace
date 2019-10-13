extern crate phf_codegen;

use std::fs;
use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::Path;
use std::env;

fn main() {
    // phf_codegen for opcodes
	let path = Path::new(&env::var("OUT_DIR").unwrap()).join("opcode.rs");
    fs::create_dir("codegen").unwrap_or_else(|why| {
        println!("! {:?}", why.kind());
    });
    let mut file = BufWriter::new(File::create(&path).unwrap());

    write!(&mut file, "static OPCODE: phf::Map<&'static str, InstEnc> = ").unwrap();

    phf_codegen::Map::new()
        .entry("ADDI",  "InstEnc{encoding: InstType::I, opcode: OP_IMM, func3: Some(0b000), func7: None}")
        .entry("SLTI",  "InstEnc{encoding: InstType::I, opcode: OP_IMM, func3: Some(0b010), func7: None}")
        .entry("SLTIU", "InstEnc{encoding: InstType::I, opcode: OP_IMM, func3: Some(0b011), func7: None}")

        .entry("ANDI",  "InstEnc{encoding: InstType::I, opcode: OP_IMM, func3: Some(0b111), func7: None}")
        .entry("ORI",   "InstEnc{encoding: InstType::I, opcode: OP_IMM, func3: Some(0b110), func7: None}")
        .entry("XORI",  "InstEnc{encoding: InstType::I, opcode: OP_IMM, func3: Some(0b100), func7: None}")

        // TODO: shamt
        // shamt - 0b0000000
        .entry("SLLI",  "InstEnc{encoding: InstType::I, opcode: OP_IMM, func3: Some(0b001), func7: None}")
        // shamt - 0b0000000
        .entry("SRLI",  "InstEnc{encoding: InstType::I, opcode: OP_IMM, func3: Some(0b101), func7: None}")
        // shamt - 0b0100000
        .entry("SRAI",  "InstEnc{encoding: InstType::I, opcode: OP_IMM, func3: Some(0b101), func7: Some(0b010_0000)}")

        .entry("LUI",   "InstEnc{encoding: InstType::U, opcode: LUI, func3: None, func7: None}")

        .entry("AUIPC", "InstEnc{encoding: InstType::U, opcode: AUIPC, func3: None, func7: None}")

        .entry("ADD",   "InstEnc{encoding: InstType::R, opcode: OP_REG, func3: Some(0b000), func7: Some(0b000_0000)}")
        .entry("SLT",   "InstEnc{encoding: InstType::R, opcode: OP_REG, func3: Some(0b010), func7: Some(0b000_0000)}")
        .entry("SLTU",  "InstEnc{encoding: InstType::R, opcode: OP_REG, func3: Some(0b011), func7: Some(0b000_0000)}")

        .entry("AND",   "InstEnc{encoding: InstType::R, opcode: OP_REG, func3: Some(0b111), func7: Some(0b000_0000)}")
        .entry("OR",    "InstEnc{encoding: InstType::R, opcode: OP_REG, func3: Some(0b110), func7: Some(0b000_0000)}")
        .entry("XOR",   "InstEnc{encoding: InstType::R, opcode: OP_REG, func3: Some(0b100), func7: Some(0b000_0000)}")

        .entry("SLL",   "InstEnc{encoding: InstType::R, opcode: OP_REG, func3: Some(0b001), func7: Some(0b000_0000)}")
        .entry("SRL",   "InstEnc{encoding: InstType::R, opcode: OP_REG, func3: Some(0b101), func7: Some(0b000_0000)}")

        .entry("SUB",   "InstEnc{encoding: InstType::R, opcode: OP_REG, func3: Some(0b000), func7: Some(0b010_0000)}")
        .entry("SRA",   "InstEnc{encoding: InstType::R, opcode: OP_REG, func3: Some(0b101), func7: Some(0b010_0000)}")

        .entry("JAL",   "InstEnc{encoding: InstType::UJ, opcode: JAL, func3: None, func7: None}")

        .entry("JALR",  "InstEnc{encoding: InstType::I, opcode: JALR, func3: Some(0b000), func7: None}")

        .entry("BEQ",   "InstEnc{encoding: InstType::SB, opcode: BRANCH, func3: Some(0b000), func7: None}")
        .entry("BNE",   "InstEnc{encoding: InstType::SB, opcode: BRANCH, func3: Some(0b001), func7: None}")

        .entry("BLT",   "InstEnc{encoding: InstType::SB, opcode: BRANCH, func3: Some(0b100), func7: None}")
        .entry("BLTU",  "InstEnc{encoding: InstType::SB, opcode: BRANCH, func3: Some(0b110), func7: None}")

        .entry("BGE",   "InstEnc{encoding: InstType::SB, opcode: BRANCH, func3: Some(0b101), func7: None}")
        .entry("BGEU",  "InstEnc{encoding: InstType::SB, opcode: BRANCH, func3: Some(0b111), func7: None}")

        .entry("LW",    "InstEnc{encoding: InstType::I, opcode: LOAD, func3: Some(0b010), func7: None}")
        .entry("LH",    "InstEnc{encoding: InstType::I, opcode: LOAD, func3: Some(0b001), func7: None}")
        .entry("LHU",   "InstEnc{encoding: InstType::I, opcode: LOAD, func3: Some(0b101), func7: None}")
        .entry("LB",    "InstEnc{encoding: InstType::I, opcode: LOAD, func3: Some(0b000), func7: None}")
        .entry("LBU",   "InstEnc{encoding: InstType::I, opcode: LOAD, func3: Some(0b100), func7: None}")

        .entry("SW",    "InstEnc{encoding: InstType::S, opcode: STORE, func3: Some(0b010), func7: None}")
        .entry("SH",    "InstEnc{encoding: InstType::S, opcode: STORE, func3: Some(0b001), func7: None}")
        .entry("SB",    "InstEnc{encoding: InstType::S, opcode: STORE, func3: Some(0b000), func7: None}")

        // TODO: custom bitfield (but its nop in the vm tho)
        // imm - custom bitfield
        .entry("FENCE",   "InstEnc{encoding: InstType::I, opcode: MISC_MEM, func3: Some(0b000), func7: None}")

        // TODO: custom layout (imm/registers/etc)
        // TODO: CSR - RDCYCLE[H], RDTIME[H], RDINSTRET[H]
        .entry("CSRRW",  "InstEnc{encoding: InstType::I, opcode: SYSTEM, func3: Some(0b001), func7: None}")
        .entry("CSRRS",  "InstEnc{encoding: InstType::I, opcode: SYSTEM, func3: Some(0b010), func7: None}")
        .entry("CSRRC",  "InstEnc{encoding: InstType::I, opcode: SYSTEM, func3: Some(0b011), func7: None}")
        .entry("CSRRWI", "InstEnc{encoding: InstType::I, opcode: SYSTEM, func3: Some(0b101), func7: None}")
        .entry("CSRRSI", "InstEnc{encoding: InstType::I, opcode: SYSTEM, func3: Some(0b110), func7: None}")
        .entry("CSRRCI", "InstEnc{encoding: InstType::I, opcode: SYSTEM, func3: Some(0b111), func7: None}")

        // PRIV
        .entry("ECALL",  "InstEnc{encoding: InstType::I, opcode: SYSTEM, func3: Some(0b000), func7: None}")
        .entry("EBREAK", "InstEnc{encoding: InstType::I, opcode: SYSTEM, func3: Some(0b000), func7: None}")

        // Extension - M type
        .entry("MUL",    "InstEnc{encoding: InstType::R, opcode: OP_REG, func3: Some(0b000), func7: Some(0b000_0001)}")
        .entry("MULH",   "InstEnc{encoding: InstType::R, opcode: OP_REG, func3: Some(0b001), func7: Some(0b000_0001)}")
        .entry("MULHU",  "InstEnc{encoding: InstType::R, opcode: OP_REG, func3: Some(0b011), func7: Some(0b000_0001)}")
        .entry("MULHSU", "InstEnc{encoding: InstType::R, opcode: OP_REG, func3: Some(0b010), func7: Some(0b000_0001)}")

        .entry("DIV",    "InstEnc{encoding: InstType::R, opcode: OP_REG, func3: Some(0b100), func7: Some(0b000_0001)}")
        .entry("DIVU",   "InstEnc{encoding: InstType::R, opcode: OP_REG, func3: Some(0b101), func7: Some(0b000_0001)}")
        .entry("REM",    "InstEnc{encoding: InstType::R, opcode: OP_REG, func3: Some(0b110), func7: Some(0b000_0001)}")
        .entry("REMU",   "InstEnc{encoding: InstType::R, opcode: OP_REG, func3: Some(0b111), func7: Some(0b000_0001)}")

        .build(&mut file)
        .unwrap();
    write!(&mut file, ";\n").unwrap();
}
