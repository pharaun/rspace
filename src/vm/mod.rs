pub mod regfile;
pub mod mem;
pub mod csr;
pub mod opcode;
pub mod cpu;


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

pub struct Emul32 {
    mem: mem::Memory,
    csr: csr::Csr,
    cpu: cpu::Cpu,
}

impl Emul32 {
    pub fn new_with_rom(rom: [u8; 4096]) -> Emul32 {
        Emul32 {
            mem: mem::Memory::new(rom, [0; 4096]),
            csr: csr::Csr::new([0; 4096]),
            cpu: cpu::Cpu::new(regfile::RegFile::new([0; 31]), 0),
        }
    }

    pub fn new(reg: regfile::RegFile, mem: mem::Memory, csr: csr::Csr, pc: usize) -> Emul32 {
        Emul32 {
            mem: mem,
            csr: csr,
            cpu: cpu::Cpu::new(reg, pc),
        }
    }

    pub fn run(&mut self) {
        self.cpu.run(
            &mut self.mem,
            &mut self.csr,
        );
    }
}


// Tests getting too long, include instead
include!("test.rs");
