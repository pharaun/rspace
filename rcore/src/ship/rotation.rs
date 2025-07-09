use bevy::prelude::*;
use std::f32::consts::PI;

const FRAC_PI_128: f32 = PI / 128.0;

// Fixed rotation: 0 = 0, 1 = π/128, 64 = π/2, 128 = π/1, ...
#[derive(Component)]
pub struct FixedRotation(pub u8);

fn fixed_to_quat(fixed: u8) -> Quat {
    Quat::from_rotation_z(FRAC_PI_128 * fixed as f32)
}


fn quat_to_fixed(quat: Quat) -> u8 {
    quat.to_euler(EulerRot::ZYX).0;
    1
}




#[derive(Component)]
pub struct Rotation {
    pub limit: f32, // Per Second?
    pub target: Quat,
}

// TODO: separate the debug stuff out to its own component/system
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

        // TODO: probs want slerp here not lerp
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


// Hacky test to at least verify the fixed quat math, you shouldn't compare floats directly
#[test]
fn test_fixed_to_quat() {
    assert_eq!(Quat::from_rotation_z(0.), fixed_to_quat(0));
    assert_eq!(Quat::from_rotation_z(PI/128.), fixed_to_quat(1));
    assert_eq!(Quat::from_rotation_z(PI/64.), fixed_to_quat(2));
    assert_eq!(Quat::from_rotation_z(PI/32.), fixed_to_quat(4));
    assert_eq!(Quat::from_rotation_z(PI/16.), fixed_to_quat(8));
    assert_eq!(Quat::from_rotation_z(PI/8.), fixed_to_quat(16));
    assert_eq!(Quat::from_rotation_z(PI/4.), fixed_to_quat(32));
    assert_eq!(Quat::from_rotation_z(PI/2.), fixed_to_quat(64));
    assert_eq!(Quat::from_rotation_z(PI), fixed_to_quat(128));
    assert_eq!(Quat::from_rotation_z(PI + PI/2.), fixed_to_quat(192));
    assert_eq!(Quat::from_rotation_z(PI + PI - PI/128.), fixed_to_quat(255));
}
