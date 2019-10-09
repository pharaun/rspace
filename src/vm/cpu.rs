use std::ops::Range;

use crate::vm::regfile;
use crate::vm::mem::Mem;
use crate::vm::csr;
use crate::vm::opcode;
use crate::vm::Trap;


//********************************************************************************
// Const shift
const MASK_4_0: u32   = const_u32_mask(4..0);
const MASK_6_0: u32   = const_u32_mask(6..0);
const MASK_7_7: u32   = const_u32_mask(7..7);
const MASK_11_7: u32  = const_u32_mask(11..7);
const MASK_11_8: u32  = const_u32_mask(11..8);
const MASK_14_12: u32 = const_u32_mask(14..12);
const MASK_19_12: u32 = const_u32_mask(19..12);
const MASK_19_15: u32 = const_u32_mask(19..15);
const MASK_20_20: u32 = const_u32_mask(20..20);
const MASK_24_20: u32 = const_u32_mask(24..20);
const MASK_30_21: u32 = const_u32_mask(30..21);
const MASK_30_25: u32 = const_u32_mask(30..25);
const MASK_31_11: u32 = const_u32_mask(31..11);
const MASK_31_12: u32 = const_u32_mask(31..12);
const MASK_31_20: u32 = const_u32_mask(31..20);
const MASK_31_25: u32 = const_u32_mask(31..25);
const MASK_31_31: u32 = const_u32_mask(31..31);

// M extensions
const MASK_31_0: u64  = const_u32_mask(31..0) as u64;
//********************************************************************************



pub struct Cpu {
    // TODO: make private when tests are broken up better
    pub reg: regfile::RegFile,
    pc: u32
}

impl Cpu {
    pub fn new(reg: regfile::RegFile, pc: u32) -> Cpu {
        Cpu {
            reg: reg,
            pc: pc
        }
    }

    pub fn set_pc(&mut self, pc: u32) {
        self.pc = pc;
    }

    pub fn step(&mut self, memory: &mut impl Mem, csrfile: &mut csr::Csr) -> Result<(), Trap> {
        let inst = fetch_instruction(&*memory, self.pc)?;

        // Decode opcode
        let opcode  = mask_and_shift(inst, MASK_6_0, 0);

        // Inst Type
        // TODO: change this over to generating the mask needed (for rspace issue #4)
        //let instType = opcode::instruction_type(opcode);

        // Prefetch the func3/7
        let func3   = mask_and_shift(inst, MASK_14_12, 12);
        let func7   = mask_and_shift(inst, MASK_31_25, 25);

        // Prefetch rd/rs1/rs2
        let rd      = mask_and_shift(inst, MASK_11_7, 7) as usize;
        let rs1     = mask_and_shift(inst, MASK_19_15, 15) as usize;
        let rs2     = mask_and_shift(inst, MASK_24_20, 20) as usize;

        // IMM types - Probably can be put in the asm steps
        let shamt   = mask_and_shift(inst, MASK_24_20, 20);
        // TODO: handle sign extend and so on as needed
        let i_imm   = mask_and_shift(inst, MASK_31_20, 20); // Some inst needs a non-sign extend?
        let s_imm   = (mask_and_shift(inst, MASK_31_25, 25) << 5)
                    | mask_and_shift(inst, MASK_11_7, 7);
        let sb_imm  = (mask_and_shift(inst, MASK_31_31, 31) << 12)
                    | (mask_and_shift(inst, MASK_7_7, 7) << 11)
                    | (mask_and_shift(inst, MASK_30_25, 25) << 5)
                    | (mask_and_shift(inst, MASK_11_8, 8) << 1);
        let u_imm   = mask_and_shift(inst, MASK_31_12, 12) << 12; // LUI doesn't sign extend?
        let uj_imm  = (mask_and_shift(inst, MASK_31_31, 31) << 20)
                    | (mask_and_shift(inst, MASK_19_12, 12) << 12)
                    | (mask_and_shift(inst, MASK_20_20, 20) << 11)
                    | (mask_and_shift(inst, MASK_30_21, 21) << 1);

        // CSR related
        // TODO: do we need to sign extend the csr_imm?
        let csr     = mask_and_shift(inst, MASK_31_20, 20) as usize; // functionally same as i_imm
        let csr_imm = mask_and_shift(inst, MASK_24_20, 20); // functionally same as rs2

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
                let shamt = mask_and_shift(self.reg[rs2], MASK_4_0, 0);
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
                let shamt = mask_and_shift(self.reg[rs2], MASK_4_0, 0);
                self.reg[rd] = self.reg[rs1] >> shamt;
            },
            (0b0100000, 0b101, opcode::OP_REG) => {
                // SRA
                let shamt = mask_and_shift(self.reg[rs2], MASK_4_0, 0);
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
                self.reg[rd] = (product & MASK_31_0) as u32;
            },
            (0b0000001, 0b001, opcode::OP_REG) => {
                // MULH
                let product: i64 = (sign_extend_32_to_64(self.reg[rs1]) as i64) * (sign_extend_32_to_64(self.reg[rs2]) as i64);
                self.reg[rd] = (((product >> 32) as u64) & MASK_31_0) as u32;
            },
            (0b0000001, 0b010, opcode::OP_REG) => {
                // MULHSU
                let product: i64 = (sign_extend_32_to_64(self.reg[rs1]) as i64) * (self.reg[rs2] as i64);
                self.reg[rd] = (((product >> 32) as u64) & MASK_31_0) as u32;
            },
            (0b0000001, 0b011, opcode::OP_REG) => {
                // MULHU
                let product: u64 = (self.reg[rs1] as u64) * (self.reg[rs2] as u64);
                self.reg[rd] = ((product >> 32) & MASK_31_0) as u32;
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
                self.reg[rd] = self.pc + 4;
                // Need to zero the last value
                //self.pc = (self.reg[rs1] + i_imm - 4) & 0xff_ff_ff_fe;
                // Because after this inst complete the pc will +4 at the end)
                self.pc = (self.reg[rs1].wrapping_add(i_imm)) & 0xff_ff_ff_fe;
            },

            // RV32 I
            (        _, 0b000, opcode::LOAD) => {
                // LB
                self.reg[rd] = sign_extend_8_to_32(
                    memory.load_byte(
                        self.reg[rs1].wrapping_add(sign_extend(inst, i_imm))
                    )?
                );
            },
            (        _, 0b001, opcode::LOAD) => {
                // LH
                self.reg[rd] = sign_extend_16_to_32(
                    memory.load_half(
                        self.reg[rs1].wrapping_add(sign_extend(inst, i_imm))
                    )?
                );
            },
            (        _, 0b010, opcode::LOAD) => {
                // LW
                self.reg[rd] = memory.load_word(
                    self.reg[rs1].wrapping_add(sign_extend(inst, i_imm))
                )?;
            },
            (        _, 0b100, opcode::LOAD) => {
                // LBU
                self.reg[rd] = memory.load_byte(
                    self.reg[rs1].wrapping_add(sign_extend(inst, i_imm))
                )?;
            },
            (        _, 0b101, opcode::LOAD) => {
                // LHU
                self.reg[rd] = memory.load_half(
                    self.reg[rs1].wrapping_add(sign_extend(inst, i_imm))
                )?;
            },

            // RV32 I
            (        _, 0b000, opcode::MISC_MEM) => {
                // FENCE
                // NOP instruction
            },

            // RV32 I
            // TODO: finish implementing these (particularly ECALL)
            // - Implement MRET for returning from trap
            // - Implement WFI - can make the emulator halt the cpu portion till a interrupt
            // fires, this depends on how we do it, if its essental, we can NOP instead -> busy
            // wait loop
            (        _, 0b000, opcode::SYSTEM) => {
                // ECALL | EBREAK
                let imm   = mask_and_shift(inst, MASK_31_20, 20);

                match imm {
                    0b000000000000 => {
                        // ECALL
                        // NOP instruction
                    },
                    0b000000000001 => {
                        // EBREAK
                        // NOP instruction
                    },
                    _ => return Err(Trap::IllegalInstruction(inst)),
                }
            },

            // RV32 Zicsr extensions
            // - if rd = x0 then csr does not read and cause read side effects (rw)
            // - if rs1 = x0 then csr does not write and cause write side effects (rs/c)
            // TODO:
            //   * such as raising illegal instruction exceptions on accesses to read-only CSRs.
            //
            //  inst    rd rs1
            // CSRRW    x0   - -> no-read, write
            // CSRRW   !x0   - -> read, write
            // CSRRS/C   -  x0 -> read, no-write
            // CSRRS/C   - !x0 -> read, write
            //
            // for Imm variant replace x0 with 0 and !0 for rs1
            (        _, 0b001, opcode::SYSTEM) => {
                // CSRRW
                if rd != 0 {
                    self.reg[rd] = csrfile.read(csr);
                }
                csrfile.write(csr, self.reg[rs1]);
            },
            (        _, 0b010, opcode::SYSTEM) => {
                // CSRRS
                self.reg[rd] = csrfile.read(csr);
                if rs1 != 0 {
                    csrfile.set(csr, self.reg[rs1]);
                }
            },
            (        _, 0b011, opcode::SYSTEM) => {
                // CSRRC
                self.reg[rd] = csrfile.read(csr);
                if rs1 != 0 {
                    csrfile.clear(csr, self.reg[rs1]);
                }
            },
            (        _, 0b101, opcode::SYSTEM) => {
                // CSRRWI
                self.reg[rd] = csrfile.read(csr);
                csrfile.write(csr, csr_imm);
            },
            (        _, 0b110, opcode::SYSTEM) => {
                // CSRRSI
                self.reg[rd] = csrfile.read(csr);
                if csr_imm != 0 {
                    csrfile.set(csr, csr_imm);
                }
            },
            (        _, 0b111, opcode::SYSTEM) => {
                // CSRRCI
                self.reg[rd] = csrfile.read(csr);
                if csr_imm != 0 {
                    csrfile.clear(csr, csr_imm);
                }
            },

            // RV32 I
            (        _, 0b000, opcode::STORE) => {
                // SB
                memory.store_byte(
                    self.reg[rs1].wrapping_add(sign_extend(inst, s_imm)),
                    self.reg[rs2],
                )?;
            },
            (        _, 0b001, opcode::STORE) => {
                // SH
                memory.store_half(
                    self.reg[rs1].wrapping_add(sign_extend(inst, s_imm)),
                    self.reg[rs2],
                )?;
            },
            (        _, 0b010, opcode::STORE) => {
                // SW
                memory.store_word(
                    self.reg[rs1].wrapping_add(sign_extend(inst, s_imm)),
                    self.reg[rs2],
                )?;
            },

            // RV32 I
            (        _, 0b000, opcode::BRANCH) => {
                // BEQ
                if self.reg[rs1] == self.reg[rs2] {
                    self.pc = sign_extend(inst, sb_imm).wrapping_add(self.pc);
                    //self.pc = self.pc - 4; // Because after this inst complete the pc will +4 at the end)
                } else {
                    self.pc += 4;
                }
            },
            (        _, 0b001, opcode::BRANCH) => {
                // BNE
                if self.reg[rs1] != self.reg[rs2] {
                    self.pc = sign_extend(inst, sb_imm).wrapping_add(self.pc);
                    //self.pc = self.pc - 4; // Because after this inst complete the pc will +4 at the end)
                } else {
                    self.pc += 4;
                }
            },
            (        _, 0b100, opcode::BRANCH) => {
                // BLT
                if (self.reg[rs1] as i32) < (self.reg[rs2] as i32) {
                    self.pc = sign_extend(inst, sb_imm).wrapping_add(self.pc);
                    //self.pc = self.pc - 4; // Because after this inst complete the pc will +4 at the end)
                } else {
                    self.pc += 4;
                }
            },
            (        _, 0b101, opcode::BRANCH) => {
                // BGE
                if (self.reg[rs1] as i32) >= (self.reg[rs2] as i32) {
                    self.pc = sign_extend(inst, sb_imm).wrapping_add(self.pc);
                    //self.pc = self.pc - 4; // Because after this inst complete the pc will +4 at the end)
                } else {
                    self.pc += 4;
                }
            },
            (        _, 0b110, opcode::BRANCH) => {
                // BLTU
                if self.reg[rs1] < self.reg[rs2] {
                    self.pc = sign_extend(inst, sb_imm).wrapping_add(self.pc);
                    //self.pc = self.pc - 4; // Because after this inst complete the pc will +4 at the end)
                } else {
                    self.pc += 4;
                }
            },
            (        _, 0b111, opcode::BRANCH) => {
                // BGEU
                if self.reg[rs1] >= self.reg[rs2] {
                    self.pc = sign_extend(inst, sb_imm).wrapping_add(self.pc);
                    //self.pc = self.pc - 4; // Because after this inst complete the pc will +4 at the end)
                } else {
                    self.pc += 4;
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
                //self.reg[rd] = u_imm.wrapping_add(self.pc);
                panic!("Auipc isn't really supported by the assembler yet");
            },
            (        _,     _, opcode::JAL) => {
                // JAL
                self.reg[rd] = self.pc + 4;

                self.pc = sign_extend(inst, uj_imm).wrapping_add(self.pc);
                //self.pc = self.pc - 4; // Because after this inst complete the pc will +4 at the end)
            },

            // TODO: handle instruction decoding failure
            (f7, f3, op) => {
                #[cfg(feature = "debug")]
                {
                    println!("FIX  PC: 0x{:04x} F7: {:07b} F3: {:03b} OP: {:07b}", self.pc, f7, f3, op);
                }

                //println!("ROM DUMP:");
                //for i in 0..(binary_u8.len()/4) {
                //    let rinst_u8: [u8; 4] = [rom[i*4], rom[i*4+1], rom[i*4+2], rom[i*4+3]];
                //    let rinst = unsafe { std::mem::transmute::<[u8; 4], u32>(rinst_u8) };

                //    println!("{:08x}", rinst);

                //    let rop = mask_and_shift(rinst, MASK_6_0, 0);
                //    let rfunc3 = mask_and_shift(rinst, MASK_14_12, 12);
                //    let rfunc7 = mask_and_shift(rinst, MASK_31_25, 25);
                //    println!("F7: {:07b} F3: {:03b} OP: {:07b}", rfunc7, rfunc3, rop);
                //}
                //break;
                return Err(Trap::IllegalInstruction(inst))
            },
        }

        #[cfg(feature = "debug")]
        {
            println!("FINE PC: 0x{:04x} F7: {:07b} F3: {:03b} OP: {:07b}", self.pc, func7, func3, opcode);
        }

        // TODO: this is a hack to handle Branch + JAL instruction, the branch will INC the PC if
        // they don't branch
        // TODO: need to verify if we want the self.pc to wrap around to 0 upon hitting upper
        // bounds
        match opcode {
            opcode::JAL     => (),
            opcode::JALR    => (),
            opcode::BRANCH  => (),
            _               => self.pc += 4,
        }

        // Everything's fine in this step
        Ok(())
    }
}


fn fetch_instruction(memory: &impl Mem, idx: u32) -> Result<u32, Trap> {
    // If inst is read from non u32 aligned address, error out (ISA specifies this)
    if idx % 4 != 0 {
        Err(Trap::UnalignedInstructionAccess(idx))
    } else {
        memory.load_word(idx).and_then(|x|
            // If inst is all 0 or all 1's error out (illegal instruction)
            if x == 0x0 {
                Err(Trap::IllegalInstruction(x))
            } else if x == 0xFF_FF_FF_FF {
                Err(Trap::IllegalInstruction(x))
            } else {
                Ok(x)
            }
        )
    }
}

// This block below is a hack around there being no const fn mask in twiddling
// and that's due to no const fn checked_shl
//********************************************************************************
// Hack around lack of const fn checked_shl
const fn const_checked_shl(lhs: u32, rhs: u32) -> u32 {
    // overflowing_shl is const fn
    let (a, b) = lhs.overflowing_shl(rhs);

    // This trick is slow but since we're const_fn this for const application of mask its fine
    // https://github.com/rust-lang/rust/issues/53718#issuecomment-500074555
    // False -> usize -> 0
    // True -> usize -> 1
    // As index we then return either shl or 0
    [a, 0][b as usize]
}

// const masks
const fn const_u32_mask(range: Range<usize>) -> u32 {
    // cshl is << but with overlong shifts resulting in 0
    let top = const_checked_shl(1, (1 + range.start - range.end) as u32);
    top.wrapping_sub(1) << range.end
}
//********************************************************************************


fn mask_and_shift(inst: u32, mask: u32, shift: u32) -> u32 {
    (inst & mask) >> shift
}


// TODO: better to move imm inst extraction here?
fn sign_extend(inst: u32, imm: u32) -> u32 {
    if (inst & 0x80_00_00_00) == 0x80_00_00_00 {
        let opcode = mask_and_shift(inst, MASK_6_0, 0);
        let mask = match opcode::instruction_type(opcode) {
            opcode::InstType::R  => 0x0,
            opcode::InstType::I  => MASK_31_11,
            opcode::InstType::S  => MASK_31_11,
            opcode::InstType::SB => MASK_31_12,
            opcode::InstType::U  => MASK_31_31,
            opcode::InstType::UJ => MASK_31_20,
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
