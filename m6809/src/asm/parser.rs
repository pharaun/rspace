use nom::{
  Parser,
  IResult,
  bytes::complete::{
      tag,
      take_while1,
  },
  character::complete::multispace0,
  combinator::{
      map_res,
      opt,
      map,
  },
  branch::alt,
  sequence::{
      preceded,
      delimited,
  },
  multi::{
      many0,
      separated_list1,
  },
};

use crate::asm::ast;


pub fn implict(input: &str) -> IResult<&str, ast::IInst> {
    alt((
        map(tag("ABX"),  |_| ast::IInst::ABX),
        map(tag("DAA"),  |_| ast::IInst::DAA),
        map(tag("MUL"),  |_| ast::IInst::MUL),
        map(tag("NOP"),  |_| ast::IInst::NOP),
        map(tag("RTI"),  |_| ast::IInst::RTI),
        map(tag("RTS"),  |_| ast::IInst::RTS),
        map(tag("SEX"),  |_| ast::IInst::SEX),
        map(tag("SEXW"), |_| ast::IInst::SEXW),
        map(tag("SWI"),  |_| ast::IInst::SWI),
        map(tag("SWI2"), |_| ast::IInst::SWI2),
        map(tag("SWI3"), |_| ast::IInst::SWI3),
        map(tag("SYNC"), |_| ast::IInst::SYNC),
    )).parse(input)
}

fn implict_imm(input: &str) -> IResult<&str, (ast::ImmInst, u8)> {
    let (input, inst) = alt((
        map(tag("ANDCC"), |_| ast::ImmInst::ANDCC),
        map(tag("BITMD"), |_| ast::ImmInst::BITMD),
        map(tag("CWAI"),  |_| ast::ImmInst::CWAI),
        map(tag("LDMD"),  |_| ast::ImmInst::LDMD),
        map(tag("ORCC"),  |_| ast::ImmInst::ORCC),
    )).parse(input)?;

    map(u8_imm, |imm| (inst, imm)).parse(input)
}

fn u8_imm(input: &str) -> IResult<&str, u8> {
    Ok((input, 0))
}

fn u16_imm(input: &str) -> IResult<&str, u8> {
    Ok((input, 0))
}

fn u32_imm(input: &str) -> IResult<&str, u8> {
    Ok((input, 0))
}


#[cfg(test)]
mod test_parser {
    use super::*;

    #[test]
    fn test_implict() {
        assert_eq!(implict("ABX"), Ok(("", ast::IInst::ABX)));
    }

    #[test]
    fn test_imm_implict() {
        assert_eq!(implict_imm("ANDCC 0xFF"), Ok(("", (ast::ImmInst::ANDCC, 0xFF))));
    }
}
