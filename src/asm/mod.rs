use vm::opcode;

use twiddle::Twiddle;

pub mod parse;
pub mod ast;


// TODO: Code quality improvement
//
// 1. First pass: Clean up the raw parser output, and uniformize into a nice AST with error tracking) (ie file location information
// 2. Second pass: Expand the macros and other relevant bits a needed
// 3. Third pass: Collect up the label and symbols
// 4. Fourth pass: rectify the inst into u32 + attribute memory location for labels
// 5. Fifth pass: Find where to put data, and then rectify reference to data/memory locations.
pub fn parse_asm(input: &str) -> Vec<u32> {
    // First pass -> Vec<(u32, or entry to retrify on 2nd pass (for labels))>
    let mut first_pass: Vec<ast::AsmLine> = Vec::new();

    // This symbol table will be a list of (label, location)
    // Will handle duplicate entries by just listing it
    let mut position: usize = 0; // Per u32 word
    let mut label_acc: Vec<ast::Labels> = Vec::new();
    let mut symbol_table: Vec<(ast::Labels, usize)> = Vec::new();

    // Assembly output
    // Second pass -> Vec<u32>
    let mut second_pass: Vec<u32> = Vec::new();

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
                Ok(asmline) => {
                    match asmline {
                        ast::AsmLine::Lab(l) => {
                            // We see a label, accumulate it till we get an asm line to attribute it to
                            label_acc.push(l);
                        },
                        ast::AsmLine::Lns(l, inst, args) => {
                            // We see a label, accumulate + handle it
                            label_acc.push(l);

                            // Put labels onto symbol list
                            while let Some(la) = label_acc.pop() {
                                symbol_table.push((la, position));
                            }

                            // Process instruction
                            first_pass.push(ast::AsmLine::Ins(inst, args));

                            // Inc line pointer
                            position += 1;
                        },
                        ast::AsmLine::Ins(inst, args) => {
                            // Check if there's any accumulated label, and if so put onto symbol list
                            while let Some(la) = label_acc.pop() {
                                symbol_table.push((la, position));
                            }

                            // Process instruction
                            first_pass.push(ast::AsmLine::Ins(inst, args));

                            // Inc line pointer
                            position += 1;
                        },
                    }
                },
            }
        }
    }

    // Debug label positioning
    println!("{:?}", position);
    println!("{:?}", symbol_table);

    // Start second pass
    position = 0; // Per u32 word
    first_pass.reverse(); // Since we pop from end, reverse order
    while let Some(ast::AsmLine::Ins(inst, args)) = first_pass.pop() {
        let upper_inst = &inst.to_uppercase();

        // 3. lookup if in the opcode lookup table
        match opcode::lookup(upper_inst) {
            // 4. if not (macro/etc, panic for now)
            None => println!("Skipping for now - {:?}", inst),
            // 5. if so proceed below
            Some(x) => {
                let binary_line = lut_to_binary(upper_inst, args, x, &symbol_table, position);

                //println!("{:?}", line);
                //println!("{:032b}", binary_line);
                //println!("{:08x}", binary_line);
                //let byte_line = unsafe { std::mem::transmute::<u32, [u8; 4]>(binary_line.to_le()) };
                //println!("{:08b} {:08b} {:08b} {:08b}", byte_line[3], byte_line[2], byte_line[1], byte_line[0]);


                // TODO: the elf dump shows it in different order, need to compare +
                // figure out if i need to rearrange the bytes in the output?
                second_pass.push(binary_line);
            },
        }

        position += 1;
    }

    second_pass
}

// TODO: support labeled memory location, for now only branches + jumps (need more support)
// Don't know how to figure out if i should generate a relative or an absolute address, branches are always relative to the inst
// supporting those for now, the jalr/auipc/lui/jal i can't figure out yet
fn lut_to_binary(inst: &str, args: Vec<ast::Args>, inst_encode: opcode::InstEnc, symbol: &Vec<(ast::Labels, usize)>, inst_pos: usize) -> u32 {
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
                "FENCE" => {
                    // Default settings with all flags toggled (just hardcode for now)
                    ret |= select_and_shift(0b11111111, 7, 0, 20);
                },
                "FENCE.I" => (),
                // func12 - ECALL- 0b000000000000
                "ECALL" => (),
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
                        "JALR" => {
                            if is_label(&args[2]) {
                                // TODO: deal with labels
                                let lab = extract_label(args[2].clone());
                                let lpos = find_label_position(lab, symbol, inst_pos);
                                let imm = encode_relative_offset(inst_pos, lpos);
                            } else {
                                // TODO: deal with imm
                                // TODO: design a function for dealing with imm (takes a list of range + shift)
                                // for extracting bytes and shifting em to relevant spot plus dealing with sign
                                // extend as needed
                                let imm  = extract_imm(&args[2]);
                                // imm[11:0]
                                ret |= select_and_shift(imm, 11, 0, 20);
                            }
                        },
                        _ => {
                            // TODO: to support addi for 'la'
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
        opcode::InstType::S | opcode::InstType::SB => {
            // 31-25, 24-20, 19-15, 14-12, 11-7, 6-0
            //   imm,   rs2,   rs1, func3,  imm, opcode
            if args.len() != 3 {
                println!("{:?}", inst);
                panic!("S - Not 3 args");
            }
            // Args rs1 rs2 imm
            ret |= extract_and_shift_register(&args[0], 15);
            ret |= extract_and_shift_register(&args[1], 20);

            match inst_encode.encoding {
                opcode::InstType::S => {
                    // TODO: deal with imm
                    let imm  = extract_imm(&args[2]);
                    // imm[4:0]
                    ret |= select_and_shift(imm, 4, 0, 7);
                    // imm[11:5]
                    ret |= select_and_shift(imm, 11, 5, 25);
                },
                opcode::InstType::SB => {
                    // TODO: objdump shows bne/beq swapped and so on
                    // plus all fixed 0x8 jump, so i'm not sure I'm
                    // encoding these quite right, but with the re-arranged
                    // order it seems to work.... ?
                    if is_label(&args[2]) {
                        let lab = extract_label(args[2].clone());
                        let lpos = find_label_position(lab, symbol, inst_pos);
                        let imm = encode_relative_offset(inst_pos, lpos);

                        // imm[11]
                        ret |= select_and_shift(imm, 11, 11, 7);
                        // imm[4:1]
                        ret |= select_and_shift(imm, 4, 1, 8);
                        // imm[10:5]
                        ret |= select_and_shift(imm, 10, 5, 25);
                        // imm[12]
                        ret |= select_and_shift(imm, 12, 12, 31);
                    } else {
                        // TODO: deal with imm
                        let imm  = extract_imm(&args[2]);
                        // imm[11]
                        ret |= select_and_shift(imm, 11, 11, 7);
                        // imm[4:1]
                        ret |= select_and_shift(imm, 4, 1, 8);
                        // imm[10:5]
                        ret |= select_and_shift(imm, 10, 5, 25);
                        // imm[12]
                        ret |= select_and_shift(imm, 12, 12, 31);
                    }
                },
                _ => (),
            }
        },
        opcode::InstType::U | opcode::InstType::UJ => {
            // 31-12, 11-7, 6-0
            //   imm,   rd, opcode
            if args.len() != 2 {
                println!("{:?}", inst);
                panic!("U - Not 2 args");
            }
            // args rd imm
            ret |= extract_and_shift_register(&args[0], 7);
            match inst_encode.encoding {
                opcode::InstType::U => {
                    // TODO: update lui + auipc tests to not use this mips legacy
                    //
                    // LUI only probably (verify)
                    // imm[31:12]
                    //
                    // Due to mips legacy, GAS takes the bottom 20 bits, not the top 20 as per
                    // the spec, but via %hi(0xFF) and %low(0xFF)... macro? we are able to access
                    // the upper 20bit (it gets shifted down 12 bits). Let's do what GAS does here
                    // so that we can assemble the output of gcc -S.
                    //
                    // This mismatch what you would expect from the docs:
                    //      LUI places the U-immediate value in the top 20 bits of the destination
                    //      register rd, filling in the lowest 12 bits with zeros.
                    //
                    // TODO: Add `li x1 0xff` and `la x1 symbol` which makes this nicer (takes the
                    // value and symbol and split it into upper 20 and lower 12 bits and load it)
                    // TODO: deal with imm
                    //let imm = extract_imm(&args[1]);
                    //ret |= select_and_shift(imm, 19, 0, 12);
                    if is_label(&args[1]) {
                        let lab = extract_label(args[1].clone());
                        let lpos = find_label_position(lab, symbol, inst_pos);
                        let imm = encode_relative_offset(inst_pos, lpos);

                        ret |= select_and_shift(imm, 19, 0, 12);
                        //ret |= select_and_shift(imm, 31, 12, 12);
                    } else {
                        // TODO: this relative offset doesn't work for LUI, we want to see label, get the value from that and store that
                        let imm = extract_imm(&args[1]);
                        ret |= select_and_shift(imm, 19, 0, 12);
                        //ret |= select_and_shift(imm, 31, 12, 12);
                    }
                },
                opcode::InstType::UJ => {
                    if is_label(&args[1]) {
                        let lab = extract_label(args[1].clone());
                        let lpos = find_label_position(lab, symbol, inst_pos);
                        let imm = encode_relative_offset(inst_pos, lpos);

                        // imm[19:12]
                        ret |= select_and_shift(imm, 19, 12, 0);
                        // imm[11]
                        ret |= select_and_shift(imm, 11, 11, 20);
                        // imm[10:1]
                        ret |= select_and_shift(imm, 10, 1, 21);
                        // imm[20]
                        ret |= select_and_shift(imm, 20, 20, 31);
                    } else {
                        // TODO: deal with imm
                        let imm = extract_imm(&args[1]);
                        // imm[19:12]
                        ret |= select_and_shift(imm, 19, 12, 0);
                        // imm[11]
                        ret |= select_and_shift(imm, 11, 11, 20);
                        // imm[10:1]
                        ret |= select_and_shift(imm, 10, 1, 21);
                        // imm[20]
                        ret |= select_and_shift(imm, 20, 20, 31);
                    }
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

fn extract_csr(arg: &ast::Args) -> u32 {
    match *arg {
        ast::Args::Csr(n) => {
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
        _ => panic!("Was a register or num or label, expected csr"),
    }
}

fn extract_imm(arg: &ast::Args) -> u32 {
    match *arg {
        ast::Args::Num(n) => {
            n
        },
        _ => panic!("Was a register or csr or label, expected Num"),
    }
}

fn extract_and_shift_register(arg: &ast::Args, shift: u32) -> u32 {
    match *arg {
        ast::Args::Reg(r) => {
            // Map x0..x31 -> 0..31
            // TODO: for now just drop the x from the registers
            r[1..].parse::<u32>().unwrap() << shift
        },
        _ => panic!("Was a num or csr or label, expected register"),
    }
}

// TODO: should be able to use this without a clone
fn extract_label<'input>(arg: ast::Args<'input>) -> ast::Labels<'input> {
    match arg {
        ast::Args::Lab(l) => {
            l
        },
        _ => panic!("Was a register or csr or num, expected Label"),
    }
}

fn is_label(arg: &ast::Args) -> bool {
    match *arg {
        ast::Args::Lab(_) => true,
        _ => false,
    }
}

// TODO: this is kinda a poor quality function, need to redo it
fn find_label_position<'input>(lab: ast::Labels<'input>, symbol: &Vec<(ast::Labels, usize)>, inst_pos: usize) -> usize {
//    println!("");
//    println!("Label to look up: {:?}", lab);
//    println!("Instruction Position: {:?}", inst_pos);

    // Decode the type of Label it is (is it a word or a numberic label)
    // If word, proceed, but if numberic,
    //      parse the letter after (b or f) for backward or forward numberic ref
    //      find inst pos then proceed backward or forward in search for the numberic ref
    // if word
    //      Assume no duplicate word label (should not happen, integrity check the symbol table)
    //      linear scan till you find the matching word label
    match lab {
        ast::Labels::WLabel(l) => {
            for val in symbol.iter() {
                match val {
                    &(ast::Labels::WLabel(sl), spos) => {
                        if sl == l {
                            //println!("Label Position: {:?}", spos);
                            return spos
                        }
                    },
                    _ => (),
                }
            }
            panic!("Did not find the label in the symbol table!")
        },
        ast::Labels::NLabel(l) => {
            let mut s = String::from(l);

            let dir = s.pop().unwrap();
            let num = &s;

            match dir {
                'f' => {
                    // Forward
                    for val in symbol.iter() {
                        match val {
                            &(ast::Labels::NLabel(sl), spos) => {
                                if (sl == num) & (spos >= inst_pos) {
                                    //println!("Label Position: {:?}", spos);
                                    return spos
                                }
                            },
                            _ => (),
                        }
                    }
                    panic!("Did not find the label in the symbol table!")
                },
                'b' => {
                    // Backward
                    for val in symbol.iter().rev() {
                        match val {
                            &(ast::Labels::NLabel(sl), spos) => {
                                if (sl == num) & (spos <= inst_pos) {
                                    //println!("Label Position: {:?}", spos);
                                    return spos
                                }
                            },
                            _ => (),
                        }
                    }
                    panic!("Did not find the label in the symbol table!")
                },
                _ => panic!("Invalid identifer, should be b or f for NLabel"),
            }
        },
    }
}

fn encode_relative_offset(inst_pos: usize, label_pos: usize) -> u32 {
    // TODO: hella truncation
    // This assumes u32 sized instruction words, the pos are integer in block of u32
    // If label pos is earlier than inst_pos its a negative offset
    // and viceverse
    // The address is in byte (hence u32 blocks)
    let inst_addr: i64 = (inst_pos as i64) * 4;
    let label_addr: i64 = (label_pos as i64) * 4;
    let offset = label_addr - inst_addr;
    offset as u32
}
