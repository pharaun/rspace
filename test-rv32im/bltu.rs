#[test]
fn bltu_inst() {
    // Branch tests
    TEST_BR2_OP_TAKEN( 2, "bltu", 0x00000000, 0x00000001 );
    TEST_BR2_OP_TAKEN( 3, "bltu", 0xfffffffe, 0xffffffff );
    TEST_BR2_OP_TAKEN( 4, "bltu", 0x00000000, 0xffffffff );

    TEST_BR2_OP_NOTTAKEN( 5, "bltu", 0x00000001, 0x00000000 );
    TEST_BR2_OP_NOTTAKEN( 6, "bltu", 0xffffffff, 0xfffffffe );
    TEST_BR2_OP_NOTTAKEN( 7, "bltu", 0xffffffff, 0x00000000 );
    TEST_BR2_OP_NOTTAKEN( 8, "bltu", 0x80000000, 0x7fffffff );
}
