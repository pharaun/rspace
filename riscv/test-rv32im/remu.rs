#[test]
fn remu_inst() {
    let neg1: u32 = (-1 as i32) as u32;
    let neg6: u32 = (-6 as i32) as u32;
    let neg20: u32 = (-20 as i32) as u32;

    // Arithmetic tests
    TEST_RR_OP( 2, "remu",   2,  20,   6 );
    TEST_RR_OP( 3, "remu",   2, neg20,   6 );
    TEST_RR_OP( 4, "remu",  20,  20,  neg6 );
    TEST_RR_OP( 5, "remu", neg20, neg20,  neg6 );

    TEST_RR_OP( 6, "remu",      0, neg1<<31,  1 );
    TEST_RR_OP( 7, "remu", neg1<<31, neg1<<31, neg1 );

    TEST_RR_OP( 8, "remu", neg1<<31, neg1<<31, 0 );
    TEST_RR_OP( 9, "remu",      1,      1, 0 );
    TEST_RR_OP(10, "remu",      0,      0, 0 );
}
