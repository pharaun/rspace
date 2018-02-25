// TODO: convert Reg + Csrs to enum or something
#[derive(Debug)]
pub enum Args <'input> {
    Num(u32),
    Reg(&'input str),
    Csr(&'input str),
    Lab(&'input str),
}

#[derive(Debug)]
pub enum AsmLine <'input> {
    Lab(&'input str),
    Ins(&'input str, Vec<Args <'input>>),
    Lns(&'input str, &'input str, Vec<Args <'input>>),
}
