use crate::vm::Emul32;


// Handle the core game state logic
pub struct Game {
    // Ships
    ships: Vec<Ship>,
}


pub struct Ship {
    team: u8,

    // Map positioning and facing
    x: u32,
    y: u32,
    facing: u8,
    velocity: u8,

    // Virtual machine
    vm: Emul32,
}
