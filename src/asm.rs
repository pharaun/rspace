use parse;
use types;
use opcode;
use std;

use core::ops::Range;
use twiddle::Twiddle;

pub fn parse_asm(input: &str) -> Vec<u32> {
    let mut asm_out: Vec<u32> = Vec::new();

    for line in input.lines() {
        let line = line.trim();
        let line = match line.find(r#"//"#) {
            Some(x) => &line[..x],
            None => line,
        };

        if !line.is_empty() {
            // 2. parse it via lalrpop (parse_AsmLine)
            let parse = parse::parse_AsmLine(line);

            match parse {
                Err(x) => {
                    println!("{:?}", line);
                    println!("{:?}", x);
                },
                Ok((inst, args)) => {
                    let upper_inst = &inst.to_uppercase();

                    // 3. lookup if in the opcode lookup table
                    match opcode::lookup(upper_inst) {
                        // 4. if not (macro/etc, panic for now)
                        None => println!("Skipping for now - {:?}", inst),
                        // 5. if so proceed below
                        Some(x) => {
                            let binary_line = lut_to_binary(upper_inst, args, x);
                            let byte_line = unsafe { std::mem::transmute::<u32, [u8; 4]>(binary_line.to_le()) };

                            //println!("{:?}", line);
                            //println!("{:032b}", binary_line);
                            //println!("{:08b} {:08b} {:08b} {:08b}", byte_line[3], byte_line[2], byte_line[1], byte_line[0]);

                            asm_out.push(binary_line);
                        },
                    }
                },
            }
        }
    }

    asm_out
}

fn lut_to_binary(inst: &str, args: Vec<types::Args>, inst_encode: opcode::InstEnc) -> u32 {
    let mut ret: u32 = 0x0;

    // Opcode
    ret |= inst_encode.opcode;

    // Func3
    ret |= match_and_shift(inst_encode.func3, 12);

    // Func7
    ret |= match_and_shift(inst_encode.func7, 25);

    // 6. i think a good step will be to use the data in the LUT to construct a binary line (u32)
    match inst_encode.encoding {
        opcode::InstType::R => {
            // 31-25, 24-20, 19-15, 14-12, 11-7, 6-0
            // func7,   rs2,   rs1, func3,   rd, opcode
            if args.len() != 3 {
                println!("{:?}", inst);
                panic!("R - Not 3 args");
            }
            // args rd, rs, r2
            ret |= extract_and_shift_register(&args[0],  7);
            ret |= extract_and_shift_register(&args[1], 15);
            ret |= extract_and_shift_register(&args[2], 20);
        },
        opcode::InstType::I => {
            // 31-20, 19-15, 14-12, 11-7, 6-0
            //   imm,   rs1, func3,   rd, opcode
            match inst {
                "FENCE" | "FENCE.I" => (),
                "ECALL" => {
                    // func12 - ECALL- 0b000000000000
                    // Do nothing
                },
                "EBREAK" => {
                    // func12 - EBREAK - 0b000000000001
                    ret |= 0x1 << 20;
                },
                "CSRRWI" | "CSRRSI" | "CSRRCI" => {
                    if args.len() != 3 {
                        println!("{:?}", inst);
                        panic!("I - Not 3 args");
                    }
                    // args rd, imm, csr
                    ret |= extract_and_shift_register(&args[0],  7);

                    let imm  = extract_imm(&args[1]);
                    ret |= select_and_shift(imm, 5, 0, 15);

                    let csrreg = extract_csr(&args[2]);
                    ret |= csrreg << 20;
                },
                _ => {
                    if args.len() != 3 {
                        println!("{:?}", inst);
                        panic!("I - Not 3 args");
                    }
                    // args rd, rs, imm
                    ret |= extract_and_shift_register(&args[0],  7);
                    ret |= extract_and_shift_register(&args[1], 15);

                    match inst {
                        "SLLI" | "SRLI" | "SRAI" => {
                            // TODO: deal with imm
                            let imm  = extract_imm(&args[2]);
                            // shamt[4:0]
                            ret |= select_and_shift(imm, 4, 0, 20);
                            // imm[11:5] - taken care by func7
                        },
                        "CSRRW" | "CSRRS" | "CSRRC" => {
                            // args rd, rs, csr
                            let csrreg = extract_csr(&args[2]);
                            ret |= csrreg << 20;
                        },
                        _ => {
                            // TODO: deal with imm
                            // TODO: design a function for dealing with imm (takes a list of range + shift)
                            // for extracting bytes and shifting em to relevant spot plus dealing with sign
                            // extend as needed
                            let imm  = extract_imm(&args[2]);
                            // imm[11:0]
                            ret |= select_and_shift(imm, 11, 0, 20);
                        },
                    }
                },
            }
        },
        opcode::InstType::S | opcode::InstType::B => {
            // 31-25, 24-20, 19-15, 14-12, 11-7, 6-0
            //   imm,   rs2,   rs1, func3,  imm, opcode
            if args.len() != 3 {
                println!("{:?}", inst);
                panic!("S - Not 3 args");
            }
            // Args rs1 rs2 imm
            ret |= extract_and_shift_register(&args[0], 15);
            ret |= extract_and_shift_register(&args[1], 20);

            // TODO: deal with imm
            let imm  = extract_imm(&args[2]);

            match inst_encode.encoding {
                opcode::InstType::S => {
                    // imm[4:0]
                    ret |= select_and_shift(imm, 4, 0, 7);
                    // imm[11:5]
                    ret |= select_and_shift(imm, 11, 5, 25);
                },
                opcode::InstType::B => {
                    // imm[11]
                    ret |= select_and_shift(imm, 11, 11, 7);
                    // imm[4:1]
                    ret |= select_and_shift(imm, 4, 1, 8);
                    // imm[10:5]
                    ret |= select_and_shift(imm, 10, 5, 25);
                    // imm[12]
                    ret |= select_and_shift(imm, 12, 12, 31);
                },
                _ => (),
            }
        },
        opcode::InstType::U | opcode::InstType::J => {
            // 31-12, 11-7, 6-0
            //   imm,   rd, opcode
            if args.len() != 2 {
                println!("{:?}", inst);
                panic!("U - Not 2 args");
            }
            // args rd imm
            ret |= extract_and_shift_register(&args[0], 7);

            // TODO: deal with imm
            let imm = extract_imm(&args[1]);

            match inst_encode.encoding {
                opcode::InstType::U => {
                    // imm[31:12]
                    ret |= select_and_shift(imm, 31, 12, 12);
                },
                opcode::InstType::J => {
                    // imm[19:12]
                    ret |= select_and_shift(imm, 19, 12, 0);
                    // imm[11]
                    ret |= select_and_shift(imm, 11, 11, 20);
                    // imm[10:1]
                    ret |= select_and_shift(imm, 10, 1, 21);
                    // imm[20]
                    ret |= select_and_shift(imm, 20, 20, 31);
                },
                _ => (),
            }
        },
    }

    // 7. check if its an instruction that needs additional/special processing, if so do the deed
    // 8. emit out the binary code (see if we can't construct a simple example that uses at least 1 of
    //    all instruction and features in gcc, then use that to get the binarycode for that, and then
    //    compare the two to make sure our result is the same)
    // 9. proceed to start work on the virtual machine
    ret
}

fn match_and_shift(byte: Option<u32>, shift: u32) -> u32 {
    match byte {
        Some(x) => x << shift,
        _ => 0x0,
    }
}

fn select_and_shift(imm: u32, hi: usize, lo: usize, shift: usize) -> u32 {
    ((imm & u32::mask(hi..lo)) >> lo) << shift
}

fn extract_csr(arg: &types::Args) -> u32 {
    match *arg {
        types::Args::Csr(n) => {
            match n {
                "CYCLE" => 0xC00,
                "CYCLEH" => 0xC80,
                "TIME" => 0xC01,
                "TIMEH" => 0xC81,
                "INSTRET" => 0xC02,
                "INSTRETH" => 0xC82,
                _ => panic!("New type of csr"),
            }
        },
        _ => panic!("Was a register or num, expected csr"),
    }
}

fn extract_imm(arg: &types::Args) -> u32 {
    match *arg {
        types::Args::Num(n) => {
            n
        },
        _ => panic!("Was a register or csr, expected Num"),
    }
}

fn extract_and_shift_register(arg: &types::Args, shift: u32) -> u32 {
    match *arg {
        types::Args::Reg(r) => {
            // Map x0..x31 -> 0..31
            // TODO: for now just drop the x from the registers
            r[1..].parse::<u32>().unwrap() << shift
        },
        _ => panic!("Was a num or csr, expected register"),
    }
}

#[test]
fn comment_test() {
	assert_eq!(";;", ";;");

    // Test number parse
    println!("{:?}", parse::parse_Number("09213"));
    println!("{:?}", parse::parse_Number("009213"));
    println!("{:?}", parse::parse_Number("0xFF"));
    println!("{:?}", parse::parse_Number("0x09123"));

    // Test register
    println!("{:?}", parse::parse_Register("x0"));
    println!("{:?}", parse::parse_Register("x31"));

    // Test CSR
    println!("{:?}", parse::parse_Csr("CYCLE"));
    println!("{:?}", parse::parse_Csr("CYCLEH"));

    // Test Arguments
    println!("{:?}", parse::parse_Arguments("x0"));
    println!("{:?}", parse::parse_Arguments("0923"));
    println!("{:?}", parse::parse_Arguments("0xFF"));

    // Test list of args
    println!("{:?}", parse::parse_VecArgs(""));
    println!("{:?}", parse::parse_VecArgs("0xFF"));
    println!("{:?}", parse::parse_VecArgs("0xFF x0"));
    println!("{:?}", parse::parse_VecArgs("0xFF x0 0923"));
    println!("{:?}", parse::parse_VecArgs("0xFF x0 0923 x2"));

    // Test Asm line
    println!("{:?}", parse::parse_AsmLine("ECALL"));
    println!("{:?}", parse::parse_AsmLine("CSRRS x0 x1 CYCLE"));
    println!("{:?}", parse::parse_AsmLine("CSRRS x0 x1 CYCLEH"));
    println!("{:?}", parse::parse_AsmLine("SFENCE.VM x0"));
    println!("{:?}", parse::parse_AsmLine("LUI x0 0xFF"));
    println!("{:?}", parse::parse_AsmLine("FCVT.W.H x0 x1"));
    println!("{:?}", parse::parse_AsmLine("FMADD.S x0 x1 x2 x3"));
    println!("{:?}", parse::parse_AsmLine("csrrci x6 0x3 INSTRET"));

    // Test lookups
    println!("{:?}", opcode::lookup("ADDI"));
    println!("{:?}", opcode::lookup("SRA"));
    println!("{:?}", opcode::lookup("NOP"));
}
