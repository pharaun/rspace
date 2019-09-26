mod regfile;
mod mem;
mod mio;
mod csr;
mod cpu;
pub mod opcode;

use std::cell::RefCell;
use std::rc::Rc;

// TODO: design ideas/how
//
// We need to have a way for various component to have these features:
// - memory read
// - memory write
// - csr read
// - csr write
//
// By maybe making mem read and write traits that should let us have a nicer
// way to maybe do memory map ie
//
// 0x0000 -> 0x00FF = Rom (memory read trait)
// 0x00FF -> 0x0F00 = Ram (Memory read/write trait)
// 0x0F00 -> 0x1000 = Null (No read/write trait?)
// 0x1000 -> 0x2000 = Memory Mapped I/O (Memory read/write trait)
//
// Then later for CSR that ie affect the component for eg the timer
//
// TimerInterrupt:
// - CSR read
// - CSR write
// - Mem Read
// - Mem Write
//
// For ie the timer interrupt that has memory i/o for configuration + reading data and
// csr for triggering the timer interrupt (?)
//
// Cpu:
// - CSR read
// - CSR write
//
// For bits (such as counters) that would be readable/usable from the cpu for eg (simple cycle
// counters)

// Support for Traps and Interrupts
// TODO: improve this, but for now this will do
#[derive(Debug, PartialEq)]
pub enum Trap {
    IllegalInstruction(u32),
    IllegalMemoryAccess(u32),
    UnalignedInstructionAccess(u32),
    InterruptTimer,
}


// TODO: consider renaming to system/machine?
pub struct Emul32 {
    mem: mem::MemMap,
    csr: csr::Csr,

    // Tickable items
    cpu: cpu::Cpu,

    // TODO: disgusting, i'm not sure how to not do this, need to redo the memory_map i think?
    timer: Rc<RefCell<mio::timer::Timer>>,
}

impl Emul32 {
    pub fn new_with_rom(rom: [u8; 4096]) -> Emul32 {
        // TODO: redo the memory map, can we store the information somehow in Emul32
        // or make reconstructing this easy/cheap/fast?
        let mut mem_map = mem::MemMap::new();
        mem_map.add(0x0,    0x1000, mem::rom::Rom::new(rom));
        mem_map.add(0x1000, 0x2000, mem::ram::Ram::new());

        // MIO region
        // TODO: may need to make mem_map take some other data structure so we can keep ownership
        //       of the mio bits for ticking em (and handing this one a csr interrupt thing so it
        //       can trigger an timer interrupt)
        let timer = Rc::new(RefCell::new(mio::timer::Timer::new(0, 0, 0x2000)));
        mem_map.add(0x2000, 0x2010, Rc::clone(&timer));

        // TODO: implement a csr_map construct (to handle similiar things to mem_map but for csr)
        // CSR & MIO is the main 2 way for an external system to interact with the cpu, maaybe
        // interrupts (but that's going to be our own PIC which probs will use CSR & MIO for
        // working with external interrupts)

        Emul32 {
            mem: mem_map,
            csr: csr::Csr::new([0; 4096]),
            cpu: cpu::Cpu::new(regfile::RegFile::new([0; 31]), 0),
            timer: timer,
        }
    }

    pub fn new(
        reg: regfile::RegFile,
        mem: mem::MemMap,
        csr: csr::Csr,
        pc: u32,
        timer: mio::timer::Timer,
    ) -> Emul32 {
        Emul32 {
            mem: mem,
            csr: csr,
            cpu: cpu::Cpu::new(reg, pc),
            timer: Rc::new(RefCell::new(timer)),
        }
    }

    pub fn run(&mut self) {
        loop {
            match self.step() {
                Ok(_)  => (),
                Err(_) => break,
            }
        }
    }

    // TODO: for now just return an option
    pub fn step(&mut self) -> Result<(), Trap> {
        // TODO: figure out how to test that the trap fired
        self.timer.borrow_mut().step();

        // TODO: should be returning a list i think
        self.cpu.step(
            &mut self.mem,
            &mut self.csr,
        )
    }
}


// Tests getting too long, include instead
include!("test.rs");
