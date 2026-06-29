use std::fmt;
use std::ops::{Index, IndexMut};

use bitfield_struct::bitfield;
use bytemuck::{must_cast_slice, must_cast_slice_mut, Pod, Zeroable};

// Macro for provisioning an Index+Mut impl for more than 1 Idx type
// such as: Acc8/Acc16/Acc32 for QuadAcc
macro_rules! acc_index {
    ($acc:ty, $idx:ty, $out:ty) => {
        impl Index<$idx> for $acc {
            type Output = $out;

            fn index(&self, idx: $idx) -> &Self::Output {
                &must_cast_slice(&self.0)[idx as usize]
            }
        }

        impl IndexMut<$idx> for $acc {
            fn index_mut(&mut self, idx: $idx) -> &mut Self::Output {
                &mut must_cast_slice_mut(&mut self.0)[idx as usize]
            }
        }
    }
}

enum Acc8 {
    A = 3,
    B = 2,
    E = 1,
    F = 0,
}

enum Acc16 {
    D = 1,
    W = 0,
}

enum Acc32 {
    Q = 0,
}

#[derive(Copy, Clone, Pod, Zeroable)]
#[repr(transparent)]
pub(super) struct QuadAcc([u32; 1]);

acc_index!(QuadAcc, Acc8, u8);
acc_index!(QuadAcc, Acc16, u16);
acc_index!(QuadAcc, Acc32, u32);

impl Default for QuadAcc {
    fn default() -> Self {
        QuadAcc::zeroed()
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
        let mut acc = QuadAcc::zeroed();

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
        let mut acc = QuadAcc::zeroed();

        acc[Acc16::D] = 0x01_01;
        acc[Acc16::W] = 0x80_80;

        assert_eq!(acc[Acc16::D], 0x01_01);
        assert_eq!(acc[Acc16::W], 0x80_80);
    }

    #[test]
    fn test_acc32() {
        let mut acc = QuadAcc::zeroed();
        acc[Acc32::Q] = 0x80_80_01_01;
        assert_eq!(acc[Acc32::Q], 0x80_80_01_01);
    }

    #[test]
    fn test_acc8_acc16_acc32() {
        let mut acc = QuadAcc::zeroed();

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
        let mut acc = QuadAcc::zeroed();

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
        let mut acc = QuadAcc::zeroed();

        acc[Acc8::E] = 0x80;
        acc[Acc8::A] = acc[Acc8::E] + 0x08;

        assert_eq!(acc[Acc8::E], 0x80);
        assert_eq!(acc[Acc8::A], 0x88);
    }
}

// TODO: how to handle the 0 register (zero)
#[derive(Debug, Default)]
pub(super) struct RegFile {
    pub acc: QuadAcc,
    // Index
    pub x: u16,
    pub y: u16,
    // Stack
    pub u: u16,
    pub s: u16,
    //Transfer
    pub v: u16,
    // Direct Page
    pub dp: u8,
    // Flags
    pub cc: ConditionCode,
    pub md: CpuMode,
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
