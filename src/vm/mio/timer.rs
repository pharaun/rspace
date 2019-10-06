use crate::vm::mem::Mem;
use crate::vm::mem::MemIO;
use crate::vm::mem::MemMapId;
use crate::vm::Trap;

// TODO: Memory-mapped read-write register in memory
// - mtime (64 bit register)
// - mtimecmp (64 bit register)
// - Details: 3.1.10 Machine Timer Registers (mtime and mtimecmp) (riscv-priv)
// - This is the timer-interrupt source
pub struct Timer {
    block_id: MemMapId,
}

impl Timer {
    // TODO: have a way for this to grab a mem block and write the specified
    // time: u64 and timecmp: u64
    pub fn new(block_id: MemMapId) -> Timer {
        Timer {
            block_id: block_id,
        }
    }

    // TODO: make this nice and provide utilities for reading out 8, 16, 32, 64 bits out of a
    // mem_map block
    pub fn step(&mut self, mem_map: &mut impl MemIO) -> Result<(), Trap> {
        // Read out the time and timecmp
        let (mut time, timecmp) = {
            let block = mem_map.get(self.block_id).unwrap();

            let mut time: u64 = read_dword(block, 0);
            let mut timecmp: u64 = read_dword(block, 8);

            (time, timecmp)
        };

        // One tick
        time += 1;

        // Write time out
        {
            let block = mem_map.get_mut(self.block_id).unwrap();
            write_dword(block, 0, time);
        }

        if time >= timecmp {
            Err(Trap::InterruptTimer)
        } else {
            Ok(())
        }
    }
}


fn read_byte(block: &[u8], offset: usize) -> u8 {
    block[offset]
}

fn read_half(block: &[u8], offset: usize) -> u16 {
    read_byte(block, offset) as u16 | (read_byte(block, offset + 1) as u16) << 8
}

fn read_word(block: &[u8], offset: usize) -> u32 {
    read_half(block, offset) as u32 | (read_byte(block, offset + 2) as u32) << 16
}

fn read_dword(block: &[u8], offset: usize) -> u64 {
    read_word(block, offset) as u64 | (read_word(block, offset + 4) as u64) << 32
}


fn write_byte(block: &mut [u8], offset: usize, data: u8) {
    block[offset] = data;
}

fn write_half(block: &mut [u8], offset: usize, data: u16) {
    write_byte(block, offset, (data & 0xFF) as u8);
    write_byte(block, offset + 1, ((data >> 8) & 0xFF) as u8);
}

fn write_word(block: &mut [u8], offset: usize, data: u32) {
    write_half(block, offset, (data & 0xFF_FF) as u16);
    write_half(block, offset + 2, ((data >> 16) & 0xFF_FF) as u16);
}

fn write_dword(block: &mut [u8], offset: usize, data: u64) {
    write_word(block, offset, (data & 0xFF_FF_FF_FF) as u32);
    write_word(block, offset + 4, ((data >> 32) & 0xFF_FF_FF_FF) as u32);
}
