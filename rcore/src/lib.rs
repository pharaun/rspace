use bevy::prelude::SystemSet;

pub mod arena;
pub mod class;
pub mod collision;
pub mod debug_weapon;
pub mod health;
pub mod math;
pub mod movement;
pub mod radar;
pub mod rotation;
pub mod script;
pub mod ship;
pub mod spawner;

// Systemset to help group systems in a defined order of operation since we now have systems that
// depends on previous systems, and this will help avoid the 1+ frame delay when using events
#[derive(SystemSet, Debug, Hash, Eq, PartialEq,  Clone)]
pub enum FixedGameSystem {
    // Motion, Rotation, Radar, Turret, etc...
    Motion,

    // Physics -> https://docs.rs/bevy_rapier2d/latest/bevy_rapier2d/plugin/enum.PhysicsSet.html
    // TODO: replace rapier with a custom collision system so it can reuse the Position/motion code
    Collision,

    // The game AI
    ShipLogic,

    // Post game AI systems for handling all of the events spawned from ShipLogic such as spawning
    // new ships, firing weapons, etc..
    Spawn,
    Weapon,
}
