use bevy::prelude::*;

use crate::ship::Radar;
use crate::math::AbsRot;

pub struct RotationPlugin;
impl Plugin for RotationPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(FixedPreUpdate, (
                init_rotation,
            ))
            .add_systems(FixedUpdate, (
                apply_rotation,
            ))
            .add_systems(RunFixedMainLoop, (
                interpolate_rotation.in_set(RunFixedMainLoopSystem::AfterFixedMainLoop),
            ))
            .add_systems(Update, (
                debug_rotation_gitzmos,
            ));
    }
}

#[derive(Component)]
#[require(Rotation)]
pub struct TargetRotation {
    pub limit: u8, // Per Second?
    pub target: AbsRot,
}

#[derive(Component, Default)]
#[require(PreviousRotation)]
pub struct Rotation(pub AbsRot);

#[derive(Component, Default)]
pub struct PreviousRotation(pub AbsRot);

#[derive(Component)]
pub struct RotDebug;

// Handles rendering
// Lifted from: https://github.com/Jondolf/bevy_transform_interpolation/tree/main
// Consider: https://github.com/Jondolf/bevy_transform_interpolation/blob/main/src/hermite.rs
// - Since we do have velocity information so we should be able to do better interpolation
pub(crate) fn interpolate_rotation(
    mut query: Query<(&mut Transform, &Rotation, &PreviousRotation, &Children), Without<Radar>>,
    mut radar_query : Query<(&Radar, &mut Transform)>,
    fixed_time: Res<Time<Fixed>>
) {
    // How much of a "partial timestep" has accumulated since the last fixed timestep run.
    // Between `0.0` and `1.0`.
    let overstep = fixed_time.overstep_fraction();

    for (mut transform, rotation, previous_rotation, children) in &mut query {
        // Note: `slerp` will always take the shortest path, but when the two rotations are more than
        // 180 degrees apart, this can cause visual artifacts as the rotation "flips" to the other side.
        transform.rotation = previous_rotation.0.transform_slerp(rotation.0, overstep);

        // Grab the child radar and undo the ship rotation
        // TODO: Find a better way to deal with this
        for child_entity in children {
            if let Ok((radar, mut rtran)) = radar_query.get_mut(*child_entity) {
                rtran.rotation = transform.rotation.inverse() * radar.offset;
            }
        }
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

// So that the first frame is correct, pre-populate the PreviousRotation with the Rotation
// upon that component being inserted
pub(crate) fn init_rotation(
    mut query: Query<(&Rotation, &mut PreviousRotation), Added<Rotation>>,
) {
    for (rotation, mut previous_rotation) in query.iter_mut() {
        previous_rotation.0 = rotation.0;
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
