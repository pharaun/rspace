#[test]
fn sb_inst() {
  TEST_ST_OP(2, "lb", "sb", 0xffffffffffffffaa, 0, "tdat");
  TEST_ST_OP(3, "lb", "sb", 0x0000000000000000, 1, "tdat");
  TEST_ST_OP(4, "lh", "sb", 0xffffffffffffefa0, 2, "tdat");
  TEST_ST_OP(5, "lb", "sb", 0x000000000000000a, 3, "tdat");

  // Test with negative offset
  let neg3: u32 = (-3 as i32) as u32;
  let neg2: u32 = (-2 as i32) as u32;
  let neg1: u32 = (-1 as i32) as u32;

  TEST_ST_OP(6, "lb", "sb", 0xffffffffffffffaa, neg3, "tdat8");
  TEST_ST_OP(7, "lb", "sb", 0x0000000000000000, neg2, "tdat8");
  TEST_ST_OP(8, "lb", "sb", 0xffffffffffffffa0, neg1, "tdat8");
  TEST_ST_OP(9, "lb", "sb", 0x000000000000000a,    0, "tdat8");
}
