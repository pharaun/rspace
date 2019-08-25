pub mod ram;
pub mod rom;

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
    fn load_byte(&self, idx: usize) -> u32;
    fn load_half(&self, idx: usize) -> u32;
    fn load_word(&self, idx: usize) -> u32;

    fn store_byte(&mut self, idx: usize, data: u32);
    fn store_half(&mut self, idx: usize, data: u32);
    fn store_word(&mut self, idx: usize, data: u32);
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

    // TODO: for now impl a instruction fetch
    // TODO: make this raise an error or cause the cpu to enter a trap state on unaligned access
    pub fn fetch_instruction(&self, idx: usize) -> u32 {
        self.load_word(idx)
    }
}


// Macro for handling the lookup of the memory table
macro_rules! dispatch_to {
    ($self:ident, $func:ident, $idx:expr) => {
        {
            let mut ret = Err(());

            for t in $self.map.iter() {
                let (start, end, mem) = t;

                if ($idx >= *start) && ($idx < *end) {
                    ret = Ok(mem.$func(($idx - *start) as usize));
                    break;
                }
            }

            match ret {
                Err(_) => panic!("No memory block at: 0x{:08x}", $idx),
                Ok(x)  => x,
            }
        }
    }
}

macro_rules! mut_dispatch_to {
    ($self:ident, $func:ident, $idx:expr, $data:expr) => {
        {
            let mut success = false;

            for t in $self.map.iter_mut() {
                let (start, end, mem) = t;

                if ($idx >= *start) && ($idx < *end) {
                    mem.$func(($idx - *start) as usize, $data);
                    success = true;
                    break;
                }
            }

            if !success {
                panic!("No memory block at: 0x{:08x}", $idx);
            }
        }
    }
}


impl Mem for MemMap {
    fn load_byte(&self, idx: usize) -> u32 {
        dispatch_to!(self, load_byte, idx as u32)
    }

    fn load_half(&self, idx: usize) -> u32 {
        dispatch_to!(self, load_half, idx as u32)
    }

    fn load_word(&self, idx: usize) -> u32 {
        dispatch_to!(self, load_word, idx as u32)
    }

    fn store_byte(&mut self, idx: usize, data: u32) {
        mut_dispatch_to!(self, store_byte, idx as u32, data);
    }

    fn store_half(&mut self, idx: usize, data: u32) {
        mut_dispatch_to!(self, store_half, idx as u32, data);
    }

    fn store_word(&mut self, idx: usize, data: u32) {
        mut_dispatch_to!(self, store_word, idx as u32, data);
    }
}


//#[test]
//fn ram_test() {
//    let mut ram = Ram::new();
//
//    ram.store_byte(1, 0x10);
//    ram.store_byte(2, 0x11);
//
//    assert_eq!(ram.ram[1], 0x10);
//    assert_eq!(ram.ram[2], 0x11);
//
//    assert_eq!(ram.load_byte(1), 0x10);
//    assert_eq!(ram.load_byte(2), 0x11);
//}
//
//#[test]
//fn mem_map_size_test() {
//    let mut mem_map = MemMap::new();
//    assert_eq!(mem_map.size(), 0);
//
//    mem_map.add(Ram::new());
//    assert_eq!(mem_map.size(), 8);
//
//    mem_map.add(Ram::new());
//    assert_eq!(mem_map.size(), 16);
//}
//
//#[test]
//fn mem_map_store_read_test() {
//    let mut mem_map = MemMap::new();
//    mem_map.add(Ram::new());
//    mem_map.add(Ram::new());
//
//    mem_map.store_byte(1, 0x10);
//    mem_map.store_byte(1+8, 0x11);
//
//    assert_eq!(mem_map.load_byte(1), 0x10);
//    assert_eq!(mem_map.load_byte(1+8), 0x11);
//}
//
//#[test]
//fn mem_map_double_store_read_test() {
//    let mut mem_map_one = MemMap::new();
//    mem_map_one.add(Ram::new());
//
//    let mut mem_map = MemMap::new();
//    mem_map.add(Ram::new());
//    mem_map.add(mem_map_one);
//
//    mem_map.store_byte(1, 0x10);
//    mem_map.store_byte(1+8, 0x11);
//
//    assert_eq!(mem_map.load_byte(1), 0x10);
//    assert_eq!(mem_map.load_byte(1+8), 0x11);
//}
