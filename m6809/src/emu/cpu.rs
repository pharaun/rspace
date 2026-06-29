use crate::emu::regfile::RegFile;

use bytemuck::bytes_of;
use bytemuck::must_cast_slice;
use bytemuck::from_bytes;

struct Cpu {
    // Program Counter
    pc: u16,
}

impl Cpu {
    #[expect(dead_code)]
    fn set_pc(&mut self, pc: u16) {
        self.pc = pc;
    }

    #[expect(dead_code)]
    fn step<M: Memory>(&mut self, _reg: &mut RegFile, _mem: &mut M) {

    }
}

struct Mem([u8; 65536]);

impl Mem {
    #[expect(dead_code)]
    fn new_with(program: &[u8], load_at: u16) -> Self {
        assert!(program.len() + (load_at as usize) < 65536);

        let mut mem = [0; 65536];
        mem[(load_at as usize)..program.len()].copy_from_slice(program);

        Mem(mem)
    }
}

impl Memory for Mem {
    fn get_u8(&self, addr: u16) -> u8 {
        self.0[addr as usize]
    }
    fn set_u8(&mut self, addr: u16, value: u8) {
        self.0[addr as usize] = value;
    }
}

// TODO: support wrapping memory access?
// - Validate this assumption
#[expect(dead_code)]
trait Memory {
    fn get_u8(&self, addr: u16) -> u8;
    fn set_u8(&mut self, addr: u16, value: u8);

    fn get_u16(&self, addr: u16) -> u16 {
        let words = [
            self.get_u8(addr),
            self.get_u8(addr + 1),
        ];
        *from_bytes(&words)
    }
    fn set_u16(&mut self, addr: u16, value: u16) {
        let words = bytes_of(&value);
        self.set_u8(addr, words[0]);
        self.set_u8(addr+1, words[1]);
    }
    fn get_u32(&self, addr: u16) -> u32 {
        let words = [
            self.get_u16(addr),
            self.get_u16(addr + 2),
        ];
        *from_bytes(must_cast_slice(&words))
    }
    fn set_u32(&mut self, addr: u16, value: u32) {
        let dwords = must_cast_slice(bytes_of(&value));
        self.set_u16(addr, dwords[0]);
        self.set_u16(addr+2, dwords[1]);
    }
}
