use twiddle::Twiddle;
use arbitrary_int::prelude::u3;
use bitfield_struct::bitfield;

use std::str::FromStr;


// Just specify full instruction AST, easier
#[derive(Debug, PartialEq, Clone)]
pub enum Inst {
    // Edge Case Instructions
    TFM(TfmMode, InterReg, InterReg),
    // LEAS, LEAU, LEAX, LEAY - Indexed
    LEA(LeaReg, IndexAddrMode),
    // Jump - Uses EA (same as LEA but it uses the full addressing modes)
    JMP(AddrMode),
    JSR(AddrMode),

    // Implict Instructions
    // ABX, DAA, MUL, NOP, RTI, RTS, SEX, SEXW, SWI, SWI2, SWI3, SYNC,
    Implict(String),

    // Immedidate Implict
    // ANDCC
    // BITMD
    // CWAI
    // LDMD
    // ORCC
    ImplictImm(String, u8),

    // Implict Register Instructions
    // ASLA, ASLB, ASLD
    // ASRA, ASRB, ASRD
    // NEGA, NEGB, NEGD
    // CLRA, CLRB, CLRD, CLRE, CLRF, CLRW
    // COMA, COMB, COMD, COME, COMF, COMW
    // DECA, DECB, DECD, DECE, DECF, DECW
    // INCA, INCB, INCD, INCE, INCF, INCW
    ImplictRegister(String, AccReg),

    // Implict Register instructions for shifting
    // LSRA, LSRB, LSRD, LSRW
    // ROLA, ROLB, ROLD, ROLW
    // RORA, RORB, RORD, RORW
    ImplictShift(String, ShiftReg),

    // Immedidate Register
    // ADCA, ADCB, ADCD
    // ANDA, ANDB, ANDD
    // BITA, BITB, BITD
    // EORA, EORB, EORD
    // ORA,  ORB,  ORD
    // SBCA, SBCB, SBCD
    // ADDA, ADDB, ADDD, ADDE, ADDF, ADDW
    // CMPA, CMPB, CMPD, CMPE, CMPF, CMPW
    // LDA,  LDB,  LDD,  LDE,  LDF,  LDW
    // STA,  STB,  STD,  STE,  STF,  STW
    // SUBA, SUBB, SUBD, SUBE, SUBF, SUBW
    ImmReg(String, AccReg, u16),

    // Stack Registers (technically a imm8 inst)
    // PSHS, PSHU, PSHSW, PSHUW
    // PULS, PULU, PULSW, PULUW
    Stack(PushPullMode, PushPullReg, PushPullPostByte),
    StackW(PushPullMode, PushPullReg),

    // DIVD - Imm8
    // DIVQ - Imm16
    // MULD - Imm16
    Imm16(String, u16),

    // LDQ
    // STQ
    Imm32(String, u32),

    // Address Register
    // Direct/NonIndirect/Indirect/Extended
    // ADCA, ADCB, ADCD
    // ANDA, ANDB, ANDD
    // BITA, BITB, BITD
    // EORA, EORB, EORD
    // ORA,  ORB,  ORD
    // SBCA, SBCB, SBCD
    // ADDA, ADDB, ADDD, ADDE, ADDF, ADDW
    // CMPA, CMPB, CMPD, CMPE, CMPF, CMPW
    // LDA,  LDB,  LDD,  LDE,  LDF,  LDW
    // STA,  STB,  STD,  STE,  STF,  STW
    // SUBA, SUBB, SUBD, SUBE, SUBF, SUBW
    AddrReg(String, AccReg, AddrMode),

    // DIVD - Imm8
    // DIVQ - Imm16
    // MULD - Imm16
    Addr16Reg(String, AddrMode),

    // LDQ
    // STQ
    Addr32Reg(String, AddrMode),

    // Immedidate Stack Register
    // CMPS, CMPU, CMPX, CMPY
    // LDS,  LDU,  LDX,  LDY
    // STS,  STU,  STX,  STY
    ImmStaReg(String, IndexStackReg, u16),

    // Address Stack Register
    // Direct/NonIndirect/Indirect/Extended
    // CMPS, CMPU, CMPX, CMPY
    // LDS,  LDU,  LDX,  LDY
    AddrStaReg(String, IndexStackReg, AddrMode),

    // Bit memory instruction
    // BAND  r, u8, u8, Direct
    // BIAND r, u8, u8, Direct
    // BEOR  r, u8, u8, Direct
    // BIEOR r, u8, u8, Direct
    // BOR   r, u8, u8, Direct
    // BIOR  r, u8, u8, Direct
    // LDBT  r, u8, u8, Direct
    // STBT  r, u8, u8, Direct
    DirectBit(String, BitReg, u3, u3, Direct),

    // Branching
    // Two mode, imm8 and imm16 - Long/Short branching
    // BCC, LBCC
    // BCS, LBCS
    // BEQ, LBEQ
    // BGE, LBGE
    // BGT, LBGT
    // BHI, LBHI
    // BHS, LBHS
    // BLE, LBLE
    // BLO, LBLO
    // BLS, LBLS
    // BLT, LBLT
    // BMI, LBMI
    // BNE, LBNE
    // BPL, LBPL
    // BRA, LBRA
    // BRN, LBRN
    // BSR, LBSR
    // BVC, LBVC
    // BVS, LBVS
    Branch(String, BranchMode),

    // Register to Register
    // ADCR
    // ADDR
    // ANDR
    // CMPR
    // EORR
    // EXG
    // ORR
    // SBCR
    // SUBR
    // TFR
    RegToReg(String, InterReg, InterReg),

    // Memory to Memory instruction (ie adjusting a byte in memory)
    // Direct/NonIndirect/Indirect/Extended
    // ASL
    // ASR
    // CLR
    // COM
    // DEC
    // INC
    // LSR
    // NEG
    // ROL
    // ROR
    // TST
    MemToMem(String, AddrMode),

    // Imm to Mem (Adjusting a byte in memory via an imm8)
    // Direct/NonIndirect/Indirect/Extended
    // AIM
    // EIM
    // OIM
    // TIM
    ImmToMem(String, u8, AddrMode),
}



#[derive(Debug, PartialEq, Clone)]
pub enum LeaReg { S, U, X, Y }

#[derive(Debug, PartialEq, Clone)]
pub struct Direct(u8);

#[derive(Debug, PartialEq, Clone)]
pub enum TfmMode {
    PlusPlus, // TFM r0+, r1+
    MinusMinus, // TFM r0-, r1-
    PlusNone, // TFM r0+, r1
    NonePlus // TFM r0, r1+
}




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
    Direct(u8),
    Indexed(IndexAddrMode),
    Extended(u16),
}

#[derive(Debug, Clone, PartialEq)]
pub enum IndexAddrMode {
    NonIndirect(IndexType),
    Indirect(IndexType),
}

// Indexed Type
#[derive(Debug, Clone, PartialEq)]
pub enum IndexType {
    ConstOffset(i16, IndexStackReg),
    ConstOffsetW(i16),
    ConstOffsetPC(i16),
    AccOffset(AccReg, IndexStackReg),
    IncOne(IndexStackReg),
    IncTwo(IndexStackReg),
    DecOne(IndexStackReg),
    DecTwo(IndexStackReg),
    IncTwoW,
    DecTwoW,
    Extended(u16),
}


// Indexed Registers
#[derive(Debug, Clone, PartialEq)]
pub enum IndexStackReg {
    X, Y, U, S,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParseIndexStackRegError { _priv: () }

impl FromStr for IndexStackReg {
    type Err = ParseIndexStackRegError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "X" => Ok(IndexStackReg::X),
            "Y" => Ok(IndexStackReg::Y),
            "U" => Ok(IndexStackReg::U),
            "S" => Ok(IndexStackReg::S),
            _   => Err(ParseIndexStackRegError { _priv: () }),
        }
    }
}

impl From<IndexStackReg> for u8 {
    fn from(original: IndexStackReg) -> u8 {
        match original {
            IndexStackReg::X => 0b00,
            IndexStackReg::Y => 0b01,
            IndexStackReg::U => 0b10,
            IndexStackReg::S => 0b11,
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


// Shift Registers
#[derive(Debug, PartialEq, Clone)]
pub enum ShiftReg {
    A, B, D, W,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParseShiftRegError { _priv: () }

impl FromStr for ShiftReg {
    type Err = ParseShiftRegError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "A" => Ok(ShiftReg::A),
            "B" => Ok(ShiftReg::B),
            "D" => Ok(ShiftReg::D),
            "W" => Ok(ShiftReg::W),
            _   => Err(ParseShiftRegError { _priv: () }),
        }
    }
}


// Registers for pushing/pulling to stack
#[derive(Debug, PartialEq, Clone)]
pub enum PushPullReg {
    S, U
}

#[derive(Debug, PartialEq, Clone)]
pub enum PushPullMode {
    Push, Pull
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

#[cfg(test)]
pub mod ast_stack_post_byte {
    use super::*;

    #[test]
    fn test_cc_bit() {
        let expect: u8 = 0b0000_0001;
        let result = PushPullPostByte::new().with_str("CC", true);
        assert_eq!(expect, result.into());
    }

    #[test]
    fn test_pc_dp_cc_bit() {
        let expect: u8 = 0b1000_1001;
        let result = PushPullPostByte::new()
            .with_str("PC", true)
            .with_str("DP", true)
            .with_str("CC", true);
        assert_eq!(expect, result.into());
    }

    #[test]
    fn test_u_bit() {
        let expect: u8 = 0b0100_0000;
        let result = PushPullPostByte::new().with_str("U", true);
        assert_eq!(expect, result.into());
    }

    #[test]
    fn test_s_bit() {
        let expect: u8 = 0b0100_0000;
        let result = PushPullPostByte::new().with_str("S", true);
        assert_eq!(expect, result.into());
    }
}
