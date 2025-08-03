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
use crate::movement;
use crate::rotation;

// TODO: look into plugins + bundles to make this better
// because right now i'm having to add multiple things to this
// file for each new system/component
pub mod radar;
pub use crate::ship::radar::Radar;
use crate::ship::radar::apply_radar;
use crate::ship::radar::RadarDebug;
use crate::ship::radar::debug_radar_gitzmos;
use crate::ship::radar::ContactEvent;

pub mod collision;
use crate::ship::collision::Collision;
use crate::ship::collision::apply_collision;
use crate::ship::collision::process_collision_event;

pub mod health;
use crate::ship::health::Health;
use crate::ship::health::HealthDebug;
use crate::ship::health::process_damage_event;
use crate::ship::health::debug_health_gitzmos;
use crate::ship::health::DamageEvent;

pub mod debug_weapon;
use crate::ship::debug_weapon::DebugWeapon;
use crate::ship::debug_weapon::apply_debug_weapon_cooldown;
use crate::ship::debug_weapon::render_debug_weapon;
use crate::ship::debug_weapon::FireDebugWeaponEvent;
use crate::ship::debug_weapon::process_fire_debug_weapon_event;

pub mod class;
use crate::ship::class::ShipClass;
use crate::ship::class::get_ship;
use crate::ship::class::get_radar;

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
pub struct ShipPlugin;
impl Plugin for ShipPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(ShapePlugin)
            //.insert_resource(Time::<Fixed>::from_hz(64.0))
            .insert_resource(Time::<Fixed>::from_hz(2.0))
            .add_event::<ContactEvent>()
            .add_event::<DamageEvent>()
            .add_event::<FireDebugWeaponEvent>()
            .add_systems(FixedUpdate, (
                // TODO: apply radar rotation, then process radar_event
                apply_radar,
                apply_debug_weapon_cooldown,
            ))
            .add_systems(RunFixedMainLoop, (
                render_debug_weapon.in_set(RunFixedMainLoopSystem::AfterFixedMainLoop),
            ))
            .add_systems(Update, (
                debug_radar_gitzmos,
                debug_health_gitzmos,

                process_collision_event,
                apply_collision.after(process_collision_event),

                process_damage_event,
                process_fire_debug_weapon_event,
            ));
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
    velocity: movement::Velocity,
    rotation: rotation::TargetRotation,
    health: Health,
    radar: Radar,
    script: Script,
    debug: DebugShip,
}

impl StarterShip {
    pub fn builder(script: Script) -> ShipBuilder {
        ShipBuilder::new(script)
    }
}

// Builder to make building a starter ship nicer
pub struct ShipBuilder {
    position: IVec2,
    velocity: movement::Velocity,
    rotation: rotation::TargetRotation,
    health: Health,
    radar: Radar,
    script: Script,
    debug: DebugShip,
}

impl ShipBuilder {
    pub fn new(script: Script) -> ShipBuilder {
        // TODO: setup so that most of these components have default() or something so that
        // they can be more self-contained without having to build them up here in the builder
        ShipBuilder {
            position: IVec2::new(0, 0),
            velocity: movement::Velocity {
                velocity: IVec2::new(0, 0),
                acceleration: 0,
                velocity_limit: 100,
            },
            rotation: rotation::TargetRotation {
                limit: 16,
                target: AbsRot(0),
            },
            health: Health {
                current: 100,
                maximum: 100,
            },
            radar: Radar {
                current: AbsRot(0),
                target: AbsRot(0),
                offset: AbsRot(0).to_quat(),
                current_arc: 64,
                target_arc: 64,
            },
            script,
            debug: DebugShip::new(),
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

    pub fn health(mut self, health: u16) -> ShipBuilder {
        self.health.current = health;
        self.health.maximum = health;
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

    pub fn debug(mut self, debug: DebugShip) -> ShipBuilder {
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
            health: self.health,
            radar: self.radar,
            script: self.script,
            debug: self.debug,
        }
    }
}

// TODO: For components that are empty (ie tags) can use component ids + insert them from a null ptr
// This will allow for a list of component ids to make it easier to add/set debug bits on a ship
// optionally
pub struct DebugShip {
    radar_debug: Option<RadarDebug>,
    mov_debug: Option<movement::MovDebug>,
    rot_debug: Option<rotation::RotDebug>,
    health_debug: Option<HealthDebug>,
}

impl DebugShip {
    pub fn new() -> DebugShip {
        DebugShip {
            radar_debug: None,
            mov_debug: None,
            rot_debug: None,
            health_debug: None,
        }
    }

    pub fn builder() -> DebugBuilder {
        DebugBuilder::new()
    }
}

pub struct DebugBuilder {
    radar_debug: Option<RadarDebug>,
    mov_debug: Option<movement::MovDebug>,
    rot_debug: Option<rotation::RotDebug>,
    health_debug: Option<HealthDebug>,
}

impl DebugBuilder {
    pub fn new() -> DebugBuilder {
        DebugBuilder {
            radar_debug: None,
            mov_debug: None,
            rot_debug: None,
            health_debug: None,
        }
    }

    pub fn radar(mut self) -> DebugBuilder {
        self.radar_debug = Some(RadarDebug);
        self
    }

    pub fn movement(mut self) -> DebugBuilder {
        self.mov_debug = Some(movement::MovDebug);
        self
    }

    pub fn rotation(mut self) -> DebugBuilder {
        self.rot_debug = Some(rotation::RotDebug);
        self
    }

    pub fn health(mut self) -> DebugBuilder {
        self.health_debug = Some(HealthDebug);
        self
    }

    pub fn build(self) -> DebugShip {
        DebugShip {
            radar_debug: self.radar_debug,
            mov_debug: self.mov_debug,
            rot_debug: self.rot_debug,
            health_debug: self.health_debug,
        }
    }
}

pub fn add_ships(
    mut commands: Commands,
    ships: Vec<StarterShip>
) {
    for ship in ships {
        let radar_target = ship.radar.target;
        let ship_target = ship.rotation.target;
        let mut transform = Transform::from_translation(vec_scale(ship.position, ARENA_SCALE).extend(0.));
        transform.rotate(ship_target.to_quat());

        let mut spawned_ship = commands.spawn((
            get_ship(
                ShipClass::Medium,
                Fill::color(bevy::color::palettes::css::GREEN),
                Stroke::new(bevy::color::palettes::css::BLACK, 2.0),
            ),
            transform,
        ));

        spawned_ship
            .insert(Ship)
            .insert(ship.script)

            // Motion components
            .insert(movement::Position(ship.position))
            .insert(ship.velocity)

            .insert(rotation::Rotation(ship_target))
            .insert(ship.rotation)

            // Health and Damage components
            .insert(ship.health)
            .insert(DebugWeapon { cooldown: 10, current: 0, damage: 34 })

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
                    get_radar(Stroke::new(bevy::color::palettes::css::MAROON, 1.5)),
                    transform,
                    ship.radar,
                ));

                if let Some(radar) = ship.debug.radar_debug {
                    spawned_radar.insert(radar);
                }
            });

        // Debug components
        if let Some(mov) = ship.debug.mov_debug {
            spawned_ship.insert(mov);
        }
        if let Some(rot) = ship.debug.rot_debug {
            spawned_ship.insert(rot);
        }
        if let Some(health) = ship.debug.health_debug {
            spawned_ship.insert(health);
        }
    }
}
