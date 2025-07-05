use bevy::prelude::*;

#[derive(Component)]
pub struct Velocity {
    pub acceleration: f32,
    pub velocity: Vec2,

    // TODO: develop the limits
    pub velocity_limit: f32,
}

// TODO: improve this to integrate in forces (ie fireing of guns for smaller ships, etc)
pub(crate) fn apply_velocity(
    time: Res<Time>,
    mut query: Query<(&mut Velocity, &mut Transform, Option<&mut MovDebug>)>
) {
    for (mut vec, mut tran, debug) in query.iter_mut() {
        // DEBUG
        match debug {
            Some(mut dbg) => {
                dbg.acceleration = vec.acceleration;
                dbg.velocity = vec.velocity;
            },
            None => (),
        }

        // TODO: figure out how to lerp? There is also an awkward sideward acceleration
        // when we rotate 180, figure out why that happens
        let mut acceleration = tran.rotation.mul_vec3(Vec3::Y * vec.acceleration).truncate();

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
        tran.translation += (vec.velocity * time.delta_secs()).extend(0.);
    }
}


#[derive(Component)]
pub struct MovDebug {
    pub velocity: Vec2,
    pub acceleration: f32,
}

pub(crate) fn debug_movement_gitzmos(
    mut gizmos: Gizmos,
    query: Query<(&Transform, &MovDebug)>
) {
    for (tran, debug) in query.iter() {
        let base = tran.translation.truncate();
        let heading = tran.rotation;

        let debug_velocity = debug.velocity;
        let debug_acceleration = heading.mul_vec3(Vec3::Y * debug.acceleration).truncate();

        // Current heading
        gizmos.line_2d(
            base,
            base + heading.mul_vec3(Vec3::Y * 70.).truncate(),
            bevy::color::palettes::css::RED,
        );

        // Velocity direction
        gizmos.line_2d(
            base,
            base + debug_velocity.normalize() * 60.,
            bevy::color::palettes::css::GREEN,
        );

        // Acceleration direction
        gizmos.line_2d(
            base,
            base + debug_acceleration.normalize() * 50.,
            bevy::color::palettes::css::YELLOW,
        );
    }
}
