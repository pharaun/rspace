extern crate riscv;

use std::fs::File;
use std::io::prelude::*;

fn main() {
    let mut rom: [u8; 4096] = [0; 4096];
    let mut file = File::open("/tmp/test").unwrap();
    file.read(&mut rom).unwrap();

    let mut vm = riscv::vm::Emul32::new_with_rom(rom);

    // Virtal machine run
    loop {
        vm.set_pc(0);
        vm.run();
    }
}
