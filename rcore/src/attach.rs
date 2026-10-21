use avian2d::prelude::PhysicsSystems;
use avian2d::prelude::Position;
use avian2d::prelude::Rotation;
use bevy::prelude::*;

use crate::GamePhase;
use crate::rotation::Heading;

// Translation Attachment subsystem.
// - This is for rotation-less "transform hierarchy"
// - Aka things like turrets, radar, shield that wants to attach to a ship but preserve
//   their own rotation.
pub struct AttachPlugin;
impl Plugin for AttachPlugin {
    fn build(&self, app: &mut App) {
        // This must be ran after avian dumps `Position` into `Transform`
        app.add_systems(
            FixedPostUpdate,
            propagate_attachment_translation
                .after(PhysicsSystems::Writeback)
                .run_if(in_state(GamePhase::Run)),
        )
        .add_systems(
            PostUpdate,
            (init_body_transform, init_attachment_transform)
                .chain()
                .before(TransformSystems::Propagate),
        );
    }
}

#[derive(Component)]
#[relationship(relationship_target = Attachments)]
pub struct AttachedTo(pub Entity);

// Despawn attachments when the parent entity despawns
#[derive(Component)]
#[relationship_target(relationship = AttachedTo, linked_spawn)]
pub struct Attachments(Vec<Entity>);

// Render only component (handling offset from ship center)
#[derive(Component, Clone, Copy)]
#[require(Transform)]
pub struct AttachOffset(pub Vec3);

fn propagate_attachment_translation(
    mut query: Query<(&AttachedTo, &AttachOffset, &mut Transform)>,
    parent_query: Query<&Transform, Without<AttachedTo>>,
) {
    for (attached_to, offset, mut transform) in query.iter_mut() {
        if let Ok(parent_transform) = parent_query.get(attached_to.0) {
            transform.translation = parent_transform.translation + offset.0;
        }
    }
}

// TODO: not 100% for sure if this belongs here, I think this may belong in movement.rs
fn init_body_transform(mut query: Query<(&Position, &Rotation, &mut Transform), Added<Position>>) {
    for (pos, rot, mut transform) in &mut query {
        transform.translation = pos.0.as_vec2().extend(transform.translation.z);
        transform.rotation = Quat::from(*rot);
    }
}

fn init_attachment_transform(
    mut query: Query<(&AttachedTo, &AttachOffset, &Heading, &mut Transform), Added<AttachOffset>>,
    parent_query: Query<&Position, Without<AttachedTo>>,
) {
    for (attached_to, offset, heading, mut transform) in &mut query {
        if let Ok(parent_pos) = parent_query.get(attached_to.0) {
            transform.translation = parent_pos.0.as_vec2().extend(0.) + offset.0;
        }
        transform.rotation = heading.0.to_quat();
    }
}
