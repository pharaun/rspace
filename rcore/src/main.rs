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

        // TODO: a way to init a new ship with some preset value to help script do custom per ship
        // things
        .add_systems(Startup, |commands: Commands, script_engine: Res<ScriptEngine>| {
            let ships = vec![
                StarterShip::new(
                    Vec2::new(50., 200.),
                    Vec2::new(-3., 1.),
                    f32::to_radians(1.0),
                    f32::to_radians(0.0),
                    new_script(&ship_script(1.), &script_engine),
                ),
                StarterShip::new(
                    Vec2::new(300., 0.),
                    Vec2::new(-2., -3.),
                    f32::to_radians(2.0),
                    f32::to_radians(45.0),
                    new_script(&ship_script(1.), &script_engine),
                ),
                StarterShip::new(
                    Vec2::new(-200., 0.),
                    Vec2::new(1., 0.),
                    f32::to_radians(0.5),
                    f32::to_radians(90.0),
                    new_script(&ship_script(1.), &script_engine),
                ),
                StarterShip::new(
                    Vec2::new(200., 0.),
                    Vec2::new(-1., 0.),
                    f32::to_radians(0.25),
                    f32::to_radians(180.0),
                    new_script(&ship_script(1.), &script_engine),
                ),
            ];

            add_ships(commands, ships);
        })
        .run();
}

fn ship_script(target_rotation: f32) -> String {
    format!(r#"
        fn on_update(pos, vel, rot) {{
            log("rot - " + rot);

            rot + {}
        }}

        fn on_collision() {{
            log("collision");
        }}
        "#,
        target_rotation,
    )
}
