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
      value,
      success,
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
        value(ast::IInst::ABX,  tag("ABX")),
        value(ast::IInst::DAA,  tag("DAA")),
        value(ast::IInst::MUL,  tag("MUL")),
        value(ast::IInst::NOP,  tag("NOP")),
        value(ast::IInst::RTI,  tag("RTI")),
        value(ast::IInst::RTS,  tag("RTS")),
        value(ast::IInst::SEX,  tag("SEX")),
        value(ast::IInst::SEXW, tag("SEXW")),
        value(ast::IInst::SWI,  tag("SWI")),
        value(ast::IInst::SWI2, tag("SWI2")),
        value(ast::IInst::SWI3, tag("SWI3")),
        value(ast::IInst::SYNC, tag("SYNC")),
    )).parse(input)
}

fn implict_imm(input: &str) -> IResult<&str, (ast::ImmInst, u8)> {
    let (input, inst) = alt((
        value(ast::ImmInst::ANDCC, tag("ANDCC")),
        value(ast::ImmInst::BITMD, tag("BITMD")),
        value(ast::ImmInst::CWAI,  tag("CWAI")),
        value(ast::ImmInst::LDMD,  tag("LDMD")),
        value(ast::ImmInst::ORCC,  tag("ORCC")),
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

fn radix(input: &str) -> IResult<&str, u32> {
    alt((
        value(16, tag("0x")),
        value(8,  tag("0o")),
        value(2,  tag("0b")),
        success(10),
    )).parse(input)
}

fn number(input: &str) -> IResult<&str, u32> {
    let (input, radix) = radix.parse(input)?;

    let is_digit = |c: char| c.is_digit(radix);
    let from_digit = |s: &str| u32::from_str_radix(s, radix);
    map_res(take_while1(is_digit), from_digit).parse(input)
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

    #[test]
    fn test_radix() {
        assert_eq!(radix("0xFF"), Ok(("FF", 16)));
    }

    #[test]
    fn test_radix_fallback() {
        assert_eq!(radix("123"), Ok(("123", 10)));
    }
}
