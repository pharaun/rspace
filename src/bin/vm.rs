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
        //jal x0 0xFFF
        // there isn't actually a ret instruction, it's a synonym for jalr x0, 0(x1)
        //jalr x0 x1 0x0

        beq x1 x6 0x1
        bne x2 x5 0x2
        blt x3 x4 0x3
        bltu x4 x3 0x4
        bge x5 x2 0x5
        bgeu x6 x1 0x6

        lw x1 x0 0x1
        lh x2 x0 0x2
        lhu x3 x0 0x3
        lb x4 x0 0x4
        lbu x5 x0 0x5

        sw x0 x1 0x1
        sh x0 x2 0x2
        sb x0 x3 0x3

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

//   0:	00f00093          	li	ra,15
//   4:	00a02113          	slti	sp,zero,10
//   8:	00903193          	sltiu	gp,zero,9
//   c:	00007093          	andi	ra,zero,0
//  10:	0ff06113          	ori	sp,zero,255
//  14:	0ff04193          	xori	gp,zero,255
//  18:	00009093          	slli	ra,ra,0x0
//  1c:	00115113          	srli	sp,sp,0x1
//  20:	4061d193          	srai	gp,gp,0x6
//  24:	034120b7          	lui	ra,0x3412
//  28:	31241117          	auipc	sp,0x31241
//  2c:	002180b3          	add	ra,gp,sp
//  30:	00212133          	slt	sp,sp,sp
//  34:	0020b1b3          	sltu	gp,ra,sp
//  38:	0021f0b3          	and	ra,gp,sp
//  3c:	00216133          	or	sp,sp,sp
//  40:	0020c1b3          	xor	gp,ra,sp
//  44:	002190b3          	sll	ra,gp,sp
//  48:	0020d1b3          	srl	gp,ra,sp
//  4c:	402180b3          	sub	ra,gp,sp
//  50:	4020d1b3          	sra	gp,ra,sp
//  5c:	00609463          	bne	ra,t1,0x64
//  64:	00510463          	beq	sp,t0,0x6c
//  6c:	0041d463          	ble	tp,gp,0x74
//  74:	00327463          	bleu	gp,tp,0x7c
//  7c:	0022c463          	blt	t0,sp,0x84
//  84:	00136463          	bltu	t1,ra,0x8c
//  8c:	00102083          	lw	ra,1(zero) # 0x1
//  90:	00201103          	lh	sp,2(zero) # 0x2
//  94:	00305183          	lhu	gp,3(zero) # 0x3
//  98:	00400203          	lb	tp,4(zero) # 0x4
//  9c:	00504283          	lbu	t0,5(zero) # 0x5
//  a0:	0000a0a3          	sw	zero,1(ra) # 0x3412001
//  a4:	00011123          	sh	zero,2(sp) # 0x3124102a
//  a8:	000181a3          	sb	zero,3(gp)
//  ac:	0ff0000f          	fence
//  b0:	0000100f          	fence.i
//  b4:	c00010f3          	csrrw	ra,cycle,zero
//  b8:	c8102173          	rdtimeh	sp
//  bc:	c02031f3          	csrrc	gp,instret,zero
//  c0:	c000d273          	csrrwi	tp,cycle,1
//  c4:	c01162f3          	csrrsi	t0,time,2
//  c8:	c821f373          	csrrci	t1,instreth,3
//  cc:	00000073          	ecall
//  d0:	00100073          	ebreak
//  d4:	02208033          	mul	zero,ra,sp
//  d8:	020110b3          	mulh	ra,sp,zero
//  dc:	02103133          	mulhu	sp,zero,ra
//  e0:	0220a033          	mulhsu	zero,ra,sp
//  e4:	020140b3          	div	ra,sp,zero
//  e8:	02105133          	divu	sp,zero,ra
//  ec:	0220e033          	rem	zero,ra,sp
//  f0:	020170b3          	remu	ra,sp,zero

    // TODO: virtual machine stuff
}
