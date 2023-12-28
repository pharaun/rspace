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

// Debugging data storage component
#[derive(Component)]
struct Debug {
    rotation_current: f32,
    rotation_target: f32,
    rotation_limit: f32,
    rotation_delta: f32,
    rotation_applied: f32,
}


// TODO: decouple the rendering stuff somewhat from the rest of the system. Ie we
// still bundle the assets in the ECS, but have all of the system interact within
// the ECS then after things settle -> have a system that takes the ship plugin content
// system and update the sprite/assets/etc to display that information on the screen
pub struct ShipPlugins;
impl Plugin for ShipPlugins {
    fn build(&self, app: &mut App) {
        app.add_plugins(ShapePlugin)
            //.insert_resource(Time::<Fixed>::from_hz(64.0))
            .insert_resource(Time::<Fixed>::from_hz(8.0))
            .add_systems(
                FixedUpdate,
                (
                    apply_velocity,
                    apply_rotation,
                ),
            )
            .add_systems(Update, debug_gitzmos)

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
// TODO: this is 64hz, and the limit is at ~per second? so need to figure out how to convert the
// limit to 64hz
fn apply_rotation(mut query: Query<(&Rotation, &mut Transform, &mut Debug)>) {
    for (rot, mut tran, mut debug) in query.iter_mut() {
        // Get current rotation vector, get the target rotation vector, do math, and then rotate
        let curr = tran.rotation;
        let targ = Quat::from_rotation_z(rot.target);

        let delta = (targ * curr.inverse()).to_euler(EulerRot::ZYX).0;
//        let delta = curr.angle_between(targ);

//        // If delta is aproximately zero we are on our heading
//        if delta.abs() < f32::EPSILON {
//            continue;
//        }

        // Identify the sign (not sure if need to negate)
        let delta_sign = f32::copysign(1., delta);

        // Clamp the rotation if needed
        let applied_angle = delta_sign * rot.limit.min(delta.abs());

        // DEBUG: update the debug component.
        debug.rotation_current = curr.to_euler(EulerRot::ZYX).0;
        debug.rotation_target = rot.target;
        debug.rotation_limit = rot.limit;
        debug.rotation_delta = delta;
        debug.rotation_applied = applied_angle;

        // TODO: rotation works, but its applied way too fast, slow down and use slerp or something
        // take in the time-delta in accord
        //tran.rotate_z(applied_angle);
    }
}

// Debug system
fn debug_gitzmos(
    mut gizmos: Gizmos,
    query: Query<(&Transform, &Debug)>
) {
    for (tran, debug) in query.iter() {
        println!(
            "cur: {}, targ: {}, limit: {}, delta: {}, applied: {}",
            debug.rotation_current,
            debug.rotation_target,
            debug.rotation_limit,
            debug.rotation_delta,
            debug.rotation_applied,
        );

        let base = tran.translation.truncate();

        let current = Quat::from_rotation_z(debug.rotation_current);
        let target = Quat::from_rotation_z(debug.rotation_target);
        let limit = Quat::from_rotation_z(debug.rotation_limit);
        let delta = Quat::from_rotation_z(debug.rotation_delta);
        let applied = Quat::from_rotation_z(debug.rotation_applied);

        gizmos.line_2d(
            base,
            base + current.mul_vec3(Vec3::Y * 90.).truncate(),
            Color::RED,
        );
        gizmos.arc_2d(
            base,
            current.to_euler(EulerRot::ZYX).0 * -1.,
            current.angle_between(current*limit*limit),
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
            current.slerp(target, 0.5).to_euler(EulerRot::ZYX).0 * -1.,
            current.angle_between(target),
            70.,
            Color::GREEN,
        );

        gizmos.line_2d(
            base,
            base + delta.mul_vec3(current.mul_vec3(Vec3::Y * 70.)).truncate(),
            Color::YELLOW,
        );
        gizmos.arc_2d(
            base,
            current.slerp(current * delta, 0.5).to_euler(EulerRot::ZYX).0 * -1.,
            current.angle_between(current * delta),
            60.,
            Color::YELLOW,
        );

        gizmos.line_2d(
            base,
            base + applied.mul_vec3(current.mul_vec3(Vec3::Y * 60.)).truncate(),
            Color::ORANGE,
        );
        gizmos.arc_2d(
            base,
            current.slerp(current * applied, 0.5).to_euler(EulerRot::ZYX).0 * -1.,
            current.angle_between(current * applied),
            50.,
            Color::ORANGE,
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
        println!("New ship - limit: {}", limit_r);
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
            .insert(Velocity(ship.velocity))
            .insert(Rotation{limit: ship.limit_r, target: ship.target_r})
            .insert(ship.script)

            // TODO: probs want collision groups (ie ship vs missile vs other ships)
            .insert(Collider::cuboid(10.0, 20.0))
            .insert(ActiveCollisionTypes::empty() | ActiveCollisionTypes::STATIC_STATIC)
            .insert(ActiveEvents::COLLISION_EVENTS)
            .insert(Sensor)

            // Debug bits
            .insert(Debug { rotation_current: 0., rotation_target: 0., rotation_limit: 0., rotation_delta: 0., rotation_applied: 0.})

            .insert(Collision(0));
    }
}
