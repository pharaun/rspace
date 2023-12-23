use bevy::prelude::*;
use bevy_prototype_lyon::prelude::*;
use bevy_rapier2d::prelude::*;

use std::iter::zip;

use crate::script::{ScriptEngine, new_script};

// TODO:
// - faction
// - heat
// - hp
// - radar + shield -> Arc (direction + arc width)
#[derive(Component)]
struct Ship;

#[derive(Component)]
pub struct Velocity(pub Vec2);

#[derive(Component)]
pub struct Rotation(pub f32);

// Ref-counted collision, if greater than zero, its colloding, otherwise
#[derive(Component)]
pub struct Collision(u32);


fn add_ships(
    script_engine: Res<ScriptEngine>,
    mut commands: Commands,
) {
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
            Fill::color(Color::GREEN),
        ))
            .insert(Ship)
            .insert(Velocity(vel))
            .insert(Rotation(rot))

            .insert(new_script(&script_engine))

            // TODO: probs want collision groups (ie ship vs missile vs other ships)
            .insert(Collider::cuboid(10.0, 20.0))
            .insert(ActiveCollisionTypes::empty() | ActiveCollisionTypes::STATIC_STATIC)
            .insert(ActiveEvents::COLLISION_EVENTS)
            .insert(Sensor)

            .insert(Collision(0));
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

fn apply_collision(mut query: Query<(&Collision, &mut Fill)>) {
    for (collision, mut fill) in query.iter_mut() {
        if collision.0 == 0 {
            fill.color = Color::GREEN;
        } else {
            fill.color = Color::RED;
        }
    }
}

// collision detection
fn process_events(
    mut collision_events: EventReader<CollisionEvent>,
    mut query: Query<&mut Collision>,
) {
    for collision_event in collision_events.read() {
        match collision_event {
            //struct Collision(u32);
            CollisionEvent::Started(e1, e2, _) => {
                if let Ok([mut e1_collision, mut e2_collision]) = query.get_many_mut([*e1, *e2]) {
                    e1_collision.0 += 1;
                    e2_collision.0 += 1;
                } else {
                    println!("ERROR - ECS - {:?}", collision_event);
                }
            },
            CollisionEvent::Stopped(e1, e2, _) => {
                if let Ok([mut e1_collision, mut e2_collision]) = query.get_many_mut([*e1, *e2]) {
                    e1_collision.0 -= 1;
                    e2_collision.0 -= 1;
                } else {
                    println!("ERROR - ECS - {:?}", collision_event);
                }
            },
        }
    }
}

// TODO: decouple the rendering stuff somewhat from the rest of the system. Ie we
// still bundle the assets in the ECS, but have all of the system interact within
// the ECS then after things settle -> have a system that takes the ship plugin content
// system and update the sprite/assets/etc to display that information on the screen
pub struct ShipPlugins;
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
            )
            .add_systems(Update, process_events)
            .add_systems(Update, apply_collision.after(process_events));
    }
}
