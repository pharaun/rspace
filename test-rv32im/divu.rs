// Arithmetic tests
TEST_RR_OP( 2, "divu",                   3,  20,   6 );
TEST_RR_OP( 3, "divu",           715827879, -20,   6 );
TEST_RR_OP( 4, "divu",                   0,  20,  -6 );
TEST_RR_OP( 5, "divu",                   0, -20,  -6 );

TEST_RR_OP( 6, "divu", -1<<31, -1<<31,  1 );
TEST_RR_OP( 7, "divu",     0,  -1<<31, -1 );

TEST_RR_OP( 8, "divu", -1, -1<<31, 0 );
TEST_RR_OP( 9, "divu", -1,      1, 0 );
TEST_RR_OP(10, "divu", -1,      0, 0 );
