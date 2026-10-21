use avian2d::physics_transform::PhysicsTransformConfig;
use avian2d::prelude::*;
use bevy::prelude::*;

pub mod assets;
pub mod attach;
pub mod environ;
pub mod math;
pub mod movement;
pub mod radar;
pub mod rotation;
pub mod script;
pub mod ship;
pub mod spawner;
pub mod time;
pub mod weapon;

#[cfg(feature = "render")]
pub mod render;

use crate::math::AbsRot;

use crate::assets::AssetGatePlugin;
use crate::assets::AssetsPlugin;
use crate::attach::AttachPlugin;
use crate::environ::EnvironPlugin;
use crate::movement::MovementPlugin;
use crate::radar::RadarPlugin;
use crate::rotation::RotationPlugin;
use crate::script::ScriptPlugins;
use crate::spawner::SpawnerPlugin;
use crate::time::TimeControlPlugin;
use crate::weapon::WeaponPlugin;

use crate::assets::field::FieldProfile;
use crate::assets::field::FieldSource;
pub use crate::assets::map::ActiveMap;

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
    // Environmental sampler -> GameLogic uses it (ie c-boost, flux, etc...) This does mean that
    // any logic that depends on the Environ for its values will be a "tick stale" (ie the c_boost
    // cap will be for the current position not the newly computed next position.)
    Sampler,

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

// State to help load up assets and manage the asset2sim & asset2render lifecycle.
#[derive(States, Default, Clone, Eq, PartialEq, Hash, Debug)]
pub enum GamePhase {
    #[default]
    Load,
    Run,
}

// Make sure that Avian does not take a Transform from bevy. The sim strictly uses Position(f64)
// where the render uses Transform(f32).
const SIM_TRANSFORM_CONFIG: PhysicsTransformConfig = PhysicsTransformConfig {
    propagate_before_physics: false,
    transform_to_position: false,
    position_to_transform: true,
    transform_to_collider_scale: false,
};

pub struct SimulationPlugin;
impl Plugin for SimulationPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(Time::<Fixed>::from_hz(f64::from(TICK_HZ)))
            // Physics
            .insert_resource(SIM_TRANSFORM_CONFIG)
            .add_plugins(PhysicsPlugins::default())
            .insert_resource(Gravity(Vec2::ZERO))
            .configure_sets(
                FixedPostUpdate,
                PhysicsSystems::StepSimulation.run_if(in_state(GamePhase::Run)),
            )
            // State management
            .init_state::<GamePhase>()
            // Simulation Asset Gates
            .add_plugins(AssetGatePlugin)
            // Simulation Support
            .add_plugins(AssetsPlugin)
            .add_plugins(AttachPlugin)
            .add_plugins(TimeControlPlugin)
            // Game Bits
            .add_plugins(EnvironPlugin)
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
                    FixedGameSystem::Sampler,
                    FixedGameSystem::GameLogic,
                    FixedGameSystem::ShipLogic,
                    FixedGameSystem::Spawn,
                    FixedGameSystem::Weapon,
                )
                    .chain()
                    .run_if(in_state(GamePhase::Run)),
            );
    }
}

#[cfg(test)]
mod avian_config {
    use super::*;

    use bevy::math::DVec2;
    use bevy::time::TimeUpdateStrategy;
    use std::time::Duration;

    fn step_position(pos: DVec2, config: PhysicsTransformConfig) -> DVec2 {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, TransformPlugin))
            .insert_resource(TimeUpdateStrategy::ManualDuration(Duration::from_secs_f64(
                1.0 / f64::from(TICK_HZ),
            )))
            .insert_resource(Time::<Fixed>::from_hz(f64::from(TICK_HZ)))
            .insert_resource(config)
            .add_plugins(PhysicsPlugins::default())
            .insert_resource(Gravity(Vec2::ZERO));
        app.finish();
        app.cleanup();

        let ship = app
            .world_mut()
            .spawn((
                RigidBody::Kinematic,
                Position(pos),
            ))
            .id();

        for _ in 0..8 {
            app.update();
        }
        app.world().get::<Position>(ship).expect("position").0
    }

    #[test]
    fn test_position_survives_transform_sync() {
        let pos = DVec2::new(1_000_000.3, -0.7);
        assert_eq!(step_position(pos, SIM_TRANSFORM_CONFIG), pos);
    }
}
