use bevy::prelude::*;
use avian2d::prelude::*;

use bevy_screen_diagnostics::ScreenDiagnosticsPlugin;
use bevy_screen_diagnostics::ScreenEntityDiagnosticsPlugin;
use bevy_screen_diagnostics::ScreenFrameDiagnosticsPlugin;

use bevy_prototype_lyon::prelude::ShapePlugin;

use std::sync::Arc;
use std::sync::Mutex;

use rcore::FixedGameSystem;
use rcore::arena_bounds_setup;

use rcore::weapon::WeaponPlugin;
use rcore::movement::MovementPlugin;
use rcore::radar::RadarPlugin;
use rcore::rotation::RotationPlugin;
use rcore::script::ScriptPlugins;
use rcore::spawner::SpawnerPlugin;

use rcore::math::AbsRot;
use rcore::math::RelRot;
use rcore::script::ShipScript;
use rcore::script::Script;
use rcore::ship::DebugBuilder;
use rcore::ship::ShipBuilder;
use rcore::ship::add_ship;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)

        // Physics
        // TODO: make sure it happens post iterpolation
        .add_plugins(PhysicsPlugins::default())
        //.add_plugins(PhysicsDebugPlugin::default())

        // FPS
        .add_plugins(ScreenDiagnosticsPlugin::default())
        .add_plugins(ScreenFrameDiagnosticsPlugin)
        .add_plugins(ScreenEntityDiagnosticsPlugin)

        // Graphics (lyon)
        .add_plugins(ShapePlugin)

        // TODO: fix up systems so i can bump it to bevy default 64hz
        .insert_resource(Time::<Fixed>::from_hz(2.0))

        // Game bits
        .add_plugins(MovementPlugin)
        .add_plugins(RadarPlugin)
        .add_plugins(RotationPlugin)
        .add_plugins(ScriptPlugins)
        .add_plugins(SpawnerPlugin)
        .add_plugins(WeaponPlugin)

        // Configure the system set ordering
        .configure_sets(FixedUpdate, (
            FixedGameSystem::GameLogic,
            FixedGameSystem::ShipLogic,
            FixedGameSystem::Spawn,
            FixedGameSystem::Weapon,
        ).chain())

        // Startup setup
        .add_systems(Startup, (
            camera_setup,
            arena_bounds_setup,
            ship_setup,
        ))
        .run();
}

// Simple ship script
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
    fn new() -> SimpleShip {
        SimpleShip {
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
    fn on_update(
        &mut self,
        pos: IVec2,
        vel: IVec2,
        rot: AbsRot
    ) -> (RelRot, i32, RelRot, Option<Entity>) {
        println!("on_update: Pos - {:?} - Vel - {:?} - Rot - {:?}", pos, vel, rot);

        if rot == AbsRot(0) || rot == AbsRot(128) {
            if vel.y < 95 && rot == AbsRot(0){
                println!("Accelerate");
                (RelRot(0), self.acc, RelRot(0), self.target_e)
            } else if vel.y > -95 && rot == AbsRot(128) {
                println!("Decelerate");
                (RelRot(0), self.dec, RelRot(0), self.target_e)
            } else {
                println!("Rotate & Radar");
                (RelRot(self.rot), 0, RelRot(64), self.target_e)
            }
        } else {
            println!("Idle");
            (RelRot(0), 0, RelRot(0), self.target_e)
        }
    }

    fn on_contact(&mut self, target_pos: IVec2, target_entity: Entity) {
        self.target_x = target_pos.x;
        self.target_y = target_pos.y;
        self.target_e = Some(target_entity);

        println!("on_contact - target.x: {:?}, target.y: {:?}", target_pos.x, target_pos.y);
    }

    fn on_collision(&mut self) {
        self.collision = true;
        println!("on_collision");
    }
}

// Non-reactive ship
struct DummyShip;

impl ShipScript for DummyShip {
    fn on_update(
        &mut self,
        _pos: IVec2,
        _vel: IVec2,
        _rot: AbsRot
    ) -> (RelRot, i32, RelRot, Option<Entity>) {
        (RelRot(0), 0, RelRot(0), None)
    }
    fn on_contact(&mut self, _target_pos: IVec2, _target_entity: Entity) {}
    fn on_collision(&mut self) {}
}

#[derive(Component)]
struct CameraMarker;

fn camera_setup(mut commands: Commands) {
    commands.spawn((
        Camera2d::default(),
        CameraMarker,
    ));
}

// TODO: a way to init a new ship with some preset value to help script do custom per ship
// things limit_r, target_r,
fn ship_setup(mut commands: Commands) {
    let ships = vec![
        ShipBuilder::new(Script { script: Arc::new(Mutex::new(SimpleShip::new())) })
            .position(0, 0)
            .velocity(0, 0)
            .velocity_limit(100)
            .rotation(AbsRot(0))
            .rotation_limit(16)
            .radar(AbsRot(0))
            .radar_arc(64)
            .debug(DebugBuilder::new()
                .radar_arc()
                .build())
            .build(),

        ShipBuilder::new(Script { script: Arc::new(Mutex::new(DummyShip)) })
            .position(3500, 0)
            .velocity(0, 0)
            .radar_arc(2)
            .shield(AbsRot(192))
            .shield_damage_reduce(0.75)
            .debug(DebugBuilder::new()
                .health()
                .shield_health()
                .shield_arc()
                .build())
            .build(),

        ShipBuilder::new(Script { script: Arc::new(Mutex::new(DummyShip)) })
            .position(-3500, 0)
            .velocity(0, 0)
            .radar_arc(2)
            .shield(AbsRot(0))
            .shield_damage_reduce(0.25)
            .debug(DebugBuilder::new()
                .health()
                .shield_health()
                .shield_arc()
                .build())
            .build(),

        ShipBuilder::new(Script { script: Arc::new(Mutex::new(DummyShip)) })
            .position(-4500, -2500)
            .velocity(0, 0)
            .radar_arc(2)
            .build(),
    ];

    for ship in ships {
        add_ship(&mut commands, ship);
    }
}
