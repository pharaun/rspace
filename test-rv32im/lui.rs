#[test]
fn lui_inst() {
    TEST_LUI(2, 0x00000000, 0x00000, 0);
    TEST_LUI(3, 0xfffff800, 0xfffff, 1);
    TEST_LUI(4, 0x000007ff, 0x7ffff, 20);
    TEST_LUI(5, 0xfffff800, 0x80000, 20);
    //TEST_LUI(6, 0x0000000000000000, 0x80000, 0); // ???

    TEST_LUI(11, 0x12345000, 0x12345, 0);
    TEST_LUI(12, 0x82345000, 0x82345, 0);
}
