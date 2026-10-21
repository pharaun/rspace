use avian2d::prelude::PhysicsSystems;
use avian2d::prelude::Position;
use avian2d::prelude::Rotation;
use bevy::prelude::*;

use crate::rotation::Heading;
use crate::ship::AttachOffset;
use crate::ship::AttachedTo;

// Sim -> Transform sync (one way)
// - Transform(f32) is strictly for rendering
// - Position(f64) is for Simulation
pub(super) struct TransformSyncPlugin;
impl Plugin for TransformSyncPlugin {
    fn build(&self, app: &mut App) {
        // Run after Avian writes back the transform + before interpolation of transform
        app.add_systems(
            FixedPostUpdate,
            (
                propagate_heading_transform,
                propagate_attachment_translation,
            )
                .after(PhysicsSystems::Writeback),
        )
        .add_systems(
            PostUpdate,
            (init_body_transform, init_attachment_transform)
                .chain()
                .before(TransformSystems::Propagate),
        );
    }
}

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

// Arc modules (radar/shield) don't have Avian Rotation, Update render transform
// directly for these. (The others are handled by Avian).
#[expect(clippy::type_complexity)]
fn propagate_heading_transform(
    mut query: Query<(&Heading, &mut Transform), (Changed<Heading>, Without<Rotation>)>,
) {
    for (heading, mut transform) in &mut query {
        transform.rotation = heading.0.to_quat();
    }
}

// Rotation-less "transform hierarchy" for attachments
fn propagate_attachment_translation(
    mut query: Query<(&AttachedTo, &AttachOffset, &mut Transform)>,
    parent_query: Query<&Transform, Without<AttachedTo>>,
) {
    for (attached_to, offset, mut transform) in &mut query {
        if let Ok(parent_transform) = parent_query.get(attached_to.0) {
            transform.translation = parent_transform.translation + offset.0;
        }
    }
}
