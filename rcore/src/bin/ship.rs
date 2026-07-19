use bevy::ecs::schedule::LogLevel;
use bevy::ecs::schedule::ScheduleBuildSettings;
use bevy::prelude::*;

use rcore::SimulationPlugin;
use rcore::math::AbsRot;
use rcore::math::RelRot;
use rcore::script::Script;
use rcore::script::ShipAction;
use rcore::script::ShipScript;
use rcore::script::ShipStatus;
use rcore::ship::DebugBuilder;
use rcore::ship::ShipBuilder;
use rcore::ship::StarterShip;
use rcore::ship::add_ship;

#[cfg(feature = "render")]
use rcore::render::camera::{CameraMode, CameraRig};

fn main() {
    App::new()
        .add_plugins(SetupPlugin)
        // Rest of the game
        .add_plugins(SimulationPlugin)
        //.add_plugins(PhysicsDebugPlugin::default())
        // Log unordered system with overlapping access, this
        // makes the sim nondeterministic.
        .edit_schedule(FixedUpdate, |schedule| {
            schedule.set_build_settings(ScheduleBuildSettings {
                ambiguity_detection: LogLevel::Warn,
                ..Default::default()
            });
        })
        // Startup ship resource for spawning initial ships
        .insert_resource(StartShip(ship_setup()))
        .run();
}

pub struct SetupPlugin;
impl Plugin for SetupPlugin {
    // Normal render Setup
    #[cfg(feature = "render")]
    fn build(&self, app: &mut App) {
        use rcore::render::RenderPlugin;
        use rcore::render::camera::{CameraPlugin, camera_setup};

        app.add_plugins(DefaultPlugins)
            .add_plugins(RenderPlugin)
            .add_plugins(CameraPlugin)
            .add_systems(Startup, add_ships.after(camera_setup));
    }

    // Headless Setup
    #[cfg(not(feature = "render"))]
    fn build(&self, app: &mut App) {
        use bevy::app::ScheduleRunnerPlugin;
        use std::time::Duration;

        app.add_plugins(DefaultPlugins.set(ScheduleRunnerPlugin::run_loop(
            Duration::from_secs_f64(1.0 / 60.0),
        )))
        .add_systems(Startup, add_ships);
    }
}

// Simple ship script
#[derive(Clone)]
struct SimpleShip {
    acc: i32,
    dec: i32,
    rot: i8,
    collision: bool,
    target_x: i32,
    target_y: i32,
    target_e: Option<Entity>,
}

impl SimpleShip {
    fn new() -> Self {
        Self {
            acc: 10,
            dec: 20,
            rot: -128,

            target_x: 0,
            target_y: 0,
            target_e: None,

            collision: false,
        }
    }
}

impl ShipScript for SimpleShip {
    // Minimal go back and forth ship script
    // TODO: figure out the X axis drift, there's a slow sideway drift due to rotation
    // - going upward it snaps between 180 and 0
    // - going downward it slowly changes between 0 to 180 and never quite snaps to 180
    // - figure out why
    fn on_update(&mut self, status: &ShipStatus) -> ShipAction {
        println!(
            "on_update: Pos - {:?} - Vel - {:?} - Rot - {:?}",
            status.position, status.velocity, status.heading,
        );

        if status.heading == AbsRot(0) || status.heading == AbsRot(128) {
            if status.velocity.y < 95 && status.heading == AbsRot(0) {
                println!("Accelerate");
                ShipAction::new()
                    .acceleration(self.acc)
                    .target_entity(self.target_e)
            } else if status.velocity.y > -95 && status.heading == AbsRot(128) {
                println!("Decelerate");
                ShipAction::new()
                    .acceleration(self.dec)
                    .target_entity(self.target_e)
            } else {
                println!("Rotate & Radar");
                ShipAction::new()
                    .heading(RelRot(self.rot))
                    .radar_heading(RelRot(64))
                    .target_entity(self.target_e)
            }
        } else {
            println!("Idle");
            ShipAction::new().target_entity(self.target_e)
        }
    }

    fn on_contact(&mut self, target_pos: IVec2, target_entity: Entity) {
        self.target_x = target_pos.x;
        self.target_y = target_pos.y;
        self.target_e = Some(target_entity);

        println!(
            "on_contact - target.x: {:?}, target.y: {:?}",
            target_pos.x, target_pos.y
        );
    }

    fn on_collision(&mut self) {
        self.collision = true;
        println!("on_collision");
    }
}

// Non-reactive ship
#[derive(Clone)]
struct DummyShip;

impl ShipScript for DummyShip {
    fn on_update(&mut self, _status: &ShipStatus) -> ShipAction {
        ShipAction::new()
    }
    fn on_contact(&mut self, _target_pos: IVec2, _target_entity: Entity) {}
    fn on_collision(&mut self) {}
}

#[derive(Resource)]
struct StartShip(Vec<StarterShip>);

// TODO: a way to init a new ship with some preset value to help script do custom per ship
// things limit_r, target_r,
fn ship_setup() -> Vec<StarterShip> {
    vec![
        ShipBuilder::new(Script {
            script: Box::new(SimpleShip::new()),
        })
        .position(0, 0)
        .velocity(0, 0)
        .velocity_limit(100)
        .rotation(AbsRot(0))
        .rotation_limit(16)
        .radar(AbsRot(0))
        .radar_arc(32)
        .debug(
            DebugBuilder::new()
                .radar()
                .radar_arc()
                .movement()
                .rotation()
                .health()
                .shield_health()
                .shield_arc()
                .build(),
        )
        .build(),
        ShipBuilder::new(Script {
            script: Box::new(DummyShip),
        })
        .position(3500, 0)
        .velocity(0, 0)
        .radar_arc(1)
        .shield(AbsRot(192))
        .shield_damage_reduce(0.75)
        .debug(
            DebugBuilder::new()
                .health()
                .shield_health()
                .shield_arc()
                .build(),
        )
        .build(),
        ShipBuilder::new(Script {
            script: Box::new(DummyShip),
        })
        .position(-3500, 0)
        .velocity(0, 0)
        .radar_arc(1)
        .shield(AbsRot(0))
        .shield_damage_reduce(0.25)
        .debug(
            DebugBuilder::new()
                .health()
                .shield_health()
                .shield_arc()
                .build(),
        )
        .build(),
        ShipBuilder::new(Script {
            script: Box::new(DummyShip),
        })
        .position(-4500, -2500)
        .velocity(0, 0)
        .radar_arc(1)
        .build(),
    ]
}

#[cfg(not(feature = "render"))]
fn add_ships(ships: Res<StartShip>, mut commands: Commands) {
    for ship in ships.0.iter() {
        add_ship(&mut commands, ship.clone());
    }
}

#[expect(clippy::explicit_iter_loop)]
#[cfg(feature = "render")]
fn add_ships(
    ships: Res<StartShip>,
    mut camera: Single<&mut CameraRig, With<Camera2d>>,
    mut commands: Commands,
) {
    let mut ship_id = vec![];
    for ship in ships.0.iter() {
        ship_id.push(add_ship(&mut commands, ship.clone()));
    }

    camera.mode = CameraMode::Follow(ship_id[0]);
}
