use avian2d::interpolation::TranslationInterpolation;
use avian2d::prelude::*;
use bevy::prelude::*;

use crate::FixedGameSystem;
use crate::math::FP_SCALE;
use crate::rotation::Heading;

pub struct MovementPlugin;
impl Plugin for MovementPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            FixedUpdate,
            (apply_thrust.after(crate::rotation::apply_rotation),)
                .in_set(FixedGameSystem::GameLogic),
        );
    }
}

#[derive(Bundle, Clone)]
pub struct MovementBundle {
    pub rigid_body: RigidBody,
    pub velocity: LinearVelocity,
    pub thrust: Thrust,
    pub position: Position,
    pub interpolation: TranslationInterpolation,
}

impl MovementBundle {
    pub fn new(position: IVec2, velocity: IVec2, velocity_limit: u32, acceleration: i32) -> Self {
        Self {
            rigid_body: RigidBody::Kinematic,
            velocity: LinearVelocity(velocity.as_vec2()),
            thrust: Thrust {
                acceleration,
                velocity_limit,
            },
            position: Position(position.as_vec2()),
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
#[require(Position, LinearVelocity)]
pub struct Thrust {
    pub acceleration: i32,

    // TODO: improve how the limits works better
    pub velocity_limit: u32,
}

#[derive(Component, Clone, Copy)]
pub struct MovDebug;

// TODO: improve this to integrate in forces (ie fireing of guns for smaller ships, etc)
#[expect(clippy::needless_pass_by_value)]
fn apply_thrust(mut query: Query<(&mut LinearVelocity, &Heading, &Thrust)>, time: Res<Time>) {
    for (mut velocity, heading, thrust) in query.iter_mut() {
        let acceleration =
            heading.0.to_heading_fp().as_vec2() / (FP_SCALE as f32) * thrust.acceleration as f32;

        // Apply Lorentz factor only if it will increase the velocity,
        // this is not realistic but permits easy deceleration for the ship
        // Inspiration: https://stackoverflow.com/a/2891162
        //
        // NOTE: This will make direction change be sluggish unless the ship decelerate enough to
        // do so. Could optionally allow for a heading change while preserving the current velocity
        let factor = if velocity.0.dot(acceleration) >= 0.0 {
            #[expect(clippy::cast_precision_loss)]
            lorentz_factor(velocity.0, thrust.velocity_limit as f32)
        } else {
            1.0
        };

        velocity.0 += acceleration * factor * time.delta_secs();
    }
}

// Lorentz: Y = 1 / Sqrt(1 - v^2/c^2)
//
// vel: (0,0) == 1.0,
// vel: (limit, limit) == 0.0
fn lorentz_factor(velocity: Vec2, c: f32) -> f32 {
    if c == 0.0 {
        return 0.0;
    }
    // Note we are dropping the invert (1 / lorentz_factor)
    // since we can just multiply by the factor
    (1.0 - velocity.length_squared() / (c * c)).max(0.0).sqrt()
}

#[expect(clippy::float_cmp)]
#[test]
fn test_lorentz_factor() {
    // 0 velocity -> 1.0
    assert_eq!(lorentz_factor(Vec2::ZERO, 100.), 1.0);

    // Check velocity beyond limit -> 0
    assert_eq!(lorentz_factor(Vec2::new(0., 100.), 100.), 0.0);
    assert_eq!(lorentz_factor(Vec2::new(300., -400.), 100.), 0.0);

    // Check limit == 0 == 0 factor
    assert_eq!(lorentz_factor(Vec2::new(1., 0.), 0.), 0.0);
    assert_eq!(lorentz_factor(Vec2::ZERO, 0.), 0.0);

    // Check that sqrt(3/4) == exact 0.75
    assert_eq!(lorentz_factor(Vec2::new(0., 50.), 100.), 0.75_f32.sqrt());
}
