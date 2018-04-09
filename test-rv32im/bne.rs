#[test]
fn bne_inst() {
    let neg1: u32 = (-1 as i32) as u32;
    // Branch tests
    TEST_BR2_OP_TAKEN( 2, "bne",  0,  1 );
    TEST_BR2_OP_TAKEN( 3, "bne",  1,  0 );
    TEST_BR2_OP_TAKEN( 4, "bne", neg1,  1 );
    TEST_BR2_OP_TAKEN( 5, "bne",  1, neg1 );

    TEST_BR2_OP_NOTTAKEN( 6, "bne",  0,  0 );
    TEST_BR2_OP_NOTTAKEN( 7, "bne",  1,  1 );
    TEST_BR2_OP_NOTTAKEN( 8, "bne", neg1, neg1 );
}
