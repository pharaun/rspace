use std::time::Duration;

use bevy::prelude::*;

use crate::health::DamageEvent;
use crate::movement::Position;
use crate::arena::ARENA_SCALE;

use crate::ship::ShipBuilder;
use crate::script::Script;
use crate::rotation::Rotation;
use crate::spawner::SpawnEvent;


// TODO: dynamic warhead distance, for now fixed
const DISTANCE: i32 = 500;
const DISTANCE_SQUARED: i32 = DISTANCE.pow(2);

pub struct WeaponPlugin;
impl Plugin for WeaponPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<FireDebugWeaponEvent>()
            .add_event::<FireDebugWarheadEvent>()
            .add_event::<FireDebugMissileEvent>()
            .add_systems(FixedUpdate, (
                apply_debug_weapon_cooldown,
                apply_debug_missile_cooldown,
            ))
            .add_systems(RunFixedMainLoop, (
                render_debug_weapon.in_set(RunFixedMainLoopSystem::AfterFixedMainLoop),
                render_debug_warhead.in_set(RunFixedMainLoopSystem::AfterFixedMainLoop),
            ))
            .add_systems(Update, (
                process_fire_debug_weapon_event,
                process_fire_debug_missile_event,
                process_fire_debug_warhead_event,
            ));
    }
}

// Basic 360 no scope test weapon, it can zap anything when told to fire
#[derive(Component, Clone)]
pub struct DebugWeapon {
    // Ticks for weapon cooldown - May want to consider timer, but that's based off physical timing
    // Need to figure out how to have a "global" tick-tock to have tick-tock timing for the
    // simulator so that later we can "speed up the simulator to the max that the cpu can do for
    // doing repative testing/tournment/etc without a render.
    pub cooldown: u16,
    pub current: u16,
    pub damage: u16,
}

#[derive(Component, Clone)]
pub struct DebugMissile {
    // Ticks for weapon cooldown - May want to consider timer, but that's based off physical timing
    // Need to figure out how to have a "global" tick-tock to have tick-tock timing for the
    // simulator so that later we can "speed up the simulator to the max that the cpu can do for
    // doing repative testing/tournment/etc without a render.
    pub cooldown: u16,
    pub current: u16,
}

#[derive(Component, Clone)]
pub struct DebugWarhead {
    pub damage: u16,
}

// New entity + component for rendering the weapon then it fades away
#[derive(Component)]
pub struct RenderDebugWeapon {
    pub origin: Vec2,
    pub target: Vec2,

    // Persist for this amount of time
    pub fade: Timer,
}

// New entity + component for rendering the weapon then it fades away
#[derive(Component)]
pub struct RenderDebugWarhead {
    pub origin: Vec2,

    // Persist for this amount of time
    pub fade: Timer,
}

// Weapon Firing event,
// TODO: probs want to look at some other option but for now we can use an event to fire
// the weapon
// 0 - self, 1 - target
#[derive(Event, Copy, Clone, Debug)]
pub struct FireDebugWeaponEvent (pub Entity, pub Entity);

// Just blow up the missile upon trigger
// 0 - self
#[derive(Event, Copy, Clone, Debug)]
pub struct FireDebugWarheadEvent (pub Entity);

// Fire the missile
// 0 - self
#[derive(Event, Copy, Clone, Debug)]
pub struct FireDebugMissileEvent (pub Entity);


pub(crate) fn apply_debug_weapon_cooldown(
    mut query: Query<&mut DebugWeapon>
) {
    for mut weapon in &mut query {
        weapon.current = weapon.current.saturating_sub(1);
    }
}

pub(crate) fn apply_debug_missile_cooldown(
    mut query: Query<&mut DebugMissile>
) {
    for mut weapon in &mut query {
        weapon.current = weapon.current.saturating_sub(1);
    }
}

pub(crate) fn render_debug_weapon(
    mut gizmos: Gizmos,
    mut commands: Commands,
    mut query: Query<(Entity, &mut RenderDebugWeapon)>,
    time: Res<Time>,
) {
    for (entity, mut render) in query.iter_mut() {
        render.fade.tick(time.delta());

        // TODO: render the beam thicker
        gizmos.line_2d(
            render.origin,
            render.target,
            bevy::color::palettes::css::RED,
        );

        // Check if fade has expired?
        // if so, despawn
        if render.fade.finished() {
            commands.entity(entity).despawn();
        }
    }
}

pub(crate) fn render_debug_warhead(
    mut gizmos: Gizmos,
    mut commands: Commands,
    mut query: Query<(Entity, &mut RenderDebugWarhead)>,
    time: Res<Time>,
) {
    for (entity, mut render) in query.iter_mut() {
        render.fade.tick(time.delta());

        // TODO: render the beam thicker
        gizmos.circle_2d(
            Isometry2d::from_translation(render.origin),
            DISTANCE as f32 / ARENA_SCALE,
            bevy::color::palettes::css::RED,
        );

        // Check if fade has expired?
        // if so, despawn
        if render.fade.finished() {
            commands.entity(entity).despawn();
        }
    }
}

pub fn process_fire_debug_weapon_event(
    mut commands: Commands,
    mut fire_debug_weapon_events: EventReader<FireDebugWeaponEvent>,
    mut events: EventWriter<DamageEvent>,
    mut query: Query<&mut DebugWeapon>,
    position: Query<&Transform>,
) {
    for FireDebugWeaponEvent(ship, target) in fire_debug_weapon_events.read() {
        if let Ok(mut weapon) = query.get_mut(*ship) {
            if weapon.current == 0 {
                weapon.current = weapon.cooldown;

                // Fetch the ship & target position
                let [ship_tran, target_tran] = position.get_many([*ship, *target]).unwrap();

                // Setup the weapon render
                commands.spawn(RenderDebugWeapon {
                    origin: ship_tran.translation.truncate(),
                    target: target_tran.translation.truncate(),
                    fade: Timer::new(Duration::from_secs_f32(5.), TimerMode::Once),
                });

                // emit damage event to the target
                events.write(DamageEvent(*target, weapon.damage));
            }
        }
    }
}

pub fn process_fire_debug_warhead_event(
    mut commands: Commands,
    mut fire_debug_warhead_events: EventReader<FireDebugWarheadEvent>,
    mut events: EventWriter<DamageEvent>,
    have_warhead: Query<&DebugWarhead>,
    render_position: Query<&Transform>,
    position: Query<(Entity, &Position)>,
) {
    for FireDebugWarheadEvent(ship) in fire_debug_warhead_events.read() {
        // does this ship (self) have a warhead component?
        if let Ok(warhead) = have_warhead.get(*ship) {
            // Fetch the ship position
            let ship_tran = render_position.get(*ship).unwrap();

            // Setup the weapon render
            commands.spawn(RenderDebugWarhead {
                origin: ship_tran.translation.truncate(),
                fade: Timer::new(Duration::from_secs_f32(1.), TimerMode::Once),
            });

            // Find target in radius and then emit damage to each target within radius
            let (base_ship, base_position) = position.get(*ship).unwrap();
            for (target_ship, target_position) in position.iter() {
                if base_ship == target_ship {
                    continue;
                }

                if base_position.0.distance_squared(target_position.0) < DISTANCE_SQUARED {
                    events.write(DamageEvent(target_ship, warhead.damage));
                }
            }

            // Warhead blew up, remove self
            commands.entity(*ship).despawn();
        }
    }
}


// TODO: for now hardcore various things, but we need to pass in the script to the missile
// That or yeet the script from parent ship and copy it over
pub fn process_fire_debug_missile_event(
    mut fire_debug_missile_events: EventReader<FireDebugMissileEvent>,
    mut parent_missile: Query<&mut DebugMissile>,
    parent_ship: Query<(&Position, &Rotation, &Script)>,
    mut spawn_ship: EventWriter<SpawnEvent>,
) {
    for FireDebugMissileEvent(ship) in fire_debug_missile_events.read() {
        // 1. does this have a missile component if so, check if we can fire
        if let Ok(mut weapon) = parent_missile.get_mut(*ship) {
            if weapon.current == 0 {
                weapon.current = weapon.cooldown;

                // 2. if yes, spawn a ship next to the parent ship
                // 3. for now yeet the script from the parent ship onto this
                let (pos, rot, parent_script) = parent_ship.get(*ship).unwrap();

                // Calculate the position of the future missile
                let offset = pos.0 + rot.0.to_quat().mul_vec3(Vec3::Y * 400.).truncate().as_ivec2();

                // 4. send it on its merry way
                let missile = ShipBuilder::new(parent_script.clone())
                    .position(offset.x, offset.y)
                    .rotation(rot.0)
                    .velocity(0, 0)
                    .radar_arc(32)
                    .warhead(100)
                    .build();

                spawn_ship.write(SpawnEvent(missile));
            }
        }
    }
}
