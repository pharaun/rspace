use bevy::prelude::*;
use bevy_rapier2d::prelude::*;

use bevy_screen_diagnostics::ScreenDiagnosticsPlugin;
use bevy_screen_diagnostics::ScreenEntityDiagnosticsPlugin;
use bevy_screen_diagnostics::ScreenFrameDiagnosticsPlugin;

use bevy_prototype_lyon::prelude::ShapePlugin;

use std::collections::HashMap;
use rust_dynamic::value::Value;

use rcore::arena_bounds_setup;

use rcore::collision::CollisionPlugin;
use rcore::debug_weapon::WeaponPlugin;
use rcore::health::HealthPlugin;
use rcore::movement::MovementPlugin;
use rcore::radar::RadarPlugin;
use rcore::rotation::RotationPlugin;
use rcore::script::ScriptPlugins;
use rcore::spawner::SpawnerPlugin;

use rcore::math::AbsRot;
use rcore::math::RelRot;
use rcore::script::Script;
use rcore::ship::DebugBuilder;
use rcore::ship::ShipBuilder;
use rcore::ship::add_ship;

fn main() {
    App::new()
        //.insert_resource(Time::<Fixed>::from_hz(64.0))
        .insert_resource(Time::<Fixed>::from_hz(2.0))

        // Physics
        .add_plugins(DefaultPlugins)
        .add_plugins(RapierPhysicsPlugin::<NoUserData>::pixels_per_meter(10.0))
        //.add_plugins(RapierDebugRenderPlugin::default())

        // FPS
        .add_plugins(ScreenDiagnosticsPlugin::default())
        .add_plugins(ScreenFrameDiagnosticsPlugin)
        .add_plugins(ScreenEntityDiagnosticsPlugin)

        // Graphics (lyon)
        .add_plugins(ShapePlugin)

        // Game bits
        .add_plugins(CollisionPlugin)
        .add_plugins(HealthPlugin)
        .add_plugins(MovementPlugin)
        .add_plugins(RadarPlugin)
        .add_plugins(RotationPlugin)
        .add_plugins(ScriptPlugins)
        .add_plugins(SpawnerPlugin)
        .add_plugins(WeaponPlugin)

        // Startup setup
        .add_systems(Startup, (
            camera_setup,
            arena_bounds_setup,
            ship_setup,
        ))
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
        ("target.e", Value::none()),
    ])
}

// Minimal go back and forth ship script
// TODO: figure out the X axis drift, there's a slow sideway drift due to rotation
// - going upward it snaps between 180 and 0
// - going downward it slowly changes between 0 to 180 and never quite snaps to 180
// - figure out why
fn on_update(state: &mut HashMap<&'static str, Value>, pos: IVec2, vel: IVec2, rot: AbsRot) -> (RelRot, i32, RelRot, Option<Entity>) {
    println!("on_update: Pos - {:?} - Vel - {:?} - Rot - {:?}", pos, vel, rot);

    let acc: i32 = state.get("acc").unwrap().cast_int().unwrap() as i32;
    let dec: i32 = state.get("dec").unwrap().cast_int().unwrap() as i32;
    let arot: i8 = state.get("rot").unwrap().cast_int().unwrap() as i8;

    let target: Option<Entity> = {
        let entity_value = state.get("target.e").unwrap();

        if entity_value.is_none() {
            None
        } else {
            Some(Entity::from_bits(entity_value.cast_int().unwrap() as u64))
        }
    };

    if rot == AbsRot(0) || rot == AbsRot(128) {
        if vel.y < 95 && rot == AbsRot(0){
            println!("Accelerate");
            (RelRot(0), acc, RelRot(0), target)
        } else if vel.y > -95 && rot == AbsRot(128) {
            println!("Decelerate");
            (RelRot(0), dec, RelRot(0), target)
        } else {
            println!("Rotate & Radar");
            (RelRot(arot), 0, RelRot(64), target)
        }
    } else {
        println!("Idle");
        (RelRot(0), 0, RelRot(0), target)
    }
}

fn on_contact(state: &mut HashMap<&'static str, Value>, target_pos: IVec2, target_entity: Entity) {
    state.insert("target.x", Value::from(target_pos.x).unwrap());
    state.insert("target.y", Value::from(target_pos.y).unwrap());
    state.insert("target.e", Value::from(target_entity.to_bits() as i64).unwrap());
    println!("on_contact - target.x: {:?}, target.y: {:?}", target_pos.x, target_pos.y);
}

fn on_collision(state: &mut HashMap<&'static str, Value>) {
    state.insert("collision", Value::from(true).unwrap());
    println!("on_collision");
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
        ShipBuilder::new(Script::new(on_init, on_update, on_contact, on_collision))
            .position(0, 0)
            .velocity(0, 0)
            .velocity_limit(100)
            .rotation(AbsRot(0))
            .rotation_limit(16)
            .radar(AbsRot(0))
            .radar_arc(64)
            .build(),

        ShipBuilder::new(Script::new(
                || HashMap::from([]),
                |_, _, _, _| (RelRot(0), 0, RelRot(0), None),
                |_, _, _| (),
                |_| (),
            ))
            .position(3500, 0)
            .velocity(0, 0)
            .radar_arc(2)
            .debug(DebugBuilder::new()
                .health()
                .build())
            .build(),

        ShipBuilder::new(Script::new(
                || HashMap::from([]),
                |_, _, _, _| (RelRot(0), 0, RelRot(0), None),
                |_, _, _| (),
                |_| (),
            ))
            .position(-3500, 0)
            .velocity(0, 0)
            .radar_arc(2)
            .debug(DebugBuilder::new()
                .health()
                .build())
            .build(),

        ShipBuilder::new(Script::new(
                || HashMap::from([]),
                |_, _, _, _| (RelRot(0), 0, RelRot(0), None),
                |_, _, _| (),
                |_| (),
            ))
            .position(-4500, 3500)
            .velocity(0, 0)
            .radar_arc(2)
            .build(),
    ];

    for ship in ships {
        add_ship(&mut commands, ship);
    }
}
