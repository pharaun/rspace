use bevy::prelude::*;
use bevy_prototype_lyon::prelude::*;

use crate::ship::movement::Position;
use crate::ship::movement::PreviousPosition;

// TODO: Temp size for now
pub const ARENA_WIDTH: f32 = 1024.0;
pub const ARENA_HEIGHT: f32 = 640.0;

#[derive(Component)]
struct CameraMarker;

#[derive(Component)]
struct ArenaMarker;

// TODO: Add more support for other things like gizmos to support the wrap
pub struct ArenaPlugins;
impl Plugin for ArenaPlugins {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, camera_setup)
            .add_systems(Startup, add_arena_bounds)
            .add_systems(PostUpdate, wrap_transform)
            .add_systems(PostUpdate, wrap_position);
    }
}

fn camera_setup(mut commands: Commands) {
    commands.spawn((
        Camera2d::default(),
        CameraMarker,
    ));
}

// Take care of any existing Transform to make sure it wraps around into the arena again
// TODO: make sure this only affects transforms for things within the arena, maybe an arena tag is
// needed
// TODO: May want to change this to instead wrap the game-areana and change this to be a render
// concern
fn wrap_transform(mut query: Query<&mut Transform, Changed<Transform>>) {
    for mut tran in query.iter_mut() {
        let res = wrap(tran.translation.truncate());
        tran.translation.y += res.y;
        tran.translation.x += res.x;
    }
}

fn wrap_position(mut query: Query<(&mut Position, &mut PreviousPosition), Changed<Position>>) {
    for (mut pos, mut ppos) in query.iter_mut() {
        let res = wrap(pos.0);
        pos.0.y += res.y;
        ppos.0.y += res.y;

        pos.0.x += res.x;
        ppos.0.x += res.x;
    }
}

fn wrap(vec: Vec2) -> Vec2 {
    let mut ret = Vec2::new(0., 0.);

    if vec.y < -(ARENA_HEIGHT / 2.0) {
        ret.y += ARENA_HEIGHT;
    } else if vec.y > (ARENA_HEIGHT / 2.0) {
        ret.y -= ARENA_HEIGHT;
    }

    if vec.x < -(ARENA_WIDTH / 2.0) {
        ret.x += ARENA_WIDTH;
    } else if vec.x > (ARENA_WIDTH / 2.0) {
        ret.x -= ARENA_WIDTH;
    }

    ret
}

fn add_arena_bounds(mut commands: Commands) {
    let path = ShapePath::new()
        .move_to(Vec2::new(-(ARENA_WIDTH / 2.0), -(ARENA_HEIGHT / 2.0)))
        .line_to(Vec2::new(ARENA_WIDTH / 2.0, -(ARENA_HEIGHT / 2.0)))
        .line_to(Vec2::new(ARENA_WIDTH / 2.0, ARENA_HEIGHT / 2.0))
        .line_to(Vec2::new(-(ARENA_WIDTH / 2.0), ARENA_HEIGHT / 2.0))
        .close();

    commands.spawn((
        ShapeBuilder::with(&path)
            .fill(Fill::color(Color::srgb(0.15, 0.15, 0.15)))
            .stroke(Stroke::new(bevy::prelude::Color::Srgba(bevy::color::palettes::css::RED), 1.0))
            .build(),
        Transform::from_xyz(0., 0., -1.),
        ArenaMarker,
    ));
}
