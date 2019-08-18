use std::ops::Index;
use std::ops::IndexMut;

// Memory access stuff
// TODO: compile time size, instead of hardcoded
//
// NOTE:
// - The memory address space is circular, so that the byte at address 2^X −1 is adjacent
//   to the byte at address zero. Accordingly, memory address computations done by the
//   hardware ignore overflow and instead wrap around modulo 2^X.
//
// - Different address ranges of a hart’s address space may (1) be vacant,
//   or (2) contain main memory, or (3) contain one or more I/O devices.
//
// - The execution environment determines what portions of the non-vacant address
//   space are accessible for each kind of memory access.
//   For example, the set of locations that can be implicitly read for instruction
//   fetch may or may not have any overlap with the set of locations that can be
//   explicitly read by a load instruction; and the set of locations that can be
//   explicitly written by a store instruction may be only a subset of locations that can be read.
//   Ordinarily, if an instruction attempts to access memory at an inaccessible address,
//   an exception is raised for the instruction.
//   Vacant locations in the address space are never accessible.
//
pub struct Memory {
    _rom_hole: u8,
    rom: [u8; 4096],
    ram: [u8; 4096]
}

impl Memory {
    pub fn new(rom: [u8; 4096], ram: [u8; 4096]) -> Memory {
        Memory {
            _rom_hole: 0,
            rom: rom,
            ram: ram
        }
    }

    // TODO: make this raise an error or cause the cpu to enter a trap state on unaligned access
    pub fn fetch_instruction(&self, idx: usize) -> u32 {
        let inst_u8: [u8; 4] = [self[idx], self[idx+1], self[idx+2], self[idx+3]];

        // TODO: better way of doing this
        //unsafe { std::mem::transmute::<[u8; 4], u32>(inst_u8) }
        ((inst_u8[0] as u32)) | ((inst_u8[1] as u32) << 8) | ((inst_u8[2] as u32) << 16) | ((inst_u8[3] as u32) << 24)
    }

    pub fn load_word(&self, idx: usize) -> u32 {
        let bytes: [u8; 4] = [self[idx], self[idx+1], self[idx+2], self[idx+3]];
        ((bytes[0] as u32)) | ((bytes[1] as u32) << 8) | ((bytes[2] as u32) << 16) | ((bytes[3] as u32) << 24)
    }

    pub fn load_half(&self, idx: usize) -> u32 {
        let bytes: [u8; 2] = [self[idx], self[idx+1]];
        ((bytes[0] as u32)) | ((bytes[1] as u32) << 8)
    }

    pub fn load_byte(&self, idx: usize) -> u32 {
        let byte = self[idx];
        ((byte as u32))
    }

    pub fn store_word(&mut self, idx: usize, data: u32) {
        let bytes: [u8; 4] = [
            ((data & 0x00_00_00_FF)) as u8,
            ((data & 0x00_00_FF_00) >> 8) as u8,
            ((data & 0x00_FF_00_00) >> 16) as u8,
            ((data & 0xFF_00_00_00) >> 24) as u8,
        ];

        self[idx]     = bytes[0];
        self[idx + 1] = bytes[1];
        self[idx + 2] = bytes[2];
        self[idx + 3] = bytes[3];
    }

    pub fn store_half(&mut self, idx: usize, data: u32) {
        let bytes: [u8; 2] = [
            ((data & 0x00_00_00_FF)) as u8,
            ((data & 0x00_00_FF_00) >> 8) as u8,
        ];

        self[idx]     = bytes[0];
        self[idx + 1] = bytes[1];
    }

    pub fn store_byte(&mut self, idx: usize, data: u32) {
        let byte  = ((data & 0x00_00_00_FF)) as u8;
        self[idx] = byte;
    }
}

impl Index<usize> for Memory {
    type Output = u8;

    fn index(&self, idx: usize) -> &u8 {
        if idx < 4096 {
            &self.rom[idx]
        } else {
            &self.ram[idx-4096]
        }
    }
}

impl IndexMut<usize> for Memory {
    fn index_mut<'a>(&'a mut self, idx: usize) -> &'a mut u8 {
        if idx < 4096 {
            & mut self._rom_hole
        } else {
            & mut self.ram[idx-4096]
        }
    }
}


// TODO: write more tests for the store+load calls for memory
#[test]
fn memory_test() {
    let mut mem = Memory::new([0; 4096], [0; 4096]);
    mem.rom[0] = 1;
    mem.rom[4095] = 2;
    mem.ram[0] = 3;
    mem.ram[4095] = 4;

    assert_eq!(mem[0], 1);
    assert_eq!(mem[4095], 2);
    assert_eq!(mem[4096], 3);
    assert_eq!(mem[8191], 4);

    // Test writing to.
    mem[0] = 11;
    mem[4095] = 12;
    mem[4096] = 13;
    mem[8191] = 14;

    // ROM hole
    assert_eq!(mem[0], 1);
    assert_eq!(mem[4095], 2);

    // RAM
    assert_eq!(mem[4096], 13);
    assert_eq!(mem[8191], 14);
}

#[test]
fn instruction_memory_test() {
    use std;

    let mut mem = Memory::new([0; 4096], [0; 4096]);

    mem.rom[0] = 1;
    mem.rom[1] = 2;
    mem.rom[2] = 3;
    mem.rom[3] = 4;

    for pc in 0..3 {
        let inst_u8: [u8; 4] = [mem[pc], mem[pc+1], mem[pc+2], mem[pc+3]];
        let inst = unsafe { std::mem::transmute::<[u8; 4], u32>(inst_u8) };

        assert_eq!(mem.fetch_instruction(pc), inst);
    }
}
