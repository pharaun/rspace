use avian2d::prelude::*;
use bevy::prelude::*;

pub mod attach;
pub mod math;
pub mod movement;
pub mod radar;
pub mod rotation;
pub mod script;
pub mod ship;
pub mod spawner;
pub mod weapon;
pub mod time;

#[cfg(feature = "render")]
pub mod render;

use crate::math::AbsRot;

use crate::attach::AttachPlugin;
use crate::movement::MovementPlugin;
use crate::radar::RadarPlugin;
use crate::rotation::RotationPlugin;
use crate::script::ScriptPlugins;
use crate::spawner::SpawnerPlugin;
use crate::time::TimeControlPlugin;
use crate::weapon::WeaponPlugin;

// Sim timing
pub const TICK_HZ: u32 = 64;

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

// We break up the game into 3 main pieces:
// 1. Core simulation engine
// 2. Render
// 3. Camera
pub struct SimulationPlugin;
impl Plugin for SimulationPlugin {
    fn build(&self, app: &mut App) {
        app
            .insert_resource(Time::<Fixed>::from_hz(f64::from(TICK_HZ)))
            // Physics
            .add_plugins(PhysicsPlugins::default())
            .insert_resource(Gravity(Vec2::ZERO))
            // Simulation Control
            .add_plugins(TimeControlPlugin)
            // Game bits
            .add_plugins(AttachPlugin)
            .add_plugins(MovementPlugin)
            .add_plugins(RadarPlugin)
            .add_plugins(RotationPlugin)
            .add_plugins(ScriptPlugins)
            .add_plugins(SpawnerPlugin)
            .add_plugins(WeaponPlugin)
            // System set ordering
            .configure_sets(
                FixedUpdate,
                (
                    FixedGameSystem::GameLogic,
                    FixedGameSystem::ShipLogic,
                    FixedGameSystem::Spawn,
                    FixedGameSystem::Weapon,
                )
                    .chain(),
            );
    }
}
