use bevy::prelude::*;
use crate::math::vec_scale;
use crate::math::un_vec_scale;

use crate::rotation::Rotation;

use crate::ARENA_SCALE;
use crate::ARENA;

pub struct MovementPlugin;
impl Plugin for MovementPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(FixedUpdate, (
                apply_movement,
                wrap_position.after(apply_movement),
            ))
            .add_systems(RunFixedMainLoop, (
                interpolate_movement.in_set(RunFixedMainLoopSystem::AfterFixedMainLoop),
            ))
            .add_systems(Update, (
                debug_movement_gitzmos,
            ));
    }
}

#[derive(Bundle, Clone)]
pub struct MovementBundle {
    pub velocity: Velocity,
    pub position: Position,
    pub previous: PreviousPosition,
}

impl MovementBundle {
    pub fn new(position: IVec2, velocity: IVec2, velocity_limit: u32, acceleration: i32) -> MovementBundle {
        MovementBundle {
            velocity: Velocity {
                acceleration,
                velocity,
                velocity_limit,
            },
            position: Position(position),
            previous: PreviousPosition(position),
        }
    }

    pub fn position(&mut self, x: i32, y: i32) {
        self.position.0 = IVec2::new(x, y);
        self.previous.0 = IVec2::new(x, y);
    }
}

// TODO: for now have a single accleration vector from the main engine only, but eventually
// I want to have RCS so that there can be a small amount of lateral and backward movement
// but you would still need the main engine for heavy acceleration.
#[derive(Component, Clone, Copy)]
#[require(Position)]
pub struct Velocity {
    pub acceleration: i32,
    pub velocity: IVec2,

    // TODO: improve how the limits works better
    pub velocity_limit: u32,
}

// Simulation position,
// Transform is separate and a visual layer, we need to redo the code to better
// separate the rendering layer from the simulation layer
#[derive(Component, Default, Clone, Copy)]
#[require(PreviousPosition)]
pub struct Position(pub IVec2);

#[derive(Component, Default, Clone, Copy)]
pub struct PreviousPosition(pub IVec2);

#[derive(Component, Clone, Copy)]
pub struct MovDebug;

// Handles rendering
// Lifted from: https://github.com/Jondolf/bevy_transform_interpolation/tree/main
// Consider: https://github.com/Jondolf/bevy_transform_interpolation/blob/main/src/hermite.rs
// - Since we do have velocity information so we should be able to do better interpolation
pub(crate) fn interpolate_movement(
    mut query: Query<(&mut Transform, &Position, &PreviousPosition)>,
    fixed_time: Res<Time<Fixed>>,
) {
    // How much of a "partial timestep" has accumulated since the last fixed timestep run.
    // Between `0.0` and `1.0`.
    let overstep = fixed_time.overstep_fraction();

    for (mut transform, position, previous_position) in &mut query {
        // Scale
        let scaled_position = vec_scale(position.0, ARENA_SCALE);
        let scaled_previous_position = vec_scale(previous_position.0, ARENA_SCALE);

        // Linearly interpolate the translation from the old position to the current one.
        transform.translation = scaled_previous_position.lerp(scaled_position, overstep).extend(0.);
    }
}

// TODO: improve this to integrate in forces (ie fireing of guns for smaller ships, etc)
pub(crate) fn apply_movement(
    mut query: Query<(&mut Velocity, &Rotation, &mut Position, &mut PreviousPosition)>,
    fixed_time: Res<Time<Fixed>>,
) {
    for (mut vec, rot, mut position, mut previous_position) in query.iter_mut() {
        previous_position.0 = position.0;

        // Calculate lorentz factor to apply to acceleration
        // NOTE: This will make direction change be sluggish unless the ship decelerate enough to
        // do so. Could optionally allow for a heading change while preserving the current velocity
        let acceleration: Vec2 = rot.0.to_quat().mul_vec3(Vec3::Y * (vec.acceleration as f32)).truncate();
        let factor = calculate_lorentz_factor(
            &vec_scale(vec.velocity, 1.0),
            &acceleration,
            vec.velocity_limit,
            &fixed_time
        );

        vec.velocity += un_vec_scale(acceleration * factor * fixed_time.delta_secs(), 1.0);
        position.0 += un_vec_scale(vec_scale(vec.velocity, 1.0) * fixed_time.delta_secs(), 1.0);
    }
}

fn calculate_lorentz_factor<T>(
    velocity: &Vec2,
    acceleration: &Vec2,
    velocity_limit: u32,
    time: &Time<T>,
) -> f32
where
    T: std::default::Default
{
    // Apply Lorentz factor only if it will increase the velocity,
    // this is not realistic but permits easy deceleration for the ship
    // Inspiration: https://stackoverflow.com/a/2891162
    let old_velocity_length = velocity.length_squared();
    let new_velocity_length = (velocity + acceleration * time.delta_secs()).length_squared();

    if new_velocity_length > old_velocity_length {
        // Y = 1 / Sqrt(1 - v^2/c^2)
        // Clamp (1 - v^2/c^2) to float min to avoid NaN and inf
        // Simplified via multiplying by the factor rather than dividing
        (1.0 - (old_velocity_length / (velocity_limit as f32).powi(2))).max(0.0).sqrt()
    } else {
        1.0
    }
}

// The gizmo renders are based off the wrapped ship position which is 1:1 at the moment.
//
// TODO: make sure this only affects transforms for things within the arena, maybe an arena tag is
// needed
// TODO: May want to change this to instead wrap the game-areana and change this to be a render
// concern
fn wrap_position(mut query: Query<(&mut Position, &mut PreviousPosition), Changed<Position>>) {
    for (mut pos, mut ppos) in query.iter_mut() {
        let res: IVec2 = {
            let mut ret = IVec2::new(0, 0);

            if pos.0.y < -(ARENA.y / 2) {
                ret.y += ARENA.y;
            } else if pos.0.y > (ARENA.y / 2) {
                ret.y -= ARENA.y;
            }

            if pos.0.x < -(ARENA.x / 2) {
                ret.x += ARENA.x;
            } else if pos.0.x > (ARENA.x / 2) {
                ret.x -= ARENA.x;
            }
            ret
        };
        pos.0.y += res.y;
        ppos.0.y += res.y;

        pos.0.x += res.x;
        ppos.0.x += res.x;
    }
}

pub(crate) fn debug_movement_gitzmos(
    query: Query<(&Transform, &Velocity), With<MovDebug>>,
    mut gizmos: Gizmos,
) {
    for (tran, vel) in query.iter() {
        let base = tran.translation.truncate();
        let heading = tran.rotation;
        let velocity = vel.velocity;
        let acceleration = heading.mul_vec3(Vec3::Y * (vel.acceleration as f32)).truncate();

        // Current heading
        gizmos.line_2d(
            base + heading.mul_vec3(Vec3::Y * 30.).truncate(),
            base + heading.mul_vec3(Vec3::Y * 60.).truncate(),
            bevy::color::palettes::css::RED,
        );

        // Velocity direction
        gizmos.line_2d(
            base + vec_scale(velocity, 1.).normalize() * 30.,
            base + vec_scale(velocity, 1.).normalize() * 50.,
            bevy::color::palettes::css::GREEN,
        );

        // Acceleration direction
        if vel.acceleration > 0 {
            gizmos.line_2d(
                base + acceleration.normalize() * 30.,
                base + acceleration.normalize() * 40.,
                bevy::color::palettes::css::YELLOW,
            );
        }
    }
}
