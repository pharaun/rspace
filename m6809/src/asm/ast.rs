use twiddle::Twiddle;

use std::str::FromStr;


#[derive(Debug, PartialEq, Clone)]
pub enum DataType { Byte, Half, Word }

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParseDataTypeError { _priv: () }

impl FromStr for DataType {
    type Err = ParseDataTypeError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "BYTE"  => Ok(DataType::Byte),
            "HALF"  => Ok(DataType::Half),
            "WORD"  => Ok(DataType::Word),
            _       => Err(ParseDataTypeError { _priv: () }),
        }
    }
}


// Inter-Registers
// Z1/Z2 == 2x encoding of 0 register
#[derive(Debug, Clone, PartialEq)]
pub enum InterReg {
    D, X, Y, U, S, PC, W, V,
    A, B, CC, DP, Z1, Z2, E, F,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParseInterRegError { _priv: () }

impl FromStr for InterReg {
    type Err = ParseInterRegError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "D"  => Ok(InterReg::D),
            "X"  => Ok(InterReg::X),
            "Y"  => Ok(InterReg::Y),
            "U"  => Ok(InterReg::U),
            "S"  => Ok(InterReg::S),
            "PC" => Ok(InterReg::PC),
            "W"  => Ok(InterReg::W),
            "V"  => Ok(InterReg::V),
            "A"  => Ok(InterReg::A),
            "B"  => Ok(InterReg::B),
            "CC" => Ok(InterReg::CC),
            "DP" => Ok(InterReg::DP),
            "0"  => Ok(InterReg::Z1), // Or Z2
            "E"  => Ok(InterReg::E),
            "F"  => Ok(InterReg::F),
            _    => Err(ParseInterRegError { _priv: () }),
        }
    }
}

impl From<InterReg> for u8 {
    fn from(original: InterReg) -> u8 {
        match original {
            InterReg::D  => 0b0000,
            InterReg::X  => 0b0001,
            InterReg::Y  => 0b0010,
            InterReg::U  => 0b0011,
            InterReg::S  => 0b0100,
            InterReg::PC => 0b0101,
            InterReg::W  => 0b0110,
            InterReg::V  => 0b0111,
            InterReg::A  => 0b1000,
            InterReg::B  => 0b1001,
            InterReg::CC => 0b1010,
            InterReg::DP => 0b1011,
            InterReg::Z1 => 0b1100,
            InterReg::Z2 => 0b1101,
            InterReg::E  => 0b1110,
            InterReg::F  => 0b1111,
        }
    }
}

pub fn inter_reg_post_byte(r1: InterReg, r2: InterReg) -> u8 {
    (u8::from(r1) << 4) | u8::from(r2)
}


// Bit-Manipulation Registers
#[derive(Debug, Clone, PartialEq)]
pub enum BitReg {
    CC, A, B, Invalid,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParseBitRegError { _priv: () }

impl FromStr for BitReg {
    type Err = ParseBitRegError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "CC" => Ok(BitReg::CC),
            "A"  => Ok(BitReg::A),
            "B"  => Ok(BitReg::B),
            _    => Err(ParseBitRegError { _priv: () }),
        }
    }
}

impl From<BitReg> for u8 {
    fn from(original: BitReg) -> u8 {
        match original {
            BitReg::CC      => 0b00,
            BitReg::A       => 0b01,
            BitReg::B       => 0b10,
            BitReg::Invalid => 0b11,
        }
    }
}

pub fn bit_reg_post_byte(r: BitReg, source: u8, dest: u8) -> u8 {
    let mask = u8::mask(2..=0);
    (u8::from(r) << 6) | ((source & mask) << 3) | (dest & mask)
}


// Addressing Mode
#[derive(Debug, Clone, PartialEq)]
pub enum AddrMode {
    Immediate8(u8),
    Immediate16(u16),
    Direct(u8),
    NonIndirect(IndexType),
    Indirect(IndexType),
    Extended(u16),
    Inherent,
}

// Indexed Type
#[derive(Debug, Clone, PartialEq)]
pub enum IndexType {
    ConstOffset(i16, IndexReg),
    ConstOffsetW(i16),
    ConstOffsetPC(i16),
    AccOffset(AccReg, IndexReg),
    IncOne(IndexReg),
    IncTwo(IndexReg),
    DecOne(IndexReg),
    DecTwo(IndexReg),
    IncTwoW,
    DecTwoW,
    Extended(u16),
}


// Indexed Registers
#[derive(Debug, Clone, PartialEq)]
pub enum IndexReg {
    X, Y, U, S,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParseIndexRegError { _priv: () }

impl FromStr for IndexReg {
    type Err = ParseIndexRegError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "X" => Ok(IndexReg::X),
            "Y" => Ok(IndexReg::Y),
            "U" => Ok(IndexReg::U),
            "S" => Ok(IndexReg::S),
            _   => Err(ParseIndexRegError { _priv: () }),
        }
    }
}

impl From<IndexReg> for u8 {
    fn from(original: IndexReg) -> u8 {
        match original {
            IndexReg::X => 0b00,
            IndexReg::Y => 0b01,
            IndexReg::U => 0b10,
            IndexReg::S => 0b11,
        }
    }
}


// Accumulator registers
#[derive(Debug, Clone, PartialEq)]
pub enum AccReg {
    A, B, D, E, F, W,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParseAccRegError { _priv: () }

impl FromStr for AccReg {
    type Err = ParseAccRegError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "A" => Ok(AccReg::A),
            "B" => Ok(AccReg::B),
            "D" => Ok(AccReg::D),
            "E" => Ok(AccReg::E),
            "F" => Ok(AccReg::F),
            "W" => Ok(AccReg::W),
            _   => Err(ParseAccRegError { _priv: () }),
        }
    }
}

impl From<AccReg> for u8 {
    fn from(original: AccReg) -> u8 {
        match original {
            AccReg::A => 0b0110,
            AccReg::B => 0b0101,
            AccReg::D => 0b1011,
            AccReg::E => 0b0111,
            AccReg::F => 0b1010,
            AccReg::W => 0b1110,
        }
    }
}


// Branch mode
#[derive(Debug, Clone, PartialEq)]
pub enum BranchMode {
    Short(i8),
    Long(i16),
}


#[cfg(test)]
pub mod ast_post_byte {
    use super::*;

    #[test]
    fn test_inter_reg_post_byte() {
        let expect: u8 = 0b1111_0000;
        let result = inter_reg_post_byte(
            InterReg::F,
            InterReg::D,
        );
        assert_eq!(expect, result);
    }

    #[test]
    fn test_bit_reg_post_byte() {
        let expect: u8 = 0b11_010_101;
        let result = bit_reg_post_byte(
            BitReg::Invalid,
            0b0000_0010,
            0b0000_0101,
        );
        assert_eq!(expect, result);
    }
}
