extern crate rspace;

fn main() {
    let mut rom: [u8; 4096] = [0; 4096];
    let mut vm = rspace::vm::Emul32::new_with_rom(rom);
    vm.step();
}
