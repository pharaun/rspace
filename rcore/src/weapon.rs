use std::time::Duration;

use bevy::prelude::*;

use crate::FixedGameSystem;

use crate::movement::Position;
use crate::ARENA_SCALE;

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
        app.add_event::<DamageEvent>()
            .add_observer(process_damage_event)
            .add_event::<FireDebugWeaponEvent>()
            .add_event::<FireDebugWarheadEvent>()
            .add_event::<FireDebugMissileEvent>()
            .add_systems(FixedUpdate, (
                apply_debug_weapon_cooldown.in_set(FixedGameSystem::GameLogic),
                process_fire_debug_weapon_event.in_set(FixedGameSystem::Weapon).after(apply_debug_weapon_cooldown),
            ))
            .add_systems(FixedUpdate, (
                apply_debug_missile_cooldown.in_set(FixedGameSystem::GameLogic),
                // Missile will spawn the next frame
                // TODO: do we want a post-shiplogic set -> missile -> spawn -> weapon sequencing
                process_fire_debug_missile_event.in_set(FixedGameSystem::Weapon).after(apply_debug_missile_cooldown),
            ))
            .add_systems(FixedUpdate, (
                process_fire_debug_warhead_event.in_set(FixedGameSystem::Weapon),
            ))
            .add_systems(RunFixedMainLoop, (
                render_debug_weapon.in_set(RunFixedMainLoopSystem::AfterFixedMainLoop),
                render_debug_warhead.in_set(RunFixedMainLoopSystem::AfterFixedMainLoop),
            ))
            .add_systems(Update, (
                debug_health_gitzmos,
            ));
    }
}

// Health and armor system for ships
//
// When a ship is hit with a weapon, this is when this system comes in play
#[derive(Component, Debug, Clone, Copy)]
pub struct Health {
    pub current: u16,
    pub maximum: u16,
}

#[derive(Component, Clone, Copy)]
pub struct HealthDebug;

// 1 - health to deduce
#[derive(Event, Copy, Clone, Debug)]
pub struct DamageEvent (pub u16);

// Basic 360 no scope test weapon, it can zap anything when told to fire
#[derive(Component, Clone)]
pub struct DebugWeapon {
    // TODO: split off the cooldown + current code to its own CoolDown component and treat it like
    // the radar/shield + Arc for that
    //
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
    // TODO: split off the cooldown + current code to its own CoolDown component and treat it like
    // the radar/shield + Arc for that
    //
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

pub fn process_damage_event(
    trigger: Trigger<DamageEvent>,
    mut commands: Commands,
    mut query: Query<&mut Health>,
) {
    let ship = trigger.target();
    if let Ok(mut health) = query.get_mut(ship) {
        if let Some(new_health) = health.current.checked_sub(trigger.event().0) {
            health.current = new_health;
        } else {
            // This ship is now dead, despawn it
            println!("Despawning - {:?}", ship);
            commands.entity(ship).despawn();
        }
    }
}

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
                commands.trigger_targets(DamageEvent(weapon.damage), target.clone());
            }
        }
    }
}

pub fn process_fire_debug_warhead_event(
    mut commands: Commands,
    mut fire_debug_warhead_events: EventReader<FireDebugWarheadEvent>,
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
                    commands.trigger_targets(DamageEvent(warhead.damage), target_ship);
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

pub(crate) fn debug_health_gitzmos(
    mut gizmos: Gizmos,
    query: Query<(&Health, &Transform), With<HealthDebug>>,
) {
    for (health, tran) in query.iter() {
        let base = tran.translation.truncate();

        // Health-line as a percentage
        let width: f32 = 35.;
        let health_bar = width * (health.current as f32 / health.maximum as f32);
        let health_offset = health_bar - (width / 2.);

        // Primitive bar-graph in gizmo form
        for v_off in 1..10 {
            gizmos.line_2d(
                base + Vec2::new(-(width / 2.), -20. - v_off as f32),
                base + Vec2::new(health_offset, -20. - v_off as f32),
                bevy::color::palettes::css::GREEN,
            );
        }
        gizmos.rect_2d(
            Isometry2d::from_translation(base + Vec2::new(0., -25.)),
            Vec2::new(width, 10.),
            bevy::color::palettes::css::RED,
        );
    }
}
