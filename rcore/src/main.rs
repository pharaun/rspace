use bevy::prelude::*;
use bevy_rapier2d::prelude::*;

mod arena;
use crate::arena::ArenaPlugins;

mod script;
use crate::script::Script;
use crate::script::ScriptPlugins;
use crate::script::ScriptEngine;

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
        // things limit_r, target_r,
        .add_systems(Startup, |commands: Commands, script_engine: Res<ScriptEngine>| {
            let ships = vec![
                StarterShip::new(
                    Vec2::new(150., 0.),
                    Vec2::new(0., 0.),
                    f32::to_radians(90.0),
                    f32::to_radians(0.0),
                    1.0,
                    Script::new(&ship_script(180., 1.), &script_engine),
                ),
                StarterShip::new(
                    Vec2::new(-150., 0.),
                    Vec2::new(0., 0.),
                    f32::to_radians(90.0),
                    f32::to_radians(0.0),
                    1.0,
                    Script::new(&ship_script(180., 1.), &script_engine),
                ),


//                // Test cases
//                //  * flip flops on direction
//                //  * Weird drifting on rotation
//                StarterShip::new(
//                    Vec2::new(-350., 0.),
//                    Vec2::new(0., 0.),
//                    f32::to_radians(45.0),
//                    f32::to_radians(0.0),
//                    0.0,
//                    Script::new(&ship_script(f32::to_radians(180.), 0.), &script_engine),
//                ),
//                StarterShip::new(
//                    Vec2::new(350., 0.),
//                    Vec2::new(0., 0.),
//                    f32::to_radians(179.0),
//                    f32::to_radians(0.0),
//                    0.0,
//                    Script::new(&ship_script(f32::to_radians(-90.), 0.), &script_engine),
//                ),
            ];

            add_ships(commands, ships);
        })
        .run();
}

// TODO: the scripting really needs to be better, this is hampering us
fn ship_script(target_rot: f32, target_vel: f32) -> String {
    format!(r#"
        fn init() {{
            if "slowdown" !in this {{
                this.slowdown = false;
            }}
        }}
        fn on_update(pos, vel, rot) {{
            let add_vel = {};
            let add_rot = {};

            if vel > 5 && !this.slowdown{{
                this.slowdown = true;
            }}

            if vel < 1  && this.slowdown {{
                this.slowdown = false;
            }}

            log("Vel - " + vel + " - slowdown - " + this.slowdown);

            if this.slowdown {{
                [rot, vel - add_vel]
            }} else {{
                [rot, vel + add_vel]
            }}
        }}

        fn on_collision() {{
            log("collision");
        }}
        "#,
        target_vel, target_rot
    )
}
