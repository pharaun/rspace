use avian2d::prelude::PhysicsSystems;
use bevy::prelude::*;

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
            propagate_attachment_translation.after(PhysicsSystems::Writeback),
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
