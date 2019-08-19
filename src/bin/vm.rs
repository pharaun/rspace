extern crate rspace;
extern crate byteorder;
extern crate twiddle;

use byteorder::{LittleEndian, WriteBytesExt};


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
    "#;

    let binary_code = rspace::asm::parse_asm(test_asm);
    let binary_u8 = {
        let mut wtr = vec![];

        for i in 0..binary_code.len() {
            let _ = wtr.write_u32::<LittleEndian>(binary_code[i]);
        }
        wtr
    };

    // Virtual machine setup
    //
    // Rom (would be nice to make this consistent sized)
    let rom = {
        let mut rom: [u8; 4096] = [0; 4096];
        for i in 0..binary_u8.len() {
            rom[i] = binary_u8[i];
        }
        rom
    };

    let mut vm = rspace::vm::Emul32::new_with_rom(rom);

    // Virtal machine run
    vm.run();
}
