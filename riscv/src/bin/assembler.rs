extern crate rspace;

use std::fs::File;
use std::io::prelude::*;
use byteorder::{LittleEndian, WriteBytesExt};

fn main() -> std::io::Result<()> {
    // TODO: this should have an cli arg and a way to read text asm file and convert it
    // to binary code that can be ingested directly by the emulator.
    let test_asm = r#"
        // TODO: not implemented inst
        // auipc x2 0x31241
        // fence
        // fence.i
        // ecall
        // ebreak

        addi x0 x0 1
        addi x1 x1 2

        slti x2 x0 0xA
        sltiu x3 x0 0x9

        andi x1 x0 0x0
        ori x2 x0 0xFF
        xori x3 x0 0x00FF

        // TODO: shamt
        slli x1 x1 0x0
        srli x2 x2 0x1
        srai x3 x3 0x6

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

        mul x0 x1 x2
        mulh x1 x2 x0
        mulhu x2 x0 x1
        mulhsu x0 x1 x2

        div x1 x2 x0
        divu x2 x0 x1

        rem x0 x1 x2
        remu x1 x2 x0

        rem zero ra sp
        rem gp tp t0
        rem t1 t2 s0
        rem fp s1 a0
        rem a1 a2 a3
        rem a4 a5 a6
        rem a7 s2 s3
        rem s4 s5 s6
        rem s7 s8 s9
        rem s10 s11 t3
        rem t4 t5 t6

        lui x1 0x3412

        // jump to next inst
        jal x0 inst

        // Loads
    da: .BYTE 0x94
    db: .HALF 0x3254
    dc: .WORD 0x32879812

    inst:
        // Load word
        lui x1 dc
        addi x1 x1 dc
        lw x2 x1 0x0

        lui x1 db
        addi x1 x1 db
        lh x3 x1 0x0
        lhu x4 x1 0x0

        lui x1 da
        addi x1 x1 da
        lb x5 x1 0x0
        lbu x6 x1 0x0

        // Improper store
        lui x1 0x1100
        addi x1 x1 0x1100

        // sw <addr> <data> <offset>
        sw x1 x2 0x0
        sh x1 x3 0x4
        sb x1 x5 0x8

        // Supported CSRs
        csrrw x1 x0 MARCHID
        csrrs x2 x0 MIMPID
        csrrc x3 x0 MHARTID
        csrrwi x4 0x1 MSTATUS
        csrrsi x5 0x2 MISA
        csrrci x6 0x3 MIE


        // This here onward lies dragons (branches and jumps)
        addi x1 x1 0x1
        jalr x2 x0 0xC
        addi x3 x3 0x1
    1:  addi x4 x4 0x1
        jal x0 3f
        addi x5 x5 0x1
    2:  addi x6 x6 0x1
        jal x0 4f
        addi x7 x7 0x1
    3:  addi x8 x8 0x1
        lui x11 2b
        addi x11 x11 2b
        jalr x9 x11 0
    4:  addi x10 x10 0x1


        // Branches -- Taken
    1:  addi x3 x0 0x1
        beq x1 x2 3f
        addi x4 x0 0x1
    2:  addi x5 x0 0x1
        beq x1 x2 4f
        addi x6 x0 0x1
    3:  addi x7 x0 0x1
        beq x1 x2 2b
        addi x8 x0 0x1
    4:  addi x9 x0 0x1
        addi x10 x0 0x1

    1:  addi x3 x0 0x1
        bne x1 x2 3f
        addi x4 x0 0x1
    2:  addi x5 x0 0x1
        bne x1 x2 4f
        addi x6 x0 0x1
    3:  addi x7 x0 0x1
        bne x1 x2 2b
        addi x8 x0 0x1
    4:  addi x9 x0 0x1
        addi x10 x0 0x1

    1:  addi x3 x0 0x1
        bge x1 x2 3f
        addi x4 x0 0x1
    2:  addi x5 x0 0x1
        bge x1 x2 4f
        addi x6 x0 0x1
    3:  addi x7 x0 0x1
        bge x1 x2 2b
        addi x8 x0 0x1
    4:  addi x9 x0 0x1
        addi x10 x0 0x1

    1:  addi x3 x0 0x1
        bgeu x1 x2 3f
        addi x4 x0 0x1
    2:  addi x5 x0 0x1
        bgeu x1 x2 4f
        addi x6 x0 0x1
    3:  addi x7 x0 0x1
        bgeu x1 x2 2b
        addi x8 x0 0x1
    4:  addi x9 x0 0x1
        addi x10 x0 0x1

    1:  addi x3 x0 0x1
        blt x1 x2 3f
        addi x4 x0 0x1
    2:  addi x5 x0 0x1
        blt x1 x2 4f
        addi x6 x0 0x1
    3:  addi x7 x0 0x1
        blt x1 x2 2b
        addi x8 x0 0x1
    4:  addi x9 x0 0x1
        addi x10 x0 0x1

    1:  addi x3 x0 0x1
        bltu x1 x2 3f
        addi x4 x0 0x1
    2:  addi x5 x0 0x1
        bltu x1 x2 4f
        addi x6 x0 0x1
    3:  addi x7 x0 0x1
        bltu x1 x2 2b
        addi x8 x0 0x1
    4:  addi x9 x0 0x1
        addi x10 x0 0x1
    "#;

    let binary_code = {
        let asm = rspace::asm::parse_asm(test_asm);
        let mut wtr = vec![];

        for i in asm {
            let _ = wtr.write_u32::<LittleEndian>(i);
        }
        wtr
    };

    let mut file = File::create("/tmp/test")?;
    file.write_all(&binary_code[..])
}
