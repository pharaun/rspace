#[test]
fn beq_inst() {
    let neg1: u32 = (-1 as i32) as u32;
    // Branch tests
    TEST_BR2_OP_TAKEN( 2, "beq",  0,  0 );
    TEST_BR2_OP_TAKEN( 3, "beq",  1,  1 );
    TEST_BR2_OP_TAKEN( 4, "beq", neg1, neg1 );

    TEST_BR2_OP_NOTTAKEN( 5, "beq",  0,  1 );
    TEST_BR2_OP_NOTTAKEN( 6, "beq",  1,  0 );
    TEST_BR2_OP_NOTTAKEN( 7, "beq", neg1,  1 );
    TEST_BR2_OP_NOTTAKEN( 8, "beq",  1, neg1 );
}
