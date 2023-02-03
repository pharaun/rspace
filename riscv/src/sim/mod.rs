use crate::vm::Emul32;


// Handle the core game state logic
#[allow(dead_code)]
pub struct Game {
    // Ships
    ships: Vec<Ship>,
}

#[allow(dead_code)]
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
