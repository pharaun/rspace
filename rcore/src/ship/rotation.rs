use bevy::prelude::*;

#[derive(Component)]
pub struct Rotation {
    pub limit: f32, // Per Second?
    pub target: Quat,
}

pub(crate) fn apply_rotation(
    time: Res<Time<Fixed>>,
    mut query: Query<(&Rotation, &mut Transform, Option<&mut RotDebug>)>
) {
    for (rot, mut tran, debug) in query.iter_mut() {
        // Get current rotation vector, get the target rotation vector, do math, and then rotate
        let current = tran.rotation;
        let target = rot.target;
        let limit = Quat::from_rotation_z(rot.limit);

        // DEBUG
        match debug {
            Some(mut dbg) => {
                dbg.rotation_current = current.to_euler(EulerRot::ZYX).0;
                dbg.rotation_target = target.to_euler(EulerRot::ZYX).0;
                dbg.rotation_limit = limit.to_euler(EulerRot::ZYX).0;
            },
            None => (),
        }

        // If this is aproximately zero we are on our heading, bail
        if (current.dot(target) - 1.).abs() < f32::EPSILON {
            continue;
        }

        // Calculate the t-factor for the rotation.lerp
        let max_angle = limit.to_euler(EulerRot::ZYX).0 * time.delta_secs();
        let angle = current.angle_between(target);
        let t = (1_f32).min(max_angle / angle);

        tran.rotation = tran.rotation.lerp(target, t);
    }
}


#[derive(Component)]
pub struct RotDebug {
    pub rotation_current: f32,
    pub rotation_target: f32,
    pub rotation_limit: f32,
}

pub(crate) fn debug_rotation_gitzmos(
    mut gizmos: Gizmos,
    query: Query<(&Transform, &RotDebug)>
) {
    for (tran, debug) in query.iter() {
        let base = tran.translation.truncate();

        let current = Quat::from_rotation_z(debug.rotation_current);
        let target = Quat::from_rotation_z(debug.rotation_target);
        let limit = Quat::from_rotation_z(debug.rotation_limit);

        gizmos.line_2d(
            base,
            base + current.mul_vec3(Vec3::Y * 90.).truncate(),
            bevy::color::palettes::css::RED,
        );
        gizmos.arc_2d(
            Isometry2d::new(
                base,
                Rot2::radians(
                    (current.to_euler(EulerRot::ZYX).0 * -1.) + (current.angle_between(current*limit) * 2.) * 0.5
                )
            ),
            current.angle_between(current*limit) * 2.,
            80.,
            bevy::color::palettes::css::RED,
        );
        gizmos.line_2d(
            base,
            base + limit.mul_vec3(current.mul_vec3(Vec3::Y * 85.)).truncate(),
            bevy::color::palettes::css::RED,
        );
        gizmos.line_2d(
            base,
            base + limit.inverse().mul_vec3(current.mul_vec3(Vec3::Y * 85.)).truncate(),
            bevy::color::palettes::css::RED,
        );

        gizmos.line_2d(
            base,
            base + target.mul_vec3(Vec3::Y * 80.).truncate(),
            bevy::color::palettes::css::GREEN,
        );
        gizmos.arc_2d(
            Isometry2d::new(
                base,
                Rot2::radians(
                    (current.lerp(target, 0.5).to_euler(EulerRot::ZYX).0 * -1.) + current.angle_between(target) * 0.5
                )
            ),
            current.angle_between(target),
            70.,
            bevy::color::palettes::css::GREEN,
        );
    }
}
