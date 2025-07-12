use bevy::prelude::*;
use std::f32::consts::PI;

#[derive(Component)]
pub struct FixedRotation(AbsRot);

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


// Stepped Rotation: inspiration bevy::math::Rot2 - Which is clamped to the range (-π, π]
const FRAC_PI_128: f32 = PI / 128.0;

// Absolute Rotation:
// 0   =   0º North
// 64  =  90º East
// 128 = 180º South
// 192 = 270º West
// Radian: 0 = 0, 1 = π/128, 64 = π/2, 128 = π/1, ...
#[derive(Debug, PartialEq)]
pub struct AbsRot(u8);

impl AbsRot {
    pub fn to_quat(&self) -> Quat {
        Quat::from_rotation_z(FRAC_PI_128 * self.0 as f32)
    }

    pub fn from_quat(quat: Quat) -> Self {
        let tmp = {
            let tmp = quat.to_euler(EulerRot::ZYX).0 / FRAC_PI_128;
            if tmp < 0.0 {
                tmp + 256.
            } else {
                tmp
            }
        };
        println!("Float: {:?}, Floor: {:?}, Rounding: {:?} Ceiling: {:?}", tmp, tmp.floor(), tmp.round(), tmp.ceil());
        AbsRot(tmp.round() as u8)
    }
}

// Relative Rotation: (Used for heading rotations)
// 0 = Direct ahead
// -64 = 90º Left
//  64 = 90º Right
// Clamped: [-128, 128)
#[derive(Debug, PartialEq)]
pub struct RelRot(i8);

impl RelRot {
}

// Hacky test to at least verify the fixed quat math, you shouldn't compare floats directly
#[test]
fn test_to_quat() {
    assert_eq!(Quat::from_rotation_z(0.),      AbsRot(0).to_quat());
    assert_eq!(Quat::from_rotation_z(PI/128.), AbsRot(1).to_quat());
    assert_eq!(Quat::from_rotation_z(PI/64.),  AbsRot(2).to_quat());
    assert_eq!(Quat::from_rotation_z(PI/32.),  AbsRot(4).to_quat());
    assert_eq!(Quat::from_rotation_z(PI/16.),  AbsRot(8).to_quat());
    assert_eq!(Quat::from_rotation_z(PI/8.),   AbsRot(16).to_quat());
    assert_eq!(Quat::from_rotation_z(PI/4.),   AbsRot(32).to_quat());
    assert_eq!(Quat::from_rotation_z(PI/2.),   AbsRot(64).to_quat());
    assert_eq!(Quat::from_rotation_z(PI),      AbsRot(128).to_quat());
    assert_eq!(Quat::from_rotation_z(PI + PI/2.), AbsRot(192).to_quat());
    assert_eq!(Quat::from_rotation_z(PI + PI - PI/128.), AbsRot(255).to_quat());
}

#[test]
fn test_from_quat() {
    assert_eq!(AbsRot::from_quat(Quat::from_rotation_z(0.)),      AbsRot(0));
    assert_eq!(AbsRot::from_quat(Quat::from_rotation_z(PI/128.)), AbsRot(1));
    assert_eq!(AbsRot::from_quat(Quat::from_rotation_z(PI/64.)),  AbsRot(2));
    assert_eq!(AbsRot::from_quat(Quat::from_rotation_z(PI/32.)),  AbsRot(4));
    assert_eq!(AbsRot::from_quat(Quat::from_rotation_z(PI/16.)),  AbsRot(8));
    assert_eq!(AbsRot::from_quat(Quat::from_rotation_z(PI/8.)),   AbsRot(16));
    assert_eq!(AbsRot::from_quat(Quat::from_rotation_z(PI/4.)),   AbsRot(32));
    assert_eq!(AbsRot::from_quat(Quat::from_rotation_z(PI/2.)),   AbsRot(64));
    assert_eq!(AbsRot::from_quat(Quat::from_rotation_z(PI)),      AbsRot(128));
    assert_eq!(AbsRot::from_quat(Quat::from_rotation_z(PI + PI/2.)), AbsRot(192));
    assert_eq!(AbsRot::from_quat(Quat::from_rotation_z(PI + PI - PI/128.)), AbsRot(255));
}
