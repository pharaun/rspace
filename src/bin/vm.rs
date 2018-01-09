extern crate rspace;
extern crate byteorder;

use rspace::asm;
use byteorder::{LittleEndian, WriteBytesExt, ReadBytesExt};
use std::fs::File;

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


    // TODO: virtual machine stuff
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
