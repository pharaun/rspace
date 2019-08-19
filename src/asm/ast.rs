use std::str::FromStr;

#[derive(Debug,Clone)]
pub enum Labels <'input> {
    NLabel(&'input str),
    WLabel(&'input str),
}

#[derive(Debug,Clone)]
pub enum Args <'input> {
    Num(u32),
    Reg(Reg),
    Csr(Csr),
    Lab(Labels<'input>),
}

#[derive(Debug)]
pub enum AsmLine <'input> {
    Lab(Labels<'input>),
    Ins(&'input str, Vec<Args <'input>>),
    Lns(Labels<'input>, &'input str, Vec<Args <'input>>),
}

#[derive(Debug, Clone, PartialEq)]
pub enum Csr {
    // Machine Information Registers
    MVENDORID, MARCHID, MIMPID, MHARTID,

    // Machine Trap Setup
    MSTATUS, MISA,
    MEDELEG, MIDELEG,
    MIE, MTVEC,
    MCOUNTEREN,

    // Machine Trap Handling
    MSCRATCH, MEPC, MCAUSE, MTVAL, MIP
}

impl From<Csr> for u32 {
    fn from(original: Csr) -> u32 {
        match original {
            Csr::MVENDORID  => 0xF11, // MRO
            Csr::MARCHID    => 0xF12, // MRO
            Csr::MIMPID     => 0xF13, // MRO
            Csr::MHARTID    => 0xF14, // MRO
            Csr::MSTATUS    => 0x300, // MRW
            Csr::MISA       => 0x301, // MRW
            Csr::MEDELEG    => 0x302, // MRW
            Csr::MIDELEG    => 0x303, // MRW
            Csr::MIE        => 0x304, // MRW
            Csr::MTVEC      => 0x305, // MRW
            Csr::MCOUNTEREN => 0x306, // MRW
            Csr::MSCRATCH   => 0x340, // MRW
            Csr::MEPC       => 0x341, // MRW
            Csr::MCAUSE     => 0x342, // MRW
            Csr::MTVAL      => 0x343, // MRW
            Csr::MIP        => 0x344, // MRW
        }
    }
}

impl FromStr for Csr {
    type Err = ParseCsrError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "MVENDORID"  => Ok(Csr::MVENDORID),
            "MARCHID"    => Ok(Csr::MARCHID),
            "MIMPID"     => Ok(Csr::MIMPID),
            "MHARTID"    => Ok(Csr::MHARTID),
            "MSTATUS"    => Ok(Csr::MSTATUS),
            "MISA"       => Ok(Csr::MISA),
            "MEDELEG"    => Ok(Csr::MEDELEG),
            "MIDELEG"    => Ok(Csr::MIDELEG),
            "MIE"        => Ok(Csr::MIE),
            "MTVEC"      => Ok(Csr::MTVEC),
            "MCOUNTEREN" => Ok(Csr::MCOUNTEREN),
            "MSCRATCH"   => Ok(Csr::MSCRATCH),
            "MEPC"       => Ok(Csr::MEPC),
            "MCAUSE"     => Ok(Csr::MCAUSE),
            "MTVAL"      => Ok(Csr::MTVAL),
            "MIP"        => Ok(Csr::MIP),
            _            => Err(ParseCsrError { _priv: () }),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParseCsrError { _priv: () }


#[derive(Debug, Clone, PartialEq)]
pub enum Reg {
    X0, X1, X2, X3, X4, X5, X6, X7, X8, X9,
    X10, X11, X12, X13, X14, X15, X16, X17, X18, X19,
    X20, X21, X22, X23, X24, X25, X26, X27, X28, X29,
    X30, X31
}

impl From<Reg> for u32 {
    fn from(original: Reg) -> u32 {
        match original {
            Reg::X0  => 0,
            Reg::X1  => 1,
            Reg::X2  => 2,
            Reg::X3  => 3,
            Reg::X4  => 4,
            Reg::X5  => 5,
            Reg::X6  => 6,
            Reg::X7  => 7,
            Reg::X8  => 8,
            Reg::X9  => 9,
            Reg::X10  => 10,
            Reg::X11  => 11,
            Reg::X12  => 12,
            Reg::X13  => 13,
            Reg::X14  => 14,
            Reg::X15  => 15,
            Reg::X16  => 16,
            Reg::X17  => 17,
            Reg::X18  => 18,
            Reg::X19  => 19,
            Reg::X20  => 20,
            Reg::X21  => 21,
            Reg::X22  => 22,
            Reg::X23  => 23,
            Reg::X24  => 24,
            Reg::X25  => 25,
            Reg::X26  => 26,
            Reg::X27  => 27,
            Reg::X28  => 28,
            Reg::X29  => 29,
            Reg::X30  => 30,
            Reg::X31  => 31,
        }
    }
}

impl FromStr for Reg {
    type Err = ParseRegError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "zero" | "x0"      => Ok(Reg::X0),
            "ra" | "x1"        => Ok(Reg::X1),
            "sp" | "x2"        => Ok(Reg::X2),
            "gp" | "x3"        => Ok(Reg::X3),
            "tp" | "x4"        => Ok(Reg::X4),
            "t0" | "x5"        => Ok(Reg::X5),
            "t1" | "x6"        => Ok(Reg::X6),
            "t2" | "x7"        => Ok(Reg::X7),
            "s0" | "fp" | "x8" => Ok(Reg::X8),
            "s1" | "x9"        => Ok(Reg::X9),
            "a0" | "x10"       => Ok(Reg::X10),
            "a1" | "x11"       => Ok(Reg::X11),
            "a2" | "x12"       => Ok(Reg::X12),
            "a3" | "x13"       => Ok(Reg::X13),
            "a4" | "x14"       => Ok(Reg::X14),
            "a5" | "x15"       => Ok(Reg::X15),
            "a6" | "x16"       => Ok(Reg::X16),
            "a7" | "x17"       => Ok(Reg::X17),
            "s2" | "x18"       => Ok(Reg::X18),
            "s3" | "x19"       => Ok(Reg::X19),
            "s4" | "x20"       => Ok(Reg::X20),
            "s5" | "x21"       => Ok(Reg::X21),
            "s6" | "x22"       => Ok(Reg::X22),
            "s7" | "x23"       => Ok(Reg::X23),
            "s8" | "x24"       => Ok(Reg::X24),
            "s9" | "x25"       => Ok(Reg::X25),
            "s10" | "x26"      => Ok(Reg::X26),
            "s11" | "x27"      => Ok(Reg::X27),
            "t3" | "x28"       => Ok(Reg::X28),
            "t4" | "x29"       => Ok(Reg::X29),
            "t5" | "x30"       => Ok(Reg::X30),
            "t6" | "x31"       => Ok(Reg::X31),
            _                  => Err(ParseRegError { _priv: () }),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParseRegError { _priv: () }
