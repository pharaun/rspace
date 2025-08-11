use bevy::prelude::*;
use bevy_prototype_lyon::prelude::*;

pub mod class;
pub mod collision;
pub mod weapon;
pub mod math;
pub mod movement;
pub mod radar;
pub mod rotation;
pub mod script;
pub mod ship;
pub mod spawner;

use crate::math::AbsRot;

// This is the display area
const DISPLAY: Vec2 = Vec2::new(1024., 640.);

// This is the actual ship-arena
pub const ARENA_SCALE: f32 = 10.0;
const ARENA: IVec2 = IVec2::new(10240, 6400);

// Systemset to help group systems in a defined order of operation since we now have systems that
// depends on previous systems, and this will help avoid the 1+ frame delay when using events
#[derive(SystemSet, Debug, Hash, Eq, PartialEq,  Clone)]
pub enum FixedGameSystem {
    // Motion, Rotation, Radar, Turret, etc...
    GameLogic,

    // Physics -> https://docs.rs/bevy_rapier2d/latest/bevy_rapier2d/plugin/enum.PhysicsSet.html
    // TODO: replace rapier with a custom collision system so it can reuse the Position/motion code
    // This is for processing the checks for collision and triggering events for all of the
    // downstream consumers (ie weapon - ramming, and ship logic on_collision)
    Collision,

    // The game AI
    ShipLogic,

    // Post game AI systems for handling all of the events spawned from ShipLogic such as spawning
    // new ships, firing weapons, etc..
    Spawn,

    // This is all of the logic that has to do with weapon damage/hits/scan/health
    Weapon,
}

// TODO: add an Arena Marker for ships and stuff for objects we want to have warping
// enabled for, versus objects we don't.
#[derive(Component)]
struct ArenaMarker;

pub fn arena_bounds_setup(mut commands: Commands) {
    // Arena Bounds
    let path = ShapePath::new()
        .move_to(Vec2::new(-(DISPLAY.x / 2.0), -(DISPLAY.y / 2.0)))
        .line_to(Vec2::new(DISPLAY.x / 2.0, -(DISPLAY.y / 2.0)))
        .line_to(Vec2::new(DISPLAY.x / 2.0, DISPLAY.y / 2.0))
        .line_to(Vec2::new(-(DISPLAY.x / 2.0), DISPLAY.y / 2.0))
        .close();

    commands.spawn((
        ShapeBuilder::with(&path)
            .fill(Fill::color(Color::srgb(0.15, 0.15, 0.15)))
            .stroke(Stroke::new(bevy::prelude::Color::Srgba(bevy::color::palettes::css::RED), 1.0))
            .build(),
        Transform::from_xyz(0., 0., -1.),
        ArenaMarker,
    ));

    // Arena Zero axis marks
    let axis = ShapePath::new()
        .move_to(Vec2::new(-(DISPLAY.x / 2.0), 0.0))
        .line_to(Vec2::new(DISPLAY.x / 2.0, 0.0))
        .move_to(Vec2::new(0.0, -(DISPLAY.y / 2.0)))
        .line_to(Vec2::new(0.0, DISPLAY.y / 2.0));

    commands.spawn((
        ShapeBuilder::with(&axis)
            .stroke(Stroke::new(Color::srgb(0.40, 0.40, 0.40), 0.5))
            .build(),
        Transform::from_xyz(0., 0., -0.9),
        ArenaMarker,
    ));

    // Axis Labels
    commands.spawn((
        Text2d::new("+X"),
        Transform::from_xyz(DISPLAY.x / 2.0 + 15., 0., -0.9),
        ArenaMarker,
    ));
    commands.spawn((
        Text2d::new("-X"),
        Transform::from_xyz(-(DISPLAY.x / 2.0 + 15.), 0., -0.9),
        ArenaMarker,
    ));
    commands.spawn((
        Text2d::new("-Y"),
        Transform::from_xyz(0., DISPLAY.y / 2.0 + 15., -0.9),
        ArenaMarker,
    ));
    commands.spawn((
        Text2d::new("+Y"),
        Transform::from_xyz(0., -(DISPLAY.y / 2.0 + 15.), -0.9),
        ArenaMarker,
    ));

    // Rotation Angle Compass
    let compass = ShapePath::new()
        .move_to(Vec2::new(-20., 0.))
        .line_to(Vec2::new(20., 0.))
        .move_to(Vec2::new(0.0, -20.))
        .line_to(Vec2::new(0.0, 20.));

    let base = Vec3::new(425., 225., -0.8);
    commands.spawn((
        ShapeBuilder::with(&compass)
            .stroke(Stroke::new(Color::srgb(0.80, 0.80, 0.80), 0.5))
            .build(),
        Transform::from_translation(base),
        ArenaMarker,
    )).with_children(|parent| {
        for angle in [0, 64, 128, 192] {
            let hdr = AbsRot(angle).to_quat().mul_vec3(Vec3::Y * 40.);

            parent.spawn((
                Text2d::new(format!("{}", angle)),
                Transform::from_translation(hdr),
                ArenaMarker,
            ));
        }
    });
}
