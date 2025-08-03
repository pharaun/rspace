use std::time::Duration;

use bevy::prelude::*;

use crate::ship::health::DamageEvent;

pub struct WeaponPlugin;
impl Plugin for WeaponPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<FireDebugWeaponEvent>()
            .add_systems(FixedUpdate, (
                apply_debug_weapon_cooldown,
            ))
            .add_systems(RunFixedMainLoop, (
                render_debug_weapon.in_set(RunFixedMainLoopSystem::AfterFixedMainLoop),
            ))
            .add_systems(Update, (
                process_fire_debug_weapon_event,
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

// New entity + component for rendering the weapon then it fades away
#[derive(Component)]
pub struct RenderDebugWeapon {
    pub origin: Vec2,
    pub target: Vec2,

    // Persist for this amount of time
    pub fade: Timer,
}

// Weapon Firing event,
// TODO: probs want to look at some other option but for now we can use an event to fire
// the weapon
// 0 - self, 1 - target
#[derive(Event, Copy, Clone, Debug)]
pub struct FireDebugWeaponEvent (pub Entity, pub Entity);

pub(crate) fn apply_debug_weapon_cooldown(
    mut query: Query<&mut DebugWeapon>
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

pub fn process_fire_debug_weapon_event(
    mut commands: Commands,
    mut fire_debug_weapon_events: EventReader<FireDebugWeaponEvent>,
    mut events: EventWriter<DamageEvent>,
    mut query: Query<&mut DebugWeapon>,
    position: Query<&Transform>,
) {
    for FireDebugWeaponEvent(ship, target) in fire_debug_weapon_events.read() {
        let mut weapon = query.get_mut(*ship).unwrap();

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
