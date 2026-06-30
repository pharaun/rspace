use std::f32::consts::PI;
use std::ops::Add;
use std::ops::AddAssign;

use bevy::prelude::EulerRot;
use bevy::prelude::IVec2;
use bevy::prelude::Quat;
use bevy::prelude::Vec2;

pub fn vec_scale(vec: IVec2, factor: f32) -> Vec2 {
    Vec2::new(vec.x as f32 / factor, vec.y as f32 / factor)
}

#[expect(clippy::cast_possible_truncation)]
pub fn un_vec_scale(vec: Vec2, factor: f32) -> IVec2 {
    IVec2::new((vec.x * factor) as i32, (vec.y * factor) as i32)
}

// TODO: figure out better math stuff for integer angle math and stuff:
// https://stackoverflow.com/questions/77480605/nextion-calculate-inverse-tan-arctan-without-trig-functions-or-floating-point
// https://github.com/ddribin/trigint
//
// Would like to avoid floating point math/rotation/etc as much as possible to allow for integer
// angles, and integer position. But for now this is good enough.

// Stepped Rotation: inspiration bevy::math::Rot2 - Which is clamped to the range (-π, π]
const FRAC_PI_128: f32 = PI / -128.0;

// Absolute Rotation:
// 0   =   0º North
// 64  =  90º East
// 128 = 180º South
// 192 = 270º West
// Radian: 0 = 0, 1 = π/128, 64 = π/2, 128 = π/1, ...
#[derive(Debug, PartialEq, Eq, Clone, Copy, Default)]
pub struct AbsRot(pub u8);

impl AbsRot {
    pub fn to_quat(&self) -> Quat {
        Quat::from_rotation_z(FRAC_PI_128 * f32::from(self.0))
    }

    pub fn from_quat(quat: Quat) -> Self {
        Self::from_angle(quat.to_euler(EulerRot::ZYX).0)
    }

    pub fn from_angle(angle: f32) -> Self {
        let tmp = {
            let tmp = angle / FRAC_PI_128;
            if tmp < 0.0 { tmp + 256. } else { tmp }
        };
        #[expect(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
        Self(tmp.round() as u8)
    }

    pub fn from_vec2_angle(base: IVec2, target: IVec2) -> Option<Self> {
        if (target - base) == IVec2::ZERO {
            None
        } else {
            // IVec2(X, Y) (+Y = 0, +X = 64, -Y = 128, -X = 192)
            Some(Self::from_angle(
                Vec2::Y.angle_to((target - base).as_vec2()),
            ))
        }
    }

    // To support an arc-width of 1, we decided to make it so that an arc is
    // equiv to AbsRot(x) +- 0.5 arc width, ie dead-on is going to return x.
    //
    // half_arc = 0 -> `self` +- 0.5
    // half_arc = 1 -> `self-1 ..= self+1`
    //
    // Clamping to 127 max, half_arcs leads it to be able to check 255/256 arcs
    // TODO: may want to consider clamping to 64 for gameplay reason (128 arc)
    //
    // TODO: this is tricky cuz we give it AbsRot so it is already discretized
    // where we might want to instead give it a quat/convert it so that it can
    // handle the +- math correctly.
    pub fn within(&self, half_arc: u8, target: Self) -> bool {
        // unsigned_abs to avoid the (-128i8).abs() overflow
        i16::from(self.angle_between(target).0).unsigned_abs() <= u16::from(half_arc.min(127))
    }

    // Debugging math
    #[expect(clippy::cast_possible_wrap)]
    #[must_use]
    pub fn cw_edge(&self, half_arc: u8) -> Self {
        *self + RelRot(half_arc.min(127) as i8)
    }

    #[expect(clippy::cast_possible_wrap)]
    #[must_use]
    pub fn ccw_edge(&self, half_arc: u8) -> Self {
        *self + RelRot(-(half_arc.min(127) as i8))
    }

    #[expect(clippy::cast_possible_truncation)]
    pub fn angle_between(&self, target: Self) -> RelRot {
        // [-128, 127] so it defaults to CCW here due to cast wrap
        RelRot((i16::from(target.0) - i16::from(self.0)) as i8)
    }

    pub fn transform_slerp(&self, end: Self, f: f32) -> Quat {
        self.to_quat().slerp(end.to_quat(), f)
    }
}

impl Add<RelRot> for AbsRot {
    type Output = Self;

    #[expect(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    fn add(self, rhs: RelRot) -> Self {
        if rhs.0 < 0 {
            Self(self.0.wrapping_sub((-i16::from(rhs.0)) as u8))
        } else {
            Self(self.0.wrapping_add(rhs.0 as u8))
        }
    }
}

impl AddAssign<RelRot> for AbsRot {
    fn add_assign(&mut self, rhs: RelRot) {
        *self = *self + rhs;
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
    #[must_use]
    #[expect(clippy::cast_possible_wrap)]
    pub fn clamp(&self, clamp: u8) -> Self {
        if clamp >= 128 {
            *self
        } else if self.0 < -(clamp as i8) {
            Self(-(clamp as i8))
        } else if self.0 > clamp as i8 {
            Self(clamp as i8)
        } else {
            *self
        }
    }
}

#[rustfmt::skip]
#[test]
fn test_to_quat() {
    // Hacky test to at least verify the fixed quat math, you shouldn't compare floats directly
    assert_eq!(Quat::from_rotation_z(0.),       AbsRot(0).to_quat());
    assert_eq!(Quat::from_rotation_z(-PI/128.), AbsRot(1).to_quat());
    assert_eq!(Quat::from_rotation_z(-PI/64.),  AbsRot(2).to_quat());
    assert_eq!(Quat::from_rotation_z(-PI/32.),  AbsRot(4).to_quat());
    assert_eq!(Quat::from_rotation_z(-PI/16.),  AbsRot(8).to_quat());
    assert_eq!(Quat::from_rotation_z(-PI/8.),   AbsRot(16).to_quat());
    assert_eq!(Quat::from_rotation_z(-PI/4.),   AbsRot(32).to_quat());
    assert_eq!(Quat::from_rotation_z(-PI/2.),   AbsRot(64).to_quat());
    assert_eq!(Quat::from_rotation_z(-PI),      AbsRot(128).to_quat());
    assert_eq!(Quat::from_rotation_z(-PI + -PI/2.), AbsRot(192).to_quat());
    assert_eq!(Quat::from_rotation_z(-PI + -PI + PI/128.), AbsRot(255).to_quat());
}

#[rustfmt::skip]
#[test]
fn test_from_quat() {
    assert_eq!(AbsRot::from_quat(Quat::from_rotation_z(0.)),       AbsRot(0));

    // CW rotations
    assert_eq!(AbsRot::from_quat(Quat::from_rotation_z(-PI/128.)), AbsRot(1));
    assert_eq!(AbsRot::from_quat(Quat::from_rotation_z(-PI/64.)),  AbsRot(2));
    assert_eq!(AbsRot::from_quat(Quat::from_rotation_z(-PI/32.)),  AbsRot(4));
    assert_eq!(AbsRot::from_quat(Quat::from_rotation_z(-PI/16.)),  AbsRot(8));
    assert_eq!(AbsRot::from_quat(Quat::from_rotation_z(-PI/8.)),   AbsRot(16));
    assert_eq!(AbsRot::from_quat(Quat::from_rotation_z(-PI/4.)),   AbsRot(32));
    assert_eq!(AbsRot::from_quat(Quat::from_rotation_z(-PI/2.)),   AbsRot(64));
    assert_eq!(AbsRot::from_quat(Quat::from_rotation_z(-PI)),      AbsRot(128));
    assert_eq!(AbsRot::from_quat(Quat::from_rotation_z(-PI + -PI/2.)), AbsRot(192));
    assert_eq!(AbsRot::from_quat(Quat::from_rotation_z(-PI + -PI + PI/128.)), AbsRot(255));

    // CCW rotation
    assert_eq!(AbsRot::from_quat(Quat::from_rotation_z(PI/128.)), AbsRot((256 - 1) as u8));
    assert_eq!(AbsRot::from_quat(Quat::from_rotation_z(PI/64.)),  AbsRot((256 - 2) as u8));
    assert_eq!(AbsRot::from_quat(Quat::from_rotation_z(PI/32.)),  AbsRot((256 - 4) as u8));
    assert_eq!(AbsRot::from_quat(Quat::from_rotation_z(PI/16.)),  AbsRot((256 - 8) as u8));
    assert_eq!(AbsRot::from_quat(Quat::from_rotation_z(PI/8.)),   AbsRot((256 - 16) as u8));
    assert_eq!(AbsRot::from_quat(Quat::from_rotation_z(PI/4.)),   AbsRot((256 - 32) as u8));
    assert_eq!(AbsRot::from_quat(Quat::from_rotation_z(PI/2.)),   AbsRot((256 - 64) as u8));
    assert_eq!(AbsRot::from_quat(Quat::from_rotation_z(PI)),      AbsRot((256 - 128) as u8));
    assert_eq!(AbsRot::from_quat(Quat::from_rotation_z(PI + PI/2.)), AbsRot((256 - 192) as u8));
    assert_eq!(AbsRot::from_quat(Quat::from_rotation_z(PI + PI - PI/128.)), AbsRot((256 - 255) as u8));
}

#[test]
fn test_angle_between() {
    assert_eq!(AbsRot(0).angle_between(AbsRot(1)), RelRot(1));
    assert_eq!(AbsRot(0).angle_between(AbsRot(255)), RelRot(-1));
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

#[rustfmt::skip]
#[test]
fn test_from_vec2_angle() {
    assert_eq!(AbsRot::from_vec2_angle(IVec2::new(0, 0), IVec2::new(0, 0)), None);

    // IVec2(X, Y) (+Y = 0, +X = 64, -Y = 128, -X = 192)
    assert_eq!(AbsRot::from_vec2_angle(IVec2::new(0, 0), IVec2::new(0, 1)), Some(AbsRot(0)));
    assert_eq!(AbsRot::from_vec2_angle(IVec2::new(0, 0), IVec2::new(1, 0)), Some(AbsRot(64)));
    assert_eq!(AbsRot::from_vec2_angle(IVec2::new(0, 0), IVec2::new(0, -1)), Some(AbsRot(128)));
    assert_eq!(AbsRot::from_vec2_angle(IVec2::new(0, 0), IVec2::new(-1, 0)), Some(AbsRot(192)));

    assert_eq!(AbsRot::from_vec2_angle(IVec2::new(0, 0), IVec2::new(1, 1)), Some(AbsRot(32)));
    assert_eq!(AbsRot::from_vec2_angle(IVec2::new(0, 0), IVec2::new(-1, -1)), Some(AbsRot(160)));
}

#[rustfmt::skip]
#[test]
fn test_within() {
    // Test an 0 arc-width
    assert_eq!(false, AbsRot(0).within(0, AbsRot(255)));
    assert_eq!(true,  AbsRot(0).within(0, AbsRot(0)));
    assert_eq!(false, AbsRot(0).within(0, AbsRot(1)));

    assert_eq!(false, AbsRot(255).within(0, AbsRot(254)));
    assert_eq!(true,  AbsRot(255).within(0, AbsRot(255)));
    assert_eq!(false, AbsRot(255).within(0, AbsRot(0)));

    // Test an 1 wide arc that jumps over 256/0
    assert_eq!(false, AbsRot(0).within(1, AbsRot(254)));
    assert_eq!(true,  AbsRot(0).within(1, AbsRot(255)));
    assert_eq!(true,  AbsRot(0).within(1, AbsRot(0)));
    assert_eq!(true,  AbsRot(0).within(1, AbsRot(1)));
    assert_eq!(false, AbsRot(0).within(1, AbsRot(2)));

    // Test an arc that doesn't have a discontinunity
    assert_eq!(false, AbsRot(64).within(1, AbsRot(62)));
    assert_eq!(true,  AbsRot(64).within(1, AbsRot(63)));
    assert_eq!(true,  AbsRot(64).within(1, AbsRot(64)));
    assert_eq!(true,  AbsRot(64).within(1, AbsRot(65)));
    assert_eq!(false, AbsRot(64).within(1, AbsRot(66)));

    // Test an max width arc
    assert_eq!(true, AbsRot(0).within(127, AbsRot(0)));
    assert_eq!(true, AbsRot(0).within(127, AbsRot(64)));
    assert_eq!(false, AbsRot(0).within(127, AbsRot(128))); // Directly back
    assert_eq!(true, AbsRot(0).within(127, AbsRot(192)));
    assert_eq!(true, AbsRot(0).within(127, AbsRot(255)));
}

#[test]
fn test_render_edges() {
    // cw_edge
    assert_eq!(AbsRot(0).cw_edge(1), AbsRot(1));
    assert_eq!(AbsRot(64).cw_edge(32), AbsRot(96));

    // ccw_edge
    assert_eq!(AbsRot(0).ccw_edge(1), AbsRot(255));
    assert_eq!(AbsRot(64).ccw_edge(32), AbsRot(32));
}
