use twiddle::Twiddle;

pub mod regfile;
pub mod mem;
pub mod csr;
pub mod opcode;


pub struct Emul32 {
    reg: regfile::RegFile,
    mem: mem::Memory,
    csr: csr::Csr,
    pc: usize
}

impl Emul32 {
    pub fn new_with_rom(rom: [u8; 4096]) -> Emul32 {
        Emul32 {
            reg: regfile::RegFile::new([0; 31]),
            mem: mem::Memory::new(rom, [0; 4096]),
            csr: csr::Csr::new([0; 4096]),
            pc: 0
        }
    }

    pub fn new(reg: regfile::RegFile, mem: mem::Memory, csr: csr::Csr, pc: usize) -> Emul32 {
        Emul32 {
            reg: reg,
            mem: mem,
            csr: csr,
            pc: pc
        }
    }

    pub fn run(&mut self) {
        // VM loop
        loop {
            // TODO: unitify memory at some point
            // TODO: deal with u32 access for inst
            // TODO: if inst is read from a non u32 aligned address, error out (ISA specifies this)
            // TODO: instruction that is all zero or all ones is an illegal instruction (trap)
            let inst = self.mem.fetch_instruction(self.pc);

            // Decode opcode
            let opcode  = select_and_shift(inst, 6, 0);

            // Inst Type
            // TODO: change this over to generating the mask needed (for rspace issue #4)
            //let instType = opcode::instruction_type(opcode);

            // Prefetch the func3/7
            let func3   = select_and_shift(inst, 14, 12);
            let func7   = select_and_shift(inst, 31, 25);

            // Prefetch rd/rs1/rs2
            let rd      = select_and_shift(inst, 11, 7) as usize;
            let rs1     = select_and_shift(inst, 19, 15) as usize;
            let rs2     = select_and_shift(inst, 24, 20) as usize;

            // IMM types - Probably can be put in the asm steps
            let shamt   = select_and_shift(inst, 24, 20);
            // TODO: handle sign extend and so on as needed
            let i_imm   = select_and_shift(inst, 31, 20);
            let s_imm   = (select_and_shift(inst, 31, 25) << 5)
                        | select_and_shift(inst, 11, 7);
            let sb_imm  = (select_and_shift(inst, 31, 31) << 12)
                        | (select_and_shift(inst, 7, 7) << 11)
                        | (select_and_shift(inst, 30, 25) << 5)
                        | (select_and_shift(inst, 11, 8) << 1);
            let u_imm   = select_and_shift(inst, 31, 12) << 12;
            let uj_imm  = (select_and_shift(inst, 31, 31) << 20)
                        | (select_and_shift(inst, 19, 12) << 12)
                        | (select_and_shift(inst, 20, 20) << 11)
                        | (select_and_shift(inst, 30, 21) << 1);

            // CSR related
            let csr     = select_and_shift(inst, 31, 20) as usize; // functionally same as i_imm
            let csr_imm = select_and_shift(inst, 24, 20); // functionally same as rs2

            // TODO: handle these items
            // - exception (invalid instruction, invalid memory access, etc...)
            // - interrupts (external async event)
            // - traps (transfer of control to a trap handler for interrupt or exception)
            //
            // NOTE:
            // - Illegal instructions:
            //   * any instructions encountered with either low bit clear should be
            //     considered illegal 30-bit instructions
            //   * Encodings with bits [15:0] all zeros are defined as illegal instructions.
            //     These instructions are considered to be of minimal length:
            //     16 bits if any 16-bit instruction-set extension is present,
            //     otherwise 32 bits. The encoding with bits [ILEN-1:0] all ones is also illegal;
            //     this instruction is considered to be ILEN bits long.
            //   * all 0 and all 1 in [15:0] are considered illegal as well
            //
            // - Instruction access exception
            //   * All are a fixed 32 bits in length and must be aligned on a four-byte
            //     boundary in memory. An instruction-address-misaligned exception is
            //     generated on a taken branch or unconditional jump if the target address
            //     is not four-byte aligned. This exception is reported on the branch or
            //     jump instruction, not on the target instruction.
            //   * No instruction-address-misaligned exception is generated for a
            //     conditional branch that is not taken.
            //
            // - Support misaligned load/store ops (optional to except misaligned load)
            //   * May need to issue misaligned address or access exception if writing to
            //     that memory region can cause side-effect (ie IOMAP - hardware addresses)
            //
            // - ecall is used to make service request to the ie outer environment (ie machine)
            //   and will call (i think) into a trap
            //
            match (func7, func3, opcode) {
                // RV32 I
                (0b0000000, 0b000, opcode::OP_REG) => {
                    // ADD
                    self.reg[rd] = self.reg[rs1].wrapping_add(self.reg[rs2]);
                },
                (0b0100000, 0b000, opcode::OP_REG) => {
                    // SUB
                    self.reg[rd] = self.reg[rs1].wrapping_sub(self.reg[rs2]);
                },
                (0b0000000, 0b001, opcode::OP_REG) => {
                    // SLL
                    let shamt = select_and_shift(self.reg[rs2], 4, 0);
                    self.reg[rd] = self.reg[rs1] << shamt;
                },
                (0b0000000, 0b010, opcode::OP_REG) => {
                    // SLT
                    self.reg[rd] = if (self.reg[rs1] as i32) < (self.reg[rs2] as i32) { 0x1 } else { 0x0 }
                },
                (0b0000000, 0b011, opcode::OP_REG) => {
                    // SLTU
                    self.reg[rd] = if self.reg[rs1] < self.reg[rs2] { 0x1 } else { 0x0 }
                },
                (0b0000000, 0b100, opcode::OP_REG) => {
                    // XOR
                    self.reg[rd] = self.reg[rs1] ^ self.reg[rs2];
                },
                (0b0000000, 0b101, opcode::OP_REG) => {
                    // SRL
                    let shamt = select_and_shift(self.reg[rs2], 4, 0);
                    self.reg[rd] = self.reg[rs1] >> shamt;
                },
                (0b0100000, 0b101, opcode::OP_REG) => {
                    // SRA
                    let shamt = select_and_shift(self.reg[rs2], 4, 0);
                    // apparently arithmetic right shift depends on type of left operator
                    self.reg[rd] = ((self.reg[rs1] as i32) >> shamt) as u32;
                },
                (0b0000000, 0b110, opcode::OP_REG) => {
                    // OR
                    self.reg[rd] = self.reg[rs1] | self.reg[rs2];
                },
                (0b0000000, 0b111, opcode::OP_REG) => {
                    // AND
                    self.reg[rd] = self.reg[rs1] & self.reg[rs2];
                },

                // RV32 M extensions
                (0b0000001, 0b000, opcode::OP_REG) => {
                    // MUL
                    let product: u64 = (self.reg[rs1] as u64) * (self.reg[rs2] as u64);
                    self.reg[rd] = (product & u64::mask(31..0)) as u32;
                },
                (0b0000001, 0b001, opcode::OP_REG) => {
                    // MULH
                    let product: i64 = (sign_extend_32_to_64(self.reg[rs1]) as i64) * (sign_extend_32_to_64(self.reg[rs2]) as i64);
                    self.reg[rd] = (((product >> 32) as u64) & u64::mask(31..0)) as u32;
                },
                (0b0000001, 0b010, opcode::OP_REG) => {
                    // MULHSU
                    let product: i64 = (sign_extend_32_to_64(self.reg[rs1]) as i64) * (self.reg[rs2] as i64);
                    self.reg[rd] = (((product >> 32) as u64) & u64::mask(31..0)) as u32;
                },
                (0b0000001, 0b011, opcode::OP_REG) => {
                    // MULHU
                    let product: u64 = (self.reg[rs1] as u64) * (self.reg[rs2] as u64);
                    self.reg[rd] = ((product >> 32) & u64::mask(31..0)) as u32;
                },
                (0b0000001, 0b100, opcode::OP_REG) => {
                    // DIV
                    let _neg: u32 = (-1 as i32) as u32;
                    self.reg[rd] = match (self.reg[rs2], self.reg[rs1]) {
                        (    0x0,             _) => (-1i32) as u32,
                        (   _neg, 0xff_ff_ff_ff) => 0xff_ff_ff_ff,
                        (divisor,      dividend) => (dividend as i32).wrapping_div(divisor as i32) as u32,
                    };
                },
                (0b0000001, 0b101, opcode::OP_REG) => {
                    // DIVU
                    if self.reg[rs2] == 0x0 {
                        self.reg[rd] = 0xff_ff_ff_ff;
                    } else {
                        self.reg[rd] = self.reg[rs1] / self.reg[rs2];
                    }
                },
                (0b0000001, 0b110, opcode::OP_REG) => {
                    // REM
                    let _neg: u32 = (-1 as i32) as u32;
                    self.reg[rd] = match (self.reg[rs2], self.reg[rs1]) {
                        (    0x0,             _) => self.reg[rs1],
                        (   _neg, 0xff_ff_ff_ff) => 0x0,
                        (divisor,      dividend) => (dividend as i32).wrapping_rem(divisor as i32) as u32,
                    };
                },
                (0b0000001, 0b111, opcode::OP_REG) => {
                    // REMU
                    if self.reg[rs2] == 0x0 {
                        self.reg[rd] = self.reg[rs1];
                    } else {
                        self.reg[rd] = self.reg[rs1] % self.reg[rs2];
                    }
                },

                // RV32 I
                (        _, 0b000, opcode::OP_IMM) => {
                    // ADDI
                    self.reg[rd] = self.reg[rs1].wrapping_add(sign_extend(inst, i_imm));
                },
                (0b0000000, 0b001, opcode::OP_IMM) => {
                    // SLLI
                    self.reg[rd] = self.reg[rs1] << shamt;
                },
                (        _, 0b010, opcode::OP_IMM) => {
                    // SLTI
                    self.reg[rd] = if (self.reg[rs1] as i32) < (sign_extend(inst, i_imm) as i32) { 0x1 } else { 0x0 }
                },
                (        _, 0b011, opcode::OP_IMM) => {
                    // SLTIU
                    self.reg[rd] = if self.reg[rs1] < sign_extend(inst, i_imm) { 0x1 } else { 0x0 }
                },
                (        _, 0b100, opcode::OP_IMM) => {
                    // XORI
                    self.reg[rd] = self.reg[rs1] ^ sign_extend(inst, i_imm);
                },
                (0b0000000, 0b101, opcode::OP_IMM) => {
                    // SRLI
                    self.reg[rd] = self.reg[rs1] >> shamt;
                },
                (0b0100000, 0b101, opcode::OP_IMM) => {
                    // SRAI
                    // apparently arithmetic right shift depends on type of left operator
                    self.reg[rd] = ((self.reg[rs1] as i32) >> shamt) as u32;
                },
                (        _, 0b110, opcode::OP_IMM) => {
                    // ORI
                    self.reg[rd] = self.reg[rs1] | sign_extend(inst, i_imm);
                },
                (        _, 0b111, opcode::OP_IMM) => {
                    // ANDI
                    self.reg[rd] = self.reg[rs1] & sign_extend(inst, i_imm);
                },

                // RV32 I
                (        _, 0b000, opcode::JALR) => {
                    // JALR
                    self.reg[rd] = (self.pc + 4) as u32;
                    // Need to zero the last value
                    self.pc = ((self.reg[rs1] + i_imm - 4) & 0xff_ff_ff_fe) as usize;
                    // Because after this inst complete the pc will +4 at the end)
                },

                // RV32 I
                (        _, 0b000, opcode::LOAD) => {
                    // LB
                    self.reg[rd] = sign_extend_8_to_32(self.mem.load_byte(
                        (self.reg[rs1].wrapping_add(sign_extend(inst, i_imm))) as usize
                    ));
                },
                (        _, 0b001, opcode::LOAD) => {
                    // LH
                    self.reg[rd] = sign_extend_16_to_32(self.mem.load_half(
                        (self.reg[rs1].wrapping_add(sign_extend(inst, i_imm))) as usize
                    ));
                },
                (        _, 0b010, opcode::LOAD) => {
                    // LW
                    self.reg[rd] = self.mem.load_word(
                        (self.reg[rs1].wrapping_add(sign_extend(inst, i_imm))) as usize
                    );
                },
                (        _, 0b100, opcode::LOAD) => {
                    // LBU
                    self.reg[rd] = self.mem.load_byte(
                        (self.reg[rs1].wrapping_add(sign_extend(inst, i_imm))) as usize
                    );
                },
                (        _, 0b101, opcode::LOAD) => {
                    // LHU
                    self.reg[rd] = self.mem.load_half(
                        (self.reg[rs1].wrapping_add(sign_extend(inst, i_imm))) as usize
                    );
                },

                // RV32 I
                (        _, 0b000, opcode::MISC_MEM) => {
                    // FENCE
                    // NOP instruction
                },

                // RV32 I
                (        _, 0b000, opcode::SYSTEM) => {
                    // ECALL | EBREAK
                    let imm   = select_and_shift(inst, 31, 20);

                    match imm {
                        0b000000000000 => {
                            // ECALL
                            // NOP instruction
                        },
                        0b000000000001 => {
                            // EBREAK
                            // NOP instruction
                        },
                        _ => panic!("FIXME"),
                    }
                },

                // RV32 Zicsr extensions
                // TODO:
                // - if rd = x0 then csr does not read and cause read side effects (rw)
                // - if rs1 = x0 then csr does not write and cause write side effects (rs/c)
                //   * such as raising illegal instruction exceptions on accesses to read-only CSRs.
                //
                //  inst    rd rs1
                // CSRRW    x0   - -> no-read, write
                // CSRRW   !x0   - -> read, write
                // CSRRS/C   -  x0 -> read, no-write
                // CSRRS/C   - !x0 -> read, write
                //
                // for Imm variant replace x0 with 0 and !0 for rs1
                //
                // TODO: decide if i want and/or care to implement these counters
                // - RDCYCLE[H], RDTIME[H], RDINSTRET[H] - since these will need
                //   main vm loop support to implement
                // - These also forms a pseudo instruction if we do implement em
                // - RDCYCLE = cycle, RDTIME = time, RDINSTRET = instret (64bit counters)
                //
                (        _, 0b001, opcode::SYSTEM) => {
                    // CSRRW
                    self.reg[rd] = self.csr.read_write(
                        csr,
                        self.reg[rs1],
                    );
                },
                (        _, 0b010, opcode::SYSTEM) => {
                    // CSRRS
                    self.reg[rd] = self.csr.read_set(
                        csr,
                        self.reg[rs1],
                    );
                },
                (        _, 0b011, opcode::SYSTEM) => {
                    // CSRRC
                    self.reg[rd] = self.csr.read_clear(
                        csr,
                        self.reg[rs1],
                    );
                },
                (        _, 0b101, opcode::SYSTEM) => {
                    // CSRRWI
                    self.reg[rd] = self.csr.read_write(
                        csr,
                        csr_imm,
                    );
                },
                (        _, 0b110, opcode::SYSTEM) => {
                    // CSRRSI
                    self.reg[rd] = self.csr.read_set(
                        csr,
                        csr_imm,
                    );
                },
                (        _, 0b111, opcode::SYSTEM) => {
                    // CSRRCI
                    self.reg[rd] = self.csr.read_clear(
                        csr,
                        csr_imm,
                    );
                },

                // RV32 I
                (        _, 0b000, opcode::STORE) => {
                    // SB
                    self.mem.store_byte(
                        (self.reg[rs1].wrapping_add(sign_extend(inst, s_imm))) as usize,
                        self.reg[rs2],
                    );
                },
                (        _, 0b001, opcode::STORE) => {
                    // SH
                    self.mem.store_half(
                        (self.reg[rs1].wrapping_add(sign_extend(inst, s_imm))) as usize,
                        self.reg[rs2],
                    );
                },
                (        _, 0b010, opcode::STORE) => {
                    // SW
                    self.mem.store_word(
                        (self.reg[rs1].wrapping_add(sign_extend(inst, s_imm))) as usize,
                        self.reg[rs2],
                    );
                },

                // RV32 I
                (        _, 0b000, opcode::BRANCH) => {
                    // BEQ
                    if self.reg[rs1] == self.reg[rs2] {
                        self.pc = (sign_extend(inst, sb_imm).wrapping_add(self.pc as u32)) as usize;
                        self.pc = self.pc - 4; // Because after this inst complete the pc will +4 at the end)
                    }
                },
                (        _, 0b001, opcode::BRANCH) => {
                    // BNE
                    if self.reg[rs1] != self.reg[rs2] {
                        self.pc = (sign_extend(inst, sb_imm).wrapping_add(self.pc as u32)) as usize;
                        self.pc = self.pc - 4; // Because after this inst complete the pc will +4 at the end)
                    }
                },
                (        _, 0b100, opcode::BRANCH) => {
                    // BLT
                    if (self.reg[rs1] as i32) < (self.reg[rs2] as i32) {
                        self.pc = (sign_extend(inst, sb_imm).wrapping_add(self.pc as u32)) as usize;
                        self.pc = self.pc - 4; // Because after this inst complete the pc will +4 at the end)
                    }
                },
                (        _, 0b101, opcode::BRANCH) => {
                    // BGE
                    if (self.reg[rs1] as i32) >= (self.reg[rs2] as i32) {
                        self.pc = (sign_extend(inst, sb_imm).wrapping_add(self.pc as u32)) as usize;
                        self.pc = self.pc - 4; // Because after this inst complete the pc will +4 at the end)
                    }
                },
                (        _, 0b110, opcode::BRANCH) => {
                    // BLTU
                    if self.reg[rs1] < self.reg[rs2] {
                        self.pc = (sign_extend(inst, sb_imm).wrapping_add(self.pc as u32)) as usize;
                        self.pc = self.pc - 4; // Because after this inst complete the pc will +4 at the end)
                    }
                },
                (        _, 0b111, opcode::BRANCH) => {
                    // BGEU
                    if self.reg[rs1] >= self.reg[rs2] {
                        self.pc = (sign_extend(inst, sb_imm).wrapping_add(self.pc as u32)) as usize;
                        self.pc = self.pc - 4; // Because after this inst complete the pc will +4 at the end)
                    }
                },

                // RV32 I
                (        _,     _, opcode::LUI) => {
                    // LUI
                    self.reg[rd] = u_imm;
                },
                (        _,     _, opcode::AUIPC) => {
                    // AUIPC
                    // TODO: TEST - don't really have a way to test yet
                    //self.reg[rd] = u_imm.wrapping_add(self.pc as u32);
                    panic!("Auipc isn't really supported by the assembler yet");
                },
                (        _,     _, opcode::JAL) => {
                    // JAL
                    self.reg[rd] = (self.pc + 4) as u32;

                    self.pc = (sign_extend(inst, uj_imm).wrapping_add(self.pc as u32)) as usize;
                    self.pc = self.pc - 4; // Because after this inst complete the pc will +4 at the end)
                },

                // TODO: handle instruction decoding failure
                (f7, f3, op) => {
                    println!("FIX  PC: 0x{:04x} F7: {:07b} F3: {:03b} OP: {:07b}", self.pc, f7, f3, op);

                    //println!("ROM DUMP:");
                    //for i in 0..(binary_u8.len()/4) {
                    //    let rinst_u8: [u8; 4] = [rom[i*4], rom[i*4+1], rom[i*4+2], rom[i*4+3]];
                    //    let rinst = unsafe { std::mem::transmute::<[u8; 4], u32>(rinst_u8) };

                    //    println!("{:08x}", rinst);

                    //    let rop = select_and_shift(rinst, 6, 0);
                    //    let rfunc3 = select_and_shift(rinst, 14, 12);
                    //    let rfunc7 = select_and_shift(rinst, 31, 25);
                    //    println!("F7: {:07b} F3: {:03b} OP: {:07b}", rfunc7, rfunc3, rop);
                    //}
                    break;
                    //panic!("FIXME")
                },
            }

            println!("FINE PC: 0x{:04x} F7: {:07b} F3: {:03b} OP: {:07b}", self.pc, func7, func3, opcode);
            self.pc += 4;
        }
    }
}


fn select_and_shift(inst: u32, hi: usize, lo: usize) -> u32 {
    (inst & u32::mask(hi..lo)) >> lo
}


// TODO: better to move imm inst extraction here?
fn sign_extend(inst: u32, imm: u32) -> u32 {
    if (inst & 0x80_00_00_00) == 0x80_00_00_00 {
        let opcode = select_and_shift(inst, 6, 0);
        let mask = match opcode::instruction_type(opcode) {
            opcode::InstType::R  => 0x0,
            opcode::InstType::I  => u32::mask(31..11),
            opcode::InstType::S  => u32::mask(31..11),
            opcode::InstType::SB => u32::mask(31..12),
            opcode::InstType::U  => u32::mask(31..31),
            opcode::InstType::UJ => u32::mask(31..20),
        };

        imm | (0xff_ff_ff_ff & mask)
    } else {
        imm
    }
}

fn sign_extend_8_to_32(imm: u32) -> u32 {
    if (imm & 0x80) == 0x80 {
        imm | 0xff_ff_ff_00
    } else {
        imm
    }
}

fn sign_extend_16_to_32(imm: u32) -> u32 {
    if (imm & 0x80_00) == 0x80_00 {
        imm | 0xff_ff_00_00
    } else {
        imm
    }
}

fn sign_extend_32_to_64(imm: u32) -> u64 {
    if (imm & 0x80_00_00_00) == 0x80_00_00_00 {
        (imm as u64) | 0xff_ff_ff_ff_00_00_00_00
    } else {
        imm as u64
    }
}


// Tests getting too long, include instead
include!("test.rs");
