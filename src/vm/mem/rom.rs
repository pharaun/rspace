use crate::vm::mem::Mem;

const MEM_SIZE: usize = 4096;

pub struct Rom {
    rom: [u8; MEM_SIZE],
}

impl Rom {
    pub fn new(data: [u8; MEM_SIZE]) -> Rom {
        Rom {
            rom: data,
        }
    }
}

impl Mem for Rom {
    fn load_byte(&self, idx: usize) -> u32 {
        self.rom[idx] as u32
    }

    fn load_half(&self, idx: usize) -> u32 {
        self.load_byte(idx) | (self.load_byte(idx+1) << 8)
    }

    fn load_word(&self, idx: usize) -> u32 {
        self.load_half(idx) | (self.load_half(idx+2) << 16)
    }

    // TODO: make these do something, for now ignore writes
    fn store_byte(&mut self, idx: usize, data: u32) {}
    fn store_half(&mut self, idx: usize, data: u32) {}
    fn store_word(&mut self, idx: usize, data: u32) {}
}
