use avian2d::interpolation::RotationInterpolation;
use avian2d::prelude::Rotation;
use bevy::prelude::*;

use crate::FixedGameSystem;
use crate::TICK_HZ;

use crate::math::AbsRot;
use crate::math::tick_step;

pub struct RotationPlugin;
impl Plugin for RotationPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            FixedUpdate,
            (apply_rotation, propagate_heading_transform)
                .chain()
                .in_set(FixedGameSystem::GameLogic),
        );
    }
}

#[derive(Bundle, Clone)]
pub struct RotationBundle {
    pub target: TargetHeading,
    pub heading: Heading,
    pub rotation: Rotation,
    pub interpolation: RotationInterpolation,
}

impl RotationBundle {
    pub fn new(rotation: AbsRot, target: AbsRot, limit: u16) -> Self {
        Self {
            target: TargetHeading {
                limit,
                target,
                carry: 0,
            },
            heading: Heading(rotation),
            rotation: rotation.to_rotation(),
            interpolation: RotationInterpolation,
        }
    }

    pub fn rotation(&mut self, rotation: AbsRot) {
        self.target.target = rotation;
        self.target.carry = 0;
        self.heading.0 = rotation;
        self.rotation = rotation.to_rotation();
    }
}

// Used by any system that uses 1/256th of an arc rotation system (radar, shield, ships)
#[derive(Component, Clone, Copy)]
#[require(Heading)]
pub struct TargetHeading {
    pub limit: u16, // rotation steps per second (>= 8192 is instant)
    pub target: AbsRot,
    pub carry: u32, // sub tick rotation
}

// Canonical heading, simulation reads this. Avian Rotation is for physics/everything else.
//
// NOTE: Do not add `AngularVelocity` to the ship.
#[derive(Component, Default, Clone, Copy)]
pub struct Heading(pub AbsRot);

#[derive(Component, Clone, Copy)]
pub struct RotDebug;

pub(crate) fn apply_rotation(
    mut query: Query<(&mut TargetHeading, &mut Heading, Option<&mut Rotation>)>,
) {
    for (mut target_heading, mut heading, opt_rotation) in query.iter_mut() {
        // If heading is the same as the target heading, bail
        if heading.0 == target_heading.target {
            continue;
        }

        // Calculate the carry so sub-tick heading rotation isn't lost
        let (step, carry) = tick_step(
            u32::from(target_heading.limit),
            target_heading.carry,
            TICK_HZ,
        );
        target_heading.carry = carry;

        // NOTE: [limit >= 8192) == instant rotation.
        // Caveat: exact 180-degree flip causes weird slerp bias issue
        //
        // Consider clamping max rotation to 127, and this will break exact
        // 128 rotations, in one shot, but if its a slower rotation.... meh.
        let angle = heading
            .0
            .angle_between(target_heading.target)
            .clamp(u8::try_from(step).unwrap_or(u8::MAX));

        heading.0 += angle;

        // Mirror it into avian for physics/etc..
        // Arc modules don't have rigid body so they don't have Rotation
        if let Some(mut rotation) = opt_rotation {
            *rotation = heading.0.to_rotation();
        }
    }
}

// Arc modules (radar/shield) don't have Avian Rotation, Update render transform
// directly for these. (The others are handled by Avian).
#[expect(clippy::type_complexity)]
pub(crate) fn propagate_heading_transform(
    mut query: Query<(&Heading, &mut Transform), (Changed<Heading>, Without<Rotation>)>,
) {
    for (heading, mut transform) in query.iter_mut() {
        transform.rotation = heading.0.to_quat();
    }
}
