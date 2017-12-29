#[derive(Debug)]
pub enum Args <'input> {
    Num(u32),
    Reg(&'input str),
}

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
pub enum InstBase {
    R { opcode: u32, rd: u32, func3: u32, rs1: u32, rs2: u32, func7: u32},
    I { opcode: u32, rd: u32, func3: u32, rs1: u32,           imm: u32},
    S { opcode: u32,          func3: u32, rs1: u32, rs2: u32, imm: u32},
    B { opcode: u32,          func3: u32, rs1: u32, rs2: u32, imm: u32}, // Subtype of S
    U { opcode: u32, rd: u32,                                 imm: u32},
    J { opcode: u32, rd: u32,                                 imm: u32}, // Subtype of U
}

// Opcode
pub const OP_IMM:   u32 = 0x0; // I - (OP-IMM in docs)
pub const LUI:      u32 = 0x0; // U
pub const AUIPC:    u32 = 0x0; // U
pub const OP_REG:   u32 = 0x0; // R - (OP in docs)
pub const JAL:      u32 = 0x0; // J - imm -> signed offset, in multiples of 2 bytes
pub const JALR:     u32 = 0x0; // I - complicated
pub const BRANCH:   u32 = 0x0; // B - signed offset in multiples of 2 + pc
pub const LOAD:     u32 = 0x0; // I - LB, LH, LW, LBU, LHU
pub const STORE:    u32 = 0x0; // S - SB, SH, SW
pub const MISC_MEM: u32 = 0x0; // I
pub const SYSTEM:   u32 = 0x0; // I - CSR (control and status registers) + other priviledged instructions

// Func3
// OP_IMM
pub const ADDI:  u32 = 0x0; // I
pub const SLTI:  u32 = 0x0; // I
pub const SLTIU: u32 = 0x0; // I

pub const ANDI: u32 = 0x0; // I
pub const ORI:  u32 = 0x0; // I
pub const XORI: u32 = 0x0; // I

pub const SLLI:  u32 = 0x0; // I - shamt - 0x00
pub const SRLI:  u32 = 0x0; // I - shamt - 0x00
pub const SRAI:  u32 = 0x0; // I - shamt - 0x20

// OP_REG
pub const ADD:  u32 = 0x0; // R - func7 - 0x00
pub const SLT:  u32 = 0x0; // R - func7 - 0x00
pub const SLTU: u32 = 0x0; // R - func7 - 0x00

pub const AND: u32 = 0x0; // R - func7 - 0x00
pub const OR:  u32 = 0x0; // R - func7 - 0x00
pub const XOR: u32 = 0x0; // R - func7 - 0x00

pub const SLL: u32 = 0x0; // R - func7 - 0x00
pub const SRL: u32 = 0x0; // R - func7 - 0x00

pub const SUB: u32 = 0x0; // R - func7 - 0x20
pub const SRA: u32 = 0x0; // R - func7 - 0x20

// BRANCH
pub const BEQ:  u32 = 0x0; // B
pub const BNE:  u32 = 0x0; // B

pub const BLT:  u32 = 0x0; // B
pub const BLTU: u32 = 0x0; // B

pub const BGE:  u32 = 0x0; // B
pub const BGEU: u32 = 0x0; // B

// MISC_MEM
pub const FENCE:    u32 = 0x0; // I - imm - custom bitfield
pub const FENCE_I:  u32 = 0x0; // I

// SYSTEM
// CSR - RDCYCLE[H], RDTIME[H], RDINSTRET[H]
pub const CSRRW:    u32 = 0x0; // I
pub const CSRRS:    u32 = 0x0; // I
pub const CSRRC:    u32 = 0x0; // I
pub const CSRRWI:   u32 = 0x0; // I
pub const CSRRSI:   u32 = 0x0; // I
pub const CSRRCI:   u32 = 0x0; // I

pub const PRIV:     u32 = 0x0; // I - func12 - ECALL, EBREAK

// Extension - M type
// OP_REG
pub const MUL:      u32 = 0x0; // R - func7 - MULDIV
pub const MULH:     u32 = 0x0; // R - func7 - MULDIV
pub const MULHU:    u32 = 0x0; // R - func7 - MULDIV
pub const MULHSU:   u32 = 0x0; // R - func7 - MULDIV

pub const DIV:  u32 = 0x0; // R - func7 - MULDIV
pub const DIVU: u32 = 0x0; // R - func7 - MULDIV
pub const REM:  u32 = 0x0; // R - func7 - MULDIV
pub const REMU: u32 = 0x0; // R - func7 - MULDIV
