use bevy::prelude::*;
use bevy_rapier2d::prelude::*;
use bevy_prototype_lyon::prelude::Shape;

pub struct CollisionPlugin;
impl Plugin for CollisionPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (
                process_collision_event,
                apply_collision.after(process_collision_event),
            ));
    }
}
// Ref-counted collision, if greater than zero, its colloding, otherwise
#[derive(Component)]
pub struct Collision(pub u32);

pub(crate) fn apply_collision(mut query: Query<(&Collision, &mut Shape)>) {
    for (collision, shape) in query.iter_mut() {
        if collision.0 == 0 {
            shape.fill.unwrap().color = bevy::prelude::Color::Srgba(bevy::color::palettes::css::GREEN);
        } else {
            shape.fill.unwrap().color = bevy::prelude::Color::Srgba(bevy::color::palettes::css::RED);
        }
    }
}

pub(crate) fn process_collision_event(
    mut collision_events: EventReader<CollisionEvent>,
    mut query: Query<&mut Collision>,
) {
    for collision_event in collision_events.read() {
        match collision_event {
            //struct Collision(u32);
            CollisionEvent::Started(e1, e2, _) => {
                if let Ok([mut e1_collision, mut e2_collision]) = query.get_many_mut([*e1, *e2]) {
                    e1_collision.0 += 1;
                    e2_collision.0 += 1;
                } else {
                    println!("ERROR - ECS - {:?}", collision_event);
                }
            },
            CollisionEvent::Stopped(e1, e2, _) => {
                if let Ok([mut e1_collision, mut e2_collision]) = query.get_many_mut([*e1, *e2]) {
                    e1_collision.0 -= 1;
                    e2_collision.0 -= 1;
                } else {
                    println!("ERROR - ECS - {:?}", collision_event);
                }
            },
        }
    }
}
