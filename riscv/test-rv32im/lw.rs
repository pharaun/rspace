#[test]
fn lw_inst() {
  TEST_LD_OP(2, "lw", 0x00ff00ff, 0,  "tdat");
  TEST_LD_OP(3, "lw", 0xff00ff00, 4,  "tdat");
  TEST_LD_OP(4, "lw", 0x0ff00ff0, 8,  "tdat");
  TEST_LD_OP(5, "lw", 0xf00ff00f, 12, "tdat");

  // Test with negative offset
  let neg12: u32 = (-12 as i32) as u32;
  let neg8: u32 = (-8 as i32) as u32;
  let neg4: u32 = (-4 as i32) as u32;

  TEST_LD_OP(6, "lw", 0x00ff00ff, neg12, "tdat4");
  TEST_LD_OP(7, "lw", 0xff00ff00, neg8,  "tdat4");
  TEST_LD_OP(8, "lw", 0x0ff00ff0, neg4,  "tdat4");
  TEST_LD_OP(9, "lw", 0xf00ff00f, 0,   "tdat4");

  // Bypassing tests
  TEST_LD_DEST_BYPASS(12, 0, "lw", 0x0ff00ff0, 4, "tdat2");
  TEST_LD_DEST_BYPASS(13, 1, "lw", 0xf00ff00f, 4, "tdat3");
  TEST_LD_DEST_BYPASS(14, 2, "lw", 0xff00ff00, 4, "tdat1");

  TEST_LD_SRC1_BYPASS(15, 0, "lw", 0x0ff00ff0, 4, "tdat2");
  TEST_LD_SRC1_BYPASS(16, 1, "lw", 0xf00ff00f, 4, "tdat3");
  TEST_LD_SRC1_BYPASS(17, 2, "lw", 0xff00ff00, 4, "tdat1");
}
