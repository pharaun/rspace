use bevy::prelude::*;
use crate::ship::Rotation;

// Simulation position,
// Transform is separate and a visual layer, we need to redo the code to better
// separate the rendering layer from the simulation layer
#[derive(Component)]
pub struct Position(pub IVec2);

#[derive(Component)]
pub struct PreviousPosition(pub IVec2);

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
        // Scale
        let scaled_position = vec_scale(position.0, 10.);
        let scaled_previous_position = vec_scale(previous_position.0, 10.);

        // Linearly interpolate the translation from the old position to the current one.
        transform.translation = scaled_previous_position.lerp(scaled_position, overstep).extend(0.);
    }
}

fn vec_scale(vec: IVec2, factor: f32) -> Vec2 {
    Vec2::new(
        vec.x as f32 / factor,
        vec.y as f32 / factor,
    )
}

fn un_vec_scale(vec: Vec2, factor: f32) -> IVec2 {
    IVec2::new(
        (vec.x * factor) as i32,
        (vec.y * factor) as i32,
    )
}


// TODO: for now have a single accleration vector from the main engine only, but eventually
// I want to have RCS so that there can be a small amount of lateral and backward movement
// but you would still need the main engine for heavy acceleration.
#[derive(Component)]
pub struct Velocity {
    pub acceleration: i32,
    pub velocity: IVec2,

    // TODO: improve how the limits works better
    pub velocity_limit: u32,
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
        let mut acceleration = rot.0.to_quat().mul_vec3(Vec3::Y * (vec.acceleration as f32)).truncate();

        println!("Accel: {:?}", acceleration);

        // Apply Lorentz factor only if it will increase the velocity
        // Inspiration: https://stackoverflow.com/a/2891162
        let old_velocity = vec_scale(vec.velocity, 1.0);
        let new_velocity = old_velocity + acceleration * time.delta_secs();

        // TODO: this is not realistic, but keeps ship controllable (ie easy deceleration)
        if new_velocity.length_squared() > old_velocity.length_squared() {
            // Y = 1 / Sqrt(1 - v^2/c^2), Clamp (1 - v^2/c^2) to float min to avoid NaN and inf
            // Simplified via multiplying by the factor rather than diving
            let lorentz = (
                (1.0 - (
                    old_velocity.length_squared() / (vec.velocity_limit as f32).powi(2)
                )).max(0.0)
            ).sqrt();

            // TODO: it does go over 10 but that's cuz of delta-time and changing acceleration
            // curves, plus floating point imprecision... See if there's a better way to do it or
            // if we need to bite the bullet and go for a integrator for these
            acceleration *= lorentz;
        }

        // NOTE: This will make direction change be sluggish unless the ship decelerate enough to
        // do so. Could optionally allow for a heading change while preserving the current velocity
        // TODO: fix this up to use integer vectors + accelerations
        vec.velocity += un_vec_scale(acceleration * time.delta_secs(), 1.0);
        position.0 += un_vec_scale(vec_scale(vec.velocity, 1.0) * time.delta_secs(), 1.0);
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
        let acceleration = heading.mul_vec3(Vec3::Y * (vel.acceleration as f32)).truncate();

        // Current heading
        gizmos.line_2d(
            base + heading.mul_vec3(Vec3::Y * 30.).truncate(),
            base + heading.mul_vec3(Vec3::Y * 60.).truncate(),
            bevy::color::palettes::css::RED,
        );

        // Velocity direction
        gizmos.line_2d(
            base + Vec2::new(velocity.x as f32, velocity.y as f32).normalize() * 30.,
            base + Vec2::new(velocity.x as f32, velocity.y as f32).normalize() * 50.,
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
