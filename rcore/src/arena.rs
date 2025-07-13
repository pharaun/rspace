use bevy::prelude::*;
use bevy_prototype_lyon::prelude::*;

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
            .add_systems(PostUpdate, wrap_arena);
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
fn wrap_arena(mut query: Query<&mut Transform, Changed<Transform>>) {
    for mut tran in query.iter_mut() {
        if tran.translation.y < -(ARENA_HEIGHT / 2.0) {
            tran.translation.y += ARENA_HEIGHT;
        } else if tran.translation.y > (ARENA_HEIGHT / 2.0) {
            tran.translation.y -= ARENA_HEIGHT;
        }

        if tran.translation.x < -(ARENA_WIDTH / 2.0) {
            tran.translation.x += ARENA_WIDTH;
        } else if tran.translation.x > (ARENA_WIDTH / 2.0) {
            tran.translation.x -= ARENA_WIDTH;
        }
    }
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
