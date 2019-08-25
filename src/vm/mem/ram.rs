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
