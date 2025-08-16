use std::time::Duration;

use bevy::prelude::*;

use crate::FixedGameSystem;

use crate::movement::Position;
use crate::ARENA_SCALE;

use crate::ship::ShipBuilder;
use crate::script::Script;
use crate::rotation::Rotation;
use crate::spawner::SpawnEvent;
use crate::rotation::NoRotationPropagation;
use crate::radar::Arc;
use crate::radar::ArcCheck;
use crate::radar::within_arc;

use crate::AbsRot;


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
                debug_shield_health_gitzmos,
            ));
    }
}

// Health and armor system for ships
//
// When a ship is hit with a weapon, this is when this system comes in play
#[derive(Component, Debug, Clone, Copy, Default)]
pub struct Health {
    pub current: u16,
    pub maximum: u16,
}

#[derive(Component, Clone, Copy)]
pub struct HealthDebug;

// 0 - Origin of the damage (for shield coverage check)
// 1 - health to deduce
#[derive(Event, Copy, Clone, Debug)]
pub struct DamageEvent (pub IVec2, pub u16);

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

// TODO: add logic to query for shield on the ship, and check
// if the shield covers where the damage is coming from, and then if so,
// apply the shield damage reduce, pass it on to the ship health, and deduce the rest
// from the shield health pool, once shield health pool is zero, then just pass full
// damage through
pub fn process_damage_event(
    trigger: Trigger<DamageEvent>,
    mut commands: Commands,
    mut query: Query<(&mut Health, &Position, &Children), Without<Shield>>,
    mut shield_query: Query<(&mut Health, &Shield, &Arc)>,
) {
    let ship = trigger.target();
    if let Ok((mut health, ship_pos, children)) = query.get_mut(ship) {
        let mut ship_damage: u16 = trigger.event().1;

        // Scan through the children to find the shield if there is one.
        // TODO: support multiple shield, for now assume one.
        for child in children.iter() {
            if let Ok((mut shield_health, shield, arc)) = shield_query.get_mut(child) {
                // Check if the shield is not at 0 health
                if shield_health.current == 0 {
                    // Pass on full damage
                    ship_damage = trigger.event().1;
                    break;
                }

                // Check if the damage source is covered by the shield arc
                match within_arc(ship_pos.0, trigger.event().0, arc.current, arc.current_arc) {
                    ArcCheck::InsideArc => {
                        // Split incoming damage into shield and ship damage
                        let shield_damage: u16 = (trigger.event().1 as f32 * shield.damage_reduce).round() as u16;

                        // If shield can't cover full shield damage, deduce and pass on to ship
                        if let Some(new_shield_health) = shield_health.current.checked_sub(shield_damage) {
                            shield_health.current = new_shield_health;
                            ship_damage = trigger.event().1 - shield_damage;
                        } else {
                            // Can't cover full damage, deduce what we can and pass it on
                            let carry_shield_damage = shield_damage - shield_health.current;
                            shield_health.current = 0;
                            ship_damage = trigger.event().1 - shield_damage + carry_shield_damage;
                        }
                    },
                    ArcCheck::OutsideArc => {
                        // Pass on full damage
                        ship_damage = trigger.event().1;
                    },
                    ArcCheck::SamePosition => {
                        // Print warning & pass on full damage
                        println!("Warning self-damaging? - {:?}", ship_pos.0);
                        ship_damage = trigger.event().1;
                    }
                }

                // It matched a shield, we are done with processing
                break;
            }
        }

        if let Some(new_health) = health.current.checked_sub(ship_damage) {
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
    mut query: Query<(&mut DebugWeapon, &Position)>,
    position: Query<&Transform>,
) {
    for FireDebugWeaponEvent(ship, target) in fire_debug_weapon_events.read() {
        if let Ok((mut weapon, ship_pos)) = query.get_mut(*ship) {
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
                commands.trigger_targets(DamageEvent(ship_pos.0, weapon.damage), target.clone());
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
                    commands.trigger_targets(DamageEvent(base_position.0, warhead.damage), target_ship);
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

fn render_bar_gizmos(
    gizmos: &mut Gizmos,
    position: Vec2,
    width: f32,
    percentage_full: f32,
    bar_color: Srgba,
) {
    let bar_offset = (width * percentage_full) - (width / 2.);

    // Primitive bar-graph in gizmo form
    for v_off in 1..10 {
        gizmos.line_2d(
            position + Vec2::new(-(width / 2.), 5. - v_off as f32),
            position + Vec2::new(bar_offset, 5. - v_off as f32),
            bar_color,
        );
    }
    gizmos.rect_2d(
        Isometry2d::from_translation(position),
        Vec2::new(width, 10.),
        bevy::color::palettes::css::RED,
    );
}

pub(crate) fn debug_health_gitzmos(
    mut gizmos: Gizmos,
    query: Query<(&Health, &Transform), (With<HealthDebug>, Without<Shield>)>,
) {
    for (health, tran) in query.iter() {
        let base = tran.translation.truncate();

        render_bar_gizmos(
            &mut gizmos,
            base + Vec2::new(0., -25.),
            35.,
            health.current as f32 / health.maximum as f32,
            bevy::color::palettes::css::GREEN,
        );
    }
}

pub(crate) fn debug_shield_health_gitzmos(
    mut gizmos: Gizmos,
    query: Query<(&Health, &ChildOf), (With<ShieldHealthDebug>, With<Shield>)>,
    ship_query: Query<&Transform>,
) {
    for (health, child_of) in query.iter() {
        let base = ship_query.get(child_of.parent()).unwrap().translation.truncate();

        render_bar_gizmos(
            &mut gizmos,
            base + Vec2::new(0., -35.),
            35.,
            health.current as f32 / health.maximum as f32,
            bevy::color::palettes::css::BLUE,
        );
    }
}

// TODO: figure out collision detection with shields for misiles and other ships but
// for now skip
#[derive(Bundle, Clone)]
pub struct ShieldBundle {
    pub arc: Arc,
    pub shield: Shield,
    pub health: Health,
    pub noprop: NoRotationPropagation,
}

impl ShieldBundle {
    pub fn new(
        current: AbsRot,
        target: AbsRot,
        current_arc: u8,
        target_arc: u8,
        damage_reduce: f32,
        health: u16,
    ) -> ShieldBundle {
        ShieldBundle {
            arc: Arc {
                current,
                target,
                current_arc,
                target_arc,
            },
            shield: Shield {
                damage_reduce,
            },
            health: Health {
                current: health,
                maximum: health,
            },
            noprop: NoRotationPropagation,
        }
    }

    pub fn rotation(&mut self, rotation: AbsRot) {
        self.arc.current = rotation;
        self.arc.target = rotation;
    }

    pub fn arc(&mut self, arc: u8) {
        self.arc.current_arc = arc;
        self.arc.target_arc = arc;
    }

    pub fn damage_reduce(&mut self, damage_reduce: f32) {
        self.shield.damage_reduce = damage_reduce;
    }

    pub fn health(&mut self, health: u16) {
        self.health.current = health;
        self.health.maximum = health;
    }
}

#[derive(Component, Clone, Copy)]
#[require(Arc)]
#[require(Health)]
pub struct Shield {
    damage_reduce: f32,
}

#[derive(Component, Clone, Copy)]
pub struct ShieldHealthDebug;
