use bevy::prelude::*;
use bevy_prototype_lyon::prelude::*;

use std::iter::zip;

#[derive(Component)]
struct Ship;

#[derive(Component)]
struct Velocity(Vec2);

#[derive(Component)]
struct Rotation(f32);

// TODO:
// - faction
// - heat
// - hp
// - radar + shield -> Arc (direction + arc width)

fn add_ships(mut commands: Commands) {

    let poss = vec![Vec2::new(50.0, 200.0), Vec2::new(300.0, 0.0), Vec2::new(-200., 0.), Vec2::new(200., 0.)];
    let velo = vec![Vec2::new(-3.0, 1.0), Vec2::new(-2.0, -3.0), Vec2::new(1.0, 0.), Vec2::new(-1.0, 0.)];
    let roto = vec![1.0, 2.0, 0.0, 0.0];

    for (pos, (vel, rot)) in zip(poss, zip(velo, roto)) {
        let path = {
            let mut path = PathBuilder::new();
            let _ = path.move_to(Vec2::new(0.0, 20.0));
            let _ = path.line_to(Vec2::new(10.0, -20.0));
            let _ = path.line_to(Vec2::new(0.0, -10.0));
            let _ = path.line_to(Vec2::new(-10.0, -20.0));
            let _ = path.close();
            path.build()
        };

        commands.spawn((
            ShapeBundle {
                path: path,
                spatial: SpatialBundle {
                    transform: Transform::from_xyz(pos.x, pos.y, 0.),
                    ..default()
                },
                ..default()
            },
            Stroke::new(Color::BLACK, 2.0),
            Fill::color(Color::RED),
        ))
            .insert(Ship)
            .insert(Velocity(vel))
            .insert(Rotation(rot));
    }
}

fn apply_velocity(mut query: Query<(&Velocity, &mut Transform)>) {
    for (vec, mut tran) in query.iter_mut() {
        tran.translation.x += vec.0.x;
        tran.translation.y += vec.0.y;
    }
}

fn apply_rotation(mut query: Query<(&Rotation, &mut Transform)>) {
    for (rot, mut tran) in query.iter_mut() {
        tran.rotation *= Quat::from_rotation_z(0.0174533 * rot.0);
    }
}

// TODO: decouple the rendering stuff somewhat from the rest of the system. Ie we
// still bundle the assets in the ECS, but have all of the system interact within
// the ECS then after things settle -> have a system that takes the ship plugin content
// system and update the sprite/assets/etc to display that information on the screen
struct ShipPlugins;
impl Plugin for ShipPlugins {
    fn build(&self, app: &mut App) {
        app.add_plugins(ShapePlugin)
            .add_systems(Startup, add_ships)

            .insert_resource(Time::<Fixed>::from_hz(64.0))

            .add_systems(
                FixedUpdate,
                (
                    apply_velocity,
                    apply_rotation,
                ),
            );
    }
}


// TODO: Temp size for now
pub const ARENA_WIDTH: f32 = 1024.0;
pub const ARENA_HEIGHT: f32 = 640.0;

struct ArenaPlugins;
impl Plugin for ArenaPlugins {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, camera_setup)
            .add_systems(Startup, add_arena_bounds)
            .add_systems(PostUpdate, wrap_arena);
    }
}

#[derive(Component)]
struct CameraMarker;

fn camera_setup(mut commands: Commands) {
    commands.spawn((
        Camera2dBundle::default(),
        CameraMarker,
    ));
}

// Take care of any existing Transform to make sure it wraps around into the arena again
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

#[derive(Component)]
struct ArenaMarker;
fn add_arena_bounds(mut commands: Commands) {
    let path = {
        let mut path = PathBuilder::new();
        let _ = path.move_to(Vec2::new(-(ARENA_WIDTH / 2.0), -(ARENA_HEIGHT / 2.0)));
        let _ = path.line_to(Vec2::new(ARENA_WIDTH / 2.0, -(ARENA_HEIGHT / 2.0)));
        let _ = path.line_to(Vec2::new(ARENA_WIDTH / 2.0, ARENA_HEIGHT / 2.0));
        let _ = path.line_to(Vec2::new(-(ARENA_WIDTH / 2.0), ARENA_HEIGHT / 2.0));
        let _ = path.close();
        path.build()
    };

    commands.spawn((
        ShapeBundle {
            path: path,
            spatial: SpatialBundle {
                transform: Transform::from_xyz(0., 0., -1.),
                ..default()
            },
            ..default()
        },
        Stroke::new(Color::RED, 1.0),
        Fill::color(Color::BLUE),
        ArenaMarker,
    ));
}

fn main() {
    App::new()
        .insert_resource(Msaa::default())
        .add_plugins(DefaultPlugins)
        .add_plugins(ArenaPlugins)
        .add_plugins(ShipPlugins)
        .run();
}
