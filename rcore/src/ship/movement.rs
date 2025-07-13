use bevy::prelude::*;
use crate::ship::Rotation;

// Simulation position,
// Transform is separate and a visual layer, we need to redo the code to better
// separate the rendering layer from the simulation layer
#[derive(Component)]
pub struct Position(pub Vec2);

#[derive(Component)]
pub struct PreviousPosition(pub Vec2);

// Handles rendering
// Lifted from: https://github.com/Jondolf/bevy_transform_interpolation/tree/main
// Consider: https://github.com/Jondolf/bevy_transform_interpolation/blob/main/src/hermite.rs
// - Since we do have velocity information so we should be able to do better interpolation
pub(crate) fn interpolate_transforms(
    mut query: Query<(&mut Transform, &Position, &PreviousPosition)>,
    fixed_time: Res<Time<Fixed>>
) {
    // How much of a "partial timestep" has accumulated since the last fixed timestep run.
    // Between `0.0` and `1.0`.
    let overstep = fixed_time.overstep_fraction();

    for (mut transform, position, previous_position) in &mut query {
        // Linearly interpolate the translation from the old position to the current one.
        transform.translation = previous_position.0.lerp(position.0, overstep).extend(0.);
    }
}


#[derive(Component)]
pub struct Velocity {
    pub acceleration: f32,
    pub velocity: Vec2,

    // TODO: develop the limits
    pub velocity_limit: f32,
}

// TODO: improve this to integrate in forces (ie fireing of guns for smaller ships, etc)
// TODO: remove dependence on Transform and instead do a fixed rotation component
// TODO: separate the debug stuff out to its own component/system
pub(crate) fn apply_velocity(
    time: Res<Time<Fixed>>,
    mut query: Query<(&mut Velocity, &Rotation, &mut Position, &mut PreviousPosition)>
) {
    for (mut vec, rot, mut position, mut previous_position) in query.iter_mut() {
        previous_position.0 = position.0;

        // TODO: figure out how to lerp? There is also an awkward sideward acceleration
        // when we rotate 180, figure out why that happens
        let mut acceleration = rot.0.to_quat().mul_vec3(Vec3::Y * vec.acceleration).truncate();

        println!("Accel: {:?}", acceleration);

        // Apply Lorentz factor only if it will increase the velocity
        // Inspiration: https://stackoverflow.com/a/2891162
        let new_velocity = vec.velocity + acceleration * time.delta_secs();

        // TODO: this is not realistic, but keeps ship controllable (ie easy deceleration)
        if new_velocity.length_squared() > vec.velocity.length_squared() {
            // Y = 1 / Sqrt(1 - v^2/c^2), Clamp (1 - v^2/c^2) to float min to avoid NaN and inf
            // Simplified via multiplying by the factor rather than diving
            let lorentz = (
                (1.0 - (
                    vec.velocity.length_squared() / vec.velocity_limit.powi(2)
                )).max(0.0)
            ).sqrt();

            // TODO: it does go over 10 but that's cuz of delta-time and changing acceleration
            // curves, plus floating point imprecision... See if there's a better way to do it or
            // if we need to bite the bullet and go for a integrator for these
            acceleration *= lorentz;
        }

        // NOTE: This will make direction change be sluggish unless the ship decelerate enough to
        // do so. Could optionally allow for a heading change while preserving the current velocity
        vec.velocity += acceleration * time.delta_secs();
        position.0 += vec.velocity * time.delta_secs();
    }
}


#[derive(Component)]
pub struct MovDebug;

pub(crate) fn debug_movement_gitzmos(
    mut gizmos: Gizmos,
    query: Query<(&Transform, &Velocity), With<MovDebug>>
) {
    for (tran, vel) in query.iter() {
        let base = tran.translation.truncate();
        let heading = tran.rotation;
        let velocity = vel.velocity;
        let acceleration = heading.mul_vec3(Vec3::Y * vel.acceleration).truncate();

        // Current heading
        gizmos.line_2d(
            base + heading.mul_vec3(Vec3::Y * 30.).truncate(),
            base + heading.mul_vec3(Vec3::Y * 60.).truncate(),
            bevy::color::palettes::css::RED,
        );

        // Velocity direction
        gizmos.line_2d(
            base + velocity.normalize() * 30.,
            base + velocity.normalize() * 50.,
            bevy::color::palettes::css::GREEN,
        );

        // Acceleration direction
        if vel.acceleration > 0.0 {
            gizmos.line_2d(
                base + acceleration.normalize() * 30.,
                base + acceleration.normalize() * 40.,
                bevy::color::palettes::css::YELLOW,
            );
        }
    }
}
