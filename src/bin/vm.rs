extern crate rspace;
extern crate byteorder;
extern crate twiddle;

use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt, ByteOrder};
use std::fs::File;

use rspace::types;
use twiddle::Twiddle;

use std::ops::Index;
use std::ops::IndexMut;


fn main() {
    // Test asm code
    let test_asm = r#"
        addi x1 x0 0xF
        slti x2 x0 0xA
        sltiu x3 x0 0x9

        andi x1 x0 0x0
        ori x2 x0 0xFF
        xori x3 x0 0x00FF

        // TODO: shamt
        slli x1 x1 0x0
        srli x2 x2 0x1
        srai x3 x3 0x6

        lui x1 0x3412
        auipc x2 0x31241

        add x1 x3 x2
        slt x2 x2 x2
        sltu x3 x1 x2

        and x1 x3 x2
        or x2 x2 x2
        xor x3 x1 x2

        sll x1 x3 x2
        srl x3 x1 x2

        sub x1 x3 x2
        sra x3 x1 x2

        // TODO: drop cos elf assemblier doesn't output these
        // and j is just jal with x0 assumed
        jal x0 0xFFF
        // there isn't actually a ret instruction, it's a synonym for jalr x0, 0(x1)
        jalr x0 x1 0x0

        //beq x1 x6 0x1
        bne x1 x6 0x8
        //bne x2 x5 0x2
        beq x2 x5 0x8
        //blt x3 x4 0x3
        bge x3 x4 0x8
        //bltu x4 x3 0x4
        bgeu x4 x3 0x8
        //bge x5 x2 0x5
        blt x5 x2 0x8
        //bgeu x6 x1 0x6
        bltu x6 x1 0x8

        lw x1 x0 0x1
        lh x2 x0 0x2
        lhu x3 x0 0x3
        lb x4 x0 0x4
        lbu x5 x0 0x5

        // TODO: the args are swapped
        //sw x0 x2 0x1
        sw x2 x0 0x1
        //sh x0 x2 0x2
        sh x2 x0 0x2
        //sb x0 x3 0x3
        sb x3 x0 0x3

        // TODO: custom bitfield (but its nop in the vm tho)
        fence
        fence.i

        // TODO: custom layout (imm/registers/etc)
        // TODO: CSR - CYCLE[H], TIME[H], INSTRET[H]
        csrrw x1 x0 CYCLE
        csrrs x2 x0 TIMEH
        csrrc x3 x0 INSTRET
        csrrwi x4 0x1 CYCLE
        csrrsi x5 0x2 TIME
        csrrci x6 0x3 INSTRETH

        ecall
        ebreak

        mul x0 x1 x2
        mulh x1 x2 x0
        mulhu x2 x0 x1
        mulhsu x0 x1 x2

        div x1 x2 x0
        divu x2 x0 x1

        rem x0 x1 x2
        remu x1 x2 x0
    "#;

    let binary_code = rspace::asm::parse_asm(test_asm);
    //compare_assembly(binary_code, test_asm);
    let binary_u8 = {
        let mut wtr = vec![];

        for i in 0..binary_code.len() {
            wtr.write_u32::<LittleEndian>(binary_code[i]);
        }
        wtr
    };

    // Virtual machine stuff

    // Registers
    let mut reg = RegFile::new([0; 31]);

    // TODO: shouldn't this be u32?
    let mut pc: usize = 0;

    // Rom (would be nice to make this consistent sized)
    let rom = {
        let mut rom: [u8; 4096] = [0; 4096];

        // TODO: verify
        // Instructions are stored in memory with each 16-bit parcel stored in a memory halfword
        // according to the implementation’s natural endianness. Parcels forming one instruction
        // are stored at increasing halfword addresses, with the lowest addressed parcel holding
        // the lowest numbered bits in the instruction specification, i.e., instructions are always
        // stored in a little-endian sequence of parcels regardless of the memory system
        // endianness.
        for i in 0..binary_u8.len() {
            rom[i] = binary_u8[i];
        }
        rom
    };

    // Memory
    let mut mem = Memory::new(rom, [0; 4096]);

    // VM loop
    loop {
        // TODO: unitify memory at some point
        // TODO: deal with u32 access for inst
        // TODO: if inst is read from a non u32 aligned address, error out (ISA specifies this)
        // TODO: instruction that is all zero or all ones is an illegal instruction (trap)
        let inst = mem.fetch_instruction(pc);

        // Decode opcode
        let opcode = select_and_shift(inst, 6, 0);

        // Inst Type
        // TODO: change this over to generating the mask needed (for rspace issue #4)
        //let instType = rspace::opcode::instruction_type(opcode);

        // Prefetch the func3/7
        let func3 = select_and_shift(inst, 14, 12);
        let func7 = select_and_shift(inst, 31, 25);

        // Prefetch rd/rs1/rs2
        let rd    = select_and_shift(inst, 11, 7) as usize;
        let rs1   = select_and_shift(inst, 19, 15) as usize;
        let rs2   = select_and_shift(inst, 24, 20) as usize;

        // IMM types - Probably can be put in the asm steps
        let shamt = select_and_shift(inst, 24, 20);
        // TODO: handle sign extend and so on as needed
        let Iimm  = select_and_shift(inst, 31, 20);
        let Simm  = (select_and_shift(inst, 31, 25) << 5)
                  | select_and_shift(inst, 11, 7);
        let SBimm = (select_and_shift(inst, 31, 31) << 12)
                  | (select_and_shift(inst, 7, 7) << 11)
                  | (select_and_shift(inst, 30, 25) << 5)
                  | (select_and_shift(inst, 11, 8) << 1);
        let Uimm  = (select_and_shift(inst, 31, 12) << 12);
        let UJimm = (select_and_shift(inst, 31, 31) << 20)
                  | (select_and_shift(inst, 19, 12) << 12)
                  | (select_and_shift(inst, 20, 20) << 11)
                  | (select_and_shift(inst, 30, 21) << 1);

        match (func7, func3, opcode) {
            // RV32 I
            (0b0000000, 0b000, rspace::opcode::OP_REG) => {
                // ADD
                reg[rd] = reg[rs1] + reg[rs2];
            },
            (0b0100000, 0b000, rspace::opcode::OP_REG) => {
                // SUB
                reg[rd] = reg[rs1] - reg[rs2];
            },
            (0b0000000, 0b001, rspace::opcode::OP_REG) => {
                // SLL
                let shamt = select_and_shift(reg[rs2], 4, 0);
                reg[rd] = reg[rs1] << shamt;
            },
            (0b0000000, 0b010, rspace::opcode::OP_REG) => {
                // SLT
                reg[rd] = if (reg[rs1] as i32) < (reg[rs2] as i32) { 0x1 } else { 0x0 }
            },
            (0b0000000, 0b011, rspace::opcode::OP_REG) => {
                // SLTU
                reg[rd] = if reg[rs1] < reg[rs2] { 0x1 } else { 0x0 }
            },
            (0b0000000, 0b100, rspace::opcode::OP_REG) => {
                // XOR
                reg[rd] = reg[rs1] ^ reg[rs2];
            },
            (0b0000000, 0b101, rspace::opcode::OP_REG) => {
                // SRL
                let shamt = select_and_shift(reg[rs2], 4, 0);
                reg[rd] = reg[rs1] >> shamt;
            },
            (0b0100000, 0b101, rspace::opcode::OP_REG) => {
                // SRA
                let shamt = select_and_shift(reg[rs2], 4, 0);
                // apparently arithmetic right shift depends on type of left operator
                reg[rd] = ((reg[rs1] as i32) >> shamt) as u32;
            },
            (0b0000000, 0b110, rspace::opcode::OP_REG) => {
                // OR
                reg[rd] = reg[rs1] | reg[rs2];
            },
            (0b0000000, 0b111, rspace::opcode::OP_REG) => {
                // AND
                reg[rd] = reg[rs1] & reg[rs2];
            },

            // RV32 M extensions
            (0b0000001, 0b000, rspace::opcode::OP_REG) => {
                // MUL
            },
            (0b0000001, 0b001, rspace::opcode::OP_REG) => {
                // MULH
            },
            (0b0000001, 0b010, rspace::opcode::OP_REG) => {
                // MULHSU
            },
            (0b0000001, 0b011, rspace::opcode::OP_REG) => {
                // MULHU
            },
            (0b0000001, 0b100, rspace::opcode::OP_REG) => {
                // DIV
            },
            (0b0000001, 0b101, rspace::opcode::OP_REG) => {
                // DIVU
            },
            (0b0000001, 0b110, rspace::opcode::OP_REG) => {
                // REM
            },
            (0b0000001, 0b111, rspace::opcode::OP_REG) => {
                // REMU
            },

            // RV32 I
            (        _, 0b000, rspace::opcode::OP_IMM) => {
                // ADDI
                // TODO: add as signed?
                reg[rd] = reg[rs1] + sign_extend(inst, Iimm);
            },
            (0b0000000, 0b001, rspace::opcode::OP_IMM) => {
                // SLLI
                reg[rd] = reg[rs1] << shamt;
            },
            (        _, 0b010, rspace::opcode::OP_IMM) => {
                // SLTI
                reg[rd] = if (reg[rs1] as i32) < (sign_extend(inst, Iimm) as i32) { 0x1 } else { 0x0 }
            },
            (        _, 0b011, rspace::opcode::OP_IMM) => {
                // SLTIU
                reg[rd] = if reg[rs1] < sign_extend(inst, Iimm) { 0x1 } else { 0x0 }
            },
            (        _, 0b100, rspace::opcode::OP_IMM) => {
                // XORI
                reg[rd] = reg[rs1] ^ sign_extend(inst, Iimm);
            },
            (0b0000000, 0b101, rspace::opcode::OP_IMM) => {
                // SRLI
                reg[rd] = reg[rs1] >> shamt;
            },
            (0b0100000, 0b101, rspace::opcode::OP_IMM) => {
                // SRAI
                // apparently arithmetic right shift depends on type of left operator
                reg[rd] = ((reg[rs1] as i32) >> shamt) as u32;
            },
            (        _, 0b110, rspace::opcode::OP_IMM) => {
                // ORI
                reg[rd] = reg[rs1] | sign_extend(inst, Iimm);
            },
            (        _, 0b111, rspace::opcode::OP_IMM) => {
                // ANDI
                reg[rd] = reg[rs1] & sign_extend(inst, Iimm);
            },

            // RV32 I
            (        _, 0b000, rspace::opcode::JALR) => {
                // JALR
                // TODO: unclear if we need to execute the jumped to instruction or +4?
                //pc = (reg[rs1] + Iimm) as usize;
                //reg[rd] = (pc + 4) as u32;
            },

            // RV32 I
            (        _, 0b000, rspace::opcode::LOAD) => {
                // LB
                // TODO: abstract this to memory?
                let byte = mem[(reg[rs1] + sign_extend(inst, Iimm)) as usize];
                reg[rd] = sign_extend_8_to_32(byte as u32);
            },
            (        _, 0b001, rspace::opcode::LOAD) => {
                // LH
                // TODO: abstract this to memory?
                let bytes: [u8; 2] = [
                    mem[(reg[rs1] + sign_extend(inst, Iimm)) as usize],
                    mem[(reg[rs1] + sign_extend(inst, Iimm) + 1) as usize]
                ];
                reg[rd] = sign_extend_16_to_32((bytes[0] as u32) | ((bytes[1] as u32) << 8));
            },
            (        _, 0b010, rspace::opcode::LOAD) => {
                // LW
                // TODO: abstract this to memory?
                let bytes: [u8; 4] = [
                    mem[(reg[rs1] + sign_extend(inst, Iimm)) as usize],
                    mem[(reg[rs1] + sign_extend(inst, Iimm) + 1) as usize],
                    mem[(reg[rs1] + sign_extend(inst, Iimm) + 2) as usize],
                    mem[(reg[rs1] + sign_extend(inst, Iimm) + 3) as usize]
                ];
                reg[rd] = ((bytes[0] as u32)) | ((bytes[1] as u32) << 8) | ((bytes[2] as u32) << 16) | ((bytes[3] as u32) << 24);
            },
            (        _, 0b100, rspace::opcode::LOAD) => {
                // LBU
                // TODO: abstract this to memory?
                let byte = mem[(reg[rs1] + sign_extend(inst, Iimm)) as usize];
                reg[rd] = (byte as u32);
            },
            (        _, 0b101, rspace::opcode::LOAD) => {
                // LHU
                // TODO: abstract this to memory?
                let bytes: [u8; 2] = [
                    mem[(reg[rs1] + sign_extend(inst, Iimm)) as usize],
                    mem[(reg[rs1] + sign_extend(inst, Iimm) + 1) as usize]
                ];
                reg[rd] = (bytes[0] as u32) | ((bytes[1] as u32) << 8);
            },

            // RV32 I
            (        _, 0b000, rspace::opcode::MISC_MEM) => {
                // FENCE
                // NOP instruction
            },
            (        _, 0b001, rspace::opcode::MISC_MEM) => {
                // FENCE.I
                // NOP instruction
            },

            // RV32 I
            (        _, 0b000, rspace::opcode::SYSTEM) => {
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
            (        _, 0b001, rspace::opcode::SYSTEM) => {
                // CSRRW
            },
            (        _, 0b010, rspace::opcode::SYSTEM) => {
                // CSRRS
            },
            (        _, 0b011, rspace::opcode::SYSTEM) => {
                // CSRRC
            },
            (        _, 0b101, rspace::opcode::SYSTEM) => {
                // CSRRWI
            },
            (        _, 0b110, rspace::opcode::SYSTEM) => {
                // CSRRSI
            },
            (        _, 0b111, rspace::opcode::SYSTEM) => {
                // CSRRCI
            },

            // RV32 I
            (        _, 0b000, rspace::opcode::STORE) => {
                // SB
                // TODO: abstract this to memory?
                mem[(reg[rs1] + sign_extend(inst, Simm)) as usize] = (reg[rs2] & 0x00_00_00_FF) as u8;
            },
            (        _, 0b001, rspace::opcode::STORE) => {
                // SH
                // TODO: abstract this to memory?
                mem[(reg[rs1] + sign_extend(inst, Simm)) as usize]     = (reg[rs2] & 0x00_00_00_FF) as u8;
                mem[(reg[rs1] + sign_extend(inst, Simm) + 1) as usize] = (reg[rs2] & 0x00_00_FF_00) as u8;
            },
            (        _, 0b010, rspace::opcode::STORE) => {
                // SW
                // TODO: abstract this to memory?
                mem[(reg[rs1] + sign_extend(inst, Simm)) as usize]     = (reg[rs2] & 0x00_00_00_FF) as u8;
                mem[(reg[rs1] + sign_extend(inst, Simm) + 1) as usize] = (reg[rs2] & 0x00_00_FF_00) as u8;
                mem[(reg[rs1] + sign_extend(inst, Simm) + 2) as usize] = (reg[rs2] & 0x00_FF_00_00) as u8;
                mem[(reg[rs1] + sign_extend(inst, Simm) + 3) as usize] = (reg[rs2] & 0xFF_00_00_00) as u8;
            },

            // RV32 I
            (        _, 0b000, rspace::opcode::BRANCH) => {
                // BEQ
                // TODO: unclear if we need to execute the jumped to instruction or +4?
                //if reg[rs1] == reg[rs2] {
                //    pc += (sign_extend(inst, SBimm) as usize);
                //}
            },
            (        _, 0b001, rspace::opcode::BRANCH) => {
                // BNE
                // TODO: unclear if we need to execute the jumped to instruction or +4?
                //if reg[rs1] != reg[rs2] {
                //    pc += (sign_extend(inst, SBimm) as usize);
                //}
            },
            (        _, 0b100, rspace::opcode::BRANCH) => {
                // BLT
                // TODO: unclear if we need to execute the jumped to instruction or +4?
                //if (reg[rs1] as i32) < (reg[rs2] as i32) {
                //    pc += (sign_extend(inst, SBimm) as usize);
                //}
            },
            (        _, 0b101, rspace::opcode::BRANCH) => {
                // BGE
                // TODO: unclear if we need to execute the jumped to instruction or +4?
                //if (reg[rs1] as i32) >= (reg[rs2] as i32) {
                //    pc += (sign_extend(inst, SBimm) as usize);
                //}
            },
            (        _, 0b110, rspace::opcode::BRANCH) => {
                // BLTU
                // TODO: unclear if we need to execute the jumped to instruction or +4?
                //if reg[rs1] < reg[rs2] {
                //    pc += (sign_extend(inst, SBimm) as usize);
                //}
            },
            (        _, 0b111, rspace::opcode::BRANCH) => {
                // BGEU
                // TODO: unclear if we need to execute the jumped to instruction or +4?
                //if reg[rs1] >= reg[rs2] {
                //    pc += (sign_extend(inst, SBimm) as usize);
                //}
            },

            // RV32 I
            (        _,     _, rspace::opcode::LUI) => {
                // LUI
                reg[rd] = Uimm;
            },
            (        _,     _, rspace::opcode::AUIPC) => {
                // AUIPC
                reg[rd] = Uimm + (pc as u32);
            },
            (        _,     _, rspace::opcode::JAL) => {
                // JAL
                // TODO: unclear if we need to execute the jumped to instruction or +4?
                //pc += sign_extend(inst, UJimm) as usize;
                //reg[rd] = (pc + 4) as u32;
            },

            // TODO: handle instruction decoding failure
            (f7, f3, op) => {
                println!("FIX  PC: 0x{:04x} F7: {:07b} F3: {:03b} OP: {:07b}", pc, f7, f3, op);

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
                panic!("FIXME")
            },
        }

        println!("FINE PC: 0x{:04x} F7: {:07b} F3: {:03b} OP: {:07b}", pc, func7, func3, opcode);
        pc += 4;
    }
}



fn select_and_shift(inst: u32, hi: usize, lo: usize) -> u32 {
    (inst & u32::mask(hi..lo)) >> lo
}


// TODO: better to move imm inst extraction here?
fn sign_extend(inst: u32, imm: u32) -> u32 {
    if ((inst & 0x80_00_00_00) == 0x80_00_00_00) {
        let opcode = select_and_shift(inst, 6, 0);
        let mask = match rspace::opcode::instruction_type(opcode) {
            rspace::opcode::InstType::R  => 0x0,
            rspace::opcode::InstType::I  => u32::mask(31..11),
            rspace::opcode::InstType::S  => u32::mask(31..11),
            rspace::opcode::InstType::SB => u32::mask(31..12),
            rspace::opcode::InstType::U  => u32::mask(31..31),
            rspace::opcode::InstType::UJ => u32::mask(31..20),
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


#[derive(Debug)]
struct RegFile {
    _x0: u32,
    reg: [u32; 31]
}

impl RegFile {
    fn new(reg: [u32; 31]) -> RegFile {
        RegFile { _x0: 0, reg: reg }
    }
}

impl Index<usize> for RegFile {
    type Output = u32;

    fn index(&self, idx: usize) -> &u32 {
        match idx {
            0 => &0,
            _ => &self.reg[idx-1],
        }
    }
}

impl IndexMut<usize> for RegFile {
    fn index_mut<'a>(&'a mut self, idx: usize) -> &'a mut u32 {
        match idx {
            // TODO: this feel like a hack, can we get rid of _x0?
            0 => & mut self._x0,
            _ => & mut self.reg[idx-1],
        }

    }
}


#[test]
fn regfile_test() {
    let mut reg = RegFile::new([0; 31]);
    reg.reg[0] = 1;
    reg.reg[1] = 2;
    reg.reg[30] = 3;

    assert_eq!(reg[0], 0);
    assert_eq!(reg[1], 1);
    assert_eq!(reg[2], 2);
    assert_eq!(reg[31], 3);

    // Test writing to.
    reg[0] = 10;
    reg[1] = 11;
    reg[2] = 12;
    reg[31] = 13;

    assert_eq!(reg[0], 0);
    assert_eq!(reg[1], 11);
    assert_eq!(reg[2], 12);
    assert_eq!(reg[31], 13);
}


// Memory access stuff
// TODO: compile time size, instead of hardcoded
struct Memory {
    _rom_hole: u8,
    rom: [u8; 4096],
    ram: [u8; 4096]
}

impl Memory {
    fn new(rom: [u8; 4096], ram: [u8; 4096]) -> Memory {
        Memory {
            _rom_hole: 0,
            rom: rom,
            ram: ram
        }
    }

    fn fetch_instruction(&self, idx: usize) -> u32 {
        let inst_u8: [u8; 4] = [self[idx], self[idx+1], self[idx+2], self[idx+3]];

        // TODO: better way of doing this
        //unsafe { std::mem::transmute::<[u8; 4], u32>(inst_u8) }
        ((inst_u8[0] as u32)) | ((inst_u8[1] as u32) << 8) | ((inst_u8[2] as u32) << 16) | ((inst_u8[3] as u32) << 24)
    }
}

impl Index<usize> for Memory {
    type Output = u8;

    fn index(&self, idx: usize) -> &u8 {
        if idx < 4096 {
            &self.rom[idx]
        } else {
            &self.ram[idx-4096]
        }
    }
}

impl IndexMut<usize> for Memory {
    fn index_mut<'a>(&'a mut self, idx: usize) -> &'a mut u8 {
        if idx < 4096 {
            & mut self._rom_hole
        } else {
            & mut self.ram[idx-4096]
        }
    }
}


#[test]
fn memory_test() {
    let mut mem = Memory::new([0; 4096], [0; 4096]);
    mem.rom[0] = 1;
    mem.rom[4095] = 2;
    mem.ram[0] = 3;
    mem.ram[4095] = 4;

    assert_eq!(mem[0], 1);
    assert_eq!(mem[4095], 2);
    assert_eq!(mem[4096], 3);
    assert_eq!(mem[8191], 4);

    // Test writing to.
    mem[0] = 11;
    mem[4095] = 12;
    mem[4096] = 13;
    mem[8191] = 14;

    // ROM hole
    assert_eq!(mem[0], 1);
    assert_eq!(mem[4095], 2);

    // RAM
    assert_eq!(mem[4096], 13);
    assert_eq!(mem[8191], 14);
}

#[test]
fn instruction_memory_test() {
    let mut mem = Memory::new([0; 4096], [0; 4096]);

    mem.rom[0] = 1;
    mem.rom[1] = 2;
    mem.rom[2] = 3;
    mem.rom[3] = 4;

    for pc in 0..3 {
        let inst_u8: [u8; 4] = [mem[pc], mem[pc+1], mem[pc+2], mem[pc+3]];
        let inst = unsafe { std::mem::transmute::<[u8; 4], u32>(inst_u8) };

        assert_eq!(mem.fetch_instruction(pc), inst);
    }
}









fn compare_assembly(binary_code: Vec<u32>, test_asm: &str) {
    // Reprocess input
    let mut other_code: Vec<u32> = Vec::new();
    let mut rtw = File::open("input.bin").unwrap();

    loop {
        match rtw.read_u32::<LittleEndian>() {
            Ok(x) => {
                if (x != 0x6f) & (x != 0x8067) {
                    other_code.push(x);
                }
            },
            _ => break,
        }
    }

    // reprocess asm
    let mut asm: Vec<&str> = Vec::new();

    for line in test_asm.lines() {
        let line = line.trim();
        let line = match line.find(r#"//"#) {
            Some(x) => &line[..x],
            None => line,
        };

        if !line.is_empty() {
            asm.push(line);
        }
    }


    // Compare and print ones that are not matched
    println!("{:?}", "asm == other_code");
    assert_eq!(asm.len(), other_code.len());

    println!("{:?}", "asm == binary_code");
    assert_eq!(asm.len(), binary_code.len());

    for (i, item) in asm.iter().enumerate() {
        if binary_code[i] != other_code[i] {
            println!("{:?}", i);
            println!("{:?}", item);

            let byte_binary_code = unsafe { std::mem::transmute::<u32, [u8; 4]>(binary_code[i].to_le()) };
            let byte_other_code = unsafe { std::mem::transmute::<u32, [u8; 4]>(other_code[i].to_le()) };

            println!("{:08b} {:08b} {:08b} {:08b}", byte_binary_code[3], byte_binary_code[2], byte_binary_code[1], byte_binary_code[0]);
            println!("{:08b} {:08b} {:08b} {:08b}", byte_other_code[3], byte_other_code[2], byte_other_code[1], byte_other_code[0]);

            //println!("{:032b}", binary_line);
            //println!("{:08x}", binary_line);
            //println!("{:08b} {:08b} {:08b} {:08b}", byte_line[3], byte_line[2], byte_line[1], byte_line[0]);
        }
    }
}
