use nom::IResult;
use nom::Parser;
use nom::branch::alt;
use nom::bytes::complete::tag;
use nom::bytes::complete::take_while1;
use nom::character::complete::space1;
use nom::character::one_of;
use nom::combinator::map;
use nom::combinator::recognize;
use nom::combinator::success;
use nom::combinator::value;
use nom::error::Error;
use nom::sequence::pair;

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
    let (input, _) = space1.parse(input)?;

    map(number, |imm| (inst, imm)).parse(input)
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
    alt((one_of("+-"), success('+'))).parse(input)
}

fn number<T: Num<FromStrRadixErr = std::num::ParseIntError>>(input: &str) -> IResult<&str, T> {
    let (input, (sign, radix)) = pair(sign, radix).parse(input)?;
    let (input, digit) = take_while1(
        |c: char| c.is_digit(radix) || c == '_'
    ).parse(input)?;

    let integer = T::from_str_radix(
        &str::replace(
            &(sign.to_string() + digit),
            "_",
            ""
        ),
        radix
    );

    // TODO: Make this better, this is a hack to work around implementing a custom error
    match integer {
        Ok(n)  => Ok((input, n)),
        Err(e) => Err(nom::Err::Error(Error::new("ParseIntError", nom::error::ErrorKind::Fail))),
    }
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
        assert_eq!(radix("123"), Ok(("123", 10)));
        assert_eq!(radix("0o44"), Ok(("44", 8)));
        assert_eq!(radix("0b1010"), Ok(("1010", 2)));
    }

    #[test]
    fn test_number_parse() {
        assert_eq!(number::<u8>("1"), Ok(("", 1)));
        assert_eq!(number::<i8>("-1"), Ok(("", -1)));
        assert_eq!(number::<u16>("512"), Ok(("", 512)));
        assert_eq!(number::<u8>("0xFF"), Ok(("", 0xFF)));
        assert_eq!(number::<u8>("0o44"), Ok(("", 0o44)));
        assert_eq!(number::<u8>("0b1001"), Ok(("", 0b1001)));
        assert_eq!(number::<i8>("-0x22"), Ok(("", -0x22)));
        assert_eq!(number::<u16>("16_000"), Ok(("", 16_000)));
        assert_eq!(number::<u16>("0xFF_FF"), Ok(("", 0xFF_FF)));
        assert_eq!(number::<u16>("0o44_33"), Ok(("", 0o44_33)));
        assert_eq!(number::<u16>("0b1010_1010"), Ok(("", 0b1010_1010)));
    }
}
