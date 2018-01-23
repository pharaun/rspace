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
        // TODO: dont bother? (get rid of inst type bits here)
        let instType = rspace::opcode::instruction_type(opcode);

        // TODO: handle sign extend and so on as needed
        match instType {
            rspace::opcode::InstType::R => {
                let rd    = select_and_shift(inst, 11, 7);
                let func3 = select_and_shift(inst, 14, 12);
                let rs1   = select_and_shift(inst, 19, 15);
                let rs2   = select_and_shift(inst, 24, 20);
                let func7 = select_and_shift(inst, 31, 25);

                match opcode {
                    rspace::opcode::OP_REG => {
                        match func7 {
                            0x0000000 => {
                                match func3 {
                                    0x000 => {
                                        // ADD
                                    },
                                    0x001 => {
                                        // SLL
                                    },
                                    0x010 => {
                                        // SLT
                                    },
                                    0x011 => {
                                        // SLTU
                                    },
                                    0x100 => {
                                        // XOR
                                    },
                                    0x101 => {
                                        // SRL
                                    },
                                    0x110 => {
                                        // OR
                                    },
                                    0x111 => {
                                        // AND
                                    },
                                    _ => panic!("FIXME"),
                                }
                            },
                            0x0000001 => {
                                // RV32 M extension
                                match func3 {
                                    0x000 => {
                                        // MUL
                                    },
                                    0x001 => {
                                        // MULH
                                    },
                                    0x010 => {
                                        // MULHSU
                                    },
                                    0x011 => {
                                        // MULHU
                                    },
                                    0x100 => {
                                        // DIV
                                    },
                                    0x101 => {
                                        // DIVU
                                    },
                                    0x110 => {
                                        // REM
                                    },
                                    0x111 => {
                                        // REMU
                                    },
                                    _ => panic!("FIXME"),
                                }
                            },
                            0x0100000 => {
                                match func3 {
                                    0x000 => {
                                        // SUB
                                    },
                                    0x101 => {
                                        // SRA
                                    },
                                    _ => panic!("FIXME"),
                                }
                            },
                            _ => panic!("FIXME"),
                        }
                    },
                    _ => panic!("FIXME"),
                }
            },
            rspace::opcode::InstType::I => {
                let rd    = select_and_shift(inst, 11, 7);
                let func3 = select_and_shift(inst, 14, 12);
                let rs1   = select_and_shift(inst, 19, 15);
                let imm   = select_and_shift(inst, 31, 20);

                match opcode {
                    rspace::opcode::OP_IMM => {
                        match func3 {
                            0x000 => {
                                // ADDI
                            },
                            0x010 => {
                                // SLTI
                            },
                            0x011 => {
                                // SLTIU
                            },
                            0x100 => {
                                // XORI
                            },
                            0x110 => {
                                // ORI
                            },
                            0x111 => {
                                // ANDI
                            },
                            0x001 => {
                                // SLLI
                                match select_and_shift(inst, 31, 25) {
                                    0x0000000 => {
                                        // SLLI
                                    },
                                    _ => panic!("FIXME"),
                                }
                            },
                            0x101 => {
                                // SLLI, SRLI, SRAI
                                match select_and_shift(inst, 31, 25) {
                                    0x0000000 => {
                                        // SRLI
                                    },
                                    0x0100000 => {
                                        // SRAI
                                    },
                                    _ => panic!("FIXME"),
                                }
                            },
                            // TODO: improve debug print, because we hit this
                            _ => panic!("I Inst type missing func3 case"),
                        }
                    },
                    rspace::opcode::JALR => {
                        match func3 {
                            0x000 => {
                                // JALR
                            },
                            _ => panic!("FIXME"),
                        }
                    },
                    rspace::opcode::LOAD => {
                        match func3 {
                            0x000 => {
                                // LB
                            },
                            0x001 => {
                                // LH
                            },
                            0x010 => {
                                // LW
                            },
                            0x100 => {
                                // LBU
                            },
                            0x101 => {
                                // LHU
                            },
                            _ => panic!("FIXME"),
                        }
                    },
                    rspace::opcode::MISC_MEM => {
                        match func3 {
                            0x000 => {
                                // FENCE
                            },
                            0x001 => {
                                // FENCE.I
                            },
                            _ => panic!("FIXME"),
                        }
                    },
                    rspace::opcode::SYSTEM => {
                        match func3 {
                            0x000 => {
                                // ECALL | EBREAK
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
                            0x001 => {
                                // CSRRW
                            },
                            0x010 => {
                                // CSRRS
                            },
                            0x011 => {
                                // CSRRC
                            },
                            0x101 => {
                                // CSRRWI
                            },
                            0x110 => {
                                // CSRRSI
                            },
                            0x111 => {
                                // CSRRCI
                            },
                            _ => panic!("FIXME"),
                        }
                    },
                    _ => panic!("FIXME"),
                }
            },
            rspace::opcode::InstType::S => {
                let func3 = select_and_shift(inst, 14, 12);
                let rs1   = select_and_shift(inst, 19, 15);
                let rs2   = select_and_shift(inst, 24, 20);
                let imm   = (select_and_shift(inst, 31, 25) << 5)
                          | select_and_shift(inst, 11, 7);

                match opcode {
                    rspace::opcode::STORE => {
                        match func3 {
                            0x000 => {
                                // SB
                            },
                            0x001 => {
                                // SH
                            },
                            0x010 => {
                                // SW
                            },
                            // TODO: improve debug print, because we hit this
                            _ => panic!("S Inst type missing func3 case"),
                        }
                    },
                    _ => panic!("FIXME"),
                }
            },
            rspace::opcode::InstType::SB => {
                let func3 = select_and_shift(inst, 14, 12);
                let rs1   = select_and_shift(inst, 19, 15);
                let rs2   = select_and_shift(inst, 24, 20);
                let imm   = (select_and_shift(inst, 31, 31) << 12)
                          | (select_and_shift(inst, 7, 7) << 11)
                          | (select_and_shift(inst, 30, 25) << 5)
                          | (select_and_shift(inst, 11, 8) << 1);

                match opcode {
                    rspace::opcode::BRANCH => {
                        match func3 {
                            0x000 => {
                                // BEQ
                            },
                            0x001 => {
                                // BNE
                            },
                            0x100 => {
                                // BLT
                            },
                            0x101 => {
                                // BGE
                            },
                            0x110 => {
                                // BLTU
                            },
                            0x111 => {
                                // BGEU
                            },
                            // TODO: improve debug print, because we hit this
                            _ => panic!("SB Inst type missing func3 case"),
                        }
                    },
                    _ => panic!("FIXME"),
                }
            },
            rspace::opcode::InstType::U => {
                let rd    = select_and_shift(inst, 11, 7);
                let imm   = (select_and_shift(inst, 31, 12) << 12);

                match opcode {
                    rspace::opcode::LUI => {
                        // LUI
                    },
                    rspace::opcode::AUIPC => {
                        // AUIPC
                    },
                    _ => panic!("FIXME"),
                }
            },
            rspace::opcode::InstType::UJ => {
                let rd    = select_and_shift(inst, 11, 7);
                let imm   = (select_and_shift(inst, 31, 31) << 20)
                          | (select_and_shift(inst, 19, 12) << 12)
                          | (select_and_shift(inst, 20, 20) << 11)
                          | (select_and_shift(inst, 30, 21) << 1);

                match opcode {
                    rspace::opcode::JAL => {
                        // JAL
                    },
                    _ => panic!("FIXME"),
                }
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
