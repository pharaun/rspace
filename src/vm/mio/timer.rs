use crate::vm::mem::Mem;
use crate::vm::Trap;

// TODO: Memory-mapped read-write register in memory
// - mtime (64 bit register)
// - mtimecmp (64 bit register)
// - Details: 3.1.10 Machine Timer Registers (mtime and mtimecmp) (riscv-priv)
// - This is the timer-interrupt source
pub struct Timer {
    time: u64,
    timecmp: u64,
    addr: u32,
}

impl Timer {
    pub fn new(time: u64, timecmp: u64, addr: u32) -> Timer {
        Timer {
            time: time,
            timecmp: timecmp,
            addr: addr,
        }
    }
}


// TODO: look into providing common utilities for '8/16/32 bit memory read/write' to various
// integer fields so that this can be abstracted a bit and made nicer for new hardware to be wired
// up to the mio
//
// Macro for handling the bounds check for read/write
macro_rules! bound_read {
    ($self:ident, $bounds:expr, $mask:expr, $idx:expr) => {
        {
            let addr_idx = $idx - $self.addr;

            if addr_idx <= $bounds {
                let combo = ($self.time as u128) | (($self.timecmp as u128) << 64);

                // 32 bit / 4 (byte addressing) = 8
                Ok(((combo >> (addr_idx * 8)) as u32) & $mask)
            } else {
                //panic!("Attempting to read at: 0x{:08x} timer base addr is: 0x{:08x}", $idx, $self.addr);
                Err(Trap::IllegalMemoryAccess($idx))
            }
        }
    }
}

macro_rules! bound_write {
    ($self:ident, $bounds:expr, $mask:expr, $idx:expr, $data:expr) => {
        {
            let int_mask = 0xFF_FF_FF_FF_FF_FF_FF_FF;
            let addr_idx = $idx - $self.addr;
            let data = $data & $mask;

            // addr + 16 -> [8 byte - timecmp] [8 byte - time] <- addr
            if addr_idx < 8 {
                // We zero out the area to insert, then we insert the value
                $self.time &= int_mask ^ ($mask << (addr_idx * 8));
                $self.time |= (data as u64) << (addr_idx * 8);

                Ok(())
            } else if (addr_idx >= 8) && (addr_idx <= $bounds) {
                let addr_idx = addr_idx - 8;

                // We zero out the area to insert, then we insert the value
                $self.timecmp &= int_mask ^ ($mask << (addr_idx * 8));
                $self.timecmp |= (data as u64) << (addr_idx * 8);

                Ok(())
            } else {
                //panic!("Attempting to write at: 0x{:08x} timer base addr is: 0x{:08x} data: 0x{:08x}", $idx, $self.addr, $data);
                Err(Trap::IllegalMemoryAccess($idx))
            }
        }
    }
}

// We implement memory access assuming addr is base ie
// addr + 16 -> [8 byte - timecmp] [8 byte - time] <- addr
// [4 byte] [4 byte] [4 byte] [4 byte]
//  F E D C  B A 9 8  7 6 5 4  3 2 1 0
impl Mem for Timer {
    fn load_byte(&self, idx: u32) -> Result<u32, Trap> {
        bound_read!(self, 12, 0x00_00_00_FF, idx)
    }

    fn load_half(&self, idx: u32) -> Result<u32, Trap> {
        bound_read!(self, 12, 0x00_00_FF_FF, idx)
    }

    fn load_word(&self, idx: u32) -> Result<u32, Trap> {
        bound_read!(self, 12, 0xFF_FF_FF_FF, idx)
    }

    fn store_byte(&mut self, idx: u32, data: u32) -> Result<(), Trap> {
        bound_write!(self, 16, 0x00_00_00_FF, idx, data)
    }

    fn store_half(&mut self, idx: u32, data: u32) -> Result<(), Trap> {
        bound_write!(self, 14, 0x00_00_FF_FF, idx, data)
    }

    fn store_word(&mut self, idx: u32, data: u32) -> Result<(), Trap> {
        bound_write!(self, 12, 0xFF_FF_FF_FF, idx, data)
    }
}


#[test]
fn time_read_word() {
    let timer = Timer::new(0x10_20_30_40_50_60_70_80, 0, 0x100);

    assert_eq!(timer.load_word(0x100).unwrap(), 0x50_60_70_80);
    assert_eq!(timer.load_word(0x104).unwrap(), 0x10_20_30_40);
}

#[test]
fn time_write_word() {
    let mut timer = Timer::new(0xFF_FF_FF_FF_FF_FF_FF_FF, 0, 0x100);

    timer.store_word(0x100, 0x50_60_70_80).unwrap();
    timer.store_word(0x104, 0x10_20_30_40).unwrap();

    assert_eq!(timer.time, 0x10_20_30_40_50_60_70_80);
}

#[test]
fn timecmp_read_word() {
    let timer = Timer::new(0, 0x10_20_30_40_50_60_70_80, 0x100);

    assert_eq!(timer.load_word(0x108).unwrap(), 0x50_60_70_80);
    assert_eq!(timer.load_word(0x10C).unwrap(), 0x10_20_30_40);
}

#[test]
fn timecmp_write_word() {
    let mut timer = Timer::new(0, 0xFF_FF_FF_FF_FF_FF_FF_FF, 0x100);

    timer.store_word(0x108, 0x50_60_70_80).unwrap();
    timer.store_word(0x10C, 0x10_20_30_40).unwrap();

    assert_eq!(timer.timecmp, 0x10_20_30_40_50_60_70_80);
}

#[test]
#[should_panic]
fn invalid_read_word() {
    let timer = Timer::new(0, 0, 0x100);
    timer.load_word(0x10D).unwrap();
}

#[test]
#[should_panic]
fn invalid_write_word() {
    let mut timer = Timer::new(0, 0, 0x100);
    timer.store_word(0x10D, 0x10).unwrap();
}

#[test]
fn time_read_half() {
    let timer = Timer::new(0x10_20_30_40_50_60_70_80, 0, 0x100);

    assert_eq!(timer.load_half(0x100).unwrap(), 0x70_80);
    assert_eq!(timer.load_half(0x102).unwrap(), 0x50_60);
    assert_eq!(timer.load_half(0x104).unwrap(), 0x30_40);
    assert_eq!(timer.load_half(0x106).unwrap(), 0x10_20);
}

#[test]
fn time_write_half() {
    let mut timer = Timer::new(0xFF_FF_FF_FF_FF_FF_FF_FF, 0, 0x100);

    timer.store_half(0x100, 0x70_80).unwrap();
    timer.store_half(0x102, 0x50_60).unwrap();
    timer.store_half(0x104, 0x30_40).unwrap();
    timer.store_half(0x106, 0x10_20).unwrap();

    assert_eq!(timer.time, 0x10_20_30_40_50_60_70_80);
}

#[test]
#[should_panic]
fn invalid_read_half() {
    let timer = Timer::new(0, 0, 0x100);
    timer.load_half(0x10F).unwrap();
}

#[test]
#[should_panic]
fn invalid_write_half() {
    let mut timer = Timer::new(0, 0, 0x100);
    timer.store_half(0x10F, 0x10).unwrap();
}

#[test]
fn time_read_byte() {
    let timer = Timer::new(0x10_20_30_40_50_60_70_80, 0, 0x100);

    assert_eq!(timer.load_byte(0x100).unwrap(), 0x80);
    assert_eq!(timer.load_byte(0x101).unwrap(), 0x70);
    assert_eq!(timer.load_byte(0x102).unwrap(), 0x60);
    assert_eq!(timer.load_byte(0x103).unwrap(), 0x50);
    assert_eq!(timer.load_byte(0x104).unwrap(), 0x40);
    assert_eq!(timer.load_byte(0x105).unwrap(), 0x30);
    assert_eq!(timer.load_byte(0x106).unwrap(), 0x20);
    assert_eq!(timer.load_byte(0x107).unwrap(), 0x10);
}

#[test]
fn time_write_byte() {
    let mut timer = Timer::new(0xFF_FF_FF_FF_FF_FF_FF_FF, 0, 0x100);

    timer.store_byte(0x100, 0x80).unwrap();
    timer.store_byte(0x101, 0x70).unwrap();
    timer.store_byte(0x102, 0x60).unwrap();
    timer.store_byte(0x103, 0x50).unwrap();
    timer.store_byte(0x104, 0x40).unwrap();
    timer.store_byte(0x105, 0x30).unwrap();
    timer.store_byte(0x106, 0x20).unwrap();
    timer.store_byte(0x107, 0x10).unwrap();

    assert_eq!(timer.time, 0x10_20_30_40_50_60_70_80);
}

#[test]
#[should_panic]
fn invalid_read_byte() {
    let timer = Timer::new(0, 0, 0x100);
    timer.load_byte(0x110).unwrap();
}

#[test]
#[should_panic]
fn invalid_write_byte() {
    let mut timer = Timer::new(0, 0, 0x100);
    timer.store_byte(0x110, 0x10).unwrap();
}
