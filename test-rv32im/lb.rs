#[test]
fn lb_inst() {
  TEST_LD_OP(2, "lb", 0xffffffffffffffff, 0, "tdat");
  TEST_LD_OP(3, "lb", 0x0000000000000000, 1, "tdat");
  TEST_LD_OP(4, "lb", 0xfffffffffffffff0, 2, "tdat");
  TEST_LD_OP(5, "lb", 0x000000000000000f, 3, "tdat");

  // Test with negative offset
  let neg3: u32 = (-3 as i32) as u32;
  let neg2: u32 = (-2 as i32) as u32;
  let neg1: u32 = (-1 as i32) as u32;

  TEST_LD_OP(6, "lb", 0xffffffffffffffff, neg3, "tdat4");
  TEST_LD_OP(7, "lb", 0x0000000000000000, neg2, "tdat4");
  TEST_LD_OP(8, "lb", 0xfffffffffffffff0, neg1, "tdat4");
  TEST_LD_OP(9, "lb", 0x000000000000000f,    0, "tdat4");

  // Bypassing tests
  TEST_LD_DEST_BYPASS(12, 0, "lb", 0xfffffffffffffff0, 1, "tdat2");
  TEST_LD_DEST_BYPASS(13, 1, "lb", 0x000000000000000f, 1, "tdat3");
  TEST_LD_DEST_BYPASS(14, 2, "lb", 0x0000000000000000, 1, "tdat1");

  TEST_LD_SRC1_BYPASS(15, 0, "lb", 0xfffffffffffffff0, 1, "tdat2");
  TEST_LD_SRC1_BYPASS(16, 1, "lb", 0x000000000000000f, 1, "tdat3");
  TEST_LD_SRC1_BYPASS(17, 2, "lb", 0x0000000000000000, 1, "tdat1");
}
