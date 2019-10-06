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

            let mut time: u64 = 0x0;
            time |= (block[0] << 0) as u64;
            time |= (block[1] << 8) as u64;
            time |= (block[2] << 16) as u64;
            time |= (block[3] << 24) as u64;
            time |= (block[4] << 32) as u64;
            time |= (block[5] << 40) as u64;
            time |= (block[6] << 48) as u64;
            time |= (block[7] << 56) as u64;

            let mut timecmp: u64 = 0x0;
            timecmp |= (block[8] << 0) as u64;
            timecmp |= (block[9] << 8) as u64;
            timecmp |= (block[10] << 16) as u64;
            timecmp |= (block[11] << 24) as u64;
            timecmp |= (block[12] << 32) as u64;
            timecmp |= (block[13] << 40) as u64;
            timecmp |= (block[14] << 48) as u64;
            timecmp |= (block[15] << 56) as u64;

            (time, timecmp)
        };

        // One tick
        time += 1;

        // Write time out
        {
            let block = mem_map.get_mut(self.block_id).unwrap();

            block[0] = ((time >> 0) & 0xFF) as u8;
            block[1] = ((time >> 8) & 0xFF) as u8;
            block[2] = ((time >> 16) & 0xFF) as u8;
            block[3] = ((time >> 24) & 0xFF) as u8;
            block[4] = ((time >> 32) & 0xFF) as u8;
            block[5] = ((time >> 40) & 0xFF) as u8;
            block[6] = ((time >> 48) & 0xFF) as u8;
            block[7] = ((time >> 56) & 0xFF) as u8;
        }

        if time >= timecmp {
            Err(Trap::InterruptTimer)
        } else {
            Ok(())
        }
    }
}
