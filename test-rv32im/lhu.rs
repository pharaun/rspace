#[test]
fn lhu_inst() {
  TEST_LD_OP(2, "lhu", 0x00000000000000ff, 0, "tdat");
  TEST_LD_OP(3, "lhu", 0x000000000000ff00, 2, "tdat");
  TEST_LD_OP(4, "lhu", 0x0000000000000ff0, 4, "tdat");
  TEST_LD_OP(5, "lhu", 0x000000000000f00f, 6, "tdat");

  // Test with negative offset
  let neg6: u32 = (-6 as i32) as u32;
  let neg4: u32 = (-4 as i32) as u32;
  let neg2: u32 = (-2 as i32) as u32;

  TEST_LD_OP( 6, "lhu", 0x00000000000000ff, neg6, "tdat4");
  TEST_LD_OP( 7, "lhu", 0x000000000000ff00, neg4, "tdat4");
  TEST_LD_OP( 8, "lhu", 0x0000000000000ff0, neg2, "tdat4");
  TEST_LD_OP( 9, "lhu", 0x000000000000f00f,    0, "tdat4");

  // Bypassing tests
  TEST_LD_DEST_BYPASS(12, 0, "lhu", 0x0000000000000ff0, 2, "tdat2");
  TEST_LD_DEST_BYPASS(13, 1, "lhu", 0x000000000000f00f, 2, "tdat3");
  TEST_LD_DEST_BYPASS(14, 2, "lhu", 0x000000000000ff00, 2, "tdat1");

  TEST_LD_SRC1_BYPASS(15, 0, "lhu", 0x0000000000000ff0, 2, "tdat2");
  TEST_LD_SRC1_BYPASS(16, 1, "lhu", 0x000000000000f00f, 2, "tdat3");
  TEST_LD_SRC1_BYPASS(17, 2, "lhu", 0x000000000000ff00, 2, "tdat1");
}
