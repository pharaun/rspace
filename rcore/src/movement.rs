use avian2d::interpolation::TranslationInterpolation;
use avian2d::prelude::*;
use bevy_transform_interpolation::TranslationEasingState;

use crate::FixedGameSystem;
use crate::math::AbsRot;
use crate::math::tick_step_i64vec2_fp;
use crate::math::tick_step_ivec2;
use crate::rotation::Rotation;

use bevy::math::I64Vec2;
use bevy::prelude::*;

use crate::ARENA;
use crate::TICK_HZ;
use crate::math::FP_SCALE;

pub struct MovementPlugin;
impl Plugin for MovementPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            FixedUpdate,
            (
                apply_movement.after(crate::rotation::apply_rotation),
                wrap_position.after(apply_movement),
            )
                .in_set(FixedGameSystem::GameLogic),
        );
    }
}

#[derive(Bundle, Clone)]
pub struct MovementBundle {
    pub rigid_body: RigidBody,
    pub velocity: Velocity,
    pub position: Position,
    pub carry: PositionCarry,
    pub interpolation: TranslationInterpolation,
}

impl MovementBundle {
    pub fn new(position: IVec2, velocity: IVec2, velocity_limit: u32, acceleration: i32) -> Self {
        Self {
            rigid_body: RigidBody::Kinematic,
            velocity: Velocity {
                acceleration,
                velocity,
                velocity_limit,
            },
            position: Position(position.as_vec2()),
            carry: PositionCarry(IVec2::ZERO),
            interpolation: TranslationInterpolation,
        }
    }

    pub fn position(&mut self, x: i32, y: i32) {
        self.position.0 = IVec2::new(x, y).as_vec2();
    }
}

// TODO: for now have a single accleration vector from the main engine only, but eventually
// I want to have RCS so that there can be a small amount of lateral and backward movement
// but you would still need the main engine for heavy acceleration.
#[derive(Component, Clone, Copy)]
#[require(Position, VelocityCarry)]
pub struct Velocity {
    pub acceleration: i32,
    pub velocity: IVec2,

    // TODO: improve how the limits works better
    pub velocity_limit: u32,
}

// Experiment with sub-tick integration (bresenham-style carries)
#[derive(Component, Default, Clone, Copy)]
pub struct PositionCarry(pub IVec2);

// Fixed point floats for handling sub-tick acceleration
#[derive(Component, Default, Clone, Copy)]
pub struct VelocityCarry(pub I64Vec2);

#[derive(Component, Clone, Copy)]
pub struct MovDebug;

// TODO: improve this to integrate in forces (ie fireing of guns for smaller ships, etc)
#[expect(clippy::needless_pass_by_value)]
pub(crate) fn apply_movement(
    mut query: Query<(
        &mut Velocity,
        &mut VelocityCarry,
        &Rotation,
        &mut Position,
        &mut PositionCarry,
    )>,
) {
    for (mut vec, mut carry_vec, rot, mut position, mut carry_position) in query.iter_mut() {
        // Handle lorentz limited acceleration
        let (velocity, carry) = lorentz_acceleration(
            vec.velocity,
            carry_vec.0,
            rot.0,
            vec.acceleration,
            vec.velocity_limit,
            TICK_HZ,
        );
        carry_vec.0 = carry;
        vec.velocity = velocity;

        // Calculate position integration
        let (step, carry) = tick_step_ivec2(vec.velocity, carry_position.0, TICK_HZ);
        carry_position.0 = carry;
        position.0 += step.as_vec2();
    }
}

fn i128_dot2(a: IVec2, b: I64Vec2) -> i128 {
    i128::from(a.x) * i128::from(b.x) + i128::from(a.y) * i128::from(b.y)
}

// Integer velocity integration, using fixed-point math for the heading
// along with fixed point lorentz factor to damper it
fn lorentz_acceleration(
    velocity: IVec2,
    carry: I64Vec2,
    rotation: AbsRot,
    acceleration: i32,
    velocity_limit: u32,
    tick_hz: u32,
) -> (IVec2, I64Vec2) {
    // TODO: support ivec2 acceleration
    let heading_acceleration = rotation.to_heading_fp().as_i64vec2() * i64::from(acceleration);

    // Apply Lorentz factor only if it will increase the velocity,
    // this is not realistic but permits easy deceleration for the ship
    // Inspiration: https://stackoverflow.com/a/2891162
    //
    // NOTE: This will make direction change be sluggish unless the ship decelerate enough to
    // do so. Could optionally allow for a heading change while preserving the current velocity
    let heading_dot = i128_dot2(velocity, heading_acceleration);

    let factor = if heading_dot >= 0 {
        calculate_lorentz_factor_fp(velocity, velocity_limit)
    } else {
        FP_SCALE
    };

    // Integrate the heading_acceleration into the velocity
    let (step, carry) = tick_step_i64vec2_fp(
        heading_acceleration * factor,
        carry,
        tick_hz,
        FP_SCALE.pow(2),
    );
    (velocity + step.as_ivec2(), carry)
}

// Lorentz: Y = 1 / Sqrt(1 - v^2/c^2)
//
// This is scaled by FP_SCALE (to permit integer math).
// vel: (0,0) == FP_SCALE,
// vel: (limit, limit) == 0
//
// As it approaches limit, the factor approach zero.
fn calculate_lorentz_factor_fp(velocity: IVec2, velocity_limit: u32) -> i64 {
    // The c^2 term
    let c_2 = u64::from(velocity_limit).pow(2);
    if c_2 == 0 {
        return 0;
    }

    // The v^2 term
    // TODO: can overflow at (MIN, MIN) domain
    let v_2 = velocity.as_i64vec2().length_squared().unsigned_abs();

    // Compute the factor + scale it, this is why the original one went
    // from sqrt(1 - v^2/c^2) -> sqrt((c^2 - v^2)/v^2) so that we can apply
    // the FP_SCALE^2 when the factor is at 1, before sqrt
    let factor_2 = u128::from(c_2.saturating_sub(v_2))
        * u128::from(FP_SCALE.pow(2).cast_unsigned())
        / u128::from(c_2);

    #[expect(clippy::cast_possible_truncation)]
    {
        factor_2.isqrt() as i64
    }
}

// The gizmo renders are based off the wrapped ship position which is 1:1 at the moment.
//
// TODO: make sure this only affects transforms for things within the arena, maybe an arena tag is
// needed
// TODO: May want to change this to instead wrap the game-areana and change this to be a render
// concern
pub(crate) fn wrap_position(
    mut query: Query<(&mut Position, &mut TranslationEasingState), Changed<Position>>,
) {
    for (mut pos, mut easing) in query.iter_mut() {
        let arena = ARENA.as_vec2();
        let res: Vec2 = {
            let mut ret = Vec2::ZERO;

            if pos.0.y < -(arena.y / 2.) {
                ret.y += arena.y;
            } else if pos.0.y > (arena.y / 2.) {
                ret.y -= arena.y;
            }

            if pos.0.x < -(arena.x / 2.) {
                ret.x += arena.x;
            } else if pos.0.x > (arena.x / 2.) {
                ret.x -= arena.x;
            }
            ret
        };
        if res == Vec2::ZERO {
            continue;
        }
        pos.0 += res;

        // Update iterpolation by the wrap
        let offset = res.extend(0.);
        if let Some(start) = easing.start.as_mut() {
            *start += offset;
        }
        if let Some(end) = easing.end.as_mut() {
            *end += offset;
        }
    }
}

#[test]
fn test_calculate_lorentz_factor_fp() {
    // 0 velocity -> FP_SCALE (aka 1.0)
    assert_eq!(calculate_lorentz_factor_fp(IVec2::ZERO, 100), FP_SCALE);

    // Check velocity beyond limit -> 0
    assert_eq!(calculate_lorentz_factor_fp(IVec2::new(0, 100), 100), 0);
    assert_eq!(calculate_lorentz_factor_fp(IVec2::new(300, -400), 100), 0);

    // Check limit == 0 == 0 factor
    assert_eq!(calculate_lorentz_factor_fp(IVec2::new(1, 0), 0), 0);
    assert_eq!(calculate_lorentz_factor_fp(IVec2::ZERO, 0), 0);

    // Check the calculation (sqrt(3/4) and multiplied by the scale
    assert_eq!(
        calculate_lorentz_factor_fp(IVec2::new(0, 50), 100),
        28377, // sqrt(3/4) * FP_SCALE
    );
}

#[test]
fn test_deceleration_lorentz() {
    let heading = AbsRot(128);
    let acc = 16;
    let mut velocity = IVec2::new(0, 1000);
    let mut carry = I64Vec2::ZERO;
    for _ in 0..(128 * 5) {
        (velocity, carry) = lorentz_acceleration(velocity, carry, heading, acc, 2000, 128);
    }
    assert_eq!(
        velocity,
        IVec2::new(0, 920),
        "heading {heading:?} acc {acc}"
    );
    assert_eq!(carry, I64Vec2::ZERO, "heading {heading:?} acc {acc}");
}

#[test]
fn test_accelerate_from_rest_any_hz() {
    for hz in 1..=128 {
        let mut velocity = IVec2::ZERO;
        let mut carry = I64Vec2::ZERO;
        for _ in 0..(hz * 10) {
            (velocity, carry) = lorentz_acceleration(velocity, carry, AbsRot(0), 16, 10_000, hz);
        }
        assert_eq!(velocity, IVec2::new(0, 159), "at {hz}hz");
    }
}

#[test]
fn test_accelerate_at_limit() {
    let mut velocity = IVec2::new(0, 100);
    let mut carry = I64Vec2::ZERO;
    for _ in 0..640 {
        (velocity, carry) = lorentz_acceleration(velocity, carry, AbsRot(0), 1000, 100, 64);
    }
    assert_eq!(velocity, IVec2::new(0, 100));
    assert_eq!(carry, I64Vec2::ZERO);
}

#[test]
fn test_sub_tick_acceleration() {
    // acc = 1 is ~0.7 per axis at 45º so it would with integer math
    // truncate to 0 or round to 1 and be wrong, this must carry
    let mut velocity = IVec2::ZERO;
    let mut carry = I64Vec2::ZERO;
    for _ in 0..640 {
        (velocity, carry) = lorentz_acceleration(velocity, carry, AbsRot(32), 1, u32::MAX, 64);
    }
    assert_eq!(velocity, IVec2::new(7, 7));
}
