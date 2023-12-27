use bevy::prelude::*;
use bevy_prototype_lyon::prelude::*;
use bevy_rapier2d::prelude::*;

use crate::script::Script;

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
pub struct Rotation {
    limit: f32, // Per Second?
    pub target: f32,
}

// Ref-counted collision, if greater than zero, its colloding, otherwise
#[derive(Component)]
pub struct Collision(u32);

// TODO: decouple the rendering stuff somewhat from the rest of the system. Ie we
// still bundle the assets in the ECS, but have all of the system interact within
// the ECS then after things settle -> have a system that takes the ship plugin content
// system and update the sprite/assets/etc to display that information on the screen
pub struct ShipPlugins;
impl Plugin for ShipPlugins {
    fn build(&self, app: &mut App) {
        app.add_plugins(ShapePlugin)
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

// TODO: figure out the time bit so we can do the system in a correct delta-time savvy way
fn apply_velocity(mut query: Query<(&Velocity, &mut Transform)>) {
    for (vec, mut tran) in query.iter_mut() {
        tran.translation.x += vec.0.x;
        tran.translation.y += vec.0.y;
    }
}

// TODO: figure out the time bit so we can do the system in a correct delta-time savvy way
// TODO: this can probs be done better + tested better
fn apply_rotation(mut query: Query<(&Rotation, &mut Transform)>) {
    for (rot, mut tran) in query.iter_mut() {
        // Get current rotation vector, get the target rotation vector, do math, and then rotate
        let curr = tran.rotation;
        let targ = Quat::from_rotation_z(rot.target);

        let delta = (targ * curr.inverse()).to_euler(EulerRot::ZYX).0;

        // If delta is aproximately zero we are on our heading
        if delta.abs() < f32::EPSILON {
            continue;
        }

        // Identify the sign (not sure if need to negate)
        let delta_sign = f32::copysign(1., delta);

        // Clamp the rotation if needed
        let applied_angle = delta_sign * rot.limit.min(delta.abs());

        tran.rotation *= Quat::from_rotation_z(applied_angle);
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

// TODO:
// - Way to load a scene (which sets up where each ships are and any other obstance or resources in
// the gameworld)
// - Way to refer each ship to an AI script
// - Possibly an way to customize the starting ship (via the AI script or some other config for
// each ship)
pub struct StarterShip {
    position: Vec2,
    velocity: Vec2,
    rotation: f32,
    target_r: f32,
    script: Script,
}

impl StarterShip {
    pub fn new(position: Vec2, velocity: Vec2, rotation: f32, target_r: f32, script: Script) -> StarterShip {
        StarterShip {
            position,
            velocity,
            rotation,
            target_r,
            script,
        }
    }
}

pub fn add_ships(
    mut commands: Commands,
    ships: Vec<StarterShip>
) {
    for ship in ships {
        let path = {
            let mut path = PathBuilder::new();
            let _ = path.move_to(Vec2::new(0.0, 20.0));
            let _ = path.line_to(Vec2::new(10.0, -20.0));
            let _ = path.line_to(Vec2::new(0.0, -10.0));
            let _ = path.line_to(Vec2::new(-10.0, -20.0));
            let _ = path.close();
            path.build()
        };

        let mut transform = Transform::from_translation(ship.position.extend(0.));
        transform.rotate_z(ship.target_r);

        commands.spawn((
            ShapeBundle {
                path: path,
                spatial: SpatialBundle {
                    transform: transform,
                    ..default()
                },
                ..default()
            },
            Stroke::new(Color::BLACK, 2.0),
            Fill::color(Color::GREEN),
        ))
            .insert(Ship)
            .insert(Velocity(ship.velocity))
            .insert(Rotation{limit: ship.rotation, target: ship.target_r})
            .insert(ship.script)

            // TODO: probs want collision groups (ie ship vs missile vs other ships)
            .insert(Collider::cuboid(10.0, 20.0))
            .insert(ActiveCollisionTypes::empty() | ActiveCollisionTypes::STATIC_STATIC)
            .insert(ActiveEvents::COLLISION_EVENTS)
            .insert(Sensor)

            .insert(Collision(0));
    }
}

