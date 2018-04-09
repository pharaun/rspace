#[cfg(test)]
mod op_tests {
    use asm;
    use super::*;
    use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};

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

    mod rr_op_tests {
        use super::*;

        include!("../test-rv32im/add.rs");
        include!("../test-rv32im/sub.rs");
        include!("../test-rv32im/xor.rs");
        include!("../test-rv32im/or.rs");
        include!("../test-rv32im/and.rs");
        include!("../test-rv32im/sll.rs");
        include!("../test-rv32im/srl.rs");
        include!("../test-rv32im/sra.rs");
        include!("../test-rv32im/slt.rs");
        include!("../test-rv32im/sltu.rs");
        include!("../test-rv32im/div.rs");
        include!("../test-rv32im/divu.rs");
        include!("../test-rv32im/mul.rs");
        include!("../test-rv32im/mulh.rs");
        include!("../test-rv32im/mulhsu.rs");
        include!("../test-rv32im/mulhu.rs");
        include!("../test-rv32im/rem.rs");
        include!("../test-rv32im/remu.rs");

        // TODO: make this more flexible (ie list of reg + value, plus expected value+reg afterward)
        fn TEST_RR_OP(_test: u8, op: &str, r: u32, a: u32, b: u32) {
            // load the rom
            let mut vm = Emul32::new_with_rom(generate_rom(&format!("{} x3 x1 x2", op)));

            // Load the registers
            vm.reg[1] = a;
            vm.reg[2] = b;

            // Validate
            assert_eq!(vm.reg[3], 0);

            // Run
            vm.run();

            // Validate
            assert_eq!(vm.reg[1], a);
            assert_eq!(vm.reg[2], b);
            assert_eq!(vm.reg[3], r);
        }

        fn TEST_RR_SRC1_EQ_DEST(_test: u8, op: &str, res: u32, a: u32, b: u32) {
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

        fn TEST_RR_SRC2_EQ_DEST(_test: u8, op: &str, res: u32, a: u32, b: u32) {
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

        fn TEST_RR_SRC12_EQ_DEST(_test: u8, op: &str, res: u32, a: u32) {
            // load the rom
            let mut vm = Emul32::new_with_rom(generate_rom(&format!("{} x1 x1 x1", op)));

            // Load the registers
            vm.reg[1] = a;

            // Run
            vm.run();

            // Validate
            assert_eq!(vm.reg[1], res);
        }

        fn TEST_RR_ZEROSRC1(_test: u8, op: &str, r: u32, b: u32) {
            // load the rom
            let mut vm = Emul32::new_with_rom(generate_rom(&format!("{} x1 x0 x2", op)));

            // Load the registers
            vm.reg[2] = b;

            // Validate
            assert_eq!(vm.reg[1], 0);

            // Run
            vm.run();

            // Validate
            assert_eq!(vm.reg[0], 0);
            assert_eq!(vm.reg[1], r);
            assert_eq!(vm.reg[2], b);
        }

        fn TEST_RR_ZEROSRC2(_test: u8, op: &str, r: u32, a: u32) {
            // load the rom
            let mut vm = Emul32::new_with_rom(generate_rom(&format!("{} x1 x2 x0", op)));

            // Load the registers
            vm.reg[2] = a;

            // Validate
            assert_eq!(vm.reg[1], 0);

            // Run
            vm.run();

            // Validate
            assert_eq!(vm.reg[0], 0);
            assert_eq!(vm.reg[1], r);
            assert_eq!(vm.reg[2], a);
        }

        fn TEST_RR_ZEROSRC12(_test: u8, op: &str, r: u32) {
            // load the rom
            let mut vm = Emul32::new_with_rom(generate_rom(&format!("{} x1 x0 x0", op)));

            // Validate
            assert_eq!(vm.reg[1], 0);

            // Run
            vm.run();

            // Validate
            assert_eq!(vm.reg[0], 0);
            assert_eq!(vm.reg[1], r);
        }

        fn TEST_RR_ZERODEST(_test: u8, op: &str, a: u32, b: u32) {
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

        fn TEST_RR_DEST_BYPASS(test: u8, _n: u32, op: &str, res: u32, a: u32, b: u32) {
            TEST_RR_OP(test, op, res, a, b);
        }

        fn TEST_RR_SRC12_BYPASS(test: u8, _n1: u32, _n2: u32, op: &str, res: u32, a: u32, b: u32) {
            TEST_RR_OP(test, op, res, a, b);
        }

        fn TEST_RR_SRC21_BYPASS(test: u8, _n1: u32, _n2: u32, op: &str, res: u32, a: u32, b: u32) {
            TEST_RR_OP(test, op, res, a, b);
        }

        fn TEST_SRL(n: u8, v: u32, a: u32) {
            let xlen = 32;
            let xlen_mask: u32 = 1 << (xlen - 1) << 1;
            let xlen_mask_two: u32 = xlen_mask.wrapping_sub(1);

            TEST_RR_OP(n, "srl", (v & xlen_mask_two) >> (a as usize), v, a)
        }
    }

    mod imm_op_tests {
        use super::*;

        include!("../test-rv32im/slli.rs");
        include!("../test-rv32im/srli.rs");
        include!("../test-rv32im/srai.rs");
        include!("../test-rv32im/addi.rs");
        include!("../test-rv32im/andi.rs");
        include!("../test-rv32im/ori.rs");
        include!("../test-rv32im/xori.rs");
        include!("../test-rv32im/slti.rs");
        include!("../test-rv32im/sltiu.rs");


        fn TEST_IMM_OP(_test: u8, op: &str, res: u32, a: u32, imm: u32) {
            // load the rom
            let mut vm = Emul32::new_with_rom(generate_rom(&format!("{} x2 x1 0x{:08x}", op, imm)));

            // Load the registers
            vm.reg[1] = a;

            // Validate
            assert_eq!(vm.reg[2], 0);

            // Run
            vm.run();

            // Validate
            assert_eq!(vm.reg[1], a);
            assert_eq!(vm.reg[2], res);
        }

        fn TEST_IMM_SRC1_EQ_DEST(_test: u8, op: &str, res: u32, a: u32, imm: u32) {
            // load the rom
            let mut vm = Emul32::new_with_rom(generate_rom(&format!("{} x1 x1 0x{:08x}", op, imm)));

            // Load the registers
            vm.reg[1] = a;

            // Run
            vm.run();

            // Validate
            assert_eq!(vm.reg[1], res);
        }

        fn TEST_IMM_ZEROSRC1(_test: u8, op: &str, res: u32, imm: u32) {
            // load the rom
            let mut vm = Emul32::new_with_rom(generate_rom(&format!("{} x1 x0 0x{:08x}", op, imm)));

            // Validate
            assert_eq!(vm.reg[2], 0);

            // Run
            vm.run();

            // Validate
            assert_eq!(vm.reg[0], 0);
            assert_eq!(vm.reg[1], res);
        }

        fn TEST_IMM_ZERODEST(_test: u8, op: &str, a: u32, imm: u32) {
            // load the rom
            let mut vm = Emul32::new_with_rom(generate_rom(&format!("{} x0 x1 0x{:08x}", op, imm)));

            // Load the registers
            vm.reg[1] = a;

            // Run
            vm.run();

            // Validate
            assert_eq!(vm.reg[0], 0);
            assert_eq!(vm.reg[1], a);
        }

        fn TEST_IMM_DEST_BYPASS(test: u8, _n: u32, op: &str, res: u32, a: u32, imm: u32) {
            TEST_IMM_OP(test, op, res, a, imm);
        }

        fn TEST_IMM_SRC1_BYPASS(test: u8, _n: u32, op: &str, res: u32, a: u32, imm: u32) {
            TEST_IMM_OP(test, op, res, a, imm);
        }

        fn TEST_SRL(n: u8, v: u32, a: u32) {
            let xlen = 32;
            let xlen_mask: u32 = 1 << (xlen - 1) << 1;
            let xlen_mask_two: u32 = xlen_mask.wrapping_sub(1);

            TEST_IMM_OP(n, "srli", (v & xlen_mask_two) >> (a as usize), v, a)
        }
    }

    mod branch_tests {
        use super::*;

        include!("../test-rv32im/beq.rs");
        include!("../test-rv32im/bge.rs");
        include!("../test-rv32im/bgeu.rs");
        include!("../test-rv32im/blt.rs");
        include!("../test-rv32im/bltu.rs");
        include!("../test-rv32im/bne.rs");


        fn TEST_BR2_OP_TAKEN(_test: u8, inst: &str, val1: u32, val2: u32) {
            // load the rom
            let mut vm = Emul32::new_with_rom(
                generate_rom(
                    &format!(
                        "\n
                        1: addi x3 x0 0x1\n
                        {} x1 x2 3f\n
                        addi x4 x0 0x1\n
                        2: addi x5 x0 0x1\n
                        {} x1 x2 4f\n
                        addi x6 x0 0x1\n
                        3: addi x7 x0 0x1\n
                        {} x1 x2 2b\n
                        addi x8 x0 0x1\n
                        4: addi x9 x0 0x1\n
                        addi x10 x0 0x1",
                        inst, inst, inst
                    )
                )
            );

            // Load the registers
            vm.reg[1] = val1;
            vm.reg[2] = val2;

            // Validate - A bit complicated, but basically we want to always take the branch
            // The sentinel here is x3, x4, and x5, and x6 to confirm completion
            assert_eq!(vm.reg[3], 0);
            assert_eq!(vm.reg[4], 0);
            assert_eq!(vm.reg[5], 0);
            assert_eq!(vm.reg[6], 0);
            assert_eq!(vm.reg[7], 0);
            assert_eq!(vm.reg[8], 0);
            assert_eq!(vm.reg[9], 0);
            assert_eq!(vm.reg[10], 0);

            // Run
            vm.run();

            // Validate
            assert_eq!(vm.reg[1], val1);
            assert_eq!(vm.reg[2], val2);
            assert_eq!(vm.reg[3], 0x1); // Jumped to
            assert_eq!(vm.reg[4], 0); // Jumped over
            assert_eq!(vm.reg[5], 0x1); // jumped to
            assert_eq!(vm.reg[6], 0); // jumped over
            assert_eq!(vm.reg[7], 0x1); // jumped to
            assert_eq!(vm.reg[8], 0); // jumped over
            assert_eq!(vm.reg[9], 0x1); // jumped to
            assert_eq!(vm.reg[10], 0x1); // Finished
        }

        fn TEST_BR2_OP_NOTTAKEN(_test: u8, inst: &str, val1: u32, val2: u32) {
            // load the rom
            let mut vm = Emul32::new_with_rom(
                generate_rom(
                    &format!(
                        "\n
                        1: addi x3 x0 0x1\n
                        {} x1 x2 3f\n
                        addi x4 x0 0x1\n
                        2: addi x5 x0 0x1\n
                        {} x1 x2 4f\n
                        addi x6 x0 0x1\n
                        3: addi x7 x0 0x1\n
                        {} x1 x2 2b\n
                        addi x8 x0 0x1\n
                        4: addi x9 x0 0x1\n
                        addi x10 x0 0x1",
                        inst, inst, inst
                    )
                )
            );

            // Load the registers
            vm.reg[1] = val1;
            vm.reg[2] = val2;

            // Validate - A bit complicated, but basically we want to always not take the branch
            // The sentinel here is x3, x4, and x5, and x6 to confirm completion
            assert_eq!(vm.reg[3], 0);
            assert_eq!(vm.reg[4], 0);
            assert_eq!(vm.reg[5], 0);
            assert_eq!(vm.reg[6], 0);
            assert_eq!(vm.reg[7], 0);
            assert_eq!(vm.reg[8], 0);
            assert_eq!(vm.reg[9], 0);
            assert_eq!(vm.reg[10], 0);

            // Run
            vm.run();

            // Validate
            assert_eq!(vm.reg[1], val1);
            assert_eq!(vm.reg[2], val2);
            assert_eq!(vm.reg[3], 0x1);
            assert_eq!(vm.reg[4], 0x1);
            assert_eq!(vm.reg[5], 0x1);
            assert_eq!(vm.reg[6], 0x1);
            assert_eq!(vm.reg[7], 0x1);
            assert_eq!(vm.reg[8], 0x1);
            assert_eq!(vm.reg[9], 0x1);
            assert_eq!(vm.reg[10], 0x1);
        }
    }

    mod misc_tests {
        use super::*;

        include!("../test-rv32im/lui.rs");

        #[test]
        fn jal_inst() {
            // load the rom
            let mut vm = Emul32::new_with_rom(
                generate_rom(
                    // TODO:
                    // LA x5 ta (to grab the address of ta to compare)
                    "addi x1 x0 0x1\n
                    jal x2 ta\n
                    addi x3 x0 0x1\n
                    ta: addi x4 x0 0x1\n
                    "
                    //auipc x5 ta\n
                    //addi x5 x5 ta"
                )
            );

            // Validate
            assert_eq!(vm.reg[1], 0);
            assert_eq!(vm.reg[2], 0);
            assert_eq!(vm.reg[3], 0);
            assert_eq!(vm.reg[4], 0);
            assert_eq!(vm.reg[5], 0);

            // Run
            vm.run();

            // Validate
            assert_eq!(vm.reg[1], 0x1);
            assert_eq!(vm.reg[2], 0x8); // Hardcode address, re-validate (la - auipc+addi)
            //assert_eq!(vm.reg[2], vm.reg[5]);
            assert_eq!(vm.reg[3], 0);
            assert_eq!(vm.reg[4], 0x1);
        }

        fn TEST_LUI(_test: u8, res: u32, num: u32, shift: u32) {
            // load the rom
            let mut vm = Emul32::new_with_rom(
                generate_rom(
                    &format!(
                        "lui x1 {}\n
                        srai x1 x1 {}",
                        num, shift
                    )
                )
            );

            // Validate
            assert_eq!(vm.reg[1], 0);

            // Run
            vm.run();

            // Validate
            assert_eq!(vm.reg[1], res);
        }
    }

        // AUIPC
        // JALR
        // STORE
        // LOADs
        // Lower Priority:
        // SYNCH (fence)
        // COUNTERS (CSR)
        // SYSTEM (scall/sbreak)/(ebreak/ecall)
}
