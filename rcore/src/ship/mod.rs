use bevy::prelude::*;
use bevy_prototype_lyon::prelude::*;

use bevy_rapier2d::prelude::ActiveCollisionTypes;
use bevy_rapier2d::prelude::ActiveEvents;
use bevy_rapier2d::prelude::Collider;
use bevy_rapier2d::prelude::Sensor;

use crate::math::AbsRot;
use crate::script::Script;
use crate::math::vec_scale;
use crate::arena::ARENA_SCALE;

pub mod movement;
use crate::ship::movement::interpolate_transforms;
use crate::ship::movement::Velocity;
use crate::ship::movement::apply_velocity;
use crate::ship::movement::MovDebug;
use crate::ship::movement::debug_movement_gitzmos;

pub mod rotation;
use crate::ship::rotation::interpolate_rotation;
use crate::ship::rotation::Rotation;
use crate::ship::rotation::apply_rotation;
use crate::ship::rotation::RotDebug;
use crate::ship::rotation::debug_rotation_gitzmos;
use crate::ship::rotation::TargetRotation;

pub mod radar;
use crate::ship::radar::Radar;
use crate::ship::radar::apply_radar;
use crate::ship::radar::RadarDebug;
use crate::ship::radar::debug_radar_gitzmos;
use crate::ship::radar::ContactEvent;

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
// TODO: probs want to have 2 or 3 separate subsystem
//  - Rendering bits for the ship
//  - Simulation bits (ie universal sim bits)
//  - Specific per ship features
pub struct ShipPlugins;
impl Plugin for ShipPlugins {
    fn build(&self, app: &mut App) {
        app.add_plugins(ShapePlugin)
            //.insert_resource(Time::<Fixed>::from_hz(64.0))
            .insert_resource(Time::<Fixed>::from_hz(2.0))
            .add_event::<ContactEvent>()
            .add_systems(
                FixedUpdate,
                (
                    apply_velocity,
                    apply_rotation,
                    // TODO: apply radar rotation, then process radar_event
                    apply_radar,
                ),
            )
            .add_systems(
                RunFixedMainLoop,
                (
                    interpolate_transforms.in_set(RunFixedMainLoopSystem::AfterFixedMainLoop),
                    interpolate_rotation.in_set(RunFixedMainLoopSystem::AfterFixedMainLoop),
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
    position: IVec2,
    velocity: Velocity,
    rotation: TargetRotation,
    radar: Radar,
    script: Script,
    debug: bool,
}

impl StarterShip {
    pub fn builder(script: Script) -> ShipBuilder {
        ShipBuilder::new(script)
    }
}

// Builder to make building a starter ship nicer
pub struct ShipBuilder {
    position: IVec2,
    velocity: Velocity,
    rotation: TargetRotation,
    radar: Radar,
    script: Script,
    debug: bool,
}

impl ShipBuilder {
    pub fn new(script: Script) -> ShipBuilder {
        ShipBuilder {
            position: IVec2::new(0, 0),
            velocity: Velocity {
                velocity: IVec2::new(0, 0),
                acceleration: 0,
                velocity_limit: 100,
            },
            rotation: TargetRotation {
                limit: 16,
                target: AbsRot(0),
            },
            radar: Radar {
                current: AbsRot(0),
                target: AbsRot(0),
                offset: AbsRot(0).to_quat(),
                current_arc: 64,
                target_arc: 64,
            },
            script,
            debug: false,
        }
    }

    // Settings
    pub fn position(mut self, x: i32, y: i32) -> ShipBuilder {
        self.position = IVec2::new(x, y);
        self
    }

    pub fn velocity(mut self, x: i32, y: i32) -> ShipBuilder {
        self.velocity.velocity = IVec2::new(x, y);
        self
    }

    pub fn acceleration(mut self, acceleration: i32) -> ShipBuilder {
        self.velocity.acceleration = acceleration;
        self
    }

    pub fn velocity_limit(mut self, limit: u32) -> ShipBuilder {
        self.velocity.velocity_limit = limit;
        self
    }

    pub fn rotation(mut self, rotation: AbsRot) -> ShipBuilder {
        self.rotation.target = rotation;
        // Default target radar same direction as the ship
        self.radar.current = rotation;
        self.radar.target = rotation;
        self
    }

    pub fn rotation_limit(mut self, limit: u8) -> ShipBuilder {
        self.rotation.limit = limit;
        self
    }

    pub fn radar(mut self, rotation: AbsRot) -> ShipBuilder {
        self.radar.current = rotation;
        self.radar.target = rotation;
        self.radar.offset = rotation.to_quat();
        self
    }

    pub fn radar_arc(mut self, arc: u8) -> ShipBuilder {
        self.radar.current_arc = arc;
        self.radar.target_arc = arc;
        self
    }

    pub fn debug(mut self, debug: bool) -> ShipBuilder {
        self.debug = debug;
        self
    }

    pub fn script(mut self, script: Script) -> ShipBuilder {
        self.script = script;
        self
    }

    // TODO: can we do it as a ref so that we can make multiple ships quickly
    pub fn build(self) -> StarterShip {
        StarterShip {
            position: self.position,
            velocity: self.velocity,
            rotation: self.rotation,
            radar: self.radar,
            script: self.script,
            debug: self.debug,
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

        let radar_target = ship.radar.target;
        let ship_target = ship.rotation.target;
        let mut transform = Transform::from_translation(vec_scale(ship.position, ARENA_SCALE).extend(0.));
        transform.rotate(ship_target.to_quat());

        let mut spawned_ship = commands.spawn((
            ShapeBuilder::with(&ship_path)
                .fill(Fill::color(bevy::color::palettes::css::GREEN))
                .stroke(Stroke::new(bevy::color::palettes::css::BLACK, 2.0))
                .build(),
            transform
        ));

        spawned_ship
            .insert(Ship)
            .insert(ship.velocity)
            .insert(ship.rotation)
            .insert(ship.script)

            // Simulation components
            .insert(movement::Position(ship.position))
            .insert(movement::PreviousPosition(ship.position))

            .insert(rotation::Rotation(ship_target))
            .insert(rotation::PreviousRotation(ship_target))

            // TODO: probs want collision groups (ie ship vs missile vs other ships)
            .insert(Collider::cuboid(10.0, 20.0))
            .insert(ActiveCollisionTypes::empty() | ActiveCollisionTypes::STATIC_STATIC)
            .insert(ActiveEvents::COLLISION_EVENTS)
            .insert(Sensor)

            .insert(Collision(0))

            // Insert the graphics for the radar dish
            .with_children(|parent| {
                let mut transform = Transform::from_translation(Vec2::new(0., -2.).extend(1.));
                // TODO: this is probs wrong and needs to be fixed
                transform.rotate(radar_target.to_quat());

                let mut spawned_radar = parent.spawn((
                    ShapeBuilder::with(&radar_path)
                        .stroke(Stroke::new(bevy::color::palettes::css::MAROON, 1.5))
                        .build(),
                    transform
                ));
                spawned_radar.insert(ship.radar);

                if ship.debug {
                    spawned_radar.insert(RadarDebug);
                }
            });

        if ship.debug {
            spawned_ship
                .insert(MovDebug)
                .insert(RotDebug);
        }
    }
}
