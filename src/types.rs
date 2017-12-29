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
pub struct InstEnc {
    encoding:   InstType,
    opcode:     u32,
    func3:      Option<u32>,
    func7:      Option<u32>,
}

// OP_IMM
pub const ADDI:  InstEnc = InstEnc{encoding: InstType::I, opcode: OP_IMM, func3: Some(0b000), func7: None};
pub const SLTI:  InstEnc = InstEnc{encoding: InstType::I, opcode: OP_IMM, func3: Some(0b010), func7: None};
pub const SLTIU: InstEnc = InstEnc{encoding: InstType::I, opcode: OP_IMM, func3: Some(0b011), func7: None};

pub const ANDI:  InstEnc = InstEnc{encoding: InstType::I, opcode: OP_IMM, func3: Some(0b111), func7: None};
pub const ORI:   InstEnc = InstEnc{encoding: InstType::I, opcode: OP_IMM, func3: Some(0b110), func7: None};
pub const XORI:  InstEnc = InstEnc{encoding: InstType::I, opcode: OP_IMM, func3: Some(0b100), func7: None};

// TODO: shamt
// shamt - 0b0000000
pub const SLLI:  InstEnc = InstEnc{encoding: InstType::I, opcode: OP_IMM, func3: Some(0b001), func7: None};
// shamt - 0b0000000
pub const SRLI:  InstEnc = InstEnc{encoding: InstType::I, opcode: OP_IMM, func3: Some(0b101), func7: None};
// shamt - 0b0100000
pub const SRAI:  InstEnc = InstEnc{encoding: InstType::I, opcode: OP_IMM, func3: Some(0b101), func7: None};

// LUI
//pub const LUI:   InstEnc = InstEnc{encoding: InstType::U, opcode: LUI, func3: None, func7: None};

// AUIPC
//pub const AUIPC: InstEnc = InstEnc{encoding: InstType::U, opcode: AUIPC, func3: None, func7: None};

// OP_REG
pub const ADD:   InstEnc = InstEnc{encoding: InstType::R, opcode: OP_REG, func3: Some(0b000), func7: Some(0b0000000)};
pub const SLT:   InstEnc = InstEnc{encoding: InstType::R, opcode: OP_REG, func3: Some(0b010), func7: Some(0b0000000)};
pub const SLTU:  InstEnc = InstEnc{encoding: InstType::R, opcode: OP_REG, func3: Some(0b011), func7: Some(0b0000000)};

pub const AND:   InstEnc = InstEnc{encoding: InstType::R, opcode: OP_REG, func3: Some(0b111), func7: Some(0b0000000)};
pub const OR:    InstEnc = InstEnc{encoding: InstType::R, opcode: OP_REG, func3: Some(0b110), func7: Some(0b0000000)};
pub const XOR:   InstEnc = InstEnc{encoding: InstType::R, opcode: OP_REG, func3: Some(0b100), func7: Some(0b0000000)};

pub const SLL:   InstEnc = InstEnc{encoding: InstType::R, opcode: OP_REG, func3: Some(0b001), func7: Some(0b0000000)};
pub const SRL:   InstEnc = InstEnc{encoding: InstType::R, opcode: OP_REG, func3: Some(0b101), func7: Some(0b0000000)};

pub const SUB:   InstEnc = InstEnc{encoding: InstType::R, opcode: OP_REG, func3: Some(0b000), func7: Some(0b0100000)};
pub const SRA:   InstEnc = InstEnc{encoding: InstType::R, opcode: OP_REG, func3: Some(0b101), func7: Some(0b0100000)};

// JAL
//pub const JAL:   InstEnc = InstEnc{encoding: InstType::U, opcode: JAL, func3: None, func7: None};

// JALR
//pub const JALR:  InstEnc = InstEnc{encoding: InstType::I, opcode: JALR, func3: Some(0b000), func7: None};

// BRANCH
pub const BEQ:   InstEnc = InstEnc{encoding: InstType::B, opcode: BRANCH, func3: Some(0b000), func7: None};
pub const BNE:   InstEnc = InstEnc{encoding: InstType::B, opcode: BRANCH, func3: Some(0b001), func7: None};

pub const BLT:   InstEnc = InstEnc{encoding: InstType::B, opcode: BRANCH, func3: Some(0b100), func7: None};
pub const BLTU:  InstEnc = InstEnc{encoding: InstType::B, opcode: BRANCH, func3: Some(0b110), func7: None};

pub const BGE:   InstEnc = InstEnc{encoding: InstType::B, opcode: BRANCH, func3: Some(0b101), func7: None};
pub const BGEU:  InstEnc = InstEnc{encoding: InstType::B, opcode: BRANCH, func3: Some(0b111), func7: None};

// LOAD
pub const LW:    InstEnc = InstEnc{encoding: InstType::I, opcode: LOAD, func3: Some(0b010), func7: None};
pub const LH:    InstEnc = InstEnc{encoding: InstType::I, opcode: LOAD, func3: Some(0b001), func7: None};
pub const LHU:   InstEnc = InstEnc{encoding: InstType::I, opcode: LOAD, func3: Some(0b101), func7: None};
pub const LB:    InstEnc = InstEnc{encoding: InstType::I, opcode: LOAD, func3: Some(0b000), func7: None};
pub const LBU:   InstEnc = InstEnc{encoding: InstType::I, opcode: LOAD, func3: Some(0b100), func7: None};

// STORE
pub const SW:    InstEnc = InstEnc{encoding: InstType::S, opcode: STORE, func3: Some(0b010), func7: None};
pub const SH:    InstEnc = InstEnc{encoding: InstType::S, opcode: STORE, func3: Some(0b001), func7: None};
pub const SB:    InstEnc = InstEnc{encoding: InstType::S, opcode: STORE, func3: Some(0b000), func7: None};

// MISC_MEM
// TODO: custom bitfield (but its nop in the vm tho)
// imm - custom bitfield
pub const FENCE:   InstEnc = InstEnc{encoding: InstType::I, opcode: MISC_MEM, func3: Some(0b000), func7: None};
pub const FENCE_I: InstEnc = InstEnc{encoding: InstType::I, opcode: MISC_MEM, func3: Some(0b001), func7: None};

// SYSTEM
// CSR - RDCYCLE[H], RDTIME[H], RDINSTRET[H]
// TODO: custom layout (imm/registers/etc)
pub const CSRRW:   InstEnc = InstEnc{encoding: InstType::I, opcode: SYSTEM, func3: Some(0b001), func7: None};
pub const CSRRS:   InstEnc = InstEnc{encoding: InstType::I, opcode: SYSTEM, func3: Some(0b010), func7: None};
pub const CSRRC:   InstEnc = InstEnc{encoding: InstType::I, opcode: SYSTEM, func3: Some(0b011), func7: None};
pub const CSRRWI:  InstEnc = InstEnc{encoding: InstType::I, opcode: SYSTEM, func3: Some(0b101), func7: None};
pub const CSRRSI:  InstEnc = InstEnc{encoding: InstType::I, opcode: SYSTEM, func3: Some(0b110), func7: None};
pub const CSRRCI:  InstEnc = InstEnc{encoding: InstType::I, opcode: SYSTEM, func3: Some(0b111), func7: None};

// func12 - ECALL- 0b000000000000, EBREAK - 0b000000000001
pub const PRIV:    InstEnc = InstEnc{encoding: InstType::I, opcode: SYSTEM, func3: Some(0b000), func7: None};

// Extension - M type
// OP_REG
pub const MUL:    InstEnc = InstEnc{encoding: InstType::R, opcode: OP_REG, func3: Some(0b000), func7: Some(0b0000001)};
pub const MULH:   InstEnc = InstEnc{encoding: InstType::R, opcode: OP_REG, func3: Some(0b001), func7: Some(0b0000001)};
pub const MULHU:  InstEnc = InstEnc{encoding: InstType::R, opcode: OP_REG, func3: Some(0b011), func7: Some(0b0000001)};
pub const MULHSU: InstEnc = InstEnc{encoding: InstType::R, opcode: OP_REG, func3: Some(0b010), func7: Some(0b0000001)};

pub const DIV:    InstEnc = InstEnc{encoding: InstType::R, opcode: OP_REG, func3: Some(0b100), func7: Some(0b0000001)};
pub const DIVU:   InstEnc = InstEnc{encoding: InstType::R, opcode: OP_REG, func3: Some(0b101), func7: Some(0b0000001)};
pub const REM:    InstEnc = InstEnc{encoding: InstType::R, opcode: OP_REG, func3: Some(0b110), func7: Some(0b0000001)};
pub const REMU:   InstEnc = InstEnc{encoding: InstType::R, opcode: OP_REG, func3: Some(0b111), func7: Some(0b0000001)};
