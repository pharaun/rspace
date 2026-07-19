use std::f32::consts::PI;
use std::ops::Add;
use std::ops::AddAssign;

use avian2d::prelude::Rotation;
use bevy::math::I64Vec2;
use bevy::prelude::EulerRot;
use bevy::prelude::IVec2;
use bevy::prelude::Quat;

// Fixed-point unit scale for (headings, lorentz factor) calculation
// to remove any remaining floating point calculation in the core sim
// loop.
//
// power of 2 (sized so that i32 rate * FP_SCALE^2 still fits in i64)
pub const FP_SCALE: i64 = 1 << 15;

// Stepped Rotation: inspiration bevy::math::Rot2 - Which is clamped to the range (-π, π]
const NEG_FRAC_PI_128: f32 = PI / -128.0;

// Quarter sin table (0-64) for fixed point math (via FP_SCALE)
//
// python3 -c 'import math; print([round(math.sin(math.pi*i/128)*32768) for i in range(65)])'
#[rustfmt::skip]
const QUARTER_SIN_FP: [i32; 65] = [
        0,   804,  1608,  2411,  3212,  4011,  4808,  5602,
     6393,  7180,  7962,  8740,  9512, 10279, 11039, 11793,
    12540, 13279, 14010, 14733, 15447, 16151, 16846, 17531,
    18205, 18868, 19520, 20160, 20788, 21403, 22006, 22595,
    23170, 23732, 24279, 24812, 25330, 25833, 26320, 26791,
    27246, 27684, 28106, 28511, 28899, 29269, 29622, 29957,
    30274, 30572, 30853, 31114, 31357, 31581, 31786, 31972,
    32138, 32286, 32413, 32522, 32610, 32679, 32729, 32758,
    32768,
];

fn sin_fp(step: u8) -> i32 {
    let i = usize::from(step);
    match step {
        0..=64 => QUARTER_SIN_FP[i],
        65..=127 => QUARTER_SIN_FP[128 - i],
        128..=191 => -QUARTER_SIN_FP[i - 128],
        192..=255 => -QUARTER_SIN_FP[256 - i],
    }
}

// If d is CCW of heading r, it is > 0
// If d is at/CW of heading r, it is <= 0
fn cross(r: AbsRot, d: I64Vec2) -> i64 {
    let v = r.to_heading_fp().as_i64vec2();
    v.x * d.y - v.y * d.x
}

// Fold a vector into the first quadrant:
// x >= 0, y >= 0, Heading [0, 64]
fn fold_i64vec2(d: I64Vec2) -> (u8, I64Vec2) {
    if d.x >= 0 && d.y > 0 {
        (0u8, d)
    } else if d.x > 0 && d.y <= 0 {
        (64, I64Vec2::new(-d.y, d.x))
    } else if d.x <= 0 && d.y < 0 {
        (128, -d)
    } else {
        (192, I64Vec2::new(d.y, -d.x))
    }
}

// Largest AbsRot(k) in [0, 63] where cross is <= 0
// This means the angle is [k, k+1]
fn binary_search_angle(d: I64Vec2) -> u8 {
    let mut low = 0u8;
    let mut high = 63u8;
    while low < high {
        let middle = (low + high).div_ceil(2);
        if cross(AbsRot(middle), d) <= 0 {
            low = middle;
        } else {
            high = middle - 1;
        }
    }
    low
}

// Binary search -> 1/256th heading.
// Safe for |delta| up to 2^47
#[expect(clippy::similar_names)]
fn from_i64vec2_angle(delta: I64Vec2) -> AbsRot {
    let (quadrant, d) = fold_i64vec2(delta);
    let low = binary_search_angle(d);

    // Round up to which of the [low, low + 1] angles
    // from binary_search. If there are ties, bias it CCW
    // use cross products (|d| * sin(distance)).
    let cw_dist = -cross(AbsRot(low), d);
    let ccw_dist = cross(AbsRot(low + 1), d);
    let step = if cw_dist > ccw_dist { low + 1 } else { low };

    AbsRot(quadrant.wrapping_add(step))
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

// Absolute Rotation:
// 0   =   0º North
// 64  =  90º East
// 128 = 180º South
// 192 = 270º West
// Radian: 0 = 0, 1 = π/128, 64 = π/2, 128 = π/1, ...
#[derive(Debug, PartialEq, Eq, Clone, Copy, Default)]
pub struct AbsRot(pub u8);

// TODO: move the render only + test only entries over to their own modules/space to avoid
// polluting the top level AbsRot
impl AbsRot {
    // Render-only
    pub fn to_quat(&self) -> Quat {
        Quat::from_rotation_z(self.to_radians())
    }

    // Test-only
    pub fn from_quat(quat: Quat) -> Self {
        Self::from_angle(quat.to_euler(EulerRot::ZYX).0)
    }

    // Render-only
    pub fn to_radians(&self) -> f32 {
        NEG_FRAC_PI_128 * f32::from(self.0)
    }

    // Render-only
    pub fn from_angle(angle: f32) -> Self {
        #[expect(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
        Self((angle / NEG_FRAC_PI_128).round().rem_euclid(256.) as u8)
    }

    // Convert from AbsRot to Avian2d Rotations (reusing the fixed point sin/cos tables)
    pub fn to_rotation(&self) -> Rotation {
        #[expect(clippy::cast_precision_loss)]
        let heading = self.to_heading_fp().as_vec2() / FP_SCALE as f32;
        Rotation::from_sin_cos(-heading.x, heading.y)
    }

    // TODO: look into having a Vec2 version to avoid Position truncations
    pub fn from_vec2_angle(base: IVec2, target: IVec2) -> Option<Self> {
        let delta = target.as_i64vec2() - base.as_i64vec2();
        if delta == I64Vec2::ZERO {
            None
        } else {
            // IVec2(X, Y) (+Y = 0, +X = 64, -Y = 128, -X = 192)
            Some(from_i64vec2_angle(delta))
        }
    }

    pub fn to_heading_fp(&self) -> IVec2 {
        IVec2::new(sin_fp(self.0), sin_fp(self.0.wrapping_add(64)))
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

#[rustfmt::skip]
#[test]
fn test_to_rotation_exact() {
    assert_eq!((AbsRot(0).to_rotation().sin,   AbsRot(0).to_rotation().cos),   (0., 1.));
    assert_eq!((AbsRot(64).to_rotation().sin,  AbsRot(64).to_rotation().cos),  (-1., 0.));
    assert_eq!((AbsRot(128).to_rotation().sin, AbsRot(128).to_rotation().cos), (0., -1.));
    assert_eq!((AbsRot(192).to_rotation().sin, AbsRot(192).to_rotation().cos), (1., 0.));
}

#[test]
fn test_to_rotation_aprox_radians_match() {
    for i in 0..=u8::MAX {
        let table = AbsRot(i).to_rotation();
        let float = Rotation::radians(AbsRot(i).to_radians());
        // Validate that the table is within margin of error to the libm sin/cos numbers
        assert!(
            (table.sin - float.sin).abs() < 3e-5 && (table.cos - float.cos).abs() < 3e-5,
            "step {i}: {table:?} vs {float:?}"
        );
    }
}

#[test]
fn test_from_vec2_angle_roundtrip() {
    for i in 0..=u8::MAX {
        let heading = AbsRot(i).to_heading_fp();
        assert_eq!(
            AbsRot::from_vec2_angle(IVec2::ZERO, heading),
            Some(AbsRot(i)),
            "step {i}"
        );
    }
}

#[test]
fn test_from_vec2_angle_matches_atan2() {
    use bevy::prelude::Vec2;

    // Spot-check a dense grid against the libm atan2 implement
    for x in -60..=60 {
        for y in -60..=60 {
            if x == 0 && y == 0 {
                continue;
            }
            let delta = IVec2::new(x, y);
            let float_ref = AbsRot::from_angle(Vec2::Y.angle_to(delta.as_vec2()));
            assert_eq!(
                AbsRot::from_vec2_angle(IVec2::ZERO, delta),
                Some(float_ref),
                "delta {delta}"
            );
        }
    }
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
fn test_to_heading_fp_directions() {
    assert_eq!(AbsRot(0).to_heading_fp(),   IVec2::new(0, 32768));
    assert_eq!(AbsRot(32).to_heading_fp(),  IVec2::new(23170, 23170));
    assert_eq!(AbsRot(64).to_heading_fp(),  IVec2::new(32768, 0));
    assert_eq!(AbsRot(96).to_heading_fp(),  IVec2::new(23170, -23170));
    assert_eq!(AbsRot(128).to_heading_fp(), IVec2::new(0, -32768));
    assert_eq!(AbsRot(160).to_heading_fp(), IVec2::new(-23170, -23170));
    assert_eq!(AbsRot(192).to_heading_fp(), IVec2::new(-32768, 0));
    assert_eq!(AbsRot(224).to_heading_fp(), IVec2::new(-23170, 23170));
}

#[test]
fn test_to_heading_fp_aprox_quat_match() {
    // Make sure that the fixed point angles
    // matches the built in to_quat angles
    for i in 0..=u8::MAX {
        let quat = AbsRot(i)
            .to_quat()
            .mul_vec3(bevy::prelude::Vec3::Y)
            .truncate();
        let fp = AbsRot(i).to_heading_fp().as_vec2() / FP_SCALE as f32;
        assert!((quat - fp).length() < 3e-5, "step {i}: {quat:?} vs {fp:?}");
    }
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
