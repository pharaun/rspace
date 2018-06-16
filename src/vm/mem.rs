use std::ops::Index;
use std::ops::IndexMut;

// Memory access stuff
// TODO: compile time size, instead of hardcoded
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

    pub fn fetch_instruction(&self, idx: usize) -> u32 {
        let inst_u8: [u8; 4] = [self[idx], self[idx+1], self[idx+2], self[idx+3]];

        // TODO: better way of doing this
        //unsafe { std::mem::transmute::<[u8; 4], u32>(inst_u8) }
        ((inst_u8[0] as u32)) | ((inst_u8[1] as u32) << 8) | ((inst_u8[2] as u32) << 16) | ((inst_u8[3] as u32) << 24)
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