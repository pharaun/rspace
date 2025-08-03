use bevy::prelude::*;

pub struct HealthPlugin;
impl Plugin for HealthPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<DamageEvent>()
            .add_systems(Update, (
                process_damage_event,
                debug_health_gitzmos,
            ));
    }
}

// Health and armor system for ships
//
// When a ship is hit with a weapon, this is when this system comes in play
#[derive(Component, Debug)]
pub struct Health {
    pub current: u16,
    pub maximum: u16,
}

#[derive(Component)]
pub struct HealthDebug;

// 0 - Entity being damaged, 1 - health to deduce
#[derive(Event, Copy, Clone, Debug)]
pub struct DamageEvent (pub Entity, pub u16);

pub fn process_damage_event(
    mut commands: Commands,
    mut damage_events: EventReader<DamageEvent>,
    mut query: Query<&mut Health>,
) {
    for DamageEvent(ship, damage) in damage_events.read() {
        if let Ok(mut health) = query.get_mut(*ship) {
            if let Some(new_health) = health.current.checked_sub(*damage) {
                health.current = new_health;
            } else {
                // This ship is now dead, despawn it
                println!("Despawning - {:?}", ship);
                commands.entity(*ship).despawn();
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
