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
                    Vec2::new(0., 0.),
                    Vec2::new(0., 0.),
                    f32::to_radians(90.0),
                    f32::to_radians(0.0),
                    1.0,
                    Script::new(&ship_script(f32::to_radians(180.), 1.), &script_engine),
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
            // Const
            this.add_vel = {:.8};
            this.add_rot = {:.8};

            // State
            this.state = 0;

            // Counter
            this.counter = 0;
        }}

        {}"#,
        target_vel, target_rot,
        r#"
        // TODO: need better vel indicator (neg and pos and lateral state)
        fn on_update(pos, vel, rot) {
            log("Vel - " + vel + " - state - " + this.state + " - counter - " + this.counter);

            switch this.state {
                // 0 = speed up
                0 => {
                    if vel > 10 {
                        this.state = 1;
                    }

                    [0.0, this.add_vel]
                },

                // 1 = flip
                1 => {
                    this.state = 2;
                    [this.add_rot, 0.0]
                },

                // 2 = countdown
                2 => {
                    if this.counter > 3 {
                        this.counter = 0;
                        this.state = 3;
                    }

                    this.counter += 1;
                    [0.0, 0.0]
                },

                // 3 = slow down
                3 => {
                    if vel < 1 {
                        this.state = 4;
                    }

                    [0.0, this.add_vel]
                },

                // 4 = flip
                4 => {
                    this.state = 5;
                    [this.add_rot, 0.0]
                },

                // 5 = countdown
                5 => {
                    if this.counter > 3 {
                        this.counter = 0;
                        this.state = 0;
                    }

                    this.counter += 1;
                    [0.0, 0.0]
                },
            }
        }

        fn on_collision() {
            log("collision");
        }
        "#,
    )
}
