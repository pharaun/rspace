use bevy::prelude::*;
use std::f32::consts::PI;

#[derive(Component)]
pub struct Rotation(pub AbsRot);

#[derive(Component)]
pub struct PreviousRotation(pub AbsRot);

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
        transform.rotation = previous_rotation.0.to_quat().slerp(rotation.0.to_quat(), overstep);
    }
}


#[derive(Component)]
pub struct TargetRotation {
    pub limit: u8, // Per Second?
    pub target: AbsRot,
}

// TODO: separate the debug stuff out to its own component/system
pub(crate) fn apply_rotation(
    time: Res<Time<Fixed>>,
    mut query: Query<(&TargetRotation, &mut Rotation, &mut PreviousRotation, Option<&mut RotDebug>)>
) {
    for (target_rot, mut rotation, mut previous_rotation, debug) in query.iter_mut() {
        // DEBUG
        match debug {
            Some(mut dbg) => {
                dbg.rotation_current = rotation.0.to_quat().to_euler(EulerRot::ZYX).0;
                dbg.rotation_target = target_rot.target.to_quat().to_euler(EulerRot::ZYX).0;
                // TODO: redo these
                dbg.rotation_limit = target_rot.target.to_quat().to_euler(EulerRot::ZYX).0;
            },
            None => (),
        }

        previous_rotation.0 = rotation.0;

        // If rotation is the same as the target rotation, bail
        if rotation.0 == target_rot.target {
            continue;
        }

        // TODO: redo this logic
        // Calculate the t-factor for the rotation.lerp
        let max_angle = target_rot.limit as f32 * time.delta_secs();
        let angle = rotation.0.angle_between(target_rot.target);
        let t = (1_f32).min(max_angle / angle as f32);

        // TODO: probs want to slerp here not lerp
        rotation.0 = rotation.0.lerp(target_rot.target, t);
    }
}

// TODO: move all of the Abs & Rel rotation stuff to its own game-math-lib spot
// Stepped Rotation: inspiration bevy::math::Rot2 - Which is clamped to the range (-π, π]
const FRAC_PI_128: f32 = PI / 128.0;

// Absolute Rotation:
// 0   =   0º North
// 64  =  90º East
// 128 = 180º South
// 192 = 270º West
// Radian: 0 = 0, 1 = π/128, 64 = π/2, 128 = π/1, ...
#[derive(Debug, PartialEq, Clone, Copy)]
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
        AbsRot(tmp.round() as u8)
    }

    // TODO: redo to use relrot and all of those logic
    pub fn angle_between(&self, target: AbsRot) -> u8 {
        if self.0 < target.0 {
            target.0 - self.0
        } else {
            self.0 - target.0
        }
    }

    // TODO: don't need lerp on absrot? somewhere else?
    pub fn lerp(&self, target: AbsRot, t: f32) -> AbsRot {
        AbsRot::from_quat(self.to_quat().lerp(target.to_quat(), t))
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
