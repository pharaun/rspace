pub mod regfile;
pub mod mem;
pub mod csr;
pub mod opcode;
pub mod cpu;


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
