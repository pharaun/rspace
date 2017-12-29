use phf;

// Instruction formats for risc-v (not compressed)
// Notes:
// Register - x1->x31 is general purpose, x0 always hardcoded to 0, pc reister
// Instruction - must be aligned on four byte memory
// Imm - always sign extended (except the 5 bit imm used in CSR instruction), sign always 31st bit
// Macro - NOP
//
// All instruction is assumed to complete immediately, and in a single cycle.
//      Instruction barrier instruction (fence.i) and Data barrier instructions (fence) are both
//          set to be NOP (except undefined instruction behavior)
//      Caches & Write buffers are not simulated.
//          All load/fetches/stores completes immediately and in order and fully synchronously
#[derive(Debug)]
#[derive(Clone)]
pub enum InstType {
    R,
    I,
    S,
    B, // Subtype of S
    U,
    J, // Subtype of U
}

// Opcode
pub const OP_IMM:   u32 = 0b0010011; // I - (OP-IMM in docs)
pub const LUI:      u32 = 0b0110111; // U
pub const AUIPC:    u32 = 0b0010111; // U
pub const OP_REG:   u32 = 0b0110011; // R - (OP in docs)
pub const JAL:      u32 = 0b1101111; // J - imm -> signed offset, in multiples of 2 bytes
pub const JALR:     u32 = 0b1100111; // I - complicated
pub const BRANCH:   u32 = 0b1100011; // B - signed offset in multiples of 2 + pc
pub const LOAD:     u32 = 0b0000011; // I
pub const STORE:    u32 = 0b0100011; // S
pub const MISC_MEM: u32 = 0b0001111; // I
pub const SYSTEM:   u32 = 0b1110011; // I - CSR (control and status registers) + other priviledged instructions

// Inst Encoding
#[derive(Debug)]
#[derive(Clone)]
pub struct InstEnc {
    encoding:   InstType,
    opcode:     u32,
    func3:      Option<u32>,
    func7:      Option<u32>,
}

// Codegen from pf_codegen
include!("../codegen/opcode.rs");

pub fn parse_keyword(keyword: &str) -> Option<InstEnc> {
    OPCODE.get(keyword).cloned()
}
