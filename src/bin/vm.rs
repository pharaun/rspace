extern crate rspace;
extern crate byteorder;
extern crate twiddle;

use byteorder::{LittleEndian, ReadBytesExt};
use std::fs::File;

use rspace::types;
use twiddle::Twiddle;

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
        //sw x0 x1 0x1
        sw x1 x0 0x1
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

    // Virtual machine stuff

    // Registers
    let mut reg: [u32; 32] = [0; 32];

    // TODO: shouldn't this be u32?
    let mut pc: usize = 0;

    // Ram
    let mut ram: [u32; 1024] = [0; 1024];

    // Rom (would be nice to make this consistent sized)
    let rom = {
        let mut rom: [u32; 1024] = [0; 1024];

        for i in 0..binary_code.len() {
            rom[i] = binary_code[i];
        }
        rom
    };

    // VM loop
    loop {
        // TODO: unitify memory at some point
        let inst = rom[pc];

        // Decode opcode
        let opcode = select_and_shift(inst, 6, 0);

        // Inst Type
        // TODO: change this over to generating the mask needed (for rspace issue #4)
        //let instType = rspace::opcode::instruction_type(opcode);

        // Prefetch the func3/7
        let func3 = select_and_shift(inst, 14, 12);
        let func7 = select_and_shift(inst, 31, 25);

        // Prefetch rd/rs1/rs2
        let rd    = select_and_shift(inst, 11, 7);
        let rs1   = select_and_shift(inst, 19, 15);
        let rs2   = select_and_shift(inst, 24, 20);

        // IMM types - Probably can be put in the asm steps
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
            (0x0000000, 0x000, rspace::opcode::OP_REG) => {
                // ADD
            },
            (0x0100000, 0x000, rspace::opcode::OP_REG) => {
                // SUB
            },
            (0x0000000, 0x001, rspace::opcode::OP_REG) => {
                // SLL
            },
            (0x0000000, 0x010, rspace::opcode::OP_REG) => {
                // SLT
            },
            (0x0000000, 0x011, rspace::opcode::OP_REG) => {
                // SLTU
            },
            (0x0000000, 0x100, rspace::opcode::OP_REG) => {
                // XOR
            },
            (0x0000000, 0x101, rspace::opcode::OP_REG) => {
                // SRL
            },
            (0x0100000, 0x101, rspace::opcode::OP_REG) => {
                // SRA
            },
            (0x0000000, 0x110, rspace::opcode::OP_REG) => {
                // OR
            },
            (0x0000000, 0x111, rspace::opcode::OP_REG) => {
                // AND
            },

            // RV32 M extensions
            (0x0000001, 0x000, rspace::opcode::OP_REG) => {
                // MUL
            },
            (0x0000001, 0x001, rspace::opcode::OP_REG) => {
                // MULH
            },
            (0x0000001, 0x010, rspace::opcode::OP_REG) => {
                // MULHSU
            },
            (0x0000001, 0x011, rspace::opcode::OP_REG) => {
                // MULHU
            },
            (0x0000001, 0x100, rspace::opcode::OP_REG) => {
                // DIV
            },
            (0x0000001, 0x101, rspace::opcode::OP_REG) => {
                // DIVU
            },
            (0x0000001, 0x110, rspace::opcode::OP_REG) => {
                // REM
            },
            (0x0000001, 0x111, rspace::opcode::OP_REG) => {
                // REMU
            },

            // RV32 I
            (        _, 0x000, rspace::opcode::OP_IMM) => {
                // ADDI
            },
            (0x0000000, 0x001, rspace::opcode::OP_IMM) => {
                // SLLI
            },
            (        _, 0x010, rspace::opcode::OP_IMM) => {
                // SLTI
            },
            (        _, 0x011, rspace::opcode::OP_IMM) => {
                // SLTIU
            },
            (        _, 0x100, rspace::opcode::OP_IMM) => {
                // XORI
            },
            (0x0000000, 0x101, rspace::opcode::OP_IMM) => {
                // SRLI
            },
            (0x0100000, 0x101, rspace::opcode::OP_IMM) => {
                // SRAI
            },
            (        _, 0x110, rspace::opcode::OP_IMM) => {
                // ORI
            },
            (        _, 0x111, rspace::opcode::OP_IMM) => {
                // ANDI
            },

            // RV32 I
            (        _, 0x000, rspace::opcode::JALR) => {
                // JALR
            },

            // RV32 I
            (        _, 0x000, rspace::opcode::LOAD) => {
                // LB
            },
            (        _, 0x001, rspace::opcode::LOAD) => {
                // LH
            },
            (        _, 0x010, rspace::opcode::LOAD) => {
                // LW
            },
            (        _, 0x100, rspace::opcode::LOAD) => {
                // LBU
            },
            (        _, 0x101, rspace::opcode::LOAD) => {
                // LHU
            },

            // RV32 I
            (        _, 0x000, rspace::opcode::MISC_MEM) => {
                // FENCE
            },
            (        _, 0x001, rspace::opcode::MISC_MEM) => {
                // FENCE.I
            },

            // RV32 I
            (        _, 0x000, rspace::opcode::SYSTEM) => {
                // ECALL | EBREAK
                let imm   = select_and_shift(inst, 31, 20);

                match imm {
                    0x000000000000 => {
                        // ECALL
                    },
                    0x000000000001 => {
                        // EBREAK
                    },
                    _ => panic!("FIXME"),
                }
            },
            (        _, 0x001, rspace::opcode::SYSTEM) => {
                // CSRRW
            },
            (        _, 0x010, rspace::opcode::SYSTEM) => {
                // CSRRS
            },
            (        _, 0x011, rspace::opcode::SYSTEM) => {
                // CSRRC
            },
            (        _, 0x101, rspace::opcode::SYSTEM) => {
                // CSRRWI
            },
            (        _, 0x110, rspace::opcode::SYSTEM) => {
                // CSRRSI
            },
            (        _, 0x111, rspace::opcode::SYSTEM) => {
                // CSRRCI
            },

            // RV32 I
            (        _, 0x000, rspace::opcode::STORE) => {
                // SB
            },
            (        _, 0x001, rspace::opcode::STORE) => {
                // SH
            },
            (        _, 0x010, rspace::opcode::STORE) => {
                // SW
            },

            // RV32 I
            (        _, 0x000, rspace::opcode::BRANCH) => {
                // BEQ
            },
            (        _, 0x001, rspace::opcode::BRANCH) => {
                // BNE
            },
            (        _, 0x100, rspace::opcode::BRANCH) => {
                // BLT
            },
            (        _, 0x101, rspace::opcode::BRANCH) => {
                // BGE
            },
            (        _, 0x110, rspace::opcode::BRANCH) => {
                // BLTU
            },
            (        _, 0x111, rspace::opcode::BRANCH) => {
                // BGEU
            },

            // RV32 I
            (        _,     _, rspace::opcode::LUI) => {
                // LUI
            },
            (        _,     _, rspace::opcode::AUIPC) => {
                // AUIPC
            },
            (        _,     _, rspace::opcode::JAL) => {
                // JAL
            },

            // TODO: handle instruction decoding failure
            (f7, f3, op) => {
                println!("F7: {:07b} F3: {:03b} OP: {:07b}", f7, f3, op);
                panic!("FIXME")
            },
        }

        // TODO: decode steps
        // 3. process the instruction and +4 to pc (32bit instructions)
        pc += 4;
    }
}



fn select_and_shift(inst: u32, hi: usize, lo: usize) -> u32 {
    (inst & u32::mask(hi..lo)) >> lo
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
