use std::f32::consts::PI;
use std::ops::Add;
use std::ops::AddAssign;

use bevy::prelude::EulerRot;
use bevy::prelude::IVec2;
use bevy::prelude::Quat;
use bevy::prelude::Vec2;

pub fn vec_scale(vec: IVec2, factor: f32) -> Vec2 {
    vec.as_vec2() / factor
}

pub fn un_vec_scale(vec: Vec2, factor: f32) -> IVec2 {
    (vec * factor).round().as_ivec2()
}

// Bresenham-style integer rate integration
//  - split a per-second rate into a per-tick rate
//  - carry the remainder for next call
//  - this fixes the (1, 1) == 2/s not 1/s issue and make it approach the true rate
// TODO: do we want to pass in the tick_hz or ?
pub fn tick_step(rate: u32, carry: u32, tick_hz: u32) -> (u32, u32) {
    let budget = rate + carry;
    (budget / tick_hz, budget % tick_hz)
}

// IVec2 version of tick_step
pub fn tick_step_ivec2(rate: IVec2, carry: IVec2, tick_hz: u32) -> (IVec2, IVec2) {
    let hz = IVec2::splat(tick_hz.cast_signed());
    let budget = rate + carry;
    (budget.div_euclid(hz), budget.rem_euclid(hz))
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
        #[expect(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
        Self((angle / FRAC_PI_128).round().rem_euclid(256.) as u8)
    }

    pub fn from_vec2_angle(base: IVec2, target: IVec2) -> Option<Self> {
        // TODO: this can overflow cuz its i32 so need to fix
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
        debug_assert!(half_arc <= 127);
        self.angle_between(target).0.unsigned_abs() <= half_arc.min(127)
    }

    // Debugging math
    #[must_use]
    pub fn cw_edge(&self, half_arc: u8) -> Self {
        debug_assert!(half_arc <= 127);
        *self + RelRot(half_arc.min(127).cast_signed())
    }

    #[must_use]
    pub fn ccw_edge(&self, half_arc: u8) -> Self {
        debug_assert!(half_arc <= 127);
        *self + RelRot(-half_arc.min(127).cast_signed())
    }

    pub fn angle_between(&self, target: Self) -> RelRot {
        // i8 == [-128, 127] so it biases ccw at 128.
        RelRot(target.0.wrapping_sub(self.0).cast_signed())
    }

    pub fn transform_slerp(&self, end: Self, f: f32) -> Quat {
        // Force the slerp to respect the ccw bias of angle_between
        let delta = self.angle_between(end);
        Quat::from_rotation_z(FRAC_PI_128 * (f32::from(self.0) + f32::from(delta.0) * f))
        //self.to_quat().slerp(end.to_quat(), f)
    }
}

impl Add<RelRot> for AbsRot {
    type Output = Self;

    fn add(self, rhs: RelRot) -> Self {
        Self(self.0.wrapping_add_signed(rhs.0))
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
#[derive(Debug, PartialEq, Eq, Clone, Copy, Default)]
pub struct RelRot(pub i8);

impl RelRot {
    #[must_use]
    pub fn clamp(&self, clamp: u8) -> Self {
        if clamp >= 128 {
            return *self;
        }
        // Since its always <128 thanks to clamp above its a valid i8
        let bound = clamp.cast_signed();
        Self(self.0.clamp(-bound, bound))
    }
}

#[test]
fn test_vec_scale_roundtrip() {
    // Make sure the code rounds properly and not just truncate it, there
    // are some factors that causes it to be off by one. Such as: 7, 10, 49...
    for factor in 1..=100 {
        // Might as well exhaustively test an large arena
        for x in -20_000..=20_000 {
            let vec = IVec2::new(x, -x);
            assert_eq!(
                un_vec_scale(vec_scale(vec, factor as f32), factor as f32),
                vec,
                "x={x} factor={factor}"
            );
        }
    }
}

#[test]
fn test_tick_step_several_rates() {
    // We can have the engine from 1..=128 hz probs, forcefully check them all
    for hz in 1..=128 {
        for rate in 0..=256 {
            let mut carry = 0;
            let mut total = 0;

            // To force the carry == 0 we loop till "1 second" has passed
            for _ in 0..hz {
                let (step, new_carry) = tick_step(rate, carry, hz);
                total += step;
                carry = new_carry;
            }
            assert_eq!(total, rate, "rate {rate} at {hz}hz");
            assert_eq!(carry, 0, "rate {rate} at {hz}hz");
        }
    }
}

#[test]
fn test_tick_step_ivec2_several_rates() {
    // We can have the engine from 1..=128 hz probs, forcefully check them all
    for hz in 1..=128 {
        // Step by 32 to avoid spinning for longer than a few second
        for x in (-256..=256).step_by(32) {
            for y in (-256..=256).step_by(32) {
                let rate = IVec2::new(x, y);
                let mut carry = IVec2::ZERO;
                let mut total = IVec2::ZERO;

                // To force the carry == 0 we loop till "1 second" has passed
                for _ in 0..hz {
                    let (step, new_carry) = tick_step_ivec2(rate, carry, hz);
                    total += step;
                    carry = new_carry;
                }
                assert_eq!(total, rate, "rate {rate} at {hz}hz");
                assert_eq!(carry, IVec2::ZERO, "rate {rate} at {hz}hz");
            }
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
fn test_from_to_quat_roundtrip() {
    // u8 is small enough to just test it all.
    for i in 0..=u8::MAX {
        assert_eq!(
            AbsRot::from_quat(AbsRot(i).to_quat()),
            AbsRot(i),
            "step {i}"
        );
    }
}

#[test]
fn test_from_angle() {
    // CW rotation
    assert_eq!(AbsRot::from_angle(-PI / 512.), AbsRot(0));
    assert_eq!(AbsRot::from_angle(-PI / 256.), AbsRot(1));
    assert_eq!(AbsRot::from_angle(-PI / 128.), AbsRot(1));
    assert_eq!(AbsRot::from_angle(-PI / 64.), AbsRot(2));

    // CCW rotation
    assert_eq!(AbsRot::from_angle(PI / 512.), AbsRot(0)); // Should be 0
    assert_eq!(AbsRot::from_angle(PI / 256.), AbsRot(255));
    assert_eq!(AbsRot::from_angle(PI / 128.), AbsRot(255));
    assert_eq!(AbsRot::from_angle(PI / 64.), AbsRot(254));
}

#[test]
fn test_angle_between() {
    assert_eq!(AbsRot(0).angle_between(AbsRot(0)), RelRot(0));
    assert_eq!(AbsRot(0).angle_between(AbsRot(1)), RelRot(1));
    assert_eq!(AbsRot(0).angle_between(AbsRot(255)), RelRot(-1));

    // Biasing to -128
    assert_eq!(AbsRot(0).angle_between(AbsRot(128)), RelRot(-128));
    assert_eq!(AbsRot(64).angle_between(AbsRot(192)), RelRot(-128));

    // Shortest picked
    assert_eq!(AbsRot(0).angle_between(AbsRot(200)), RelRot(-56));

    // Wrapping cross 0 check
    assert_eq!(AbsRot(200).angle_between(AbsRot(10)), RelRot(66));
    assert_eq!(AbsRot(10).angle_between(AbsRot(200)), RelRot(-66));
}

#[cfg(test)]
fn assert_rot_eq(a: Quat, b: Quat) {
    let diff = a.dot(b).abs();
    assert!(
        diff > 0.999_99,
        "rotations differ: {a:?} vs {b:?} - differ by: {diff}"
    );
}

#[rustfmt::skip]
#[test]
fn test_transform_slerp_biases_ccw() {
    // Easy exact angles to validate quickly
    assert_eq!(AbsRot(0).transform_slerp(AbsRot(64), 0.),  AbsRot(0).to_quat());
    assert_eq!(AbsRot(0).transform_slerp(AbsRot(64), 0.5), AbsRot(32).to_quat());
    assert_eq!(AbsRot(0).transform_slerp(AbsRot(64), 1.),  AbsRot(64).to_quat());

    // Check the 256/0 rotation
    assert_eq!(AbsRot(200).transform_slerp(AbsRot(10), 0.5), AbsRot(233).to_quat());

    // Check that it biases CCW like angle_between
    assert_rot_eq(AbsRot(0).transform_slerp(AbsRot(128), 0.25), AbsRot(224).to_quat());
    assert_rot_eq(AbsRot(0).transform_slerp(AbsRot(128), 0.5),  AbsRot(192).to_quat());
    assert_rot_eq(AbsRot(0).transform_slerp(AbsRot(128), 0.75), AbsRot(160).to_quat());
}

#[test]
fn test_clamp() {
    assert_eq!(RelRot(5).clamp(0), RelRot(0));
    assert_eq!(RelRot(0).clamp(1), RelRot(0));
    assert_eq!(RelRot(1).clamp(1), RelRot(1));
    assert_eq!(RelRot(2).clamp(1), RelRot(1));
    assert_eq!(RelRot(-2).clamp(1), RelRot(-1));

    // Ignore big clamp values
    assert_eq!(RelRot(-128).clamp(128), RelRot(-128));
    assert_eq!(RelRot(-128).clamp(200), RelRot(-128));

    // [-128, 127] bias, check that 127 catches both sides
    assert_eq!(RelRot(-128).clamp(127), RelRot(-127));
    assert_eq!(RelRot(127).clamp(127), RelRot(127));
}

#[test]
fn test_add_rel_to_abs() {
    assert_eq!(AbsRot(0) + RelRot(0), AbsRot(0));
    assert_eq!(AbsRot(0) + RelRot(1), AbsRot(1));
    assert_eq!(AbsRot(0) + RelRot(-1), AbsRot(255));
    assert_eq!(AbsRot(0) + RelRot(-128), AbsRot(128));

    // Wrapping
    assert_eq!(AbsRot(255) + RelRot(1), AbsRot(0));
    assert_eq!(AbsRot(200) + RelRot(100), AbsRot(44));
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
