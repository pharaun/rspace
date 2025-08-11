use bevy::prelude::*;

use crate::FixedGameSystem;
use crate::math::AbsRot;

pub struct RotationPlugin;
impl Plugin for RotationPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(FixedUpdate, (
                apply_rotation.in_set(FixedGameSystem::GameLogic),
            ))
            .add_systems(RunFixedMainLoop, (
                interpolate_rotation.in_set(RunFixedMainLoopSystem::AfterFixedMainLoop),
            ))
            .add_systems(Update, (
                debug_rotation_gitzmos,
            ))
            .add_systems(PostUpdate, (
                disable_rotation_propagation.after(TransformSystem::TransformPropagate),
            ));
    }
}

#[derive(Bundle, Clone)]
pub struct RotationBundle {
    pub target: TargetRotation,
    pub rotation: Rotation,
    pub previous: PreviousRotation,
}

impl RotationBundle {
    pub fn new(rotation: AbsRot, target: AbsRot, limit: u8) -> RotationBundle {
        RotationBundle {
            target: TargetRotation {
                limit,
                target,
            },
            rotation: Rotation(rotation),
            previous: PreviousRotation(rotation),
        }
    }

    pub fn rotation(&mut self, rotation: AbsRot) {
        self.target.target = rotation;
        self.rotation.0 = rotation;
        self.previous.0 = rotation;
    }
}

#[derive(Component, Clone, Copy)]
#[require(Rotation)]
pub struct TargetRotation {
    pub limit: u8, // Per Second?
    pub target: AbsRot,
}

#[derive(Component, Default, Clone, Copy)]
#[require(PreviousRotation)]
pub struct Rotation(pub AbsRot);

#[derive(Component, Default, Clone, Copy)]
pub struct PreviousRotation(pub AbsRot);

#[derive(Component, Clone)]
pub struct NoRotationPropagation;

#[derive(Component, Clone, Copy)]
pub struct RotDebug;

// Handles rendering
// Lifted from: https://github.com/Jondolf/bevy_transform_interpolation/tree/main
// Consider: https://github.com/Jondolf/bevy_transform_interpolation/blob/main/src/hermite.rs
// - Since we do have velocity information so we should be able to do better interpolation
pub(crate) fn interpolate_rotation(
    mut query: Query<(&mut Transform, &Rotation, &PreviousRotation)>,
    fixed_time: Res<Time<Fixed>>
) {
    // How much of a "partial timestep" has accumulated since the last fixed timestep run.
    // Between `0.0` and `1.0`.
    let overstep = fixed_time.overstep_fraction();

    for (mut transform, rotation, previous_rotation) in &mut query {
        // Note: `slerp` will always take the shortest path, but when the two rotations are more than
        // 180 degrees apart, this can cause visual artifacts as the rotation "flips" to the other side.
        transform.rotation = previous_rotation.0.transform_slerp(rotation.0, overstep);
    }
}

pub(crate) fn apply_rotation(
    time: Res<Time<Fixed>>,
    mut query: Query<(&TargetRotation, &mut Rotation, &mut PreviousRotation)>
) {
    for (target_rot, mut rotation, mut previous_rotation) in query.iter_mut() {
        previous_rotation.0 = rotation.0;

        // If rotation is the same as the target rotation, bail
        if rotation.0 == target_rot.target {
            continue;
        }

        // Clamp the rotation
        let limit = target_rot.limit as f32 * time.delta_secs();
        let angle = rotation.0.angle_between(target_rot.target).clamp(limit.round() as u8);
        rotation.0 += angle;
    }
}

pub(crate) fn disable_rotation_propagation(
    query: Query<(&Children, &Transform)>,
    mut child_query: Query<(&Transform, &mut GlobalTransform), With<NoRotationPropagation>>,
) {
    for (childrens, parent_tran) in query.iter() {
        for child_entity in childrens.iter() {
            if let Ok((child_tran, mut child_global_tran)) = child_query.get_mut(child_entity) {
                *child_global_tran = child_global_tran
                    .compute_transform()
                    .with_rotation(child_tran.rotation)
                    .with_translation(parent_tran.translation + child_tran.translation)
                    .into();
            }
        }
    }
}

pub(crate) fn debug_rotation_gitzmos(
    mut gizmos: Gizmos,
    query: Query<(&Transform, &TargetRotation), With<RotDebug>>
) {
    for (tran, target) in query.iter() {
        let base = tran.translation.truncate();
        let heading = tran.rotation;
        let qtarget = target.target.to_quat();

        let cw_limit  = heading * AbsRot(target.limit).to_quat();
        let ccw_limit = heading * AbsRot(255 - target.limit).to_quat();

        // Current heading
        gizmos.line_2d(
            base + heading.mul_vec3(Vec3::Y * 70.).truncate(),
            base + heading.mul_vec3(Vec3::Y * 100.).truncate(),
            bevy::color::palettes::css::RED,
        );

        // Target heading
        gizmos.line_2d(
            base + qtarget.mul_vec3(Vec3::Y * 70.).truncate(),
            base + qtarget.mul_vec3(Vec3::Y * 90.).truncate(),
            bevy::color::palettes::css::GREEN,
        );
        gizmos.short_arc_2d_between(
            base,
            base + heading.mul_vec3(Vec3::Y * 80.).truncate(),
            base + qtarget.mul_vec3(Vec3::Y * 80.).truncate(),
            bevy::color::palettes::css::GREEN,
        );

        // Limit + Arcs for rotation direction
        gizmos.line_2d(
            base + cw_limit.mul_vec3(Vec3::Y * 70.).truncate(),
            base + cw_limit.mul_vec3(Vec3::Y * 80.).truncate(),
            bevy::color::palettes::css::YELLOW,
        );
        gizmos.short_arc_2d_between(
            base,
            base + heading.mul_vec3(Vec3::Y * 70.).truncate(),
            base + cw_limit.mul_vec3(Vec3::Y * 70.).truncate(),
            bevy::color::palettes::css::YELLOW,
        );
        gizmos.line_2d(
            base + ccw_limit.mul_vec3(Vec3::Y * 70.).truncate(),
            base + ccw_limit.mul_vec3(Vec3::Y * 80.).truncate(),
            bevy::color::palettes::css::YELLOW,
        );
        gizmos.short_arc_2d_between(
            base,
            base + heading.mul_vec3(Vec3::Y * 70.).truncate(),
            base + ccw_limit.mul_vec3(Vec3::Y * 70.).truncate(),
            bevy::color::palettes::css::YELLOW,
        );
    }
}
