#[derive(Debug,Clone)]
pub enum Labels <'input> {
    NLabel(&'input str),
    WLabel(&'input str),
}

// TODO: convert Reg + Csrs to enum or something
#[derive(Debug,Clone)]
pub enum Args <'input> {
    Num(u32),
    Reg(&'input str),
    Csr(&'input str),
    Lab(Labels<'input>),
}

#[derive(Debug)]
pub enum AsmLine <'input> {
    Lab(Labels<'input>),
    Ins(&'input str, Vec<Args <'input>>),
    Lns(Labels<'input>, &'input str, Vec<Args <'input>>),
}



// TODO: not sure if these pair of func is good or if it should go back into the parser?
pub fn is_csr(csr: &str) -> bool {
    match csr {
        "CYCLE"   | "CYCLEH"    => true,
        "TIME"    | "TIMEH"     => true,
        "INSTRET" | "INSTRETH"  => true,
        _                       => false,
    }
}

// TODO: not sure if these pair of func is good or if it should go back into the parser?
pub fn map_reg(reg: &str) -> &str {
    match reg {
        "zero"  => "x0",
        "ra"    => "x1",
        "sp"    => "x2",
        "gp"    => "x3",
        "tp"    => "x4",
        "t0"    => "x5",
        "t1"    => "x6",
        "t2"    => "x7",
        "s0"    => "x8",
        "fp"    => "x8",
        "s1"    => "x9",
        "a0"    => "x10",
        "a1"    => "x11",
        "a2"    => "x12",
        "a3"    => "x13",
        "a4"    => "x14",
        "a5"    => "x15",
        "a6"    => "x16",
        "a7"    => "x17",
        "s2"    => "x18",
        "s3"    => "x19",
        "s4"    => "x20",
        "s5"    => "x21",
        "s6"    => "x22",
        "s7"    => "x23",
        "s8"    => "x24",
        "s9"    => "x25",
        "s10"   => "x26",
        "s11"   => "x27",
        "t3"    => "x28",
        "t4"    => "x29",
        "t5"    => "x30",
        "t6"    => "x31",
        // x[0-9]+ style
        _ => reg,
    }
}
