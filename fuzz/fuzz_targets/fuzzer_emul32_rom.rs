#![no_main]
#[macro_use]
extern crate libfuzzer_sys;
extern crate rspace;

fuzz_target!(|data: &[u8]| {
    let rom = {
        let mut rom: [u8; 4096] = [0; 4096];
        for i in 0..data.len() {
            rom[i] = data[i];
        }
        rom
    };

    // fuzzed code goes here
    let mut vm = rspace::vm::Emul32::new_with_rom(rom);

    // Can't infinite run, so do a hundred (?) steps then call it quits
    for _ in 0..100 {
        vm.step();
    }
});
