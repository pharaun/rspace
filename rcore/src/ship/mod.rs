use bevy::prelude::*;
use bevy_prototype_lyon::prelude::*;

use bevy_rapier2d::prelude::ActiveCollisionTypes;
use bevy_rapier2d::prelude::ActiveEvents;
use bevy_rapier2d::prelude::Collider;
use bevy_rapier2d::prelude::Sensor;

use crate::script::Script;

pub mod movement;
use crate::ship::movement::Velocity;
use crate::ship::movement::apply_velocity;
use crate::ship::movement::MovDebug;
use crate::ship::movement::debug_movement_gitzmos;

pub mod rotation;
use crate::ship::rotation::Rotation;
use crate::ship::rotation::apply_rotation;
use crate::ship::rotation::RotDebug;
use crate::ship::rotation::debug_rotation_gitzmos;

pub mod radar;
use crate::ship::radar::Radar;
use crate::ship::radar::apply_radar_rotation;
use crate::ship::radar::RadarDebug;
use crate::ship::radar::debug_radar_gitzmos;

pub mod collision;
use crate::ship::collision::Collision;
use crate::ship::collision::apply_collision;
use crate::ship::collision::process_collision_event;

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

            .add_systems(Update, process_collision_event)
            .add_systems(Update, apply_collision.after(process_collision_event));
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
        let ship_path = ShapePath::new()
            .move_to(Vec2::new(0.0, 20.0))
            .line_to(Vec2::new(10.0, -20.0))
            .line_to(Vec2::new(0.0, -10.0))
            .line_to(Vec2::new(-10.0, -20.0))
            .close();

        let radar_path = ShapePath::new()
            .move_to(Vec2::new(5.0, 0.0))
            .arc(Vec2::new(0.0, 0.0), Vec2::new(5.0, 4.5), f32::to_radians(-180.0), f32::to_radians(0.0))
            .move_to(Vec2::new(0.0, 2.0))
            .line_to(Vec2::new(0.0, -4.5));

        let mut transform = Transform::from_translation(ship.position.extend(0.));
        transform.rotate_z(ship.target_r);

        commands.spawn((
            ShapeBuilder::with(&ship_path)
                .fill(Fill::color(bevy::color::palettes::css::GREEN))
                .stroke(Stroke::new(bevy::color::palettes::css::BLACK, 2.0))
                .build(),
            transform
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
            .insert(RotDebug { rotation_current: 0., rotation_target: 0., rotation_limit: 0.})
            .insert(MovDebug { velocity: ship.velocity, acceleration: 0. })
            .insert(RadarDebug { rotation_current: 0., rotation_target: 0., rotation_limit: 0., radar_length: 0., radar_arc: 0.})

            .insert(Collision(0))

            // Insert the graphics for the radar dish
            .with_children(|parent| {
                let mut transform = Transform::from_translation(Vec2::new(0., -2.).extend(1.));
                transform.rotate_z(ship.target_radar);

                parent.spawn((
                    ShapeBuilder::with(&radar_path)
                        .stroke(Stroke::new(bevy::color::palettes::css::MAROON, 1.5))
                        .build(),
                    transform
                ));
            });
    }
}
