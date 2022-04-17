extern crate riscv;

use std::io::prelude::*;
use std::io::{self, Read};

fn main() -> io::Result<()> {
    let mut rom: [u8; 4096] = [0; 4096];

    io::stdin().read(&mut rom)?;

    // fuzzed code goes here
    let mut vm = riscv::vm::Emul32::new_with_rom(rom);

    // Can't infinite run, so do a thousand (?) steps then call it quits
    // exit early if Trap fires
    for _ in 0..1024 {
        match vm.step() {
            Ok(_)   => (),
            Err(_)  => break,
        }
    }

    Ok(())
}
