pub mod ram;
pub mod rom;

use crate::vm::Trap;

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
// TODO: Memory-mapped read-write register in memory
// - mtime (64 bit register)
// - mtimecmp (64 bit register)
// - Details: 3.1.10 Machine Timer Registers (mtime and mtimecmp) (riscv-priv)
// - This is the timer-interrupt source
//
// TODO:
// - implement some sort of memory map (ie this block is ro, this block is rw, this block is i/o
// for xyz) (see sector 3.5 - PMA - physical memory attributes)
// - 3.5.1 Main Memory versus I/O versus Empty Regions
pub trait Mem {
    fn load_byte(&self, idx: u32) -> Result<u32, Trap>;
    fn load_half(&self, idx: u32) -> Result<u32, Trap>;
    fn load_word(&self, idx: u32) -> Result<u32, Trap>;

    // TODO: consider maybe two memory traits (one for read one for write)?
    fn store_byte(&mut self, idx: u32, data: u32) -> Result<(), Trap>;
    fn store_half(&mut self, idx: u32, data: u32) -> Result<(), Trap>;
    fn store_word(&mut self, idx: u32, data: u32) -> Result<(), Trap>;
}


// Memory Map
pub struct MemMap {
    // Start (inclusive), End (exclusive), Memory
    map: Vec<(u32, u32, Box<dyn Mem>)>,
}

impl MemMap {
    pub fn new() -> MemMap {
        MemMap {
            map: vec![],
        }
    }

    // TODO: do more work upfront to make fetching cheaper, but for now be dumb
    pub fn add<T: Mem + 'static>(&mut self, start: u32, end: u32, mem: T) {
        self.map.push(
            (start, end, Box::new(mem))
        );
    }

    pub fn fetch_instruction(&self, idx: u32) -> Result<u32, Trap> {
        // If inst is read from non u32 aligned address, error out (ISA specifies this)
        if idx % 4 != 0 {
            Err(Trap::UnalignedInstructionAccess(idx))
        } else {
            self.load_word(idx).and_then(|x|
                // If inst is all 0 or all 1's error out (illegal instruction)
                if x == 0x0 {
                    Err(Trap::IllegalInstruction(x))
                } else if x == 0xFF_FF_FF_FF {
                    Err(Trap::IllegalInstruction(x))
                } else {
                    Ok(x)
                }
            )
        }
    }
}


// Macro for handling the lookup of the memory table
macro_rules! dispatch_to {
    ($self:ident, $func:ident, $idx:expr) => {
        {
            for t in $self.map.iter() {
                let (start, end, mem) = t;

                if ($idx >= *start) && ($idx < *end) {
                    return mem.$func($idx - *start).or(
                        Err(Trap::IllegalMemoryAccess($idx))
                    );
                }
            }

            // TODO: outside bounds of memory map
            // panic!("No memory block at: 0x{:08x}", $idx);
            Err(Trap::IllegalMemoryAccess($idx))
        }
    }
}

macro_rules! mut_dispatch_to {
    ($self:ident, $func:ident, $idx:expr, $data:expr) => {
        {
            for t in $self.map.iter_mut() {
                let (start, end, mem) = t;

                if ($idx >= *start) && ($idx < *end) {
                    return mem.$func($idx - *start, $data).or(
                        Err(Trap::IllegalMemoryAccess($idx))
                    );
                }
            }

            // TODO: outside bounds of memory map
            // panic!("No memory block at: 0x{:08x}", $idx);
            Err(Trap::IllegalMemoryAccess($idx))
        }
    }
}


impl Mem for MemMap {
    fn load_byte(&self, idx: u32) -> Result<u32, Trap> {
        dispatch_to!(self, load_byte, idx)
    }

    fn load_half(&self, idx: u32) -> Result<u32, Trap> {
        dispatch_to!(self, load_half, idx)
    }

    fn load_word(&self, idx: u32) -> Result<u32, Trap> {
        dispatch_to!(self, load_word, idx)
    }

    fn store_byte(&mut self, idx: u32, data: u32) -> Result<(), Trap> {
        mut_dispatch_to!(self, store_byte, idx, data)
    }

    fn store_half(&mut self, idx: u32, data: u32) -> Result<(), Trap> {
        mut_dispatch_to!(self, store_half, idx, data)
    }

    fn store_word(&mut self, idx: u32, data: u32) -> Result<(), Trap> {
        mut_dispatch_to!(self, store_word, idx, data)
    }
}


#[test]
fn roundtrip_byte() {
    let mut mem_map = MemMap::new();
    mem_map.add(0x0, 0x1000, ram::Ram::new());

    mem_map.store_byte(1, 0x10).unwrap();
    assert_eq!(mem_map.load_byte(1).unwrap(), 0x10);
}

#[test]
fn roundtrip_half() {
    let mut mem_map = MemMap::new();
    mem_map.add(0x0, 0x1000, ram::Ram::new());

    mem_map.store_half(1, 0x2010).unwrap();
    assert_eq!(mem_map.load_half(1).unwrap(), 0x2010);
}

#[test]
fn roundtrip_word() {
    let mut mem_map = MemMap::new();
    mem_map.add(0x0, 0x1000, ram::Ram::new());

    mem_map.store_word(1, 0x40302010).unwrap();
    assert_eq!(mem_map.load_word(1).unwrap(), 0x40302010);
}

#[test]
fn dispatch_test() {
    let mut mem1 = [0; 4096];
    mem1[10] = 0x10;

    let mut mem2 = [0; 4096];
    mem2[20] = 0x20;

    let mut mem_map = MemMap::new();
    mem_map.add(0x0000, 0x1000, rom::Rom::new(mem1));
    mem_map.add(0x1000, 0x2000, rom::Rom::new(mem2));

    // Ensure its where we expect it to be
    assert_eq!(mem_map.load_byte(10).unwrap(), 0x10);
    assert_eq!(mem_map.load_byte(20 + 0x1000).unwrap(), 0x20);
}
