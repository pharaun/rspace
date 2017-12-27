#[derive(Debug)]
pub enum Args <'input> {
    Num(u32),
    Reg(&'input str),
}
