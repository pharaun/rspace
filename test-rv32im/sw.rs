#[test]
fn sw_inst() {
  TEST_ST_OP(2, "lw", "sw", 0x0000000000aa00aa, 0,  "tdat");
  TEST_ST_OP(3, "lw", "sw", 0xffffffffaa00aa00, 4,  "tdat");
  TEST_ST_OP(4, "lw", "sw", 0x000000000aa00aa0, 8,  "tdat");
  TEST_ST_OP(5, "lw", "sw", 0xffffffffa00aa00a, 12, "tdat");

  // Test with negative offset
  let neg12: u32 = (-12 as i32) as u32;
  let neg8: u32 = (-8 as i32) as u32;
  let neg4: u32 = (-4 as i32) as u32;

  TEST_ST_OP(6, "lw", "sw", 0x0000000000aa00aa, neg12, "tdat8");
  TEST_ST_OP(7, "lw", "sw", 0xffffffffaa00aa00,  neg8, "tdat8");
  TEST_ST_OP(8, "lw", "sw", 0x000000000aa00aa0,  neg4, "tdat8");
  TEST_ST_OP(9, "lw", "sw", 0xffffffffa00aa00a,     0, "tdat8");
}
