use crate::vm::mem::MemIO;
use crate::vm::mem::MemMapId;
use crate::vm::Trap;

use byteorder::{ByteOrder, LittleEndian};

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

            let time: u64 = read_dword(block, 0);
            let timecmp: u64 = read_dword(block, 8);

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
    LittleEndian::read_u16(&block[offset..=(offset+1)])
}

fn read_word(block: &[u8], offset: usize) -> u32 {
    LittleEndian::read_u32(&block[offset..=(offset+3)])
}

fn read_dword(block: &[u8], offset: usize) -> u64 {
    LittleEndian::read_u64(&block[offset..=(offset+7)])
}


fn write_byte(block: &mut [u8], offset: usize, data: u8) {
    block[offset] = data;
}

fn write_half(block: &mut [u8], offset: usize, data: u16) {
    LittleEndian::write_u16(&mut block[offset..=(offset+1)], data);
}

fn write_word(block: &mut [u8], offset: usize, data: u32) {
    LittleEndian::write_u32(&mut block[offset..=(offset+3)], data);
}

fn write_dword(block: &mut [u8], offset: usize, data: u64) {
    LittleEndian::write_u64(&mut block[offset..=(offset+7)], data);
}
