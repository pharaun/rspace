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
#[derive(Debug, Clone)]
pub enum InstType {
    R,
    I,
    S,
    SB, // Subtype of S
    U,
    UJ, // Subtype of U
}

// Opcode
pub const OP_IMM:   u32 = 0b001_0011; // I - (OP-IMM in docs)
pub const LUI:      u32 = 0b011_0111; // U
pub const AUIPC:    u32 = 0b001_0111; // U
pub const OP_REG:   u32 = 0b011_0011; // R - (OP in docs)
pub const JAL:      u32 = 0b110_1111; // UJ - imm -> signed offset, in multiples of 2 bytes
pub const JALR:     u32 = 0b110_0111; // I - complicated
pub const BRANCH:   u32 = 0b110_0011; // SB - signed offset in multiples of 2 + pc
pub const LOAD:     u32 = 0b000_0011; // I
pub const STORE:    u32 = 0b010_0011; // S
pub const MISC_MEM: u32 = 0b000_1111; // I
pub const SYSTEM:   u32 = 0b111_0011; // I - CSR (control and status registers) + other priviledged instructions

// Inst Encoding
#[derive(Debug, Clone)]
pub struct InstEnc {
    pub encoding:   InstType,
    pub opcode:     u32,
    pub func3:      Option<u32>,
    pub func7:      Option<u32>,
}

// Map from opcode to inst type
pub fn instruction_type(opcode: u32) -> InstType {
    match opcode {
        OP_IMM | JALR | LOAD | MISC_MEM | SYSTEM => InstType::I,
        LUI | AUIPC => InstType::U,
        JAL    => InstType::UJ,
        OP_REG => InstType::R,
        BRANCH => InstType::SB,
        STORE  => InstType::S,

        // Handle this case
        _ => InstType::I,
    }
}

// Codegen from pf_codegen
include!(concat!(env!("OUT_DIR"), "/opcode.rs"));

pub fn lookup(keyword: &str) -> Option<InstEnc> {
    OPCODE.get(keyword).cloned()
}
