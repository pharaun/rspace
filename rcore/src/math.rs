use std::f32::consts::PI;
use std::ops::AddAssign;
use std::ops::Add;

use bevy::prelude::EulerRot;
use bevy::prelude::Quat;

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
pub struct AbsRot(pub u8);

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

    // TODO: Hack flipped the sign, need to figure out what we want semantics wise first
    pub fn angle_between(&self, target: AbsRot) -> RelRot {
        RelRot(-(self.0 as i16 - target.0 as i16) as i8)
    }
}

impl Add<RelRot> for AbsRot {
    type Output = AbsRot;

    fn add(self, rhs: RelRot) -> AbsRot {
        if rhs.0 < 0 {
            AbsRot(self.0.wrapping_sub((-(rhs.0 as i16)) as u8))
        } else {
            AbsRot(self.0.wrapping_add(rhs.0 as u8))
        }
    }
}

impl AddAssign<RelRot> for AbsRot {
    fn add_assign(&mut self, rhs: RelRot) {
        *self = *self + rhs
    }
}

// Relative Rotation: (Used for heading rotations)
// 0 = Direct ahead
// -64 = 90º Left
//  64 = 90º Right
// Clamped: [-128, 128)
#[derive(Debug, PartialEq, Clone, Copy)]
pub struct RelRot(pub i8);

impl RelRot {
    pub fn clamp(&self, clamp: u8) -> RelRot {
        if clamp >= 128 {
            *self
        } else if self.0 < -(clamp as i8) {
            RelRot(-(clamp as i8))
        } else if self.0 > clamp as i8 {
            RelRot(clamp as i8)
        } else {
            *self
        }
    }
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

// TODO: I think we need to identify/do hand-ness conversion/checks here because yeah
#[test]
fn test_angle_between() {
    assert_eq!(AbsRot(0).angle_between(AbsRot(1)), RelRot(-1));
    assert_eq!(AbsRot(0).angle_between(AbsRot(255)), RelRot(1));
}

#[test]
fn test_clamp() {
    assert_eq!(RelRot(0).clamp(1), RelRot(0));
    assert_eq!(RelRot(2).clamp(1), RelRot(1));
    assert_eq!(RelRot(-2).clamp(1), RelRot(-1));
}

#[test]
fn test_add_rel_to_abs() {
    assert_eq!(AbsRot(0) + RelRot(0), AbsRot(0));
    assert_eq!(AbsRot(0) + RelRot(1), AbsRot(1));
    assert_eq!(AbsRot(0) + RelRot(-1), AbsRot(255));
}


