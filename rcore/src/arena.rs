use bevy::prelude::*;
use bevy_prototype_lyon::prelude::*;

use crate::ship::movement::Position;
use crate::ship::movement::PreviousPosition;

// This is the display area
const DISPLAY: Vec2 = Vec2::new(1024., 640.);

// This is the actual ship-arena
pub const ARENA_SCALE: f32 = 10.0;
const ARENA: IVec2 = IVec2::new(10240, 6400);

#[derive(Component)]
struct CameraMarker;

// TODO: add an Arena Marker for ships and stuff for objects we want to have warping
// enabled for, versus objects we don't.
#[derive(Component)]
struct ArenaMarker;

pub struct ArenaPlugins;
impl Plugin for ArenaPlugins {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, camera_setup)
            .add_systems(Startup, add_arena_bounds)
            .add_systems(PostUpdate, wrap_position);
    }
}

fn camera_setup(mut commands: Commands) {
    commands.spawn((
        Camera2d::default(),
        CameraMarker,
    ));
}

// The gizmo renders are based off the wrapped ship position which is 1:1 at the moment.
//
// TODO: make sure this only affects transforms for things within the arena, maybe an arena tag is
// needed
// TODO: May want to change this to instead wrap the game-areana and change this to be a render
// concern
fn wrap_position(mut query: Query<(&mut Position, &mut PreviousPosition), Changed<Position>>) {
    for (mut pos, mut ppos) in query.iter_mut() {
        let res: IVec2 = {
            let mut ret = IVec2::new(0, 0);

            if pos.0.y < -(ARENA.y / 2) {
                ret.y += ARENA.y;
            } else if pos.0.y > (ARENA.y / 2) {
                ret.y -= ARENA.y;
            }

            if pos.0.x < -(ARENA.x / 2) {
                ret.x += ARENA.x;
            } else if pos.0.x > (ARENA.x / 2) {
                ret.x -= ARENA.x;
            }
            ret
        };
        pos.0.y += res.y;
        ppos.0.y += res.y;

        pos.0.x += res.x;
        ppos.0.x += res.x;
    }
}

fn add_arena_bounds(mut commands: Commands) {
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
        Transform::from_xyz(DISPLAY.x / 2.0 + 15., 0., -0.8),
        ArenaMarker,
    ));
    commands.spawn((
        Text2d::new("-X"),
        Transform::from_xyz(-(DISPLAY.x / 2.0 + 15.), 0., -0.8),
        ArenaMarker,
    ));
    commands.spawn((
        Text2d::new("+Y"),
        Transform::from_xyz(0., DISPLAY.y / 2.0 + 15., -0.8),
        ArenaMarker,
    ));
    commands.spawn((
        Text2d::new("-Y"),
        Transform::from_xyz(0., -(DISPLAY.y / 2.0 + 15.), -0.8),
        ArenaMarker,
    ));

    // Rotation Angle Compass
}
