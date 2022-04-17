#[test]
fn div_inst() {
    let neg1: u32 = (-1 as i32) as u32;
    let neg3: u32 = (-3 as i32) as u32;
    let neg6: u32 = (-6 as i32) as u32;
    let neg20: u32 = (-20 as i32) as u32;

    // Arithmetic tests
    TEST_RR_OP( 2, "div",  3,  20,   6 );
    TEST_RR_OP( 3, "div", neg3, neg20,   6 );
    TEST_RR_OP( 4, "div", neg3,  20,  neg6 );
    TEST_RR_OP( 5, "div",  3, neg20,  neg6 );

    TEST_RR_OP( 6, "div", neg1<<31, neg1<<31,  1 );
    TEST_RR_OP( 7, "div", neg1<<31, neg1<<31, neg1 );

    TEST_RR_OP( 8, "div", neg1, neg1<<31, 0 );
    TEST_RR_OP( 9, "div", neg1,      1, 0 );
    TEST_RR_OP(10, "div", neg1,      0, 0 );
}
