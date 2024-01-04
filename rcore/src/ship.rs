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
pub struct Velocity {
    pub acceleration: f32,
    pub velocity: Vec2,

    // TODO: develop the limits
}

#[derive(Component)]
pub struct Rotation {
    limit: f32, // Per Second?
    pub target: Quat,
}

// Ref-counted collision, if greater than zero, its colloding, otherwise
#[derive(Component)]
pub struct Collision(u32);

// Debugging data storage component
#[derive(Component)]
struct RotDebug {
    rotation_current: f32,
    rotation_target: f32,
    rotation_limit: f32,
}

// Debugging data storage component
#[derive(Component)]
struct MovDebug {
    velocity: Vec2,
    acceleration: f32,
}


// TODO: decouple the rendering stuff somewhat from the rest of the system. Ie we
// still bundle the assets in the ECS, but have all of the system interact within
// the ECS then after things settle -> have a system that takes the ship plugin content
// system and update the sprite/assets/etc to display that information on the screen
pub struct ShipPlugins;
impl Plugin for ShipPlugins {
    fn build(&self, app: &mut App) {
        app.add_plugins(ShapePlugin)
            .insert_resource(Time::<Fixed>::from_hz(64.0))
            //.insert_resource(Time::<Fixed>::from_hz(2.0))
            .add_systems(
                FixedUpdate,
                (
                    apply_velocity,
                    apply_rotation,
                ),
            )
//            .add_systems(Update, debug_rotation_gitzmos)
            .add_systems(Update, debug_movement_gitzmos)

            .add_systems(Update, process_events)
            .add_systems(Update, apply_collision.after(process_events));
    }
}

fn apply_velocity(
    time: Res<Time>,
    mut query: Query<(&mut Velocity, &mut Transform, Option<&mut MovDebug>)>
) {
    for (mut vec, mut tran, debug) in query.iter_mut() {
        // DEBUG
        match debug {
            Some(mut dbg) => {
                dbg.acceleration = vec.acceleration;
                dbg.velocity = vec.velocity;
            },
            None => (),
        }

        let acceleration = tran.rotation.mul_vec3(Vec3::Y * vec.acceleration).truncate();
        vec.velocity += acceleration * time.delta_seconds();
        tran.translation += (vec.velocity * time.delta_seconds()).extend(0.);
    }
}

fn debug_movement_gitzmos(
    mut gizmos: Gizmos,
    query: Query<(&Transform, &MovDebug)>
) {
    for (tran, debug) in query.iter() {
        let base = tran.translation.truncate();
        let heading = tran.rotation;

        let debug_velocity = debug.velocity;
        let debug_acceleration = heading.mul_vec3(Vec3::Y * debug.acceleration).truncate();

        // Current heading
        gizmos.line_2d(
            base,
            base + heading.mul_vec3(Vec3::Y * 70.).truncate(),
            Color::RED,
        );

        // Velocity direction
        gizmos.line_2d(
            base,
            base + debug_velocity.normalize() * 60.,
            Color::GREEN,
        );

        // Acceleration direction
        gizmos.line_2d(
            base,
            base + debug_acceleration.normalize() * 50.,
            Color::YELLOW,
        );

        //let zero_speed = draw_bar_gitzmo(base, current, 10., 25.);
    }
}

fn draw_bar_gitzmo(
    base: Vec2,
    rot: Quat,
    width: f32,
    distance: f32,
) -> (Vec2, Vec2) {
    let part_one = Vec3::Y * distance + Vec3::X * (width / 2.);
    let part_two = Vec3::Y * distance + Vec3::NEG_X * (width / 2.);

    (base + rot.mul_vec3(part_one).truncate(),
    base + rot.mul_vec3(part_two).truncate())
}

fn apply_rotation(
    time: Res<Time>,
    mut query: Query<(&Rotation, &mut Transform, Option<&mut RotDebug>)>
) {
    for (rot, mut tran, debug) in query.iter_mut() {
        // Get current rotation vector, get the target rotation vector, do math, and then rotate
        let current = tran.rotation;
        let target = rot.target;
        let limit = Quat::from_rotation_z(rot.limit);

        // DEBUG
        match debug {
            Some(mut dbg) => {
                dbg.rotation_current = current.to_euler(EulerRot::ZYX).0;
                dbg.rotation_target = target.to_euler(EulerRot::ZYX).0;
                dbg.rotation_limit = limit.to_euler(EulerRot::ZYX).0;
            },
            None => (),
        }

        // If this is aproximately zero we are on our heading, bail
        if (current.dot(target) - 1.).abs() < f32::EPSILON {
            continue;
        }

        // Calculate the t-factor for the rotation.lerp
        let max_angle = limit.to_euler(EulerRot::ZYX).0 * time.delta_seconds();
        let angle = current.angle_between(target);
        let t = (1_f32).min(max_angle / angle);

        tran.rotation = tran.rotation.lerp(target, t);
    }
}

// Debug system
fn debug_rotation_gitzmos(
    mut gizmos: Gizmos,
    query: Query<(&Transform, &RotDebug)>
) {
    for (tran, debug) in query.iter() {
        let base = tran.translation.truncate();

        let current = Quat::from_rotation_z(debug.rotation_current);
        let target = Quat::from_rotation_z(debug.rotation_target);
        let limit = Quat::from_rotation_z(debug.rotation_limit);

        gizmos.line_2d(
            base,
            base + current.mul_vec3(Vec3::Y * 90.).truncate(),
            Color::RED,
        );
        gizmos.arc_2d(
            base,
            current.to_euler(EulerRot::ZYX).0 * -1.,
            current.angle_between(current*limit) * 2.,
            80.,
            Color::RED,
        );
        gizmos.line_2d(
            base,
            base + limit.mul_vec3(current.mul_vec3(Vec3::Y * 85.)).truncate(),
            Color::RED,
        );
        gizmos.line_2d(
            base,
            base + limit.inverse().mul_vec3(current.mul_vec3(Vec3::Y * 85.)).truncate(),
            Color::RED,
        );

        gizmos.line_2d(
            base,
            base + target.mul_vec3(Vec3::Y * 80.).truncate(),
            Color::GREEN,
        );
        gizmos.arc_2d(
            base,
            current.lerp(target, 0.5).to_euler(EulerRot::ZYX).0 * -1.,
            current.angle_between(target),
            70.,
            Color::GREEN,
        );
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
    limit_r: f32,
    target_r: f32,
    script: Script,
}

impl StarterShip {
    pub fn new(position: Vec2, velocity: Vec2, limit_r: f32, target_r: f32, script: Script) -> StarterShip {
        StarterShip {
            position,
            velocity,
            limit_r,
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
            .insert(Velocity{velocity: ship.velocity, acceleration: 0. })
            .insert(Rotation{limit: ship.limit_r, target: Quat::from_rotation_z(ship.target_r)})
            .insert(ship.script)

            // TODO: probs want collision groups (ie ship vs missile vs other ships)
            .insert(Collider::cuboid(10.0, 20.0))
            .insert(ActiveCollisionTypes::empty() | ActiveCollisionTypes::STATIC_STATIC)
            .insert(ActiveEvents::COLLISION_EVENTS)
            .insert(Sensor)

            // Debug bits
            //.insert(RotDebug { rotation_current: 0., rotation_target: 0., rotation_limit: 0.})
            .insert(MovDebug { velocity: ship.velocity, acceleration: 0. })

            .insert(Collision(0));
    }
}
