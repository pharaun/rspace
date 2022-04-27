use bevy::prelude::*;
use bevy_prototype_lyon::prelude::*;

#[derive(Component)]
struct Ship;

#[derive(Component)]
struct Position {
    x: f32,
    y: f32,
}

// TODO:
// - faction
// - heat
// - hp
// - radar + shield -> Arc (direction + arc width)

fn add_ships(mut commands: Commands) {
    commands.spawn()
        .insert(Ship)
        .insert(Position { x: 1.0, y: 4.0 });

    commands.spawn()
        .insert(Ship)
        .insert(Position { x: 6.0, y: 0.0 });
}

fn print_position(query: Query<&Position, With<Ship>>) {
    for pos in query.iter() {
        println!("x: {}, y: {}", pos.x, pos.y);
    }
}


fn add_shape(mut commands: Commands) {
    let shape = shapes::RegularPolygon {
        sides: 6,
        feature: shapes::RegularPolygonFeature::Radius(200.0),
        ..shapes::RegularPolygon::default()
    };

    commands.spawn_bundle(OrthographicCameraBundle::new_2d());
    commands.spawn_bundle(GeometryBuilder::build_as(
        &shape,
        DrawMode::Outlined {
            fill_mode: FillMode::color(Color::CYAN),
            outline_mode: StrokeMode::new(Color::BLACK, 10.0),
        },
        Transform::default(),
    ));
}

fn main() {
    App::new()
        .insert_resource(Msaa { samples: 4 })
        .add_plugins(DefaultPlugins)
        .add_plugin(ShapePlugin)
        .add_startup_system(add_shape)
        .add_startup_system(add_ships)
        .add_system(print_position)
        .run();
}
