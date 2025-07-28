use bevy::prelude::*;

// Health and armor system for ships
//
// When a ship is hit with a weapon, this is when this system comes in play

#[derive(Component, Debug)]
pub struct Health {
    pub health: u16,
    pub max_health: u16,
}

// 0 - Entity being damaged, 1 - health to deduce
#[derive(Event, Copy, Clone, Debug)]
pub struct DamageEvent (pub Entity, pub u16);

pub fn process_damage_event(
    mut damage_events: EventReader<DamageEvent>,
    mut query: Query<&mut Health>,
) {
    for damage_event in damage_events.read() {

    }
}

#[derive(Component)]
pub struct HealthDebug;

pub(crate) fn debug_health_gitzmos(
    mut gizmos: Gizmos,
    query: Query<&Health>,
) {
}
