use bevy::prelude::*;
use bevy_rapier2d::prelude::*;

use iyes_perf_ui::PerfUiPlugin;
use iyes_perf_ui::ui::root::PerfUiRoot;
use iyes_perf_ui::entries::diagnostics::PerfUiEntryFPSWorst;
use iyes_perf_ui::entries::diagnostics::PerfUiEntryFPS;

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
                ShipBuilder::new(Script::new(on_update, on_collision))
                    .position(Vec2::new(0., 0.))
                    .velocity(Vec2::new(0., 0.))
                    .velocity_limit(10.)
                    .rotation_limit(90.)
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
//                    .rotation_limit(45.)
//                    .rotation(0.)
//                    .radar_limit(5.)
//                    .radar_arc(180.)
//                    .build(),
//
//                ShipBuilder::new(Script::new(&ship_script(f32::to_radians(-90.), 0.), &script_engine))
//                    .position(Vec2::new(350., 0.))
//                    .velocity(Vec2::new(0., 0.))
//                    .velocity_limit(10.)
//                    .rotation_limit(179.)
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
fn on_update(pos: Vec2, vel: Vec2, rot: f32) -> (f32, f32) {
    println!("on_update");
    (0., 0.)
}

fn on_collision() {
    println!("on_collision");
}


// TODO: the scripting really needs to be better, this is hampering us
fn ship_script(target_rot: f32, acceleration: f32) -> String {
    format!(r#"
        fn init() {{
            // Const
            this.acceleration = {:.8};
            this.add_rot = {:.8};

            // State
            this.state = 0;
            this.next_state = [];

            // Counter
            this.counter = 0;
        }}

        {}"#,
        acceleration, target_rot,
        r#"
        // TODO: need better vel indicator (neg and pos and lateral state)
        fn on_update(pos, vel, rot) {
            log("Pos - " + pos + " - Vel - " + vel + " - " + vel.length());

            // If next_state is empty, it has ran out, so restock it with sequences
            if this.next_state.is_empty() {
                this.state = 0;
                this.next_state += [1, 1, 2, 1, 1, 3, 1, 1, 2, 1, 1, 0]
            }

            switch this.state {
                // 0 = speed up
                0 => {
                    if vel.length() > 10 {
                        this.state = this.next_state.shift();
                    }

                    log("State - Speedup");
                    [0.0, this.acceleration]
                },

                // 1 = pause
                1 => {
                    if this.counter > 3 {
                        this.counter = 0;
                        this.state = this.next_state.shift();
                    }

                    this.counter += 1;

                    log("State - Pause");
                    [0.0, 0.0]
                }

                // 2 = flip
                2 => {
                    this.state = this.next_state.shift();

                    log("State - Flip");
                    [this.add_rot, 0.0]
                },

                // 3 = slow down
                3 => {
                    if vel.length() < 2 {
                        this.state = this.next_state.shift();
                    }

                    log("State - Slowdown");
                    [0.0, this.acceleration]
                },
            }
        }

        fn on_collision() {
            log("collision");
        }
        "#,
    )
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
