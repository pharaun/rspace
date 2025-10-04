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
use nom::sequence::separated_pair;
use nom::sequence::preceded;

use num_traits::Num;
use bitfield_struct::bitfield;


#[derive(Debug, PartialEq, Copy, Clone)]
pub enum HalfAcc {A, B, D}

fn half_acc(input: &str) -> IResult<&str, HalfAcc> {
    alt((
        value(HalfAcc::A, tag("A")),
        value(HalfAcc::B, tag("B")),
        value(HalfAcc::D, tag("D")),
    )).parse(input)
}

#[derive(Debug, PartialEq, Copy, Clone)]
pub enum FullAcc {A, B, D, E, F, W}

fn full_acc(input: &str) -> IResult<&str, FullAcc> {
    alt((
        value(FullAcc::A, tag("A")),
        value(FullAcc::B, tag("B")),
        value(FullAcc::D, tag("D")),
        value(FullAcc::E, tag("E")),
        value(FullAcc::F, tag("F")),
        value(FullAcc::W, tag("W")),
    )).parse(input)
}

#[derive(Debug, PartialEq, Copy, Clone)]
pub enum ShiftAcc {A, B, D, W}

fn shift_acc(input: &str) -> IResult<&str, ShiftAcc> {
    alt((
        value(ShiftAcc::A, tag("A")),
        value(ShiftAcc::B, tag("B")),
        value(ShiftAcc::D, tag("D")),
        value(ShiftAcc::W, tag("W")),
    )).parse(input)
}

#[derive(Debug, PartialEq, Copy, Clone)]
pub enum Inherent {
    // Simple Inherent instruction
    ABX, DAA, MUL, NOP, RTI, RTS, SYNC,
    PSHSW, PSHUW, PULSW, PULUW,
    SEX, SEXW, SWI, SWI2, SWI3,

    // Half Acc Inherent
    ASL(HalfAcc), ASR(HalfAcc), NEG(HalfAcc),

    // Full Acc Inherent
    CLR(FullAcc), COM(FullAcc), DEC(FullAcc), INC(FullAcc), TST(FullAcc),

    // Shift Acc Inherent
    LSR(ShiftAcc), ROL(ShiftAcc), ROR(ShiftAcc),
}

fn simple_inherent(input: &str) -> IResult<&str, Inherent> {
    alt((
        value(Inherent::ABX,   tag("ABX")),
        value(Inherent::DAA,   tag("DAA")),
        value(Inherent::MUL,   tag("MUL")),
        value(Inherent::NOP,   tag("NOP")),
        value(Inherent::RTI,   tag("RTI")),
        value(Inherent::RTS,   tag("RTS")),
        value(Inherent::SYNC,  tag("SYNC")),
        value(Inherent::PSHSW, tag("PSHSW")),
        value(Inherent::PSHUW, tag("PSHUW")),
        value(Inherent::PULSW, tag("PULSW")),
        value(Inherent::PULUW, tag("PULUW")),
        value(Inherent::SEXW,  tag("SEXW")),
        value(Inherent::SEX,   tag("SEX")),
        value(Inherent::SWI3,  tag("SWI3")),
        value(Inherent::SWI2,  tag("SWI2")),
        value(Inherent::SWI,   tag("SWI")),
    )).parse(input)
}

fn inherent(input: &str) -> IResult<&str, Inherent> {
    alt((
        // Simple
        simple_inherent,
        // Half
        map(pair(tag("ASL"), half_acc), |(_, acc)| Inherent::ASL(acc)),
        map(pair(tag("LSL"), half_acc), |(_, acc)| Inherent::ASL(acc)), // ASL/LSL
        map(pair(tag("ASR"), half_acc), |(_, acc)| Inherent::ASR(acc)),
        map(pair(tag("NEG"), half_acc), |(_, acc)| Inherent::NEG(acc)),
        // Full
        map(pair(tag("CLR"), full_acc), |(_, acc)| Inherent::CLR(acc)),
        map(pair(tag("COM"), full_acc), |(_, acc)| Inherent::COM(acc)),
        map(pair(tag("DEC"), full_acc), |(_, acc)| Inherent::DEC(acc)),
        map(pair(tag("INC"), full_acc), |(_, acc)| Inherent::INC(acc)),
        map(pair(tag("TST"), full_acc), |(_, acc)| Inherent::TST(acc)),
        // Shift
        map(pair(tag("LSR"), shift_acc), |(_, acc)| Inherent::LSR(acc)),
        map(pair(tag("ROL"), shift_acc), |(_, acc)| Inherent::ROL(acc)),
        map(pair(tag("ROR"), shift_acc), |(_, acc)| Inherent::ROR(acc)),
    )).parse(input)
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum InterReg {
    D  = 0b0000,
    X  = 0b0001,
    Y  = 0b0010,
    U  = 0b0011,
    S  = 0b0100,
    PC = 0b0101,
    W  = 0b0110,
    V  = 0b0111,
    A  = 0b1000,
    B  = 0b1001,
    CC = 0b1010,
    DP = 0b1011,
    // Z1/Z2 == 2x encoding of 0 register
    Z1 = 0b1100,
    Z2 = 0b1101,
    E  = 0b1110,
    F  = 0b1111,
}

fn inter_reg(input: &str) -> IResult<&str, InterReg> {
    alt((
        value(InterReg::D,  tag("D")),
        value(InterReg::X,  tag("X")),
        value(InterReg::Y,  tag("Y")),
        value(InterReg::U,  tag("U")),
        value(InterReg::S,  tag("S")),
        value(InterReg::PC, tag("PC")),
        value(InterReg::W,  tag("W")),
        value(InterReg::V,  tag("V")),
        value(InterReg::A,  tag("A")),
        value(InterReg::B,  tag("B")),
        value(InterReg::CC, tag("CC")),
        value(InterReg::DP, tag("DP")),
        value(InterReg::Z1, tag("0")), // Could also return Z2
        value(InterReg::E,  tag("E")),
        value(InterReg::F,  tag("F")),
    )).parse(input)
}

pub fn inter_reg_post_byte(r0: InterReg, r1: InterReg) -> u8 {
    let mask: u8 = 0b0000_1111;
    (((r0 as u8) & mask) << 4) | ((r1 as u8) & mask)
}

#[derive(Debug, PartialEq, Copy, Clone)]
pub enum TfmMode {
    PlusPlus, // TFM r0+, r1+
    MinusMinus, // TFM r0-, r1-
    PlusNone, // TFM r0+, r1
    NonePlus // TFM r0, r1+
}

#[derive(Debug, PartialEq, Copy, Clone)]
pub enum Imm8 {
    // Reg to Reg
    ADCR, ADDR, ANDR, CMPR, EORR, ORR, SBCR, SUBR, EXG, TFR,

    // Stack PostByte
    PSHS, PSHU, PULS, PULU,

    // Condition Code Flags
    ANDCC, ORCC, CWAI,

    // Weird
    BITMD, LDMD, TFM(TfmMode),
}

fn Imm8(input: &str) -> IResult<&str, (Imm8, u8)> {
    alt((
        // Reg to Reg
        pair(value(Imm8::ADCR,  tag("ADCR")), preceded(space1, reg_to_reg)),
        pair(value(Imm8::ADDR,  tag("ADDR")), preceded(space1, reg_to_reg)),
        pair(value(Imm8::ANDR,  tag("ANDR")), preceded(space1, reg_to_reg)),
        pair(value(Imm8::CMPR,  tag("CMPR")), preceded(space1, reg_to_reg)),
        pair(value(Imm8::EORR,  tag("EORR")), preceded(space1, reg_to_reg)),
        pair(value(Imm8::ORR,   tag("ORR")),  preceded(space1, reg_to_reg)),
        pair(value(Imm8::SBCR,  tag("SBCR")), preceded(space1, reg_to_reg)),
        pair(value(Imm8::SUBR,  tag("SUBR")), preceded(space1, reg_to_reg)),
        pair(value(Imm8::EXG,   tag("EXG")),  preceded(space1, reg_to_reg)),
        pair(value(Imm8::TFR,   tag("TFR")),  preceded(space1, reg_to_reg)),
        // Stack PostByte
//        pair(value(Imm8::PSHS,  tag("PSHS")), stack_postbyte),
//        pair(value(Imm8::PSHU,  tag("PSHU")), stack_postbyte),
//        pair(value(Imm8::PULS,  tag("PULS")), stack_postbyte),
//        pair(value(Imm8::PULU,  tag("PULU")), stack_postbyte),
//        // Condition Code Flags
//        pair(value(Imm8::ANDCC, tag("ANDCC")), cc_flags),
//        pair(value(Imm8::ORCC,  tag("ORCC")), cc_flags),
//        pair(value(Imm8::CWAI,  tag("CWAI")), cc_flags),
//        // Weird
//        pair(value(Imm8::BITMD, tag("BITMD")), raw_imm8),
//        pair(value(Imm8::LDMD,  tag("LDMD")),  raw_imm8),
//        map(pair(tag("TFM"), tfm_reg), |(_, (mode, reg))| (Imm8::TFM(mode), reg)),
    )).parse(input)
}

fn reg_to_reg(input: &str) -> IResult<&str, u8> {
    map(separated_pair(inter_reg, tag(","), inter_reg), |(r0, r1)| inter_reg_post_byte(r0, r1)).parse(input)
}




#[bitfield(u8, order=Msb)]
#[derive(PartialEq)]
pub struct PushPullPostByte {
    pc: bool, // 0b1000_0000
    us: bool,
    y:  bool,
    x:  bool,
    dp: bool,
    b:  bool,
    a:  bool,
    cc: bool, // 0b0000_0001
}

impl PushPullPostByte {
    // Enable using a string to toggle a field on or off
    pub fn with_str(&self, reg: &str, val: bool) -> Self {
        match reg {
            "PC" => self.with_pc(val),
            "U"  => self.with_us(val),
            "S"  => self.with_us(val),
            "Y"  => self.with_y(val),
            "X"  => self.with_x(val),
            "DP" => self.with_dp(val),
            "B"  => self.with_b(val),
            "A"  => self.with_a(val),
            "CC" => self.with_cc(val),
            _ => *self,
        }
    }
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
    fn test_inherent() {
        let data = vec![
            // Simple
            ("ABX", Inherent::ABX), ("SYNC", Inherent::SYNC),
            ("PSHSW", Inherent::PSHSW), ("PULUW", Inherent::PULUW),
            ("SWI", Inherent::SWI), ("SWI2", Inherent::SWI2),
            // Half
            ("ASLA", Inherent::ASL(HalfAcc::A)),
            ("LSLA", Inherent::ASL(HalfAcc::A)),
            ("ASRB", Inherent::ASR(HalfAcc::B)),
            ("NEGD", Inherent::NEG(HalfAcc::D)),
            // Full
            ("CLRA", Inherent::CLR(FullAcc::A)),
            ("COMB", Inherent::COM(FullAcc::B)),
            ("DECD", Inherent::DEC(FullAcc::D)),
            ("INCE", Inherent::INC(FullAcc::E)),
            ("TSTW", Inherent::TST(FullAcc::W)),
        ];

        for (s,e) in data {
            assert_eq!(inherent(s), Ok(("", e)));
        }
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
