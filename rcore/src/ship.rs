use bevy::prelude::*;
use bevy_prototype_lyon::prelude::*;
use bevy_rapier2d::prelude::*;

use crate::script::Script;

// INFO:
// - Ship class: Tiny, Small, Med, Large where they would occupy roughly
//  * Missile/mines
//  * Fighter
//  * Gunship/frigate
//  * Cruiser/Construction ship
//
// But there is some flexibity ie:
//  * missile with no engine in it -> mine
//  * missile with engine but no computer/radar/etc -> rocket/torpedo
//  * missile with intelligence/radar/fuel/engine -> missile
//
// Unclear if i want to also allow for that level of flexibity in ie fighter/gunship tier of ships
// possibly up to cruiser, image a cruiser sized missile, but that kinda seems ehh, so i feel like
// useful missiles would be tiny or small (torpedos for eg) but anything larger is not.
//
// Each thing takes a certain amount of power/fuel while active ie:
// - radar, computer, bomb, gun, etc... so a misssile can accelerate with some fuel then go into
// sleep till its close to the enemy and wake up and do final targeting adjustment/burst of
// acceleration to hit the target.
// - Unclear if we want to allow for guns on missile, ie it expires and fires off a laser at the
// enemy
// - This could be a interesting variant, instead of missiles actually hitting the enemy we can
// have them having an explode final step which implodes into a powerful xray-laser that then goes
// in a specified direction. then fighter, possibly say gunship can do the same for a even more
// powerful laser.
//
// Maybe there is an concept where a cruiser doesn't have weapons but it can offload/generate:
//  - missile/fighter/gunship sized laser-bomb and then when they blow up they can serve as the
//  cruiser gun for eg.
//
// Balancing:
// - probs have to figure out the explosion->laser effect on how to balance/make it work, and
// figure out if we still want normal types of ammo at all or if we want it to be mainly cruisers
// and cruisers that can build other cruisers, then everything smaller than a cruiser can be ...
// well... weapons for the cruiser (ie medium frigate sized bomb-laser) to accelerate an immense
// amount of power/particle at the enemy at a decent range.
//
// This would make targeting/gunnery kida interesting where cruisers would focus on finding targets
// or it could delegate it to a target-finder ship and then that would then instruct various groups
// of weapons to fire in a particular direction or accelerate toward that direction and fire when
// near.

// TODO:
// - faction
// - heat - affect radar discovery & engine and other system health
// - hp - collision/damaging (ammo/missiles/etc)
// - shield -> Arc (direction + arc width) - less wide == more damage reduction where if its
// pinsized its nearly 100% but if its 360 its nearly 0% damage reduction
// - ship types (missiles, big, middle, small size ship)
// - Ship energy (fuel for engine? and heat production)
// - Ship construction (each ship can build a ship same size or smaller than itself?
#[derive(Component)]
struct Ship;

#[derive(Component)]
pub struct Velocity {
    pub acceleration: f32,
    pub velocity: Vec2,

    // TODO: develop the limits
    velocity_limit: f32,
}

#[derive(Component)]
pub struct Rotation {
    limit: f32, // Per Second?
    pub target: Quat,
}

// Ref-counted collision, if greater than zero, its colloding, otherwise
#[derive(Component)]
pub struct Collision(u32);

// Radar:
//  TODO: Other types such as fixed radar (missiles?) and rotating radar
//  - Direction + arc-width (boosting detection distance)
#[derive(Component)]
pub struct Radar {
    limit: f32, // Per second?
    pub arc: f32, // Radian the width of the arc
    pub target: Quat, // Direction the radar should be pointing in
}

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

// Debugging radar
#[derive(Component)]
struct RadarDebug {
    rotation_current: f32,
    rotation_target: f32,
    rotation_limit: f32,

    radar_length: f32,
    radar_arc: f32,
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
            .insert_resource(Time::<Fixed>::from_hz(2.0))
            .add_systems(
                FixedUpdate,
                (
                    apply_velocity,
                    apply_rotation,
                    apply_radar_rotation,
                ),
            )
            .add_systems(Update, debug_rotation_gitzmos)
            .add_systems(Update, debug_movement_gitzmos)
            .add_systems(Update, debug_radar_gitzmos)

            .add_systems(Update, process_events)
            .add_systems(Update, apply_collision.after(process_events));
    }
}

// TODO: improve this to integrate in forces (ie fireing of guns for smaller ships, etc)
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

        // TODO: figure out how to lerp? There is also an awkward sideward acceleration
        // when we rotate 180, figure out why that happens
        let mut acceleration = tran.rotation.mul_vec3(Vec3::Y * vec.acceleration).truncate();

        // Apply Lorentz factor only if it will increase the velocity
        // Inspiration: https://stackoverflow.com/a/2891162
        let new_velocity = vec.velocity + acceleration * time.delta_secs();

        // TODO: this is not realistic, but keeps ship controllable (ie easy deceleration)
        if new_velocity.length_squared() > vec.velocity.length_squared() {
            // Y = 1 / Sqrt(1 - v^2/c^2), Clamp (1 - v^2/c^2) to float min to avoid NaN and inf
            // Simplified via multiplying by the factor rather than diving
            let lorentz = (
                (1.0 - (
                    vec.velocity.length_squared() / vec.velocity_limit.powi(2)
                )).max(0.0)
            ).sqrt();

            // TODO: it does go over 10 but that's cuz of delta-time and changing acceleration
            // curves, plus floating point imprecision... See if there's a better way to do it or
            // if we need to bite the bullet and go for a integrator for these
            acceleration *= lorentz;
        }

        // NOTE: This will make direction change be sluggish unless the ship decelerate enough to
        // do so. Could optionally allow for a heading change while preserving the current velocity
        vec.velocity += acceleration * time.delta_secs();
        tran.translation += (vec.velocity * time.delta_secs()).extend(0.);
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
            bevy::color::palettes::css::RED,
        );

        // Velocity direction
        gizmos.line_2d(
            base,
            base + debug_velocity.normalize() * 60.,
            bevy::color::palettes::css::GREEN,
        );

        // Acceleration direction
        gizmos.line_2d(
            base,
            base + debug_acceleration.normalize() * 50.,
            bevy::color::palettes::css::YELLOW,
        );
    }
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
        let max_angle = limit.to_euler(EulerRot::ZYX).0 * time.delta_secs();
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
            bevy::color::palettes::css::RED,
        );
        gizmos.arc_2d(
            Isometry2d::new(
                base,
                Rot2::radians(
                    (current.to_euler(EulerRot::ZYX).0 * -1.) + (current.angle_between(current*limit) * 2.) * 0.5
                )
            ),
            current.angle_between(current*limit) * 2.,
            80.,
            bevy::color::palettes::css::RED,
        );
        gizmos.line_2d(
            base,
            base + limit.mul_vec3(current.mul_vec3(Vec3::Y * 85.)).truncate(),
            bevy::color::palettes::css::RED,
        );
        gizmos.line_2d(
            base,
            base + limit.inverse().mul_vec3(current.mul_vec3(Vec3::Y * 85.)).truncate(),
            bevy::color::palettes::css::RED,
        );

        gizmos.line_2d(
            base,
            base + target.mul_vec3(Vec3::Y * 80.).truncate(),
            bevy::color::palettes::css::GREEN,
        );
        gizmos.arc_2d(
            Isometry2d::new(
                base,
                Rot2::radians(
                    (current.lerp(target, 0.5).to_euler(EulerRot::ZYX).0 * -1.) + current.angle_between(target) * 0.5
                )
            ),
            current.angle_between(target),
            70.,
            bevy::color::palettes::css::GREEN,
        );
    }
}

// TODO:
// - radar rotation system
// - radar arc2length via area rule system?
// - radar detection system -> emits contact events.
// - Script subsystem listen for contact event and act upon it
fn apply_radar_rotation(
    time: Res<Time>,
    mut query: Query<(&Radar, &mut Transform, Option<&mut RadarDebug>)>
) {
    for (radar, mut tran, debug) in query.iter_mut() {
        // Get current rotation vector, get the target rotation vector, do math, and then rotate
        let current = tran.rotation;
        let target = radar.target;
        let limit = Quat::from_rotation_z(radar.limit);

        // DEBUG
        match debug {
            Some(mut dbg) => {
                dbg.rotation_current = current.to_euler(EulerRot::ZYX).0;
                dbg.rotation_target = target.to_euler(EulerRot::ZYX).0;
                dbg.rotation_limit = limit.to_euler(EulerRot::ZYX).0;

                dbg.radar_length = 0f32;
                dbg.radar_arc = radar.arc;
            },
            None => (),
        }
    }
}

// Probs a universal debugger that debug rotation + arc2length, and detection?
fn debug_radar_gitzmos(
    mut gizmos: Gizmos,
    query: Query<(&Transform, &RadarDebug)>
) {
    for (tran, debug) in query.iter() {
    }
}

fn apply_collision(mut query: Query<(&Collision, &mut Fill)>) {
    for (collision, mut fill) in query.iter_mut() {
        if collision.0 == 0 {
            fill.color = bevy::prelude::Color::Srgba(bevy::color::palettes::css::GREEN);
        } else {
            fill.color = bevy::prelude::Color::Srgba(bevy::color::palettes::css::RED);
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
// - Dig into ECS archtype to help with some of these setup stuff
pub struct StarterShip {
    position: Vec2,
    velocity: Vec2,
    limit_v: f32,
    limit_r: f32,
    target_r: f32,
    limit_radar: f32,
    arc_radar: f32,
    target_radar: f32,
    script: Script,
}

// TODO: time to implement a builder pattern since this is getting tedious
impl StarterShip {
    pub fn new(
        position: Vec2,
        velocity: Vec2,
        limit_v: f32,
        limit_r: f32,
        target_r: f32,
        limit_radar: f32,
        arc_radar: f32,
        target_radar: f32,
        script: Script
    ) -> StarterShip {
        StarterShip {
            position,
            velocity,
            limit_v,
            limit_r,
            target_r,
            limit_radar,
            arc_radar,
            target_radar,
            script,
        }
    }
}

pub fn add_ships(
    mut commands: Commands,
    ships: Vec<StarterShip>
) {
    for ship in ships {
        let ship_path = {
            let mut path = PathBuilder::new();
            let _ = path.move_to(Vec2::new(0.0, 20.0));
            let _ = path.line_to(Vec2::new(10.0, -20.0));
            let _ = path.line_to(Vec2::new(0.0, -10.0));
            let _ = path.line_to(Vec2::new(-10.0, -20.0));
            let _ = path.close();
            path.build()
        };

        let radar_path = {
            let mut path = PathBuilder::new();
            let _ = path.move_to(Vec2::new(5.0, 0.0));
            let _ = path.arc(Vec2::new(0.0, 0.0), Vec2::new(5.0, 4.5), f32::to_radians(-180.0), f32::to_radians(0.0));
            let _ = path.move_to(Vec2::new(0.0, 2.0));
            let _ = path.line_to(Vec2::new(0.0, -4.5));
            path.build()
        };

        let mut transform = Transform::from_translation(ship.position.extend(0.));
        transform.rotate_z(ship.target_r);

        commands.spawn((
            ShapeBundle {
                path: ship_path,
                transform: transform,
                ..default()
            },
            Stroke::new(bevy::color::palettes::css::BLACK, 2.0),
            Fill::color(bevy::color::palettes::css::GREEN),
        ))
            .insert(Ship)
            .insert(Velocity{velocity: ship.velocity, acceleration: 0., velocity_limit: ship.limit_v})
            .insert(Rotation{limit: ship.limit_r, target: Quat::from_rotation_z(ship.target_r)})
            .insert(Radar{limit: ship.limit_radar, arc: ship.arc_radar, target: Quat::from_rotation_z(ship.target_radar)})
            .insert(ship.script)

            // TODO: probs want collision groups (ie ship vs missile vs other ships)
            .insert(Collider::cuboid(10.0, 20.0))
            .insert(ActiveCollisionTypes::empty() | ActiveCollisionTypes::STATIC_STATIC)
            .insert(ActiveEvents::COLLISION_EVENTS)
            .insert(Sensor)

            // Debug bits
            //.insert(RotDebug { rotation_current: 0., rotation_target: 0., rotation_limit: 0.})
            //.insert(MovDebug { velocity: ship.velocity, acceleration: 0. })
            .insert(RadarDebug { rotation_current: 0., rotation_target: 0., rotation_limit: 0., radar_length: 0., radar_arc: 0.})

            .insert(Collision(0))

            // Insert the graphics for the radar dish
            .with_children(|parent| {
                let mut transform = Transform::from_translation(Vec2::new(0., -2.).extend(1.));
                transform.rotate_z(ship.target_radar);

                parent.spawn((
                    ShapeBundle {
                        path: radar_path,
                        transform: transform,
                        ..default()
                    },
                    Stroke::new(bevy::color::palettes::css::MAROON, 1.5),
                ));
            });
    }
}
