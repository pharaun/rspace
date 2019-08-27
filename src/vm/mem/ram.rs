use crate::vm::mem::Mem;

const MEM_SIZE: usize = 4096;

pub struct Ram {
    ram: [u8; MEM_SIZE],
}

impl Ram {
    pub fn new() -> Ram {
        Ram {
            ram: [0; MEM_SIZE],
        }
    }
}

impl Mem for Ram {
    fn load_byte(&self, idx: usize) -> u32 {
        self.ram[idx] as u32
    }

    fn load_half(&self, idx: usize) -> u32 {
        self.load_byte(idx) | (self.load_byte(idx+1) << 8)
    }

    fn load_word(&self, idx: usize) -> u32 {
        self.load_half(idx) | (self.load_half(idx+2) << 16)
    }

    fn store_byte(&mut self, idx: usize, data: u32) {
        self.ram[idx] = ((data & 0x00_00_00_FF) as u8);
    }

    fn store_half(&mut self, idx: usize, data: u32) {
        self.store_byte(idx, data);
        self.store_byte(idx+1, data >> 8);
    }

    fn store_word(&mut self, idx: usize, data: u32) {
        self.store_half(idx, data);
        self.store_half(idx+2, data >> 16);
    }
}

#[test]
fn byte() {
    let mut ram = Ram::new();

    assert_eq!(ram.ram[1], 0x0);

    ram.store_byte(1, 0x10);
    assert_eq!(ram.load_byte(1), 0x10);

    assert_eq!(ram.ram[1], 0x10);
}

#[test]
fn half() {
    let mut ram = Ram::new();

    assert_eq!(ram.ram[1], 0x0);
    assert_eq!(ram.ram[2], 0x0);

    ram.store_half(1, 0x2010);
    assert_eq!(ram.load_half(1), 0x2010);

    assert_eq!(ram.ram[1], 0x10);
    assert_eq!(ram.ram[2], 0x20);
}

#[test]
fn word() {
    let mut ram = Ram::new();

    assert_eq!(ram.ram[1], 0x0);
    assert_eq!(ram.ram[2], 0x0);
    assert_eq!(ram.ram[3], 0x0);
    assert_eq!(ram.ram[4], 0x0);

    ram.store_word(1, 0x40302010);
    assert_eq!(ram.load_word(1), 0x40302010);

    assert_eq!(ram.ram[1], 0x10);
    assert_eq!(ram.ram[2], 0x20);
    assert_eq!(ram.ram[3], 0x30);
    assert_eq!(ram.ram[4], 0x40);
}
