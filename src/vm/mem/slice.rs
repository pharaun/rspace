use std::slice::SliceIndex;

use crate::vm::Trap;

// Base memory size
const MEM_SIZE: usize = 4096 * 2;

pub trait MemT {
    fn load_byte(&self, idx: u32) -> Result<u32, Trap>;
    fn store_byte(&mut self, idx: u32, data: u32) -> Result<(), Trap>;
}

// Memory Map block Id (ie this block belongs to ram, rom, timer...)
#[derive(Debug, PartialEq, Clone)]
struct MemMapId(u8);


// Memory Map attributes (ie read write, or read only)
#[derive(Debug, PartialEq)]
pub enum MemMapAttr { RW, RO }


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
struct MemMapT {
    // TODO: for now start with a preallocated block
    memory: [u8; MEM_SIZE],

    map: Vec<MemMapBlock>,

    // Holds the id for MemMapId
    next_map_id: u8,
}


impl MemMapT {
    pub fn new() -> MemMapT {
        MemMapT {
            memory: [0; MEM_SIZE],
            map: vec![],
            next_map_id: 0,
        }
    }

    pub fn add(&mut self, start: u32, size: u32, offset: u32, attr: MemMapAttr) -> MemMapId {
        let id = MemMapId(self.next_map_id);
        self.next_map_id += 1;

        self.map.push(
            MemMapBlock {
                // TODO: is clone right here? or can we do it with refs?
                id: id.clone(),
                start: start,
                size: size,
                offset: offset,
                attr: attr,
            }
        );

        id
    }


    // TODO: memory block support?
    // when you do an add you get an id for that block, and then you can do a get
    // that id to get a slice of the block you defined in your add
    pub fn get<I: SliceIndex<[u8]>>(&self, idx: I) -> Option<&I::Output> {
        self.memory.get(idx)
    }

    pub fn get_mut<I: SliceIndex<[u8]>>(&mut self, idx: I) -> Option<&mut I::Output> {
        self.memory.get_mut(idx)
    }
}

impl MemT for MemMapT {
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

    fn store_byte(&mut self, idx: u32, data: u32) -> Result<(), Trap> {
        for mb in self.map.iter() {
            if (idx >= mb.start) && (idx < (mb.start + mb.size)) {
                return match self.memory.get_mut((mb.offset + (idx - mb.start)) as usize) {
                    None    => Err(Trap::IllegalMemoryAccess(idx)),
                    Some(x) => {
                        *x = (data & 0x00_00_00_ff) as u8;
                        Ok(())
                    },
                };
            }
        }

        // TODO: outside bounds of memory map
        // panic!("No memory block at: 0x{:08x}", $idx);
        Err(Trap::IllegalMemoryAccess(idx))
    }
}


#[test]
fn roundtrip_byte() {
    let mut mem_map = MemMapT::new();
    mem_map.add(0x0, 0x1000, 0, MemMapAttr::RW);

    mem_map.store_byte(1, 0x10).unwrap();
    assert_eq!(mem_map.load_byte(1).unwrap(), 0x10);
}

#[test]
fn round_trip_element() {
    let mut mem_map = MemMapT::new();
    mem_map.add(0x0, 0x1000, 0, MemMapAttr::RW);

    mem_map.store_byte(1, 0x10).unwrap();

    assert_eq!(mem_map.get(1), Some(&0x10));

    *(mem_map.get_mut(1).unwrap()) = 0x20 as u8;
    assert_eq!(mem_map.get(1), Some(&0x20));

    match mem_map.get_mut(1) {
        None    => (),
        Some(x) => *x = 0x30 as u8,
    }
    assert_eq!(mem_map.load_byte(1).unwrap(), 0x30);
}

// Dummy impl of a cpu and a timer
fn cpu_step(mem_map: &mut MemMapT) {
    *(mem_map.get_mut(1).unwrap()) = 0x20 as u8;
}

fn timer_step(mem_map: &mut MemMapT) {
    if *(mem_map.get(1).unwrap()) == 0x20 {
        *(mem_map.get_mut(1).unwrap()) = 0x30 as u8;
    }
}

#[test]
fn basic_step() {
    let mut mem_map = MemMapT::new();
    mem_map.add(0x0, 0x1000, 0, MemMapAttr::RW);

    mem_map.store_byte(1, 0x10).unwrap();

    // Do a cpu step
    cpu_step(&mut mem_map);

    assert_eq!(mem_map.load_byte(1).unwrap(), 0x20);

    // Do a timer step
    timer_step(&mut mem_map);

    assert_eq!(mem_map.load_byte(1).unwrap(), 0x30);
}

#[test]
fn round_trip_slice() {
    let mut mem_map = MemMapT::new();
    mem_map.add(0x0, 0x1000, 0, MemMapAttr::RW);

    mem_map.store_byte(1, 0x10).unwrap();
    mem_map.store_byte(2, 0x20).unwrap();

    assert_eq!(mem_map.get(1..3), Some(&[0x10, 0x20][..]));

    {
        let x = mem_map.get_mut(1..3).unwrap();

        x[0] = 0x30 as u8;
        x[1] = 0x40 as u8;
    }

    assert_eq!(mem_map.load_byte(1).unwrap(), 0x30);
    assert_eq!(mem_map.load_byte(2).unwrap(), 0x40);
}
