use bevy::prelude::*;
use bevy_rapier2d::prelude::*;

use iyes_perf_ui::PerfUiPlugin;
use iyes_perf_ui::ui::root::PerfUiRoot;
use iyes_perf_ui::entries::diagnostics::PerfUiEntryFPSWorst;
use iyes_perf_ui::entries::diagnostics::PerfUiEntryFPS;

use std::collections::HashMap;
use rust_dynamic::value::Value;

use rcore::arena::ArenaPlugins;
use rcore::script::Script;
use rcore::script::ScriptPlugins;
use rcore::ship::ShipPlugins;
use rcore::ship::add_ships;
use rcore::ship::ShipBuilder;

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

fn approx_equal(a: f32, b: f32) -> bool {
    (a - b).abs() < 0.01
}

// Minimal go back and forth ship script
// TODO: figure out the X axis drift, there's a slow sideway drift due to rotation
// - going upward it snaps between 180 and 0
// - going downward it slowly changes between 0 to 180 and never quite snaps to 180
// - figure out why
fn on_update(_state: &mut HashMap<&'static str, Value>, pos: Vec2, vel: Vec2, rot: f32) -> (f32, f32) {
    println!("on_update: Pos - {:?} - Vel - {:?} - {:?} - Rot - {:?}", pos, vel, vel.length(), rot);

    if vel.length() < 10. && (approx_equal(rot, 0.) || approx_equal(rot, -3.1415925)) {
        (0., 1.)
    } else if vel.length() > 10. && (approx_equal(rot, 0.) || approx_equal(rot, -3.1415925)) {
        (f32::to_radians(180.), 0.)
    } else {
        (0., 0.)
    }
}

fn on_collision(_state: &mut HashMap<&'static str, Value>) {
    println!("on_collision");
}

pub struct FpsPlugins;
impl Plugin for FpsPlugins {
    fn build(&self, app: &mut App) {
        // we want Bevy to measure these values for us:
        app.add_plugins(bevy::diagnostic::FrameTimeDiagnosticsPlugin::default())
            .add_plugins(PerfUiPlugin)
            .add_systems(Startup, |mut commands: Commands| {
                commands.spawn((
                    PerfUiRoot {
                        display_labels: false,
                        layout_horizontal: true,
                        ..default()
                    },
                    PerfUiEntryFPS::default(),
                    PerfUiEntryFPSWorst::default(),
                ));
            });
    }
}
