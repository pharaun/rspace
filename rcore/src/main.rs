use bevy::prelude::*;
use bevy_rapier2d::prelude::*;

mod arena;
use crate::arena::ArenaPlugins;

mod script;
use crate::script::ScriptPlugins;
use crate::script::ScriptEngine;
use crate::script::new_script;

mod ship;
use crate::ship::ShipPlugins;
use crate::ship::StarterShip;
use crate::ship::add_ships;

fn main() {
    App::new()
        .insert_resource(Msaa::default())
        .add_plugins(DefaultPlugins)
        .add_plugins(RapierPhysicsPlugin::<NoUserData>::pixels_per_meter(10.0))
        //.add_plugins(RapierDebugRenderPlugin::default())

        .add_plugins(ArenaPlugins)
        .add_plugins(ScriptPlugins)
        .add_plugins(ShipPlugins)

        .add_systems(Startup, |commands: Commands, script_engine: Res<ScriptEngine>| {
            let script = r#"
            fn on_update(pos, vel, rot) {
                log("pos - " + pos + " vel - " + vel + " rot - " + rot);
            }

            fn on_collision() {
                log("collision");
            }
            "#;

            let ships = vec![
                StarterShip::new(
                    Vec2::new(50., 200.),
                    Vec2::new(-3., 1.),
                    0.0174533,
                    new_script(&script, &script_engine),
                ),
                StarterShip::new(
                    Vec2::new(300., 0.),
                    Vec2::new(-2., -3.),
                    0.0174533 * 2.,
                    new_script(&script, &script_engine),
                ),
                StarterShip::new(
                    Vec2::new(-200., 0.),
                    Vec2::new(1., 0.),
                    0.,
                    new_script(&script, &script_engine),
                ),
                StarterShip::new(
                    Vec2::new(200., 0.),
                    Vec2::new(-1., 0.),
                    0.,
                    new_script(&script, &script_engine),
                ),
            ];

            add_ships(commands, ships);
        })
        .run();
}
