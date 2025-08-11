use bevy::prelude::*;

// TODO: convert this to observers
pub struct HealthPlugin;
impl Plugin for HealthPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<DamageEvent>()
            .add_observer(process_damage_event)
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
