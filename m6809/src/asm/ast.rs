use twiddle::Twiddle;

use std::str::FromStr;


// Just specify full instruction AST, easier
#[derive(Debug, PartialEq, Clone)]
pub enum Inst {
    // Inherit
    ABX, DAA, MUL, NOP, RTI, RTS, SEX, SEXW,
    SWI, SWI2, SWI3, SYNC,

    // ADCA, ADCB, - imm8
    // ADCD, - imm16
    ADC(Reg6809, AddrMode),

    // SBCA, SBCB - imm8
    // SBCD - imm16
    SBC(Reg6809, AddrMode),

    // ORA, ORB - imm8
    // ORD - imm16
    OR(Reg6809, AddrMode),

    // ANDA, ANDB - imm8
    // ANDD - imm16
    AND(Reg6809, AddrMode),

    // ADDA, ADDB, ADDE, ADDF, - imm8
    // ADDD, ADDW - imm16
    ADD(Reg6309, AddrMode),

    // SUBA, SUBB, SUBE, SUBF - imm8
    // SUBD, SUBW - imm16
    SUB(Reg6309, AddrMode),

    // BITA, BITB - imm8
    // BITD - imm16
    BIT(Reg6309, AddrMode),

    // ASLA, ASLB, ASLD
    // Also LSL
    ASL(Reg6809),

    // ASRA, ASRB, ASRD
    ASR(Reg6809),

    // CLRA, CLRB, CLRD
    // CLRE, CLRF, CLRW
    CLR(Reg6309),

    // COMA, COMB, COMD
    // COME, COMF, COMW
    COM(Reg6309),

    // DECA, DECB, DECD
    // DECE, DECF, DECW
    DEC(Reg6309),

    // INCA, INCB, INCD
    // INCE, INCF, INCW
    INC(Reg6309),

    // DECA, DECB, DECD
    // DECE, DECF, DECW
    TST(Reg6309),

    // NEGA, NEGB, NEGD
    NEG(Reg6809),

    // LSRA, LSRB, LSRD, LSRW
    LSR(ShiftReg),

    // CMPA, CMPB, CMPE, CMPF - imm8
    // CMPD, CMPW - imm16
    // CMPS, CMPU, CMPX, CMPY - imm16
    CMP(Reg6309Stack, AddrMode),

    // DIVD -imm8
    // DIVQ -imm16
    DIV(DivReg, AddrMode),

    // EORA, EORB, EORD
    EOR(Reg6809, AddrMode),

    // ROLA, ROLB -imm8
    // ROLD, ROLW -imm16
    ROL(ShiftReg),

    // RORA, RORB - imm8
    // RORD, RORW - imm16
    ROR(ShiftReg),

    // LDA, LDB, LDE, LDF - imm8
    // LDD, LDW - imm16
    // LDS, LDU, LDX, LDY - imm16
    LD(Reg6309Stack, AddrMode),
    // LDQ - imm32
    LDQ(AddrMode),

    // STA, STB, STE, STF - imm8
    // STD, STW - imm16
    // STS, STU, STX, STY - imm16
    ST(Reg6309Stack, AddrMode),
    // STQ - imm32
    STQ(AddrMode),

    // MULD - imm16
    MULD(AddrMode),

    // BAND r, u8, u8, Direct
    // BEOR r, u8, u8, Direct
    // BIAND r, u8, u8, Direct
    // BIEOR r, u8, u8, Direct
    // BIOR r, u8, u8, Direct
    // BOR r, u8, u8, Direct
    BAND(BitReg, u8, u8, Direct),
    BEOR(BitReg, u8, u8, Direct),
    BIAND(BitReg, u8, u8, Direct),
    BIEOR(BitReg, u8, u8, Direct),
    BIOR(BitReg, u8, u8, Direct),
    BOR(BitReg, u8, u8, Direct),
    LDBT(BitReg, u8, u8, Direct),
    STBT(BitReg, u8, u8, Direct),

    // Branching
    // Two mode, imm8 and imm16
    // BCC - imm8, LBCC - imm16
    BCC(BranchMode),
    BCS(BranchMode),
    BEQ(BranchMode),
    BGE(BranchMode),
    BGT(BranchMode),
    BHI(BranchMode),
    BHS(BranchMode),
    BLE(BranchMode),
    BLO(BranchMode),
    BLS(BranchMode),
    BLT(BranchMode),
    BMI(BranchMode),
    BNE(BranchMode),
    BPL(BranchMode),
    BRA(BranchMode),
    BRN(BranchMode),
    BSR(BranchMode),
    BVC(BranchMode),
    BVS(BranchMode),


    // ANDCC - imm8
    // BITMD - imm8
    // CWAI - imm8
    // LDMD - imm8
    // ORCC - imm8
    ANDCC(u8),
    BITMD(u8),
    CWAI(u8),
    LDMD(u8),
    ORCC(u8),

    // Stack Registers (technically a imm8 inst)
    // PSHS, PSHU
    PSH(StackReg, Vec<StackSubReg>),
    PSHW(StackReg),

    // PULS, PULU
    PUL(StackReg, Vec<StackSubReg>),
    PULW(StackReg),

    // Register to Register
    // TODO: could swap the typing here
    ADCR(InterReg, InterReg),
    SBCR(InterReg, InterReg),
    ADDR(InterReg, InterReg),
    ANDR(InterReg, InterReg),
    CMPR(InterReg, InterReg),
    EORR(InterReg, InterReg),
    EXG(InterReg, InterReg),
    ORR(InterReg, InterReg),
    SUBR(InterReg, InterReg),
    TFR(InterReg, InterReg),

    // Weird one - uses InterReg but only admits
    // X, Y, U, S, D
    TFM(TfmMode, InterReg, InterReg),

    // LEAS, LEAU, LEAX, LEAY - Indexed
    //NonIndirect(IndexType),
    //Indirect(IndexType),
    LEA(LeaReg, AddrMode),

    // Imm8 & Address
    // Direct/NonIndirect/Indirect/Extended
    AIM(u8, AddrMode),
    EIM(u8, AddrMode),
    OIM(u8, AddrMode),
    TIM(u8, AddrMode),

    // Direct/NonIndirect/Indirect/Extended
    ASLaddr(AddrMode), // ALSO LSLaddr
    ASRaddr(AddrMode),
    CLRaddr(AddrMode),
    COMaddr(AddrMode),
    DECaddr(AddrMode),
    INCaddr(AddrMode),
    LSRaddr(AddrMode),
    NEGaddr(AddrMode),
    ROLaddr(AddrMode),
    RORaddr(AddrMode),
    TSTaddr(AddrMode),

    // TODO: Figure out Effective Address thing, there's a few instruction that uses it, validate
    JMP(AddrMode),
    JSR(AddrMode),
}

#[derive(Debug, PartialEq, Clone)]
pub enum Reg6809 { A, B, D }

#[derive(Debug, PartialEq, Clone)]
pub enum Reg6309 { A, B, E, F, D, W }

#[derive(Debug, PartialEq, Clone)]
pub enum Reg6309Stack { A, B, E, F, D, W, S, U, X, Y }

#[derive(Debug, PartialEq, Clone)]
pub enum StackReg { S, U }

#[derive(Debug, PartialEq, Clone)]
pub enum LeaReg { S, U, X, Y }

#[derive(Debug, PartialEq, Clone)]
pub enum ShiftReg { A, B, D, W }

#[derive(Debug, PartialEq, Clone)]
pub enum DivReg { D, Q }

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
    // TODO: separate the Imm8 from imm16
    Immediate8(u8),
    Immediate16(u16),

    // Rest of memory access
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


// Stack registers
#[derive(Debug, Clone, PartialEq)]
pub enum StackSubReg {
    PC, US, Y, X, DP, B, A, CC
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParseStackSubRegError { _priv: () }

impl FromStr for StackSubReg {
    type Err = ParseStackSubRegError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "PC" => Ok(StackSubReg::PC),
            "U"  => Ok(StackSubReg::US),
            "S"  => Ok(StackSubReg::US),
            "Y"  => Ok(StackSubReg::Y),
            "X"  => Ok(StackSubReg::X),
            "DP" => Ok(StackSubReg::DP),
            "B"  => Ok(StackSubReg::B),
            "A"  => Ok(StackSubReg::A),
            "CC" => Ok(StackSubReg::CC),
            _   => Err(ParseStackSubRegError { _priv: () }),
        }
    }
}

impl From<StackSubReg> for u8 {
    fn from(original: StackSubReg) -> u8 {
        match original {
            StackSubReg::PC => 0b1000_0000,
            StackSubReg::US => 0b0100_0000,
            StackSubReg::Y  => 0b0010_0000,
            StackSubReg::X  => 0b0001_0000,
            StackSubReg::DP => 0b0000_1000,
            StackSubReg::B  => 0b0000_0100,
            StackSubReg::A  => 0b0000_0010,
            StackSubReg::CC => 0b0000_0001,
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
