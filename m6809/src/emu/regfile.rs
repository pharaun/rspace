use std::fmt;
use std::ops::{Index, IndexMut};

use bitfield_struct::bitfield;

#[derive(Debug)]
enum Acc8 {
    A = 3,
    B = 2,
    E = 1,
    F = 0,
}

#[derive(Debug)]
enum Acc16 {
    D = 1,
    W = 0,
}

#[derive(Debug)]
enum Acc32 {
    Q = 0,
}

union QuadAcc {
    // A / B / E / F
    acc8: [u8; 4],
    // D / W
    acc16: [u16; 2],
    // Q
    acc32: [u32; 1],
}

impl Default for QuadAcc {
    fn default() -> Self { QuadAcc { acc32: [0; 1] } }
}

impl Index<Acc8> for QuadAcc {
    type Output = u8;

    fn index(&self, idx: Acc8) -> &Self::Output {
        unsafe {
            &self.acc8[idx as usize]
        }
    }
}

impl IndexMut<Acc8> for QuadAcc {
    fn index_mut(&mut self, idx: Acc8) -> &mut Self::Output {
        unsafe {
            &mut self.acc8[idx as usize]
        }
    }
}

impl Index<Acc16> for QuadAcc {
    type Output = u16;

    fn index(&self, idx: Acc16) -> &Self::Output {
        unsafe {
            &self.acc16[idx as usize]
        }
    }
}

impl IndexMut<Acc16> for QuadAcc {
    fn index_mut(&mut self, idx: Acc16) -> &mut Self::Output {
        unsafe {
            &mut self.acc16[idx as usize]
        }
    }
}

impl Index<Acc32> for QuadAcc {
    type Output = u32;

    fn index(&self, idx: Acc32) -> &Self::Output {
        unsafe {
            &self.acc32[idx as usize]
        }
    }
}

impl IndexMut<Acc32> for QuadAcc {
    fn index_mut(&mut self, idx: Acc32) -> &mut Self::Output {
        unsafe {
            &mut self.acc32[idx as usize]
        }
    }
}

impl fmt::Debug for QuadAcc {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "QuadAcc {{ ABEF: [A: {:02X} B: {:02X} E: {:02X} F: {:02X}], DW: [D: {:04X} W: {:04X}], Q: {:08X} }}",
            self[Acc8::A], self[Acc8::B], self[Acc8::E], self[Acc8::F],
            self[Acc16::D], self[Acc16::W],
            self[Acc32::Q],
        )
    }
}

#[cfg(test)]
mod test_quad_acc {
    use super::*;

    #[test]
    fn test_acc8() {
        let mut acc = QuadAcc::default();

        acc[Acc8::A] = 0b1000_0001;
        acc[Acc8::B] = 0b0100_0010;
        acc[Acc8::E] = 0b0010_0100;
        acc[Acc8::F] = 0b0001_1000;

        assert_eq!(acc[Acc8::A], 0b1000_0001);
        assert_eq!(acc[Acc8::B], 0b0100_0010);
        assert_eq!(acc[Acc8::E], 0b0010_0100);
        assert_eq!(acc[Acc8::F], 0b0001_1000);
    }

    #[test]
    fn test_acc16() {
        let mut acc = QuadAcc::default();

        acc[Acc16::D] = 0x01_01;
        acc[Acc16::W] = 0x80_80;

        assert_eq!(acc[Acc16::D], 0x01_01);
        assert_eq!(acc[Acc16::W], 0x80_80);
    }

    #[test]
    fn test_acc32() {
        let mut acc = QuadAcc::default();
        acc[Acc32::Q] = 0x80_80_01_01;
        assert_eq!(acc[Acc32::Q], 0x80_80_01_01);
    }

    #[test]
    fn test_acc8_acc16_acc32() {
        let mut acc = QuadAcc::default();

        // A/B
        acc[Acc8::A] = 0b1000_0001;
        acc[Acc8::B] = 0b0100_0010;
        assert_eq!(acc[Acc16::D], 0b1000_0001_0100_0010);

        // E/F
        acc[Acc8::E] = 0b0010_0100;
        acc[Acc8::F] = 0b0001_1000;
        assert_eq!(acc[Acc16::W], 0b0010_0100_0001_1000);

        // Q
        assert_eq!(acc[Acc32::Q], 0b1000_0001_0100_0010_0010_0100_0001_1000);
    }

    #[test]
    fn test_acc32_acc16_acc8() {
        let mut acc = QuadAcc::default();

        acc[Acc32::Q] = 0x80_40_20_10;

        // D/W
        assert_eq!(acc[Acc16::D], 0x80_40);
        assert_eq!(acc[Acc16::W], 0x20_10);

        // A/B/E/F
        assert_eq!(acc[Acc8::A], 0x80);
        assert_eq!(acc[Acc8::B], 0x40);
        assert_eq!(acc[Acc8::E], 0x20);
        assert_eq!(acc[Acc8::F], 0x10);
    }

    #[test]
    fn test_copy_acc8() {
        let mut acc = QuadAcc::default();

        acc[Acc8::E] = 0x80;
        acc[Acc8::A] = acc[Acc8::E] + 0x08;

        assert_eq!(acc[Acc8::E], 0x80);
        assert_eq!(acc[Acc8::A], 0x88);
    }
}

// TODO: figure out if Index/IndexMut will help the ergonomics of this regfile but for now
// let's treat it as a regular structure since there's no overlapping fields or anything
#[derive(Debug)]
pub struct RegFile {
    acc: QuadAcc,
    // Index
    x: u16, y: u16,
    // Stack
    u: u16, s: u16,
    // Program Counter
    pc: u16,
    //Transfer
    v: u16,
    // Zero register?

    // Direct Page
    dp: u8,

    // Flags
    cc: ConditionCode,
    md: CpuMode,
}

#[bitfield(u8, order=Msb)]
#[derive(PartialEq)]
struct ConditionCode {
    e: bool, // 0b1000_0000
    f: bool,
    h: bool,
    i: bool,
    n: bool,
    z: bool,
    v: bool,
    c: bool, // 0b0000_0001
}

#[bitfield(u8, order=Msb)]
#[derive(PartialEq)]
struct CpuMode {
    // Readable
    d0: bool, // 0b1000_0000
    il: bool,
    // Unused
    #[bits(4, default = 0)]
    __: u8,
    // Writeable
    fm: bool,
    nm: bool, // 0b0000_0001
}
