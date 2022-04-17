use crate::vm::mem::MemIO;
use crate::vm::mem::MemMapId;
use crate::vm::Trap;
use crate::vm::mem_util;

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
            block_id,
        }
    }

    pub fn step(&mut self, mem_map: &mut impl MemIO) -> Result<(), Trap> {
        // Read out the time and timecmp
        let (mut time, timecmp) = {
            let block = mem_map.get(self.block_id).unwrap();

            let time: u64 = mem_util::read_dword(block, 0);
            let timecmp: u64 = mem_util::read_dword(block, 8);

            (time, timecmp)
        };

        // One tick
        time += 1;

        // Write time out
        {
            let block = mem_map.get_mut(self.block_id).unwrap();
            mem_util::write_dword(block, 0, time);
        }

        if time >= timecmp {
            Err(Trap::InterruptTimer)
        } else {
            Ok(())
        }
    }
}
