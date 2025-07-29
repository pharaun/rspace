use bevy::prelude::*;

// Basic 360 no scope test weapon, it can zap anything when told to fire
#[derive(Component)]
pub struct DebugWeapon {
    // Ticks for weapon cooldown
    pub cooldown: u16,
    pub current: u16,
    pub damage: u16,
}

pub(crate) fn apply_debug_weapon_cooldown(
    time: Res<Time<Fixed>>,
    mut query: Query<&mut DebugWeapon>
) {
    // TODO: tick down the weapon cooldown till its ready
}

// New entity + component for rendering the weapon then it fades away
#[derive(Component)]
pub struct RenderDebugWeapon {
    pub origin: Vec2,
    pub target: Vec2,

    // Persist for this amount of seconds
    pub fade: f32,
}

pub(crate) fn render_debug_weapon(
    mut commands: Commands,
    mut query: Query<&RenderDebugWeapon>,
    time: Res<Time>,
) {
    // TODO: render the weapon beam for fade amount of time, and once fade expire, despawn
}

// Weapon Firing event,
// TODO: probs want to look at some other option but for now we can use an event to fire
// the weapon
// 0 - self, 1 - target
#[derive(Event, Copy, Clone, Debug)]
pub struct FireDebugWeaponEvent (pub Entity, pub Entity);

pub fn process_fire_debug_weapon_event(
    mut commands: Commands,
    mut fire_debug_weapon_events: EventReader<FireDebugWeaponEvent>,
    mut query: Query<&mut DebugWeapon>,
) {
    for FireDebugWeaponEvent(ship, target) in fire_debug_weapon_events.read() {
        // TODO: check if its safe to fire the weapon
        // 1. cooldown
        // 2. spawn the RenderDebugWeapon component+entity
        // 3. emit damage event to the target
    }
}
