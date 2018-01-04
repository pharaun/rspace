// TODO: convert Reg + Csrs to enum or something
#[derive(Debug)]
pub enum Args <'input> {
    Num(u32),
    Reg(&'input str),
    Csr(&'input str),
}
