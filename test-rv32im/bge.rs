#[test]
fn bge_inst() {
    let neg1: u32 = (-1 as i32) as u32;
    let neg2: u32 = (-2 as i32) as u32;
    // Branch tests
    TEST_BR2_OP_TAKEN( 2, "bge",  0,  0 );
    TEST_BR2_OP_TAKEN( 3, "bge",  1,  1 );
    TEST_BR2_OP_TAKEN( 4, "bge", neg1, neg1 );
    TEST_BR2_OP_TAKEN( 5, "bge",  1,  0 );
    TEST_BR2_OP_TAKEN( 6, "bge",  1, neg1 );
    TEST_BR2_OP_TAKEN( 7, "bge", neg1, neg2 );

    TEST_BR2_OP_NOTTAKEN(  8, "bge",  0,  1 );
    TEST_BR2_OP_NOTTAKEN(  9, "bge", neg1,  1 );
    TEST_BR2_OP_NOTTAKEN( 10, "bge", neg2, neg1 );
    TEST_BR2_OP_NOTTAKEN( 11, "bge", neg2,  1 );
}
