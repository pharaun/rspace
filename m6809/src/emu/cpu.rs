use crate::emu::regfile::RegFile;

struct Cpu {
    reg: RegFile,
    mem: [u8; 65536],
}

impl Cpu {
    fn new_with(program: &[u8], load_at: u16) -> Self {
        assert!(program.len() + (load_at as usize) < 65536);

        let mut mem = [0; 65536];
        mem[(load_at as usize)..program.len()].copy_from_slice(program);

        Cpu {
            reg: RegFile::default(),
            mem: mem,
        }
    }

    fn set_pc(&mut self, pc: u16) {
        self.reg.pc = pc;
    }

    fn step(&mut self) {

    }
}
