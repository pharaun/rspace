use bevy::prelude::*;
use bevy_prototype_lyon::prelude::*;

#[derive(Component)]
struct Ship;

#[derive(Component)]
struct Velocity(Vec2);

// TODO:
// - faction
// - heat
// - hp
// - radar + shield -> Arc (direction + arc width)

fn add_ships(mut commands: Commands) {
    // TODO: draw an actual ship
    let shape = shapes::RegularPolygon {
        sides: 6,
        feature: shapes::RegularPolygonFeature::Radius(20.0),
        ..shapes::RegularPolygon::default()
    };

    let poss = vec![Vec2::new(50.0, 200.0), Vec2::new(300.0, 0.0)];

    for pos in poss {
        commands
            .spawn_bundle(GeometryBuilder::build_as(
                &shape,
                DrawMode::Outlined {
                    fill_mode: FillMode::color(Color::CYAN),
                    outline_mode: StrokeMode::new(Color::BLACK, 5.0),
                },
                Transform {
                    translation: pos.extend(0.0),
                    ..default()
                },
            ))
            .insert(Ship)
            .insert(Velocity(Vec2::new(1.0, 5.0)));
    }
}

struct VelocityTimer(Timer);
fn apply_velocity(
    time: Res<Time>,
    mut timer: ResMut<VelocityTimer>,
    mut query: Query<(&Velocity, &mut Transform)>
) {
    if timer.0.tick(time.delta()).just_finished() {
        for (vec, mut tran) in query.iter_mut() {
            println!("POS - x: {}, y: {}", tran.translation.x, tran.translation.y);
            println!("VEC - x: {}, y: {}", vec.0.x, vec.0.y);

            tran.translation.x += vec.0.x;
            tran.translation.y += vec.0.y;
        }
    }
}

struct ShipPlugin;
impl Plugin for ShipPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(ShapePlugin)
            .add_startup_system(add_ships)
            .insert_resource(VelocityTimer(Timer::from_seconds(1.0, true)))
            .add_system(apply_velocity);
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
