use avian2d::prelude::*;
use bevy::prelude::*;

pub mod attach;
pub mod field;
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

use crate::attach::AttachPlugin;
use crate::field::FieldPlugin;
use crate::movement::MovementPlugin;
use crate::radar::RadarPlugin;
use crate::rotation::RotationPlugin;
use crate::script::ScriptPlugins;
use crate::spawner::SpawnerPlugin;
use crate::time::TimeControlPlugin;
use crate::weapon::WeaponPlugin;

use crate::field::FieldProfile;
use crate::field::FieldSource;

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

// We break up the game into 3 main pieces:
// 1. Core simulation engine
// 2. Render
// 3. Camera
pub struct SimulationPlugin;
impl Plugin for SimulationPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(Time::<Fixed>::from_hz(f64::from(TICK_HZ)))
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
            .add_plugins(FieldPlugin)
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
                    .chain(),
            );
    }
}

// Star
#[derive(Component)]
struct StarMarker;

pub fn star_setup(
    mut profiles: ResMut<Assets<FieldProfile>>,
    //asset_server: Res<AssetServer>,
    mut commands: Commands,
) {
    // TODO: Workaround for now
    // This works - cuz we register the component (above) and then we forcefully load the assets
    // So this works to prove the hook logic works, but the hook logic probs should be done in a
    // better way
    let raw = std::fs::read_to_string("assets/profiles/g_class.ron").expect("g_class.ron missing");
    let profile: FieldProfile = ron::from_str(&raw).expect("g_class.ron malformed");

    commands.spawn((
        StarMarker,
        Transform::from_xyz(0.0, -1000.0, 0.0),
        //FieldSource(asset_server.load("profiles/g_class.ron")),
        FieldSource(profiles.add(profile)),
        // This breaks the render separation
        Sprite::from_color(Color::srgb(0.0, 1.0, 1.0), Vec2::splat(200.0)),
    ));
}
