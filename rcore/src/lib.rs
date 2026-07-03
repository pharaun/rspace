use bevy::prelude::*;

pub mod math;
pub mod movement;
pub mod radar;
pub mod rotation;
pub mod script;
pub mod ship;
pub mod spawner;
pub mod weapon;
pub mod render;

pub use render::RenderPlugin;

use crate::math::AbsRot;

// This is the actual ship-arena
pub const ARENA_SCALE: f32 = 10.0;
const ARENA: IVec2 = IVec2::new(10240, 6400);

// Systemset to help group systems in a defined order of operation since we now have systems that
// depends on previous systems, and this will help avoid the 1+ frame delay when using events
//
// TODO: figure out how to integrate collision, it probs should be post GameLogic but avian is
// in the FixedPostUpdate stage. Need to ensure it happens post interpolation, so collision events
// are going to be 1-frame delayed for now
#[derive(SystemSet, Debug, Hash, Eq, PartialEq, Clone)]
pub enum FixedGameSystem {
    // Motion, Rotation, Radar, Turret, etc...
    GameLogic,

    // The game AI
    ShipLogic,

    // Post game AI systems for handling all of the events spawned from ShipLogic such as spawning
    // new ships, firing weapons, etc..
    Spawn,

    // This is all of the logic that has to do with weapon damage/hits/scan/health
    Weapon,
}

// TODO: add an Arena Marker for ships and stuff for objects we want to have warping
// enabled for, versus objects we don't.
#[derive(Component)]
struct ArenaMarker;
