use crate::vm::mem::Mem;
use crate::vm::Trap;

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
    fn load_byte(&self, idx: usize) -> Result<u32, Trap> {
        if idx >= MEM_SIZE {
            Err(Trap::IllegalMemoryAccess(idx as u32))
        } else {
            Ok(self.rom[idx] as u32)
        }
    }

    fn load_half(&self, idx: usize) -> Result<u32, Trap> {
        match (self.load_byte(idx), self.load_byte(idx+1)) {
            (Ok(x), Ok(y))  => Ok(x | (y << 8)),
            (Err(x), _)     => Err(x),
            (_, Err(x))     => Err(x),
        }
    }

    fn load_word(&self, idx: usize) -> Result<u32, Trap> {
        match (self.load_half(idx), self.load_half(idx+2)) {
            (Ok(x), Ok(y))  => Ok(x | (y << 16)),
            (Err(x), _)     => Err(x),
            (_, Err(x))     => Err(x),
        }
    }

    fn store_byte(&mut self, idx: usize, _data: u32) -> Result<(), Trap> {
        //panic!("Tried to write 0x{:02x} to 0x{:08x}", data, idx);
        Err(Trap::IllegalMemoryAccess(idx as u32))
    }
    fn store_half(&mut self, idx: usize, _data: u32) -> Result<(), Trap> {
        //panic!("Tried to write 0x{:02x} to 0x{:08x}", data, idx);
        Err(Trap::IllegalMemoryAccess(idx as u32))
    }
    fn store_word(&mut self, idx: usize, _data: u32) -> Result<(), Trap> {
        //panic!("Tried to write 0x{:02x} to 0x{:08x}", data, idx);
        Err(Trap::IllegalMemoryAccess(idx as u32))
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

    assert_eq!(rom.load_byte(1).unwrap(), 0x10);
}

#[test]
fn load_half() {
    let mut mem = [0; MEM_SIZE];
    mem[1] = 0x10;
    mem[2] = 0x20;
    mem[3] = 0x30;
    mem[4] = 0x40;

    let mut rom = Rom::new(mem);

    assert_eq!(rom.load_half(1).unwrap(), 0x2010);
}

#[test]
fn load_word() {
    let mut mem = [0; MEM_SIZE];
    mem[1] = 0x10;
    mem[2] = 0x20;
    mem[3] = 0x30;
    mem[4] = 0x40;

    let mut rom = Rom::new(mem);

    assert_eq!(rom.load_word(1).unwrap(), 0x40302010);
}

#[test]
#[should_panic]
fn store_byte() {
    let mut rom = Rom::new([0; MEM_SIZE]);
    rom.store_byte(1, 0x10).unwrap();
}

#[test]
#[should_panic]
fn store_half() {
    let mut rom = Rom::new([0; MEM_SIZE]);
    rom.store_half(1, 0x2010).unwrap();
}

#[test]
#[should_panic]
fn store_word() {
    let mut rom = Rom::new([0; MEM_SIZE]);
    rom.store_word(1, 0x40302010).unwrap();
}
