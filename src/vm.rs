use opcode;
use mem;
use regfile;
use asm;

use twiddle::Twiddle;

pub struct Emul32 {
    reg: regfile::RegFile,
    mem: mem::Memory,
    pc: usize
}

impl Emul32 {
    pub fn new_with_rom(rom: [u8; 4096]) -> Emul32 {
        Emul32 {
            reg: regfile::RegFile::new([0; 31]),
            mem: mem::Memory::new(rom, [0; 4096]),
            pc: 0
        }
    }

    pub fn new(reg: regfile::RegFile, mem: mem::Memory, pc: usize) -> Emul32 {
        Emul32 {
            reg: reg,
            mem: mem,
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
            let opcode = select_and_shift(inst, 6, 0);

            // Inst Type
            // TODO: change this over to generating the mask needed (for rspace issue #4)
            //let instType = opcode::instruction_type(opcode);

            // Prefetch the func3/7
            let func3  = select_and_shift(inst, 14, 12);
            let func7  = select_and_shift(inst, 31, 25);

            // Prefetch rd/rs1/rs2
            let rd     = select_and_shift(inst, 11, 7) as usize;
            let rs1    = select_and_shift(inst, 19, 15) as usize;
            let rs2    = select_and_shift(inst, 24, 20) as usize;

            // IMM types - Probably can be put in the asm steps
            let shamt  = select_and_shift(inst, 24, 20);
            // TODO: handle sign extend and so on as needed
            let i_imm  = select_and_shift(inst, 31, 20);
            let s_imm  = (select_and_shift(inst, 31, 25) << 5)
                       | select_and_shift(inst, 11, 7);
            let sb_imm = (select_and_shift(inst, 31, 31) << 12)
                       | (select_and_shift(inst, 7, 7) << 11)
                       | (select_and_shift(inst, 30, 25) << 5)
                       | (select_and_shift(inst, 11, 8) << 1);
            let u_imm  = select_and_shift(inst, 31, 12) << 12;
            let uj_imm = (select_and_shift(inst, 31, 31) << 20)
                       | (select_and_shift(inst, 19, 12) << 12)
                       | (select_and_shift(inst, 20, 20) << 11)
                       | (select_and_shift(inst, 30, 21) << 1);

            match (func7, func3, opcode) {
                // RV32 I
                (0b0000000, 0b000, opcode::OP_REG) => {
                    // ADD
                    self.reg[rd] = self.reg[rs1].wrapping_add(self.reg[rs2]);
                },
                (0b0100000, 0b000, opcode::OP_REG) => {
                    // SUB
                    self.reg[rd] = self.reg[rs1] - self.reg[rs2];
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
                },
                (0b0000001, 0b001, opcode::OP_REG) => {
                    // MULH
                },
                (0b0000001, 0b010, opcode::OP_REG) => {
                    // MULHSU
                },
                (0b0000001, 0b011, opcode::OP_REG) => {
                    // MULHU
                },
                (0b0000001, 0b100, opcode::OP_REG) => {
                    // DIV
                },
                (0b0000001, 0b101, opcode::OP_REG) => {
                    // DIVU
                },
                (0b0000001, 0b110, opcode::OP_REG) => {
                    // REM
                },
                (0b0000001, 0b111, opcode::OP_REG) => {
                    // REMU
                },

                // RV32 I
                (        _, 0b000, opcode::OP_IMM) => {
                    // ADDI
                    // TODO: add as signed?
                    self.reg[rd] = self.reg[rs1] + sign_extend(inst, i_imm);
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
                    // TODO: unclear if we need to execute the jumped to instruction or +4?
                    //self.pc = (self.reg[rs1] + i_imm) as usize;
                    //self.reg[rd] = (self.pc + 4) as u32;
                },

                // RV32 I
                (        _, 0b000, opcode::LOAD) => {
                    // LB
                    // TODO: abstract this to memory?
                    let byte = self.mem[(self.reg[rs1] + sign_extend(inst, i_imm)) as usize];
                    self.reg[rd] = sign_extend_8_to_32(byte as u32);
                },
                (        _, 0b001, opcode::LOAD) => {
                    // LH
                    // TODO: abstract this to memory?
                    let bytes: [u8; 2] = [
                        self.mem[(self.reg[rs1] + sign_extend(inst, i_imm)) as usize],
                        self.mem[(self.reg[rs1] + sign_extend(inst, i_imm) + 1) as usize]
                    ];
                    self.reg[rd] = sign_extend_16_to_32((bytes[0] as u32) | ((bytes[1] as u32) << 8));
                },
                (        _, 0b010, opcode::LOAD) => {
                    // LW
                    // TODO: abstract this to memory?
                    let bytes: [u8; 4] = [
                        self.mem[(self.reg[rs1] + sign_extend(inst, i_imm)) as usize],
                        self.mem[(self.reg[rs1] + sign_extend(inst, i_imm) + 1) as usize],
                        self.mem[(self.reg[rs1] + sign_extend(inst, i_imm) + 2) as usize],
                        self.mem[(self.reg[rs1] + sign_extend(inst, i_imm) + 3) as usize]
                    ];
                    self.reg[rd] = ((bytes[0] as u32)) | ((bytes[1] as u32) << 8) | ((bytes[2] as u32) << 16) | ((bytes[3] as u32) << 24);
                },
                (        _, 0b100, opcode::LOAD) => {
                    // LBU
                    // TODO: abstract this to memory?
                    let byte = self.mem[(self.reg[rs1] + sign_extend(inst, i_imm)) as usize];
                    self.reg[rd] = byte as u32;
                },
                (        _, 0b101, opcode::LOAD) => {
                    // LHU
                    // TODO: abstract this to memory?
                    let bytes: [u8; 2] = [
                        self.mem[(self.reg[rs1] + sign_extend(inst, i_imm)) as usize],
                        self.mem[(self.reg[rs1] + sign_extend(inst, i_imm) + 1) as usize]
                    ];
                    self.reg[rd] = (bytes[0] as u32) | ((bytes[1] as u32) << 8);
                },

                // RV32 I
                (        _, 0b000, opcode::MISC_MEM) => {
                    // FENCE
                    // NOP instruction
                },
                (        _, 0b001, opcode::MISC_MEM) => {
                    // FENCE.I
                    // NOP instruction
                },

                // RV32 I
                (        _, 0b000, opcode::SYSTEM) => {
                    // ECALL | EBREAK
                    let imm   = select_and_shift(inst, 31, 20);

                    match imm {
                        0b000000000000 => {
                            // ECALL
                        },
                        0b000000000001 => {
                            // EBREAK
                        },
                        _ => panic!("FIXME"),
                    }
                },
                (        _, 0b001, opcode::SYSTEM) => {
                    // CSRRW
                },
                (        _, 0b010, opcode::SYSTEM) => {
                    // CSRRS
                },
                (        _, 0b011, opcode::SYSTEM) => {
                    // CSRRC
                },
                (        _, 0b101, opcode::SYSTEM) => {
                    // CSRRWI
                },
                (        _, 0b110, opcode::SYSTEM) => {
                    // CSRRSI
                },
                (        _, 0b111, opcode::SYSTEM) => {
                    // CSRRCI
                },

                // RV32 I
                (        _, 0b000, opcode::STORE) => {
                    // SB
                    // TODO: abstract this to memory?
                    self.mem[(self.reg[rs1] + sign_extend(inst, s_imm)) as usize] = (self.reg[rs2] & 0x00_00_00_FF) as u8;
                },
                (        _, 0b001, opcode::STORE) => {
                    // SH
                    // TODO: abstract this to memory?
                    self.mem[(self.reg[rs1] + sign_extend(inst, s_imm)) as usize]     = (self.reg[rs2] & 0x00_00_00_FF) as u8;
                    self.mem[(self.reg[rs1] + sign_extend(inst, s_imm) + 1) as usize] = (self.reg[rs2] & 0x00_00_FF_00) as u8;
                },
                (        _, 0b010, opcode::STORE) => {
                    // SW
                    // TODO: abstract this to memory?
                    self.mem[(self.reg[rs1] + sign_extend(inst, s_imm)) as usize]     = (self.reg[rs2] & 0x00_00_00_FF) as u8;
                    self.mem[(self.reg[rs1] + sign_extend(inst, s_imm) + 1) as usize] = (self.reg[rs2] & 0x00_00_FF_00) as u8;
                    self.mem[(self.reg[rs1] + sign_extend(inst, s_imm) + 2) as usize] = (self.reg[rs2] & 0x00_FF_00_00) as u8;
                    self.mem[(self.reg[rs1] + sign_extend(inst, s_imm) + 3) as usize] = (self.reg[rs2] & 0xFF_00_00_00) as u8;
                },

                // RV32 I
                (        _, 0b000, opcode::BRANCH) => {
                    // BEQ
                    // TODO: unclear if we need to execute the jumped to instruction or +4?
                    //if self.reg[rs1] == self.reg[rs2] {
                    //    self.pc += (sign_extend(inst, sb_imm) as usize);
                    //}
                },
                (        _, 0b001, opcode::BRANCH) => {
                    // BNE
                    // TODO: unclear if we need to execute the jumped to instruction or +4?
                    //if self.reg[rs1] != self.reg[rs2] {
                    //    self.pc += (sign_extend(inst, sb_imm) as usize);
                    //}
                },
                (        _, 0b100, opcode::BRANCH) => {
                    // BLT
                    // TODO: unclear if we need to execute the jumped to instruction or +4?
                    //if (self.reg[rs1] as i32) < (self.reg[rs2] as i32) {
                    //    self.pc += (sign_extend(inst, sb_imm) as usize);
                    //}
                },
                (        _, 0b101, opcode::BRANCH) => {
                    // BGE
                    // TODO: unclear if we need to execute the jumped to instruction or +4?
                    //if (self.reg[rs1] as i32) >= (self.reg[rs2] as i32) {
                    //    self.pc += (sign_extend(inst, sb_imm) as usize);
                    //}
                },
                (        _, 0b110, opcode::BRANCH) => {
                    // BLTU
                    // TODO: unclear if we need to execute the jumped to instruction or +4?
                    //if self.reg[rs1] < self.reg[rs2] {
                    //    self.pc += (sign_extend(inst, sb_imm) as usize);
                    //}
                },
                (        _, 0b111, opcode::BRANCH) => {
                    // BGEU
                    // TODO: unclear if we need to execute the jumped to instruction or +4?
                    //if self.reg[rs1] >= self.reg[rs2] {
                    //    self.pc += (sign_extend(inst, sb_imm) as usize);
                    //}
                },

                // RV32 I
                (        _,     _, opcode::LUI) => {
                    // LUI
                    self.reg[rd] = u_imm;
                },
                (        _,     _, opcode::AUIPC) => {
                    // AUIPC
                    self.reg[rd] = u_imm + (self.pc as u32);
                },
                (        _,     _, opcode::JAL) => {
                    // JAL
                    // TODO: unclear if we need to execute the jumped to instruction or +4?
                    //self.pc += sign_extend(inst, uj_imm) as usize;
                    //self.reg[rd] = (self.pc + 4) as u32;
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
                    panic!("FIXME")
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


#[cfg(test)]
mod rr_op_tests {
    use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt, ByteOrder};
    use super::*;

    #[test]
    fn add_inst() {
      // Arithmetic tests
      TEST_RR_OP(2,  "add", 0x00000000, 0x00000000, 0x00000000);
      TEST_RR_OP(3,  "add", 0x00000002, 0x00000001, 0x00000001);
      TEST_RR_OP(4,  "add", 0x0000000a, 0x00000003, 0x00000007);

      TEST_RR_OP(5,  "add", 0xffff8000, 0x00000000, 0xffff8000);
      TEST_RR_OP(6,  "add", 0x80000000, 0x80000000, 0x00000000);
      TEST_RR_OP(7,  "add", 0x7fff8000, 0x80000000, 0xffff8000);

      TEST_RR_OP(8,  "add", 0x00007fff, 0x00000000, 0x00007fff);
      TEST_RR_OP(9,  "add", 0x7fffffff, 0x7fffffff, 0x00000000);
      TEST_RR_OP(10, "add", 0x80007ffe, 0x7fffffff, 0x00007fff);

      TEST_RR_OP(11, "add", 0x80007fff, 0x80000000, 0x00007fff);
      TEST_RR_OP(12, "add", 0x7fff7fff, 0x7fffffff, 0xffff8000);

      TEST_RR_OP(13, "add", 0xffffffff, 0x00000000, 0xffffffff);
      TEST_RR_OP(14, "add", 0x00000000, 0xffffffff, 0x00000001);
      TEST_RR_OP(15, "add", 0xfffffffe, 0xffffffff, 0xffffffff);

      TEST_RR_OP(16, "add", 0x80000000, 0x00000001, 0x7fffffff);

      // Source/Destination tests
      TEST_RR_SRC1_EQ_DEST(17, "add", 24, 13, 11);
      TEST_RR_SRC2_EQ_DEST(18, "add", 25, 14, 11);
      TEST_RR_SRC12_EQ_DEST(19, "add", 26, 13);

      // Bypassing tests
      TEST_RR_DEST_BYPASS(20, 0, "add", 24, 13, 11);
      TEST_RR_DEST_BYPASS(21, 1, "add", 25, 14, 11);
      TEST_RR_DEST_BYPASS(22, 2, "add", 26, 15, 11);

      TEST_RR_SRC12_BYPASS(23, 0, 0, "add", 24, 13, 11);
      TEST_RR_SRC12_BYPASS(24, 0, 1, "add", 25, 14, 11);
      TEST_RR_SRC12_BYPASS(25, 0, 2, "add", 26, 15, 11);
      TEST_RR_SRC12_BYPASS(26, 1, 0, "add", 24, 13, 11);
      TEST_RR_SRC12_BYPASS(27, 1, 1, "add", 25, 14, 11);
      TEST_RR_SRC12_BYPASS(28, 2, 0, "add", 26, 15, 11);

      TEST_RR_SRC21_BYPASS(29, 0, 0, "add", 24, 13, 11);
      TEST_RR_SRC21_BYPASS(30, 0, 1, "add", 25, 14, 11);
      TEST_RR_SRC21_BYPASS(31, 0, 2, "add", 26, 15, 11);
      TEST_RR_SRC21_BYPASS(32, 1, 0, "add", 24, 13, 11);
      TEST_RR_SRC21_BYPASS(33, 1, 1, "add", 25, 14, 11);
      TEST_RR_SRC21_BYPASS(34, 2, 0, "add", 26, 15, 11);

      TEST_RR_ZEROSRC1(35, "add", 15, 15);
      TEST_RR_ZEROSRC2(36, "add", 32, 32);
      TEST_RR_ZEROSRC12(37, "add", 0);
      TEST_RR_ZERODEST(38, "add", 16, 30);
    }


    // TODO: Put in some sort of generic test suite utilities
    fn generate_rom(opcodes: &str) -> [u8; 4096] {
        let mut rom: [u8; 4096] = [0; 4096];
        let asm_u8 = {
            let asm = asm::parse_asm(opcodes);
            let mut wtr = vec![];

            for i in 0..asm.len() {
                wtr.write_u32::<LittleEndian>(asm[i]);
            }
            wtr
        };

        for i in 0..asm_u8.len() {
            rom[i] = asm_u8[i];
        }
        rom
    }

    // TODO: make this more flexible (ie list of reg + value, plus expected value+reg afterward)
    fn TEST_RR_OP(test: u8, op: &str, r: u32, a: u32, b: u32) {
        // load the rom
        let mut vm = Emul32::new_with_rom(generate_rom(&format!("{} x3 x1 x2", op)));

        // Load the registers
        vm.reg[1] = a;
        vm.reg[2] = b;

        // Run
        vm.run();

        // Validate
        assert_eq!(vm.reg[1], a);
        assert_eq!(vm.reg[2], b);
        assert_eq!(vm.reg[3], r);
    }

    fn TEST_RR_SRC1_EQ_DEST(test: u8, op: &str, res: u32, a: u32, b: u32) {
        // load the rom
        let mut vm = Emul32::new_with_rom(generate_rom(&format!("{} x1 x1 x2", op)));

        // Load the registers
        vm.reg[1] = a;
        vm.reg[2] = b;

        // Run
        vm.run();

        // Validate
        assert_eq!(vm.reg[1], res);
        assert_eq!(vm.reg[2], b);
    }

    fn TEST_RR_SRC2_EQ_DEST(test: u8, op: &str, res: u32, a: u32, b: u32) {
        // load the rom
        let mut vm = Emul32::new_with_rom(generate_rom(&format!("{} x2 x1 x2", op)));

        // Load the registers
        vm.reg[1] = a;
        vm.reg[2] = b;

        // Run
        vm.run();

        // Validate
        assert_eq!(vm.reg[1], a);
        assert_eq!(vm.reg[2], res);
    }

    fn TEST_RR_SRC12_EQ_DEST(test: u8, op: &str, res: u32, a: u32) {
        // load the rom
        let mut vm = Emul32::new_with_rom(generate_rom(&format!("{} x1 x1 x1", op)));

        // Load the registers
        vm.reg[1] = a;

        // Run
        vm.run();

        // Validate
        assert_eq!(vm.reg[1], res);
    }

    fn TEST_RR_ZEROSRC1(test: u8, op: &str, r: u32, b: u32) {
        // load the rom
        let mut vm = Emul32::new_with_rom(generate_rom(&format!("{} x1 x0 x2", op)));

        // Load the registers
        vm.reg[2] = b;

        // Run
        vm.run();

        // Validate
        assert_eq!(vm.reg[0], 0);
        assert_eq!(vm.reg[1], r);
        assert_eq!(vm.reg[2], b);
    }

    fn TEST_RR_ZEROSRC2(test: u8, op: &str, r: u32, a: u32) {
        // load the rom
        let mut vm = Emul32::new_with_rom(generate_rom(&format!("{} x1 x2 x0", op)));

        // Load the registers
        vm.reg[2] = a;

        // Run
        vm.run();

        // Validate
        assert_eq!(vm.reg[0], 0);
        assert_eq!(vm.reg[1], r);
        assert_eq!(vm.reg[2], a);
    }

    fn TEST_RR_ZEROSRC12(test: u8, op: &str, r: u32) {
        // load the rom
        let mut vm = Emul32::new_with_rom(generate_rom(&format!("{} x1 x0 x0", op)));

        // Run
        vm.run();

        // Validate
        assert_eq!(vm.reg[0], 0);
        assert_eq!(vm.reg[1], r);
    }

    fn TEST_RR_ZERODEST(test: u8, op: &str, a: u32, b: u32) {
        // load the rom
        let mut vm = Emul32::new_with_rom(generate_rom(&format!("{} x0 x1 x2", op)));

        // Load the registers
        vm.reg[1] = a;
        vm.reg[2] = b;

        // Run
        vm.run();

        // Validate
        assert_eq!(vm.reg[0], 0);
        assert_eq!(vm.reg[1], a);
        assert_eq!(vm.reg[2], b);
    }

    fn TEST_RR_DEST_BYPASS(test: u8, n: u32, op: &str, res: u32, a: u32, b: u32) {
        TEST_RR_OP(test, op, res, a, b);
    }

    fn TEST_RR_SRC12_BYPASS(test: u8, n1: u32, n2: u32, op: &str, res: u32, a: u32, b: u32) {
        TEST_RR_OP(test, op, res, a, b);
    }

    fn TEST_RR_SRC21_BYPASS(test: u8, n1: u32, n2: u32, op: &str, res: u32, a: u32, b: u32) {
        TEST_RR_OP(test, op, res, a, b);
    }
}
