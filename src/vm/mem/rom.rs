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

    fn store_byte(&mut self, idx: usize, data: u32) {
        panic!("Tried to write 0x{:02x} to 0x{:08x}", data, idx);
    }
    fn store_half(&mut self, idx: usize, data: u32) {
        panic!("Tried to write 0x{:04x} to 0x{:08x}", data, idx);
    }
    fn store_word(&mut self, idx: usize, data: u32) {
        panic!("Tried to write 0x{:08x} to 0x{:08x}", data, idx);
    }
}


#[test]
fn load_byte() {
    let mut mem = [0; MEM_SIZE];
    mem[1] = 0x10;
    mem[2] = 0x20;
    mem[3] = 0x30;
    mem[4] = 0x40;

    let mut rom = Rom::new(mem);

    assert_eq!(rom.load_byte(1), 0x10);
}

#[test]
fn load_half() {
    let mut mem = [0; MEM_SIZE];
    mem[1] = 0x10;
    mem[2] = 0x20;
    mem[3] = 0x30;
    mem[4] = 0x40;

    let mut rom = Rom::new(mem);

    assert_eq!(rom.load_half(1), 0x2010);
}

#[test]
fn load_word() {
    let mut mem = [0; MEM_SIZE];
    mem[1] = 0x10;
    mem[2] = 0x20;
    mem[3] = 0x30;
    mem[4] = 0x40;

    let mut rom = Rom::new(mem);

    assert_eq!(rom.load_word(1), 0x40302010);
}

#[test]
#[should_panic]
fn store_byte() {
    let mut rom = Rom::new([0; MEM_SIZE]);
    rom.store_byte(1, 0x10);
}

#[test]
#[should_panic]
fn store_half() {
    let mut rom = Rom::new([0; MEM_SIZE]);
    rom.store_half(1, 0x2010);
}

#[test]
#[should_panic]
fn store_word() {
    let mut rom = Rom::new([0; MEM_SIZE]);
    rom.store_word(1, 0x40302010);
}
