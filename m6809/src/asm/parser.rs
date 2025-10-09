use nom::IResult;
use nom::Parser;
use nom::branch::alt;
use nom::bytes::complete::tag;
use nom::bytes::complete::take_while1;
use nom::character::complete::one_of;
use nom::character::complete::space1;
use nom::combinator::map;
use nom::combinator::recognize;
use nom::combinator::success;
use nom::combinator::value;
use nom::combinator::opt;
use nom::error::Error;
use nom::multi::separated_list1;
use nom::sequence::pair;
use nom::sequence::preceded;
use nom::sequence::separated_pair;
use nom::sequence::terminated;
use nom::sequence::delimited;

use num_traits::Num;
use bitfield_struct::bitfield;


#[derive(Debug, PartialEq, Copy, Clone)]
enum HalfAcc {A, B, D}

fn half_acc(input: &str) -> IResult<&str, HalfAcc> {
    alt((
        value(HalfAcc::A, tag("A")),
        value(HalfAcc::B, tag("B")),
        value(HalfAcc::D, tag("D")),
    )).parse(input)
}

#[repr(u8)]
#[derive(Debug, PartialEq, Copy, Clone)]
enum FullAcc {
    A = 0b0110,
    B = 0b0101,
    D = 0b1011,
    E = 0b0111,
    F = 0b1010,
    W = 0b1110
}

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
enum ShiftAcc {A, B, D, W}

fn shift_acc(input: &str) -> IResult<&str, ShiftAcc> {
    alt((
        value(ShiftAcc::A, tag("A")),
        value(ShiftAcc::B, tag("B")),
        value(ShiftAcc::D, tag("D")),
        value(ShiftAcc::W, tag("W")),
    )).parse(input)
}

#[derive(Debug, PartialEq, Copy, Clone)]
enum Inherent {
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
enum InterReg {
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
        value(InterReg::PC, tag("PC")),
        value(InterReg::CC, tag("CC")),
        value(InterReg::DP, tag("DP")),
        value(InterReg::D,  tag("D")),
        value(InterReg::X,  tag("X")),
        value(InterReg::Y,  tag("Y")),
        value(InterReg::U,  tag("U")),
        value(InterReg::S,  tag("S")),
        value(InterReg::W,  tag("W")),
        value(InterReg::V,  tag("V")),
        value(InterReg::A,  tag("A")),
        value(InterReg::B,  tag("B")),
        value(InterReg::Z1, tag("0")), // Could also return Z2
        value(InterReg::E,  tag("E")),
        value(InterReg::F,  tag("F")),
    )).parse(input)
}

fn inter_reg_post_byte(r0: InterReg, r1: InterReg) -> u8 {
    ((r0 as u8) << 4) | (r1 as u8)
}

#[bitfield(u8, order=Msb)]
#[derive(PartialEq)]
struct StackPostByte {
    pc: bool, // 0b1000_0000
    us: bool,
    y:  bool,
    x:  bool,
    dp: bool,
    b:  bool,
    a:  bool,
    cc: bool, // 0b0000_0001
}

impl StackPostByte {
    // Enable using a string to toggle a field on or off
    fn with_str(&self, reg: &str, val: bool) -> Self {
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

#[bitfield(u8, order=Msb)]
#[derive(PartialEq)]
struct ConditionCodeByte {
    e: bool, // 0b1000_0000
    f: bool,
    h: bool,
    i: bool,
    n: bool,
    z: bool,
    v: bool,
    c: bool, // 0b0000_0001
}

impl ConditionCodeByte {
    // Enable using a char to toggle a field on or off
    fn with_char(&self, reg: char, val: bool) -> Self {
        match reg {
            'E' => self.with_e(val),
            'F' => self.with_f(val),
            'H' => self.with_h(val),
            'I' => self.with_i(val),
            'N' => self.with_n(val),
            'Z' => self.with_z(val),
            'V' => self.with_v(val),
            'C' => self.with_c(val),
            _ => *self,
        }
    }
}

#[derive(Debug, PartialEq, Copy, Clone)]
enum TfmMode {
    PlusPlus, // TFM r0+, r1+
    MinusMinus, // TFM r0-, r1-
    PlusNone, // TFM r0+, r1
    NonePlus // TFM r0, r1+
}

#[derive(Debug, PartialEq, Copy, Clone)]
enum Imm8 {
    // Reg to Reg
    ADCR, ADDR, ANDR, CMPR, EORR, ORR, SBCR, SUBR, EXG, TFR,

    // Stack PostByte
    PSHS, PSHU, PULS, PULU,

    // Condition Code Flags
    ANDCC, ORCC, CWAI,

    // Weird
    BITMD, LDMD, TFM(TfmMode),
}

fn imm8(input: &str) -> IResult<&str, (Imm8, u8)> {
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
        pair(value(Imm8::PSHS,  tag("PSHS")), preceded(space1, stack_postbyte)),
        pair(value(Imm8::PSHU,  tag("PSHU")), preceded(space1, stack_postbyte)),
        pair(value(Imm8::PULS,  tag("PULS")), preceded(space1, stack_postbyte)),
        pair(value(Imm8::PULU,  tag("PULU")), preceded(space1, stack_postbyte)),
        // Condition Code Flags
        pair(value(Imm8::ANDCC, tag("ANDCC")), preceded(space1, condition_code)),
        pair(value(Imm8::ORCC,  tag("ORCC")),  preceded(space1, condition_code)),
        pair(value(Imm8::CWAI,  tag("CWAI")),  preceded(space1, condition_code)),
        // Weird
        pair(value(Imm8::BITMD, tag("BITMD")), preceded(space1, bitmd_imm8)),
        pair(value(Imm8::LDMD,  tag("LDMD")),  preceded(space1, ldmd_imm8)),
        map(pair(tag("TFM"), preceded(space1, tfm_reg)), |(_, (mode, reg))| (Imm8::TFM(mode), reg)),
    )).parse(input)
}

fn reg_to_reg(input: &str) -> IResult<&str, u8> {
    map(separated_pair(inter_reg, tag(","), inter_reg), |(r0, r1)| inter_reg_post_byte(r0, r1)).parse(input)
}

fn stack_postbyte(input: &str) -> IResult<&str, u8> {
    separated_list1(
        tag(","),
        alt((
            tag("PC"),
            tag("DP"),
            tag("CC"),
            recognize(one_of("USYXBA"))
        )),
    ).parse(input).map(
        |(input, items)| (input, items.into_iter().fold(
            StackPostByte::new(),
            |acc, x| acc.with_str(x, true),
        ).into())
    )
}

fn condition_code(input: &str) -> IResult<&str, u8> {
    separated_list1(
        tag(","),
        one_of("EFHINZVC"),
    ).parse(input).map(
        |(input, items)| (input, items.into_iter().fold(
            ConditionCodeByte::new(),
            |acc, x| acc.with_char(x, true),
        ).into())
    )
}

fn bitmd_imm8(input: &str) -> IResult<&str, u8> {
    separated_list1(
        tag(","),
        alt((
            value(0b1000_0000, tag("/0")),
            value(0b0100_0000, tag("IL")),
        )),
    ).parse(input).map(
        |(input, items)| (input, items.into_iter().fold(
            0,
            |acc, x| acc | x,
        ))
    )
}

fn ldmd_imm8(input: &str) -> IResult<&str, u8> {
    separated_list1(
        tag(","),
        alt((
            value(0b0000_0010, tag("FM")),
            value(0b0000_0001, tag("NM")),
        )),
    ).parse(input).map(
        |(input, items)| (input, items.into_iter().fold(
            0,
            |acc, x| acc | x,
        ))
    )
}

fn tfm_mode_reg(input: &str) -> IResult<&str, (InterReg, char)> {
    recognize(
        one_of("XYUSD")
    ).and_then(
        inter_reg
    ).and(
        alt((
            one_of("+-"),
            success(' ')
        ))
    ).parse(input)
}

fn tfm_reg(input: &str) -> IResult<&str, (TfmMode, u8)> {
    let (input, (r0, r1)) = separated_pair(
        tfm_mode_reg,
        tag(","),
        tfm_mode_reg,
    ).parse(input)?;

    let post_byte = inter_reg_post_byte(r0.0, r1.0);
    let tfm_mode = match (r0.1, r1.1) {
        ('+', '+') => Ok(TfmMode::PlusPlus),
        ('-', '-') => Ok(TfmMode::MinusMinus),
        ('+', ' ') => Ok(TfmMode::PlusNone),
        (' ', '+') => Ok(TfmMode::NonePlus),
        _ => Err(nom::Err::Error(Error::new("ParseTfmModeError", nom::error::ErrorKind::Fail))),
    }?;

    Ok((input, (tfm_mode, post_byte)))
}

#[derive(Debug, PartialEq, Copy, Clone)]
enum BitMode { AND, EOR, OR }

fn bit_mode(input: &str) -> IResult<&str, BitMode> {
    alt((
        value(BitMode::AND, tag("AND")),
        value(BitMode::EOR, tag("EOR")),
        value(BitMode::OR, tag("OR")),
    )).parse(input)
}

#[derive(Debug, PartialEq, Copy, Clone)]
enum BitInv { AsIs, Inverted }

fn bit_inv(input: &str) -> IResult<&str, BitInv> {
    preceded(
        tag("B"),
        alt((
            value(BitInv::Inverted, tag("I")),
            success(BitInv::AsIs),
        )),
    ).parse(input)
}

#[derive(Debug, PartialEq, Copy, Clone)]
enum DirectBit {
    // Load/Store
    LDBT, STBT,

    // Bit Mutation
    BitMut(BitMode, BitInv),
}

fn direct_bit(input: &str) -> IResult<&str, (DirectBit, (u8, u8))> {
    alt((
        // Load/Store
        pair(value(DirectBit::LDBT, tag("LDBT")), preceded(space1, bit_arg)),
        pair(value(DirectBit::STBT, tag("STBT")), preceded(space1, bit_arg)),

        // Bit Mutation
        map(
            pair(pair(bit_inv, bit_mode), preceded(space1, bit_arg)),
            |((bi, bm), ba)| (DirectBit::BitMut(bm, bi), ba),
        ),
    )).parse(input)
}

fn bit_arg(input: &str) -> IResult<&str, (u8, u8)> {
    // BIAND r, sBit, dBit, addr
    map((
        // r == CC = 0b00, A = 0b01, B = 0b10, invalid = 0b11
        terminated(bit_reg, tag(",")),
        // sBit/dBit == 0-7 -> 3bit
        terminated(bit_sel, tag(",")),
        terminated(bit_sel, tag(",")),
        ),
        |(reg, s_bit, d_bit)| {
            reg << 6 | s_bit << 3 | d_bit
        },
    ).and(
        // addr -> u8 byte
        direct_addr
    ).parse(input)
}

fn bit_reg(input: &str) -> IResult<&str, u8> {
    alt((
        value(0b00, tag("CC")),
        value(0b01, tag("A")),
        value(0b10, tag("B")),
        // 0b11 == Invalid
    )).parse(input)
}

fn bit_sel(input: &str) -> IResult<&str, u8> {
    alt((
        value(0b000, tag("0")),
        value(0b001, tag("1")),
        value(0b010, tag("2")),
        value(0b011, tag("3")),
        value(0b100, tag("4")),
        value(0b101, tag("5")),
        value(0b110, tag("6")),
        value(0b111, tag("7")),
    )).parse(input)
}

fn direct_addr(input: &str) -> IResult<&str, u8> {
    preceded(
        tag("<"),
        number::<u8>,
    ).parse(input)
}

#[repr(u8)]
#[derive(Debug, PartialEq, Copy, Clone)]
enum StackReg {
    X = 0b00,
    Y = 0b01,
    U = 0b10,
    S = 0b11,
}

fn stack_reg(input: &str) -> IResult<&str, StackReg> {
    alt((
        value(StackReg::X, tag("X")),
        value(StackReg::Y, tag("Y")),
        value(StackReg::U, tag("U")),
        value(StackReg::S, tag("S")),
    )).parse(input)
}

#[repr(u8)]
#[derive(Debug, PartialEq, Copy, Clone)]
enum IndexType {
    NonIndirect = 0b1111,
    Indirect    = 0b0000,
}

impl IndexType {
    fn into_bool(&self) -> bool{
        match self {
            IndexType::NonIndirect => false,
            IndexType::Indirect => true,
        }
    }
}

#[repr(u8)]
#[derive(Debug, PartialEq, Copy, Clone)]
enum ModeW {
    Offset0  = 0b00,
    Offset16 = 0b01,
    IncInc   = 0b10,
    DecDec   = 0b11,
}

#[repr(u8)]
#[derive(Debug, PartialEq, Copy, Clone)]
enum IndexMode {
    // Offset mode
    Offset0  = 0b0100,
    Offset8  = 0b1000,
    Offset16 = 0b1001,

    // Accumulator Offset
    AccA = 0b0110,
    AccB = 0b0101,
    AccD = 0b1011,
    AccE = 0b0111,
    AccF = 0b1010,
    AccW = 0b1110,

    // Inc/Dec of RR register
    IncInc = 0b0001,
    DecDec = 0b0011,

    // PC offset
    PCR8  = 0b1100,
    PCR16 = 0b1101,
}

#[derive(Debug, PartialEq, Copy, Clone)]
enum IndexPostByte {
    // - 5bit offset
    Offset5(StackReg, u8),
    // - Single Inc/Dec
    Inc(StackReg),
    Dec(StackReg),
    // - Extended Indirect
    ExtendedIndirect,
    // W register
    // Non Indirect / Indirect
    RegW(ModeW, IndexType),
    // Standard index modes
    Standard(StackReg, IndexMode, IndexType),
}

#[bitfield(u8, order=Msb)]
#[derive(PartialEq)]
struct PackedIndexPostByte {
    #[bits(1, default = true)]
    __: bool,
    #[bits(2)]
    rr: u8,
    #[bits(1)]
    indirect: bool,
    #[bits(4)]
    mode: u8,
}

fn index_post_byte(index: IndexPostByte) -> u8 {
    match index {
        IndexPostByte::Offset5(rr, imm) => {
            let imm5_mask: u8 = 0b0001_1111;
            ((rr as u8) << 5) | (imm & imm5_mask)
        },
        IndexPostByte::ExtendedIndirect => {
            0b10011111
        },
        IndexPostByte::Inc(rr) => {
            PackedIndexPostByte::new()
                .with_rr(rr as u8)
                .with_indirect(false)
                .with_mode(0b0000)
                .into()
        },
        IndexPostByte::Dec(rr) => {
            PackedIndexPostByte::new()
                .with_rr(rr as u8)
                .with_indirect(false)
                .with_mode(0b0010)
                .into()
        },
        IndexPostByte::RegW(mode, typ) => {
            PackedIndexPostByte::new()
                .with_rr(mode as u8)
                .with_indirect(typ.into_bool())
                .with_mode(typ as u8)
                .into()
        },
        IndexPostByte::Standard(rr, mode, typ) => {
            PackedIndexPostByte::new()
                .with_rr(rr as u8)
                .with_indirect(typ.into_bool())
                .with_mode(mode as u8)
                .into()

        },
    }
}

#[derive(Debug, PartialEq, Copy, Clone)]
enum IndexBytes {
    None,
    One(u8),
    Two(u16),
}

#[derive(Debug, PartialEq, Copy, Clone)]
enum Indexed {
    LEA(StackReg),
}

fn indexed(input: &str) -> IResult<&str, (Indexed, u8, IndexBytes)> {
    map(
        pair(
            preceded(tag("LEA"), stack_reg),
            preceded(space1, index_addr),
        ),
        |(s_reg, (i_typ, (i_arg, w_stack)))| {
            let (index_post, index_bytes) = index_parse_to_post_byte(
                i_typ, i_arg, w_stack
            );
            let index_pb = index_post_byte(index_post);

            (Indexed::LEA(s_reg), index_pb, index_bytes)
        },
    ).parse(input)
}

#[derive(Debug, PartialEq, Copy, Clone)]
enum WStack {W, PCR, Stack(StackReg), None}

#[derive(Debug, PartialEq, Copy, Clone)]
enum IncDec {Inc, IncInc, Dec, DecDec, None}

#[derive(Debug, PartialEq, Copy, Clone)]
enum IndexArg {IncDec(IncDec), Imm(i16), Acc(FullAcc), UImm(u16)}

fn zero_offset_parse(input: &str) -> IResult<&str, (IndexArg, WStack)> {
    preceded(
        pair(opt(tag("0")), tag(",")),
        alt((
            // 0,--R ~= ,--R
            map(pair(tag("--"), stack_reg), |(_, s)| (IndexArg::IncDec(IncDec::DecDec), WStack::Stack(s))),
            // 0,--W ~= ,--W
            value((IndexArg::IncDec(IncDec::DecDec), WStack::W), tag("--W")),
            // 0,R++ ~= ,R++
            map(pair(stack_reg, tag("++")), |(s, _)| (IndexArg::IncDec(IncDec::IncInc), WStack::Stack(s))),
            // 0,W++ ~= ,W++
            value((IndexArg::IncDec(IncDec::IncInc), WStack::W), tag("W++")),
            // 0,-R ~= ,-R
            map(pair(tag("-"), stack_reg), |(_, s)| (IndexArg::IncDec(IncDec::Dec), WStack::Stack(s))),
            // 0,R+ ~= ,R+
            map(pair(stack_reg, tag("+")), |(s, _)| (IndexArg::IncDec(IncDec::Inc), WStack::Stack(s))),
            // 0,R ~= ,R
            map(stack_reg, |s| (IndexArg::IncDec(IncDec::None), WStack::Stack(s))),
            // 0,W ~= ,W
            value((IndexArg::IncDec(IncDec::None), WStack::W), tag("W")),
        ))
    ).parse(input)
}

fn imm_parse(input: &str) -> IResult<&str, (IndexArg, WStack)> {
    separated_pair(
        map(number::<i16>, IndexArg::Imm),
        tag(","),
        alt((
            // n,R   - n = imm5, imm8, imm16
            map(stack_reg, |s| WStack::Stack(s)),
            // n,W   - n = imm16
            value(WStack::W, tag("W")),
            // n,PCR - n = imm8, imm16
            value(WStack::PCR, tag("PCR")),
        )),
    ).parse(input)
}

fn acc_stack(input: &str) -> IResult<&str, (IndexArg, WStack)> {
    // acc,R
    separated_pair(
        map(full_acc, IndexArg::Acc),
        tag(","),
        map(stack_reg, WStack::Stack),
    ).parse(input)
}

fn index_addr(input: &str) -> IResult<&str, (IndexType, (IndexArg, WStack))> {
    alt((
        pair(
            success(IndexType::Indirect),
            delimited(
                tag("["),
                alt((
                    zero_offset_parse,
                    imm_parse,
                    acc_stack,
                    pair(
                        map(number::<u16>, IndexArg::UImm),
                        success(WStack::None),
                    ),
                )),
                tag("]")
            ),
        ),
        pair(
            success(IndexType::NonIndirect),
            alt((
                zero_offset_parse,
                imm_parse,
                acc_stack,
            )),
        ),
    )).parse(input)
}

fn index_parse_to_post_byte(it: IndexType, ia: IndexArg, ws: WStack) -> (IndexPostByte, IndexBytes) {
    match (it, ia, ws) {
        // Indirect Specials
        // [n]
        (IndexType::Indirect, IndexArg::UImm(addr), WStack::None) => {
            (IndexPostByte::ExtendedIndirect, IndexBytes::Two(addr))
        },
        // NonIndirect Specials
        // Inc
        (IndexType::NonIndirect, IndexArg::IncDec(IncDec::Inc), WStack::Stack(s)) => {
            (IndexPostByte::Inc(s), IndexBytes::None)
        },
        // Dec
        (IndexType::NonIndirect, IndexArg::IncDec(IncDec::Dec), WStack::Stack(s)) => {
            (IndexPostByte::Dec(s), IndexBytes::None)
        },
        // W register Specials
        // 0,W++ ~= ,W++
        (it, IndexArg::IncDec(IncDec::IncInc), WStack::W) => {
            (IndexPostByte::RegW(ModeW::IncInc, it), IndexBytes::None)
        },
        // 0,--W ~= ,--W
        (it, IndexArg::IncDec(IncDec::DecDec), WStack::W) => {
            (IndexPostByte::RegW(ModeW::DecDec, it), IndexBytes::None)
        },
        // 0,W ~= ,W
        (it, IndexArg::IncDec(IncDec::None), WStack::W) => {
            (IndexPostByte::RegW(ModeW::Offset0, it), IndexBytes::None)
        },
        // n,W   - n = imm16
        (it, IndexArg::Imm(offset), WStack::W) => {
            (IndexPostByte::RegW(ModeW::Offset16, it), IndexBytes::Two(offset as u16))
        },
        // Standard index modes
        // RR IncInc
        (it, IndexArg::IncDec(IncDec::IncInc), WStack::Stack(s)) => {
            (IndexPostByte::Standard(s, IndexMode::IncInc, it), IndexBytes::None)
        },
        // RR DecDec
        (it, IndexArg::IncDec(IncDec::DecDec), WStack::Stack(s)) => {
            (IndexPostByte::Standard(s, IndexMode::DecDec, it), IndexBytes::None)
        },
        // RR AccA/B/D/E/F/W
        (it, IndexArg::Acc(acc), WStack::Stack(s)) => {
            let acc_mode = match acc {
                FullAcc::A => IndexMode::AccA,
                FullAcc::B => IndexMode::AccB,
                FullAcc::D => IndexMode::AccD,
                FullAcc::E => IndexMode::AccE,
                FullAcc::F => IndexMode::AccF,
                FullAcc::W => IndexMode::AccW,
            };
            (IndexPostByte::Standard(s, acc_mode, it), IndexBytes::None)
        },
        // PCR8
        // PCR16
        (it, IndexArg::Imm(offset), WStack::PCR) => {
            // if fit in 8bit emit pcr8, otherwise pcr16
            let (pcr_mode, ib) = match i8::try_from(offset) {
                Ok(imm8) => (IndexMode::PCR8,  IndexBytes::One(imm8 as u8)),
                Err(_)   => (IndexMode::PCR16, IndexBytes::Two(offset as u16)),
            };
            // The stack register is ignored, just hardcode 1 here
            (IndexPostByte::Standard(StackReg::X, pcr_mode, it), ib)
        },
        // Offset mode
        // Offset0
        (it, IndexArg::IncDec(IncDec::None), WStack::Stack(s)) => {
            (IndexPostByte::Standard(s, IndexMode::Offset0, it), IndexBytes::None)
        },
        // Offset5 - Special (-16 to +15)
        // Offset8
        // Offset16
        (it, IndexArg::Imm(offset), WStack::Stack(s)) => {
            // Check if it fits in 5bit otherwise 8/16 bit
            if (offset <= 15) && (offset >= -16) {
                (IndexPostByte::Offset5(s, offset as u8), IndexBytes::None)
            } else {
                let (offset_mode, ib) = match i8::try_from(offset) {
                    Ok(imm8) => (IndexMode::Offset8, IndexBytes::One(imm8 as u8)),
                    Err(_)   => (IndexMode::Offset16, IndexBytes::Two(offset as u16)),
                };
                (IndexPostByte::Standard(s, offset_mode, it), ib)
            }
        },
        _ => panic!("Should not reach here from the parser"),
    }
}

#[derive(Debug, PartialEq, Copy, Clone)]
enum StoreLoad {A, B, D, E, F, W, Q, S, U, X, Y}

fn store_load(input: &str) -> IResult<&str, StoreLoad> {
    alt((
        value(StoreLoad::A, tag("A")),
        value(StoreLoad::B, tag("B")),
        value(StoreLoad::D, tag("D")),
        value(StoreLoad::E, tag("E")),
        value(StoreLoad::F, tag("F")),
        value(StoreLoad::W, tag("W")),
        value(StoreLoad::Q, tag("Q")),
        value(StoreLoad::S, tag("S")),
        value(StoreLoad::U, tag("U")),
        value(StoreLoad::X, tag("X")),
        value(StoreLoad::Y, tag("Y")),
    )).parse(input)
}

// For now hard code < as direct > as extended sigils
#[derive(Debug, PartialEq, Copy, Clone)]
enum MemAddrMode {
    Direct(u8),
    Extended(u16),
    // PostByte + Additional Bytes
    Indexed(u8, IndexBytes),
}

fn extended_addr(input: &str) -> IResult<&str, u16> {
    preceded(
        tag(">"),
        number::<u16>,
    ).parse(input)
}

fn mem_addr_mode(input: &str) -> IResult<&str, MemAddrMode> {
    alt((
        map(direct_addr, MemAddrMode::Direct),
        map(extended_addr, MemAddrMode::Extended),
        map(
            index_addr,
            |(i_typ, (i_arg, w_stack))| {
                let (index_post, index_bytes) = index_parse_to_post_byte(
                    i_typ, i_arg, w_stack
                );
                let index_pb = index_post_byte(index_post);

                MemAddrMode::Indexed(index_pb, index_bytes)
            },
        ),
    )).parse(input)
}

#[derive(Debug, PartialEq, Copy, Clone)]
enum DirectMem {
    ASL, // LSL
    ASR,
    CLR,
    COM,
    DEC,
    INC,
    JMP,
    JSR,
    LSR,
    NEG,
    ROL,
    ROR,
    TST,
    ST(StoreLoad),
}

fn direct_mem(input: &str) -> IResult<&str, (DirectMem, MemAddrMode)> {
    alt((
        pair(value(DirectMem::ASL, tag("ASL")), preceded(space1, mem_addr_mode)),
        pair(value(DirectMem::ASL, tag("LSL")), preceded(space1, mem_addr_mode)), // ASL
        pair(value(DirectMem::ASR, tag("ASR")), preceded(space1, mem_addr_mode)),
        pair(value(DirectMem::CLR, tag("CLR")), preceded(space1, mem_addr_mode)),
        pair(value(DirectMem::COM, tag("COM")), preceded(space1, mem_addr_mode)),
        pair(value(DirectMem::DEC, tag("DEC")), preceded(space1, mem_addr_mode)),
        pair(value(DirectMem::INC, tag("INC")), preceded(space1, mem_addr_mode)),
        pair(value(DirectMem::JMP, tag("JMP")), preceded(space1, mem_addr_mode)),
        pair(value(DirectMem::JSR, tag("JSR")), preceded(space1, mem_addr_mode)),
        pair(value(DirectMem::LSR, tag("LSR")), preceded(space1, mem_addr_mode)),
        pair(value(DirectMem::NEG, tag("NEG")), preceded(space1, mem_addr_mode)),
        pair(value(DirectMem::ROL, tag("ROL")), preceded(space1, mem_addr_mode)),
        pair(value(DirectMem::ROR, tag("ROR")), preceded(space1, mem_addr_mode)),
        pair(value(DirectMem::TST, tag("TST")), preceded(space1, mem_addr_mode)),
        // Store
        preceded(tag("ST"), map(pair(store_load, preceded(space1, mem_addr_mode)), |(sl, mem)| (DirectMem::ST(sl), mem))),
    )).parse(input)
}

#[derive(Debug, PartialEq, Copy, Clone)]
enum LogicalMem {
    AIM,
    EIM,
    OIM,
    TIM,
}

fn logical_mem(input: &str) -> IResult<&str, (LogicalMem, u8, MemAddrMode)> {
    (
        alt((
            value(LogicalMem::AIM, tag("AIM")),
            value(LogicalMem::EIM, tag("EIM")),
            value(LogicalMem::OIM, tag("OIM")),
            value(LogicalMem::TIM, tag("TIM")),
        )),
        delimited(
            space1,
            number::<u8>,
            tag(";"),
        ),
        mem_addr_mode,
    ).parse(input)
}

#[derive(Debug, PartialEq, Copy, Clone)]
enum BranchMode {
    Short(i8),
    Long(i16),
}

#[derive(Debug, PartialEq, Copy, Clone)]
enum Branch {
    BRA, BRN, BHI, BLS,
    BCC, //BHS
    BCS, //BLO
    BNE, BEQ, BVC, BVS,
    BPL, BMI, BGE, BLT,
    BGT, BLE, BSR,
}

fn branch_inst(input: &str) -> IResult<&str, Branch> {
    alt((
        value(Branch::BRA, tag("BRA")),
        value(Branch::BRN, tag("BRN")),
        value(Branch::BHI, tag("BHI")),
        value(Branch::BLS, tag("BLS")),
        value(Branch::BCC, tag("BCC")),
        value(Branch::BCC, tag("BHS")), // BCC
        value(Branch::BCS, tag("BCS")),
        value(Branch::BCS, tag("BLO")), // BCS
        value(Branch::BNE, tag("BNE")),
        value(Branch::BEQ, tag("BEQ")),
        value(Branch::BVC, tag("BVC")),
        value(Branch::BVS, tag("BVS")),
        value(Branch::BPL, tag("BPL")),
        value(Branch::BMI, tag("BMI")),
        value(Branch::BGE, tag("BGE")),
        value(Branch::BLT, tag("BLT")),
        value(Branch::BGT, tag("BGT")),
        value(Branch::BLE, tag("BLE")),
        value(Branch::BSR, tag("BSR")),
    )).parse(input)
}

fn branch(input: &str) -> IResult<&str, (Branch, BranchMode)> {
    alt((
        preceded(tag("L"), map(pair(branch_inst, preceded(space1, number::<i16>)), |(bi, ba)| (bi, BranchMode::Long(ba)))),
        map(pair(branch_inst, preceded(space1, number::<i8>)), |(bi, ba)| (bi, BranchMode::Short(ba))),
    )).parse(input)
}

// Remaining instructions
// imm8:
//
// imm16:
//
// imm32:






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
        Err(_) => Err(nom::Err::Error(Error::new("ParseIntError", nom::error::ErrorKind::Fail))),
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
    fn test_parse_inter_reg() {
        assert_eq!(inter_reg("D"), Ok(("", InterReg::D)));
        assert_eq!(inter_reg("PC"), Ok(("", InterReg::PC)));
        assert_eq!(inter_reg("CC"), Ok(("", InterReg::CC)));
        assert_eq!(inter_reg("DP"), Ok(("", InterReg::DP)));
        assert_eq!(inter_reg("0"), Ok(("", InterReg::Z1)));
        assert_eq!(inter_reg("F"), Ok(("", InterReg::F)));
    }

    #[test]
    fn test_inter_reg_post_byte() {
        assert_eq!(
            inter_reg_post_byte(InterReg::CC, InterReg::PC),
            0b1010_0101,
        );
    }

    #[test]
    fn test_stack_post_byte() {
        assert_eq!(stack_postbyte("PC"), Ok(("", 0b1000_0000)));
        assert_eq!(stack_postbyte("U"), Ok(("", 0b0100_0000)));
        assert_eq!(stack_postbyte("S"), Ok(("", 0b0100_0000)));
        assert_eq!(stack_postbyte("PC,U,DP,A"), Ok(("", 0b1100_1010)));
    }

    #[test]
    fn test_condition_code_byte() {
        assert_eq!(condition_code("E"), Ok(("", 0b1000_0000)));
        assert_eq!(condition_code("C"), Ok(("", 0b0000_0001)));
        assert_eq!(condition_code("E,I,N,C"), Ok(("", 0b1001_1001)));
    }

    #[test]
    fn test_bitmd_byte() {
        assert_eq!(bitmd_imm8("/0"), Ok(("", 0b1000_0000)));
        assert_eq!(bitmd_imm8("IL"), Ok(("", 0b0100_0000)));
        assert_eq!(bitmd_imm8("IL,/0"), Ok(("", 0b1100_0000)));
    }

    #[test]
    fn test_ldmd_byte() {
        assert_eq!(ldmd_imm8("FM"), Ok(("", 0b0000_0010)));
        assert_eq!(ldmd_imm8("NM"), Ok(("", 0b0000_0001)));
        assert_eq!(ldmd_imm8("FM,NM"), Ok(("", 0b0000_0011)));
    }

    #[test]
    fn test_tfm_mode_reg() {
        assert_eq!(tfm_mode_reg("X+"), Ok(("", (InterReg::X, '+'))));
        assert_eq!(tfm_mode_reg("Y-"), Ok(("", (InterReg::Y, '-'))));
        assert_eq!(tfm_mode_reg("U"), Ok(("", (InterReg::U, ' '))));
    }

    #[test]
    fn test_tfm_reg_byte() {
        let data = vec![
            ("X+,Y+", (TfmMode::PlusPlus,   inter_reg_post_byte(InterReg::X, InterReg::Y))),
            ("X-,Y-", (TfmMode::MinusMinus, inter_reg_post_byte(InterReg::X, InterReg::Y))),
            ("X+,Y",  (TfmMode::PlusNone,   inter_reg_post_byte(InterReg::X, InterReg::Y))),
            ("X,Y+",  (TfmMode::NonePlus,   inter_reg_post_byte(InterReg::X, InterReg::Y))),
        ];

        for (s,e) in data {
            assert_eq!(tfm_reg(s), Ok(("", e)));
        }
    }

    #[test]
    fn test_imm8() {
        let data = vec![
            // Reg to Reg
            ("ADCR A,B", (Imm8::ADCR, inter_reg_post_byte(InterReg::A, InterReg::B))),
            ("CMPR DP,0", (Imm8::CMPR, inter_reg_post_byte(InterReg::DP, InterReg::Z1))),
            // Stack PostByte
            ("PSHS PC", (Imm8::PSHS, StackPostByte::new().with_pc(true).into())),
            ("PULU PC,CC,A", (
                Imm8::PULU,
                StackPostByte::new().with_pc(true).with_cc(true).with_a(true).into()
            )),
            // Condition Code Flags
            ("ANDCC E", (Imm8::ANDCC, ConditionCodeByte::new().with_e(true).into())),
            ("ORCC E,V", (Imm8::ORCC, ConditionCodeByte::new().with_e(true).with_v(true).into())),
            // Weird
            ("BITMD /0", (Imm8::BITMD, 0b1000_0000)),
            ("BITMD /0,IL", (Imm8::BITMD, 0b1100_0000)),
            ("LDMD NM", (Imm8::LDMD, 0b0000_0001)),
            ("LDMD FM,NM", (Imm8::LDMD, 0b0000_0011)),
            // TFM
            ("TFM X+,Y+", (Imm8::TFM(TfmMode::PlusPlus), inter_reg_post_byte(InterReg::X, InterReg::Y))),
            ("TFM X-,Y-", (Imm8::TFM(TfmMode::MinusMinus), inter_reg_post_byte(InterReg::X, InterReg::Y))),
            ("TFM X+,Y", (Imm8::TFM(TfmMode::PlusNone), inter_reg_post_byte(InterReg::X, InterReg::Y))),
            ("TFM X,Y+", (Imm8::TFM(TfmMode::NonePlus), inter_reg_post_byte(InterReg::X, InterReg::Y))),
        ];

        for (s,e) in data {
            assert_eq!(imm8(s), Ok(("", e)));
        }
    }

    #[test]
    fn test_direct_bit() {
        let data = vec![
            // Load/Store
            ("LDBT CC,0,7,<0xFF", (DirectBit::LDBT, (0b00_000_111, 0xFF))),
            ("STBT B,7,0,<0x00",  (DirectBit::STBT, (0b10_111_000, 0x00))),

            // Bit Mutation
            ("BAND B,7,0,<0xAF",  (DirectBit::BitMut(BitMode::AND, BitInv::AsIs), (0b10_111_000, 0xAF))),
            ("BIAND B,7,0,<0xAF", (DirectBit::BitMut(BitMode::AND, BitInv::Inverted), (0b10_111_000, 0xAF))),
            ("BEOR B,7,0,<0xAF",  (DirectBit::BitMut(BitMode::EOR, BitInv::AsIs), (0b10_111_000, 0xAF))),
            ("BIEOR B,7,0,<0xAF", (DirectBit::BitMut(BitMode::EOR, BitInv::Inverted), (0b10_111_000, 0xAF))),
            ("BOR B,7,0,<0xAF",   (DirectBit::BitMut(BitMode::OR, BitInv::AsIs), (0b10_111_000, 0xAF))),
            ("BIOR B,7,0,<0xAF",  (DirectBit::BitMut(BitMode::OR, BitInv::Inverted), (0b10_111_000, 0xAF))),
        ];

        for (s,e) in data {
            assert_eq!(direct_bit(s), Ok(("", e)));
        }
    }

    #[test]
    fn test_index_post_byte() {
        let data = vec![
            (0b00101000, IndexPostByte::Offset5(StackReg::Y, 0b0_1000)),
            (0b10100000, IndexPostByte::Inc(StackReg::Y)),
            (0b10100010, IndexPostByte::Dec(StackReg::Y)),
            (0b10011111, IndexPostByte::ExtendedIndirect),
            (0b10001111, IndexPostByte::RegW(ModeW::Offset0, IndexType::NonIndirect)),
            (0b10101111, IndexPostByte::RegW(ModeW::Offset16, IndexType::NonIndirect)),
            (0b10010000, IndexPostByte::RegW(ModeW::Offset0, IndexType::Indirect)),
            (0b10110000, IndexPostByte::RegW(ModeW::Offset16, IndexType::Indirect)),
            (0b11001111, IndexPostByte::RegW(ModeW::IncInc, IndexType::NonIndirect)),
            (0b11101111, IndexPostByte::RegW(ModeW::DecDec, IndexType::NonIndirect)),
            (0b11010000, IndexPostByte::RegW(ModeW::IncInc, IndexType::Indirect)),
            (0b11110000, IndexPostByte::RegW(ModeW::DecDec, IndexType::Indirect)),
            (0b11100100, IndexPostByte::Standard(StackReg::S, IndexMode::Offset0, IndexType::NonIndirect)),
            (0b11101000, IndexPostByte::Standard(StackReg::S, IndexMode::Offset8, IndexType::NonIndirect)),
            (0b11101001, IndexPostByte::Standard(StackReg::S, IndexMode::Offset16, IndexType::NonIndirect)),
            (0b11100110, IndexPostByte::Standard(StackReg::S, IndexMode::AccA, IndexType::NonIndirect)),
            (0b11100101, IndexPostByte::Standard(StackReg::S, IndexMode::AccB, IndexType::NonIndirect)),
            (0b11101011, IndexPostByte::Standard(StackReg::S, IndexMode::AccD, IndexType::NonIndirect)),
            (0b11100111, IndexPostByte::Standard(StackReg::S, IndexMode::AccE, IndexType::NonIndirect)),
            (0b11101010, IndexPostByte::Standard(StackReg::S, IndexMode::AccF, IndexType::NonIndirect)),
            (0b11101110, IndexPostByte::Standard(StackReg::S, IndexMode::AccW, IndexType::NonIndirect)),
            (0b11100001, IndexPostByte::Standard(StackReg::S, IndexMode::IncInc, IndexType::NonIndirect)),
            (0b11100011, IndexPostByte::Standard(StackReg::S, IndexMode::DecDec, IndexType::NonIndirect)),
            (0b11101100, IndexPostByte::Standard(StackReg::S, IndexMode::PCR8, IndexType::NonIndirect)),
            (0b11101101, IndexPostByte::Standard(StackReg::S, IndexMode::PCR16, IndexType::NonIndirect)),
            (0b11110100, IndexPostByte::Standard(StackReg::S, IndexMode::Offset0, IndexType::Indirect)),
            (0b11111000, IndexPostByte::Standard(StackReg::S, IndexMode::Offset8, IndexType::Indirect)),
            (0b11111001, IndexPostByte::Standard(StackReg::S, IndexMode::Offset16, IndexType::Indirect)),
            (0b11110110, IndexPostByte::Standard(StackReg::S, IndexMode::AccA, IndexType::Indirect)),
            (0b11110101, IndexPostByte::Standard(StackReg::S, IndexMode::AccB, IndexType::Indirect)),
            (0b11111011, IndexPostByte::Standard(StackReg::S, IndexMode::AccD, IndexType::Indirect)),
            (0b11110111, IndexPostByte::Standard(StackReg::S, IndexMode::AccE, IndexType::Indirect)),
            (0b11111010, IndexPostByte::Standard(StackReg::S, IndexMode::AccF, IndexType::Indirect)),
            (0b11111110, IndexPostByte::Standard(StackReg::S, IndexMode::AccW, IndexType::Indirect)),
            (0b11110001, IndexPostByte::Standard(StackReg::S, IndexMode::IncInc, IndexType::Indirect)),
            (0b11110011, IndexPostByte::Standard(StackReg::S, IndexMode::DecDec, IndexType::Indirect)),
            (0b11111100, IndexPostByte::Standard(StackReg::S, IndexMode::PCR8, IndexType::Indirect)),
            (0b11111101, IndexPostByte::Standard(StackReg::S, IndexMode::PCR16, IndexType::Indirect)),
        ];

        for (e,s) in data {
            assert_eq!(
                index_post_byte(s),
                e,
                "index_post_byte = {:#010b} expect = {:#010b}",
                index_post_byte(s),
                e,
            )
        }
    }

    #[test]
    fn test_zero_offset_parse() {
        let data = vec![
            // Stack Reg
            ("0,X",   (IndexArg::IncDec(IncDec::None), WStack::Stack(StackReg::X))),
            (",X",    (IndexArg::IncDec(IncDec::None), WStack::Stack(StackReg::X))),
            ("0,-X",  (IndexArg::IncDec(IncDec::Dec), WStack::Stack(StackReg::X))),
            (",-X",   (IndexArg::IncDec(IncDec::Dec), WStack::Stack(StackReg::X))),
            ("0,X+",  (IndexArg::IncDec(IncDec::Inc), WStack::Stack(StackReg::X))),
            (",X+",   (IndexArg::IncDec(IncDec::Inc), WStack::Stack(StackReg::X))),
            ("0,--X", (IndexArg::IncDec(IncDec::DecDec), WStack::Stack(StackReg::X))),
            (",--X",  (IndexArg::IncDec(IncDec::DecDec), WStack::Stack(StackReg::X))),
            ("0,X++", (IndexArg::IncDec(IncDec::IncInc), WStack::Stack(StackReg::X))),
            (",X++",  (IndexArg::IncDec(IncDec::IncInc), WStack::Stack(StackReg::X))),

            // W Register
            ("0,W",   (IndexArg::IncDec(IncDec::None), WStack::W)),
            (",W",    (IndexArg::IncDec(IncDec::None), WStack::W)),
            ("0,--W", (IndexArg::IncDec(IncDec::DecDec), WStack::W)),
            (",--W",  (IndexArg::IncDec(IncDec::DecDec), WStack::W)),
            ("0,W++", (IndexArg::IncDec(IncDec::IncInc), WStack::W)),
            (",W++",  (IndexArg::IncDec(IncDec::IncInc), WStack::W)),
        ];

        for (s,e) in data {
            assert_eq!(zero_offset_parse(s), Ok(("", e)));
        }
    }

    #[test]
    fn test_imm_parse() {
        let data = vec![
            // n,R   - n = imm5, imm8, imm16
            ("4,Y",    (IndexArg::Imm(4), WStack::Stack(StackReg::Y))),
            ("-4,Y",   (IndexArg::Imm(-4), WStack::Stack(StackReg::Y))),
            ("128,Y",  (IndexArg::Imm(128), WStack::Stack(StackReg::Y))),
            ("-128,Y", (IndexArg::Imm(-128), WStack::Stack(StackReg::Y))),
            ("512,Y",  (IndexArg::Imm(512), WStack::Stack(StackReg::Y))),
            ("-512,Y", (IndexArg::Imm(-512), WStack::Stack(StackReg::Y))),
            // n,W   - n = imm16
            ("512,W",  (IndexArg::Imm(512), WStack::W)),
            ("-512,W", (IndexArg::Imm(-512), WStack::W)),
            // n,PCR - n = imm8, imm16
            ("64,PCR",   (IndexArg::Imm(64), WStack::PCR)),
            ("-64,PCR",  (IndexArg::Imm(-64), WStack::PCR)),
            ("512,PCR",  (IndexArg::Imm(512), WStack::PCR)),
            ("-512,PCR", (IndexArg::Imm(-512), WStack::PCR)),
        ];

        for (s,e) in data {
            assert_eq!(imm_parse(s), Ok(("", e)));
        }
    }

    #[test]
    fn test_acc_stack() {
        let data = vec![
            // acc,R
            ("A,X", (IndexArg::Acc(FullAcc::A), WStack::Stack(StackReg::X))),
            ("D,X", (IndexArg::Acc(FullAcc::D), WStack::Stack(StackReg::X))),
            ("W,U", (IndexArg::Acc(FullAcc::W), WStack::Stack(StackReg::U))),
        ];

        for (s,e) in data {
            assert_eq!(acc_stack(s), Ok(("", e)));
        }
    }

    #[test]
    fn test_index_addr() {
        // We know that the zero_offset_parse/imm_parse/acc_stack parsers work
        // Just pick one example from each to validate the parse
        let data = vec![
            // NonIndirect
            ("0,--X", (IndexType::NonIndirect, (IndexArg::IncDec(IncDec::DecDec), WStack::Stack(StackReg::X)))),
            ("512,W", (IndexType::NonIndirect, (IndexArg::Imm(512), WStack::W))),
            ("W,U",   (IndexType::NonIndirect, (IndexArg::Acc(FullAcc::W), WStack::Stack(StackReg::U)))),
            // Indirect
            ("[0,--X]", (IndexType::Indirect, (IndexArg::IncDec(IncDec::DecDec), WStack::Stack(StackReg::X)))),
            ("[512,W]", (IndexType::Indirect, (IndexArg::Imm(512), WStack::W))),
            ("[W,U]",   (IndexType::Indirect, (IndexArg::Acc(FullAcc::W), WStack::Stack(StackReg::U)))),
            // Extended Indirect
            ("[0xFFFF]", (IndexType::Indirect, (IndexArg::UImm(0xFFFF), WStack::None))),
        ];

        for (s,e) in data {
            assert_eq!(index_addr(s), Ok(("", e)));
        }
    }

    #[test]
    fn test_index_parse_to_post_byte() {
        // Focus on offset logic, the rest are just straight forward transformations
        let data = vec![
            // 0,W ~= ,W
            (
                (IndexType::NonIndirect, IndexArg::IncDec(IncDec::None), WStack::W),
                (IndexPostByte::RegW(ModeW::Offset0, IndexType::NonIndirect), IndexBytes::None),
            ),
            // n,W   - n = imm16
            (
                (IndexType::Indirect, IndexArg::Imm(-512), WStack::W),
                (IndexPostByte::RegW(ModeW::Offset16, IndexType::Indirect), IndexBytes::Two(-512i16 as u16)),
            ),
            // PCR8
            (
                (IndexType::Indirect, IndexArg::Imm(64), WStack::PCR),
                (IndexPostByte::Standard(StackReg::X, IndexMode::PCR8, IndexType::Indirect), IndexBytes::One(64)),
            ),
            // PCR16
            (
                (IndexType::Indirect, IndexArg::Imm(-512), WStack::PCR),
                (IndexPostByte::Standard(StackReg::X, IndexMode::PCR16, IndexType::Indirect), IndexBytes::Two(-512i16 as u16)),
            ),
            // Offset0
            (
                (IndexType::Indirect, IndexArg::IncDec(IncDec::None), WStack::Stack(StackReg::X)),
                (IndexPostByte::Standard(StackReg::X, IndexMode::Offset0, IndexType::Indirect), IndexBytes::None),
            ),
            // Offset5
            (
                (IndexType::NonIndirect, IndexArg::Imm(8), WStack::Stack(StackReg::X)),
                (IndexPostByte::Offset5(StackReg::X, 8), IndexBytes::None),
            ),
            // Offset8
            (
                (IndexType::Indirect, IndexArg::Imm(64), WStack::Stack(StackReg::X)),
                (IndexPostByte::Standard(StackReg::X, IndexMode::Offset8, IndexType::Indirect), IndexBytes::One(64)),
            ),
            // Offset16
            (
                (IndexType::Indirect, IndexArg::Imm(512), WStack::Stack(StackReg::X)),
                (IndexPostByte::Standard(StackReg::X, IndexMode::Offset16, IndexType::Indirect), IndexBytes::Two(512)),
            ),
        ];

        for ((it, ia, ws), e) in data {
            assert_eq!(index_parse_to_post_byte(it, ia, ws), e);
        }
    }

    #[test]
    fn test_indexed() {
        assert_eq!(
            indexed("LEAY [0xFFFF]"),
            Ok(("", (
                Indexed::LEA(StackReg::Y),
                0b10011111,
                IndexBytes::Two(0xFFFF),
            ))),
        );
    }

    #[test]
    fn test_mem_addr_mode() {
        assert_eq!(mem_addr_mode("<0xFF"), Ok(("", MemAddrMode::Direct(0xFF))));
        assert_eq!(mem_addr_mode(">0xFFFF"), Ok(("", MemAddrMode::Extended(0xFFFF))));
        assert_eq!(mem_addr_mode("[0xFFFF]"), Ok(("", MemAddrMode::Indexed(0b10011111, IndexBytes::Two(0xFFFF)))));
    }

    #[test]
    fn test_direct_mem() {
        let data = vec![
            ("ASL <0xFF", (DirectMem::ASL, MemAddrMode::Direct(0xFF))),
            ("LSL <0xFF", (DirectMem::ASL, MemAddrMode::Direct(0xFF))), // ASL
            ("INC >0xFFFF", (DirectMem::INC, MemAddrMode::Extended(0xFFFF))),
            ("STQ [0xFFFF]", (DirectMem::ST(StoreLoad::Q), MemAddrMode::Indexed(0b10011111, IndexBytes::Two(0xFFFF)))),
        ];

        for (s,e) in data {
            assert_eq!(direct_mem(s), Ok(("", e)));
        }
    }

    #[test]
    fn test_logical_mem() {
        assert_eq!(
            logical_mem("AIM 0x43;[64,W]"),
            Ok((
                "",
                (LogicalMem::AIM, 0x43, MemAddrMode::Indexed(0b10110000, IndexBytes::Two(64))),
            ))
        );
    }

    #[test]
    fn test_branch() {
        assert_eq!(
            branch("BCC -16"),
            Ok(("", (Branch::BCC, BranchMode::Short(-16)))),
        );
        assert_eq!(
            branch("LBHS -256"), // BCC
            Ok(("", (Branch::BCC, BranchMode::Long(-256)))),
        );
        assert_eq!(
            branch("BCS -16"),
            Ok(("", (Branch::BCS, BranchMode::Short(-16)))),
        );
        assert_eq!(
            branch("LBLO -256"), // BCS
            Ok(("", (Branch::BCS, BranchMode::Long(-256)))),
        );
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
        assert_eq!(number::<i16>("-16_000"), Ok(("", -16_000)));
    }

    #[test]
    fn test_number_too_big_parse() {
    }
}
