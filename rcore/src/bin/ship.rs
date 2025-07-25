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
                ShipBuilder::new(Script::new(on_init, on_update, on_contact, on_collision))
                    .position(0, 0)
                    .velocity(0, 0)
                    .velocity_limit(100)
                    .rotation(AbsRot(0))
                    .rotation_limit(16)
                    .radar(AbsRot(0))
                    .radar_arc(64)
                    .debug(true)
                    .build(),

                ShipBuilder::new(Script::new(
                        || HashMap::from([]),
                        |_, _, _, _| (RelRot(0), 0, RelRot(0)),
                        |_, _| (),
                        |_| (),
                    ))
                    .position(3500, 0)
                    .velocity(0, 0)
                    .build(),

                ShipBuilder::new(Script::new(
                        || HashMap::from([]),
                        |_, _, _, _| (RelRot(0), 0, RelRot(0)),
                        |_, _| (),
                        |_| (),
                    ))
                    .position(-3500, 0)
                    .velocity(0, 0)
                    .build(),

                ShipBuilder::new(Script::new(
                        || HashMap::from([]),
                        |_, _, _, _| (RelRot(0), 0, RelRot(0)),
                        |_, _| (),
                        |_| (),
                    ))
                    .position(-4500, 3500)
                    .velocity(0, 0)
                    .build(),
            ];

            add_ships(commands, ships);
        })
        .run();
}


// Function for the ship
fn on_init() -> HashMap<&'static str, Value> {
    HashMap::from([
        // Const
        ("acc", Value::from(10).unwrap()),
        ("dec", Value::from(20).unwrap()),
        ("rot", Value::from(-128).unwrap()),
        // Variables
        ("collision", Value::from(false).unwrap()),
        ("target.x", Value::from(0).unwrap()),
        ("target.y", Value::from(0).unwrap()),
    ])
}

// Minimal go back and forth ship script
// TODO: figure out the X axis drift, there's a slow sideway drift due to rotation
// - going upward it snaps between 180 and 0
// - going downward it slowly changes between 0 to 180 and never quite snaps to 180
// - figure out why
fn on_update(state: &mut HashMap<&'static str, Value>, pos: IVec2, vel: IVec2, rot: AbsRot) -> (RelRot, i32, RelRot) {
    println!("on_update: Pos - {:?} - Vel - {:?} - Rot - {:?}", pos, vel, rot);

    let acc: i32 = state.get("acc").unwrap().cast_int().unwrap() as i32;
    let dec: i32 = state.get("dec").unwrap().cast_int().unwrap() as i32;
    let arot: i8 = state.get("rot").unwrap().cast_int().unwrap() as i8;

    if rot == AbsRot(0) || rot == AbsRot(128) {
        if vel.y < 95 && rot == AbsRot(0){
            println!("Accelerate");
            (RelRot(0), acc, RelRot(0))
        } else if vel.y > -95 && rot == AbsRot(128) {
            println!("Decelerate");
            (RelRot(0), dec, RelRot(0))
        } else {
            println!("Rotate & Radar");
            (RelRot(arot), 0, RelRot(64))
        }
    } else {
        println!("Idle");
        (RelRot(0), 0, RelRot(0))
    }
}

fn on_contact(state: &mut HashMap<&'static str, Value>, target_pos: IVec2) {
    state.insert("target.x", Value::from(target_pos.x).unwrap());
    state.insert("target.y", Value::from(target_pos.y).unwrap());
    println!("on_contact - target.x: {:?}, target.y: {:?}", target_pos.x, target_pos.y);
}

fn on_collision(state: &mut HashMap<&'static str, Value>) {
    state.insert("collision", Value::from(true).unwrap());
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
