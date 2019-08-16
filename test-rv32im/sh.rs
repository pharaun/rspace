#[test]
fn sh_inst() {
  TEST_ST_OP(2, "lh", "sh", 0x00000000000000aa, 0, "tdat");
  TEST_ST_OP(3, "lh", "sh", 0xffffffffffffaa00, 2, "tdat");
  TEST_ST_OP(4, "lw", "sh", 0xffffffffbeef0aa0, 4, "tdat");
  TEST_ST_OP(5, "lh", "sh", 0xffffffffffffa00a, 6, "tdat");

  // Test with negative offset
  let neg6: u32 = (-6 as i32) as u32;
  let neg4: u32 = (-4 as i32) as u32;
  let neg2: u32 = (-2 as i32) as u32;

  TEST_ST_OP(6, "lh", "sh", 0x00000000000000aa, neg6, "tdat8");
  TEST_ST_OP(7, "lh", "sh", 0xffffffffffffaa00, neg4, "tdat8");
  TEST_ST_OP(8, "lh", "sh", 0x0000000000000aa0, neg2, "tdat8");
  TEST_ST_OP(9, "lh", "sh", 0xffffffffffffa00a,    0, "tdat8");
}
