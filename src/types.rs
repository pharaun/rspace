#[derive(Debug,Clone)]
pub enum Labels <'input> {
    NLabel(&'input str),
    WLabel(&'input str),
}

// TODO: convert Reg + Csrs to enum or something
#[derive(Debug)]
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
