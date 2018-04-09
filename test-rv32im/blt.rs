#[test]
fn blt_inst() {
    let neg1: u32 = (-1 as i32) as u32;
    let neg2: u32 = (-2 as i32) as u32;
    // Branch tests
    TEST_BR2_OP_TAKEN( 2, "blt",  0,  1 );
    TEST_BR2_OP_TAKEN( 3, "blt", neg1,  1 );
    TEST_BR2_OP_TAKEN( 4, "blt", neg2, neg1 );

    TEST_BR2_OP_NOTTAKEN( 5, "blt",  1,  0 );
    TEST_BR2_OP_NOTTAKEN( 6, "blt",  1, neg1 );
    TEST_BR2_OP_NOTTAKEN( 7, "blt", neg1, neg2 );
    TEST_BR2_OP_NOTTAKEN( 8, "blt",  1, neg2 );
}
