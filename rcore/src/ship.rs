use bevy::prelude::*;
use bevy_prototype_lyon::prelude::*;

use avian2d::prelude::*;

use crate::math::AbsRot;
use crate::script::Script;
use crate::math::vec_scale;

use crate::ARENA;
use crate::ARENA_SCALE;

use crate::movement::MovementBundle;
use crate::movement::MovDebug;

use crate::rotation::RotationBundle;
use crate::rotation::RotDebug;

use crate::class::ShipClass;
use crate::class::get_ship;
use crate::class::get_radar;

use crate::radar::RadarDebug;
use crate::radar::ArcDebug;
use crate::radar::RadarBundle;

use crate::weapon::Health;
use crate::weapon::HealthDebug;
use crate::weapon::DebugWeapon;
use crate::weapon::DebugMissile;
use crate::weapon::DebugWarhead;


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

// TODO:
// - Way to load a scene (which sets up where each ships are and any other obstance or resources in
// the gameworld)
// - Way to refer each ship to an AI script
// - Possibly an way to customize the starting ship (via the AI script or some other config for
// each ship)
// - Dig into ECS archtype to help with some of these setup stuff
#[derive(Clone)]
pub struct StarterShip {
    movement: MovementBundle,
    rotation: RotationBundle,
    radar: RadarBundle,
    health: Health,
    warhead: Option<DebugWarhead>,
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
    movement: MovementBundle,
    rotation: RotationBundle,
    radar: RadarBundle,
    health: Health,
    warhead: Option<DebugWarhead>,
    script: Script,
    debug: DebugShip,
}

// Look into impl Bundler
// https://discord.com/channels/691052431525675048/1403836135045726339/1403837230111522917
// It'll permit us to have a "bundle builder" that builds a ship
impl ShipBuilder {
    pub fn new(script: Script) -> ShipBuilder {
        // TODO: setup so that most of these components have default() or something so that
        // they can be more self-contained without having to build them up here in the builder
        ShipBuilder {
            movement: MovementBundle::new(
                IVec2::new(0, 0),
                IVec2::new(0, 0),
                100,
                0,
            ),
            rotation: RotationBundle::new(
                AbsRot(0),
                AbsRot(0),
                16,
            ),
            radar: RadarBundle::new(
                AbsRot(0),
                AbsRot(0),
                64,
                64,
            ),
            health: Health {
                current: 100,
                maximum: 100,
            },
            warhead: None,
            script,
            debug: DebugShip::new(),
        }
    }

    // Settings
    pub fn position(mut self, x: i32, y: i32) -> ShipBuilder {
        self.movement.position(x, y);
        // Warn if its outside arena bounds since it will then warp the next frame
        if (y < -(ARENA.y / 2)) || (y > (ARENA.y / 2)) || (x < -(ARENA.x / 2)) || (x > (ARENA.x / 2)) {
            println!("WARNING: Set position outside of arena bounds - x: {:?}, y: {:?}", x, y);
        }
        self
    }

    pub fn velocity(mut self, x: i32, y: i32) -> ShipBuilder {
        self.movement.velocity.velocity = IVec2::new(x, y);
        self
    }

    pub fn acceleration(mut self, acceleration: i32) -> ShipBuilder {
        self.movement.velocity.acceleration = acceleration;
        self
    }

    pub fn velocity_limit(mut self, limit: u32) -> ShipBuilder {
        self.movement.velocity.velocity_limit = limit;
        self
    }

    pub fn rotation(mut self, rotation: AbsRot) -> ShipBuilder {
        self.rotation.rotation(rotation);
        // Target radar in same direction as the ship
        self.radar.rotation(rotation);
        self
    }

    pub fn rotation_limit(mut self, limit: u8) -> ShipBuilder {
        self.rotation.target.limit = limit;
        self
    }

    pub fn health(mut self, health: u16) -> ShipBuilder {
        self.health.current = health;
        self.health.maximum = health;
        self
    }

    pub fn radar(mut self, rotation: AbsRot) -> ShipBuilder {
        self.radar.rotation(rotation);
        self
    }

    pub fn radar_arc(mut self, arc: u8) -> ShipBuilder {
        self.radar.arc(arc);
        self
    }

    pub fn warhead(mut self, damage: u16) -> ShipBuilder {
        self.warhead = Some(DebugWarhead {
            damage
        });
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
            movement: self.movement,
            rotation: self.rotation,
            radar: self.radar,
            health: self.health,
            warhead: self.warhead,
            script: self.script,
            debug: self.debug,
        }
    }
}

// TODO: For components that are empty (ie tags) can use component ids + insert them from a null ptr
// This will allow for a list of component ids to make it easier to add/set debug bits on a ship
// optionally
#[derive(Clone)]
pub struct DebugShip {
    radar_debug: Option<RadarDebug>,
    arc_debug: Option<ArcDebug>,
    mov_debug: Option<MovDebug>,
    rot_debug: Option<RotDebug>,
    health_debug: Option<HealthDebug>,
}

impl DebugShip {
    pub fn new() -> DebugShip {
        DebugShip {
            radar_debug: None,
            arc_debug: None,
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
    arc_debug: Option<ArcDebug>,
    mov_debug: Option<MovDebug>,
    rot_debug: Option<RotDebug>,
    health_debug: Option<HealthDebug>,
}

impl DebugBuilder {
    pub fn new() -> DebugBuilder {
        DebugBuilder {
            radar_debug: None,
            arc_debug: None,
            mov_debug: None,
            rot_debug: None,
            health_debug: None,
        }
    }

    pub fn radar(mut self) -> DebugBuilder {
        self.radar_debug = Some(RadarDebug);
        self
    }

    pub fn arc(mut self) -> DebugBuilder {
        self.arc_debug = Some(ArcDebug);
        self
    }

    pub fn movement(mut self) -> DebugBuilder {
        self.mov_debug = Some(MovDebug);
        self
    }

    pub fn rotation(mut self) -> DebugBuilder {
        self.rot_debug = Some(RotDebug);
        self
    }

    pub fn health(mut self) -> DebugBuilder {
        self.health_debug = Some(HealthDebug);
        self
    }

    pub fn build(self) -> DebugShip {
        DebugShip {
            radar_debug: self.radar_debug,
            arc_debug: self.arc_debug,
            mov_debug: self.mov_debug,
            rot_debug: self.rot_debug,
            health_debug: self.health_debug,
        }
    }
}

pub fn add_ship(
    commands: &mut Commands,
    ship: StarterShip
) {
    let radar_target = ship.radar.arc.target;
    let ship_target = ship.rotation.target.target;
    let mut transform = Transform::from_translation(vec_scale(ship.movement.position.0, ARENA_SCALE).extend(0.));
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
        .insert(ship.movement)
        .insert(ship.rotation)

        // Health
        .insert(ship.health)

        // TODO: probs want collision groups (ie ship vs missile vs other ships)
        .insert(Collider::circle(15.0))
        .insert(CollisionEventsEnabled)

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

            if let Some(arc) = ship.debug.arc_debug {
                spawned_radar.insert(arc);
            }
        });

    // Weapons
    if ship.warhead.is_none() {
        spawned_ship
            .insert(DebugWeapon { cooldown: 10, current: 0, damage: 34 })
            .insert(DebugMissile { cooldown: 10, current: 0 });
    } else {
        spawned_ship.insert(ship.warhead.unwrap());
    }

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
