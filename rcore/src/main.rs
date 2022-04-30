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
    let path = {
        let mut path = PathBuilder::new();
        let _ = path.move_to(Vec2::new(0.0, 20.0));
        let _ = path.line_to(Vec2::new(10.0, -20.0));
        let _ = path.line_to(Vec2::new(0.0, -10.0));
        let _ = path.line_to(Vec2::new(-10.0, -20.0));
        let _ = path.close();
        path.build()
    };

    let poss = vec![Vec2::new(50.0, 200.0), Vec2::new(300.0, 0.0)];
    let velo = vec![Vec2::new(-3.0, 1.0), Vec2::new(-2.0, -3.0)];
    let roto = vec![1.0, 2.0];

    for (pos, (vel, rot)) in zip(poss, zip(velo, roto)) {
        commands
            .spawn_bundle(GeometryBuilder::build_as(
                &path,
                DrawMode::Outlined {
                    fill_mode: FillMode::color(Color::CYAN),
                    outline_mode: StrokeMode::new(Color::BLACK, 2.0),
                },
                Transform {
                    translation: pos.extend(0.0),
                    ..default()
                },
            ))
            .insert(Ship)
            .insert(Velocity(vel))
            .insert(Rotation(rot));
    }
}

struct VelocityTimer(Timer);
fn apply_velocity(
    windows: Res<Windows>,
    time: Res<Time>,
    mut timer: ResMut<VelocityTimer>,
    mut query: Query<(&Velocity, &mut Transform)>
) {
    let window = windows.get_primary().unwrap();
    if timer.0.tick(time.delta()).just_finished() {
        for (vec, mut tran) in query.iter_mut() {
            tran.translation.x += vec.0.x;
            tran.translation.y += vec.0.y;

            // Wrap it if needed
            if tran.translation.y < -(window.height() as f32 / 2.0) {
                tran.translation.y += window.height() as f32;
            } else if tran.translation.y > (window.height() as f32 / 2.0) {
                tran.translation.y -= window.height() as f32;
            }

            if tran.translation.x < -(window.width() as f32 / 2.0) {
                tran.translation.x += window.width() as f32;
            } else if tran.translation.x > (window.width() as f32 / 2.0) {
                tran.translation.x -= window.width() as f32;
            }
        }
    }
}

struct RotationTimer(Timer);
fn apply_rotation(
    time: Res<Time>,
    mut timer: ResMut<RotationTimer>,
    mut query: Query<(&mut Rotation, &mut Transform)>
) {
    if timer.0.tick(time.delta()).just_finished() {
        for (mut rot, mut tran) in query.iter_mut() {
            tran.rotation = Quat::from_rotation_z(
                rot.0
            );
            rot.0 += 0.0174533;
        }
    }
}

struct ShipPlugin;
impl Plugin for ShipPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(ShapePlugin)
            .add_startup_system(add_ships)
            .insert_resource(VelocityTimer(Timer::from_seconds(1.0 / 10.0, true)))
            .insert_resource(RotationTimer(Timer::from_seconds(1.0 / 30.0, true)))
            .add_system(apply_velocity)
            .add_system(apply_rotation);
    }
}


fn global_setup(mut commands: Commands) {
    commands.spawn_bundle(OrthographicCameraBundle::new_2d());
}


fn main() {
    App::new()
        .insert_resource(Msaa { samples: 4 })
        .add_plugins(DefaultPlugins)
        .add_startup_system(global_setup)
        .add_plugin(ShipPlugin)
        .run();
}
