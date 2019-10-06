use crate::vm::Trap;

// Base memory size
// TODO: make the mem map build up instead of static alloc
const MEM_SIZE: usize = 4096 * 3;

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


// Trait for MIO devices to ask the memory mapper subsystem for the block of
// memory that belongs to each specified MIO device
pub trait MemIO {
    // TODO: shouldn't need the option because we shouldn't have invalid map id
    fn get(&self, block_id: MemMapId) -> Option<&[u8]>;
    fn get_mut(&mut self, block_id: MemMapId) -> Option<&mut [u8]>;
}


// Memory Map attributes (ie read write, or read only)
#[derive(Debug, PartialEq)]
pub enum MemMapAttr { RW, RO }


// Memory Map block Id (ie this block belongs to ram, rom, timer...)
#[derive(Debug, PartialEq, Clone, Copy)]
pub struct MemMapId(u8);


// Memory map block
struct MemMapBlock {
    // Slot Id
    id: MemMapId,
    // Start (inclusive)
    start: u32,
    // Size
    size: u32,
    // self.memory offset
    offset: u32,
    // Attr (RO, RW, etc)
    attr: MemMapAttr,
}


// Memory Mapper itself
pub struct MemMap {
    // TODO: for now start with a preallocated block
    memory: [u8; MEM_SIZE],

    map: Vec<MemMapBlock>,

    // Holds the id for MemMapId
    next_map_id: u8,
}


impl MemMap {
    pub fn new() -> MemMap {
        MemMap {
            memory: [0; MEM_SIZE],
            map: vec![],
            next_map_id: 0,
        }
    }

    // TODO: do more in depth checks to make sure we don't
    // - overlap
    // - run outside bounds
    pub fn add(&mut self, start: u32, size: u32, offset: u32, attr: MemMapAttr) -> MemMapId {
        let id = MemMapId(self.next_map_id);
        self.next_map_id += 1;

        self.map.push(
            MemMapBlock {
                id: id,
                start: start,
                size: size,
                offset: offset,
                attr: attr,
            }
        );

        id
    }
}


impl MemIO for MemMap {
    fn get(&self, block_id: MemMapId) -> Option<&[u8]> {
        for mb in self.map.iter() {
            if mb.id == block_id {
                return self.memory.get((mb.offset as usize)..((mb.offset + mb.size) as usize));
            }
        }

        None
    }

    fn get_mut(&mut self, block_id: MemMapId) -> Option<&mut [u8]> {
        for mb in self.map.iter() {
            if mb.id == block_id {
                return self.memory.get_mut((mb.offset as usize)..((mb.offset + mb.size) as usize));
            }
        }

        None
    }
}


impl Mem for MemMap {
    fn load_byte(&self, idx: u32) -> Result<u32, Trap> {
        for mb in self.map.iter() {
            if (idx >= mb.start) && (idx < (mb.start + mb.size)) {
                return match self.memory.get((mb.offset + (idx - mb.start)) as usize) {
                    None    => Err(Trap::IllegalMemoryAccess(idx)),
                    Some(x) => Ok(*x as u32),
                };
            }
        }

        // TODO: outside bounds of memory map
        // panic!("No memory block at: 0x{:08x}", $idx);
        Err(Trap::IllegalMemoryAccess(idx))
    }

    // TODO: turn this into a macro to handle these
    fn load_half(&self, idx: u32) -> Result<u32, Trap> {
        match (self.load_byte(idx), self.load_byte(idx+1)) {
            (Ok(x), Ok(y))  => Ok(x | (y << 8)),
            (Err(x), _)     => Err(x),
            (_, Err(x))     => Err(x),
        }
    }

    fn load_word(&self, idx: u32) -> Result<u32, Trap> {
        match (self.load_half(idx), self.load_half(idx+2)) {
            (Ok(x), Ok(y))  => Ok(x | (y << 16)),
            (Err(x), _)     => Err(x),
            (_, Err(x))     => Err(x),
        }
    }

    fn store_byte(&mut self, idx: u32, data: u32) -> Result<(), Trap> {
        for mb in self.map.iter() {
            if (idx >= mb.start) && (idx < (mb.start + mb.size)) {
                return match mb.attr {
                    MemMapAttr::RO => Err(Trap::IllegalMemoryAccess(idx)),
                    MemMapAttr::RW => {
                        match self.memory.get_mut((mb.offset + (idx - mb.start)) as usize) {
                            None    => Err(Trap::IllegalMemoryAccess(idx)),
                            Some(x) => {
                                *x = (data & 0x00_00_00_ff) as u8;
                                Ok(())
                            },
                        }
                    },
                };
            }
        }

        // TODO: outside bounds of memory map
        // panic!("No memory block at: 0x{:08x}", $idx);
        Err(Trap::IllegalMemoryAccess(idx))
    }

    fn store_half(&mut self, idx: u32, data: u32) -> Result<(), Trap> {
        match (self.store_byte(idx, data), self.store_byte(idx+1, data >> 8)) {
            (Ok(_), Ok(_))  => Ok(()),
            (Err(x), _)     => Err(x),
            (_, Err(x))     => Err(x),
        }
    }

    fn store_word(&mut self, idx: u32, data: u32) -> Result<(), Trap> {
        match (self.store_half(idx, data), self.store_half(idx+2, data >> 16)) {
            (Ok(_), Ok(_))  => Ok(()),
            (Err(x), _)     => Err(x),
            (_, Err(x))     => Err(x),
        }
    }
}




#[test]
fn roundtrip_byte() {
    let mut mem_map = MemMap::new();
    mem_map.add(0x0, 0x1000, 0, MemMapAttr::RW);

    mem_map.store_byte(1, 0x10).unwrap();
    assert_eq!(mem_map.load_byte(1).unwrap(), 0x10);
}

#[test]
fn roundtrip_half() {
    let mut mem_map = MemMap::new();
    mem_map.add(0x0, 0x1000, 0, MemMapAttr::RW);

    mem_map.store_half(1, 0x2010).unwrap();
    assert_eq!(mem_map.load_half(1).unwrap(), 0x2010);
}

#[test]
fn roundtrip_word() {
    let mut mem_map = MemMap::new();
    mem_map.add(0x0, 0x1000, 0, MemMapAttr::RW);

    mem_map.store_word(1, 0x40302010).unwrap();
    assert_eq!(mem_map.load_word(1).unwrap(), 0x40302010);
}

// Dummy impl of a cpu and a timer
fn cpu_step(mem_map: &mut impl Mem) {
    mem_map.store_byte(1, 0x20).unwrap();
}

fn timer_step(block_id: MemMapId, mem_map: &mut impl MemIO) {
    let ck: u8 = mem_map.get(block_id).unwrap()[1];

    if ck == 0x20 {
        mem_map.get_mut(block_id).unwrap()[1] = 0x30 as u8;
    }
}

#[test]
fn basic_step() {
    let mut mem_map = MemMap::new();
    let id = mem_map.add(0x0, 0x1000, 0, MemMapAttr::RW);

    mem_map.store_byte(1, 0x10).unwrap();

    // Do a cpu step
    cpu_step(&mut mem_map);

    assert_eq!(mem_map.load_byte(1).unwrap(), 0x20);

    // Do a timer step
    timer_step(id, &mut mem_map);

    assert_eq!(mem_map.load_byte(1).unwrap(), 0x30);
}

#[test]
fn byte() {
    let mut ram = MemMap::new();
    ram.add(0x0, 0x1000, 0, MemMapAttr::RW);

    assert_eq!(ram.memory[1], 0x0);

    ram.store_byte(1, 0x10).unwrap();
    assert_eq!(ram.load_byte(1).unwrap(), 0x10);

    assert_eq!(ram.memory[1], 0x10);
}

#[test]
fn half() {
    let mut ram = MemMap::new();
    ram.add(0x0, 0x1000, 0, MemMapAttr::RW);

    assert_eq!(ram.memory[1], 0x0);
    assert_eq!(ram.memory[2], 0x0);

    ram.store_half(1, 0x2010).unwrap();
    assert_eq!(ram.load_half(1).unwrap(), 0x2010);

    assert_eq!(ram.memory[1], 0x10);
    assert_eq!(ram.memory[2], 0x20);
}

#[test]
fn word() {
    let mut ram = MemMap::new();
    ram.add(0x0, 0x1000, 0, MemMapAttr::RW);

    assert_eq!(ram.memory[1], 0x0);
    assert_eq!(ram.memory[2], 0x0);
    assert_eq!(ram.memory[3], 0x0);
    assert_eq!(ram.memory[4], 0x0);

    ram.store_word(1, 0x40302010).unwrap();
    assert_eq!(ram.load_word(1).unwrap(), 0x40302010);

    assert_eq!(ram.memory[1], 0x10);
    assert_eq!(ram.memory[2], 0x20);
    assert_eq!(ram.memory[3], 0x30);
    assert_eq!(ram.memory[4], 0x40);
}

#[test]
fn load_byte() {
    let mut rom = MemMap::new();
    rom.add(0x0, 0x1000, 0, MemMapAttr::RO);

    rom.memory[1] = 0x10;
    rom.memory[2] = 0x20;
    rom.memory[3] = 0x30;
    rom.memory[4] = 0x40;

    assert_eq!(rom.load_byte(1).unwrap(), 0x10);
}

#[test]
fn load_half() {
    let mut rom = MemMap::new();
    rom.add(0x0, 0x1000, 0, MemMapAttr::RO);

    rom.memory[1] = 0x10;
    rom.memory[2] = 0x20;
    rom.memory[3] = 0x30;
    rom.memory[4] = 0x40;

    assert_eq!(rom.load_half(1).unwrap(), 0x2010);
}

#[test]
fn load_word() {
    let mut rom = MemMap::new();
    rom.add(0x0, 0x1000, 0, MemMapAttr::RO);

    rom.memory[1] = 0x10;
    rom.memory[2] = 0x20;
    rom.memory[3] = 0x30;
    rom.memory[4] = 0x40;

    assert_eq!(rom.load_word(1).unwrap(), 0x40302010);
}

#[test]
#[should_panic]
fn store_byte() {
    let mut rom = MemMap::new();
    rom.add(0x0, 0x1000, 0, MemMapAttr::RO);

    rom.store_byte(1, 0x10).unwrap();
}

#[test]
#[should_panic]
fn store_half() {
    let mut rom = MemMap::new();
    rom.add(0x0, 0x1000, 0, MemMapAttr::RO);

    rom.store_half(1, 0x2010).unwrap();
}

#[test]
#[should_panic]
fn store_word() {
    let mut rom = MemMap::new();
    rom.add(0x0, 0x1000, 0, MemMapAttr::RO);

    rom.store_word(1, 0x40302010).unwrap();
}