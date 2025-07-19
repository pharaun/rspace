use bevy::prelude::*;
use bevy_rapier2d::prelude::*;

use bevy_screen_diagnostics::ScreenDiagnosticsPlugin;
use bevy_screen_diagnostics::ScreenEntityDiagnosticsPlugin;
use bevy_screen_diagnostics::ScreenFrameDiagnosticsPlugin;

use std::collections::HashMap;
use rust_dynamic::value::Value;

use rcore::arena::ArenaPlugins;
use rcore::script::Script;
use rcore::script::ScriptPlugins;
use rcore::ship::ShipPlugins;
use rcore::ship::add_ships;
use rcore::ship::ShipBuilder;
use rcore::math::RelRot;
use rcore::math::AbsRot;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(RapierPhysicsPlugin::<NoUserData>::pixels_per_meter(10.0))
        //.add_plugins(RapierDebugRenderPlugin::default())

        // FPS
        .add_plugins(FpsPlugins)

        // Game bits
        .add_plugins(ArenaPlugins)
        .add_plugins(ScriptPlugins)
        .add_plugins(ShipPlugins)

        // TODO: a way to init a new ship with some preset value to help script do custom per ship
        // things limit_r, target_r,
        .add_systems(Startup, |commands: Commands| {
            let ships = vec![
                ShipBuilder::new(Script::new(on_init, on_update, on_collision))
                    .position(Vec2::new(0., 0.))
                    .velocity(Vec2::new(0., 0.))
                    .velocity_limit(10.)
                    .rotation_limit(16)
                    .rotation(0.)
                    .radar_limit(5.)
                    .radar_arc(180.)
                    .build(),

                // Test cases
                //  * flip flops on direction
                //  * Weird drifting on rotation
//                ShipBuilder::new(Script::new(&ship_script(f32::to_radians(180.), 0.), &script_engine))
//                    .position(Vec2::new(-350., 0.))
//                    .velocity(Vec2::new(0., 0.))
//                    .velocity_limit(10.)
//                    .rotation_limit(32)
//                    .rotation(0.)
//                    .radar_limit(5.)
//                    .radar_arc(180.)
//                    .build(),
//
//                ShipBuilder::new(Script::new(&ship_script(f32::to_radians(-90.), 0.), &script_engine))
//                    .position(Vec2::new(350., 0.))
//                    .velocity(Vec2::new(0., 0.))
//                    .velocity_limit(10.)
//                    .rotation_limit(192)
//                    .rotation(0.)
//                    .radar_limit(5.)
//                    .radar_arc(180.)
//                    .build(),
            ];

            add_ships(commands, ships);
        })
        .run();
}


// Function for the ship
fn on_init() -> HashMap<&'static str, Value> {
    HashMap::from([
        // Const
        ("acceleration", Value::from(1.).unwrap()),
        ("add_rot", Value::from(180.).unwrap()),
    ])
}

// Minimal go back and forth ship script
// TODO: figure out the X axis drift, there's a slow sideway drift due to rotation
// - going upward it snaps between 180 and 0
// - going downward it slowly changes between 0 to 180 and never quite snaps to 180
// - figure out why
fn on_update(_state: &mut HashMap<&'static str, Value>, pos: Vec2, vel: Vec2, rot: AbsRot) -> (RelRot, f32) {
    println!("on_update: Pos - {:?} - Vel - {:?} - {:?} - Rot - {:?}", pos, vel, vel.length(), rot);

    if rot == AbsRot(0) || rot == AbsRot(128) {
        if vel.y < 10. && rot == AbsRot(0){
            println!("Accelerate");
            (RelRot(0), 1.)
        } else if vel.y > -10. && rot == AbsRot(128) {
            println!("Decelerate");
            (RelRot(0), 1.)
        } else {
            println!("Rotate");
            (RelRot(-128), 0.)
        }
    } else {
        println!("Idle");
        (RelRot(0), 0.)
    }
}

fn on_collision(_state: &mut HashMap<&'static str, Value>) {
    println!("on_collision");
}

pub struct FpsPlugins;
impl Plugin for FpsPlugins {
    fn build(&self, app: &mut App) {
        // we want Bevy to measure these values for us:
        app
            .add_plugins(ScreenDiagnosticsPlugin::default())
            .add_plugins(ScreenFrameDiagnosticsPlugin)
            .add_plugins(ScreenEntityDiagnosticsPlugin);
    }
}
