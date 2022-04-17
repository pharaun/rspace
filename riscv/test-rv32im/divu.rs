#[test]
fn divu_inst() {
    let neg1: u32 = (-1 as i32) as u32;
    let neg6: u32 = (-6 as i32) as u32;
    let neg20: u32 = (-20 as i32) as u32;

    // Arithmetic tests
    TEST_RR_OP( 2, "divu",                   3,  20,   6 );
    TEST_RR_OP( 3, "divu",           715827879, neg20,   6 );
    TEST_RR_OP( 4, "divu",                   0,  20,  neg6 );
    TEST_RR_OP( 5, "divu",                   0, neg20,  neg6 );

    TEST_RR_OP( 6, "divu", neg1<<31, neg1<<31,  1 );
    TEST_RR_OP( 7, "divu",     0,  neg1<<31, neg1 );

    TEST_RR_OP( 8, "divu", neg1, neg1<<31, 0 );
    TEST_RR_OP( 9, "divu", neg1,      1, 0 );
    TEST_RR_OP(10, "divu", neg1,      0, 0 );
}
