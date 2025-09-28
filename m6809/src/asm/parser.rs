use nom::{
  Parser,
  IResult,
  bytes::complete::{
      tag,
      take_while1,
  },
  character::one_of,
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
      pair,
  },
  multi::{
      many0,
      separated_list1,
  },
};
use num_traits::Num;

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

fn sign(input: &str) -> IResult<&str, char> {
    alt((
        one_of("+-"),
        success('+'),
    )).parse(input)
}

fn number<T: Num>(input: &str) -> IResult<&str, Result<T, T::FromStrRadixErr>> {
    let (input, (sign, radix)) = pair(sign, radix).parse(input)?;
    let (input, digit) = take_while1(|c: char| c.is_digit(radix)).parse(input)?;

    Ok((
        input,
        T::from_str_radix(&(sign.to_string() + digit), radix),
    ))
}

fn from_str<T: Num>(s: &str) -> Result<T, T::FromStrRadixErr> {
    T::from_str_radix(s, 16)
}





#[cfg(test)]
mod test_parser {
    use super::*;


    #[test]
    fn test_from_str() {
        assert_eq!(0xFF, from_str::<u8>("FF").unwrap());
        assert_eq!(0xFF, from_str::<u8>("-FF").unwrap());
        assert_eq!(0xFF, from_str::<u16>("FF").unwrap());
        assert_eq!(0xFF, from_str::<u32>("FF").unwrap());
        assert_eq!(0xFFu8 as i8, from_str::<i8>("-01").unwrap());
    }



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
