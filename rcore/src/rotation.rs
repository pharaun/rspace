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
            (apply_rotation.in_set(FixedGameSystem::GameLogic),),
        )
        .add_systems(
            PostUpdate,
            (disable_rotation_propagation.after(TransformSystems::Propagate),),
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
    pub fn new(rotation: AbsRot, target: AbsRot, limit: u8) -> Self {
        Self {
            target: TargetHeading {
                limit,
                target,
                carry: 0,
            },
            heading: Heading(rotation),
            rotation: Rotation::radians(rotation.to_radians()),
            interpolation: RotationInterpolation,
        }
    }

    pub fn rotation(&mut self, rotation: AbsRot) {
        self.target.target = rotation;
        self.heading.0 = rotation;
        self.rotation = Rotation::radians(rotation.to_radians());
    }
}

#[derive(Component, Clone, Copy)]
#[require(Heading)]
pub struct TargetHeading {
    pub limit: u8, // rotation per second
    pub target: AbsRot,
    pub carry: u32, // sub tick rotation
}

// Canonical heading of the ship, simulation reads this. Avian Rotation
// is for physics/everything else.
//
// NOTE: Do not add `AngularVelocity` to the ship.
#[derive(Component, Default, Clone, Copy)]
pub struct Heading(pub AbsRot);

#[derive(Component, Clone)]
pub struct NoRotationPropagation;

#[derive(Component, Clone, Copy)]
pub struct RotDebug;

#[expect(clippy::needless_pass_by_value)]
pub(crate) fn apply_rotation(mut query: Query<(&mut TargetHeading, &mut Heading, &mut Rotation)>) {
    for (mut target_heading, mut heading, mut rotation) in query.iter_mut() {
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

        let angle = heading
            .0
            .angle_between(target_heading.target)
            .clamp(u8::try_from(step).unwrap_or(u8::MAX));

        heading.0 += angle;

        // Mirror it into avian for physics/everything else
        *rotation = Rotation::radians(heading.0.to_radians());
    }
}

pub(crate) fn disable_rotation_propagation(
    query: Query<(&Children, &Transform)>,
    mut child_query: Query<(&Transform, &mut GlobalTransform), With<NoRotationPropagation>>,
) {
    for (childrens, parent_tran) in query.iter() {
        for child_entity in childrens.iter() {
            if let Ok((child_tran, mut child_global_tran)) = child_query.get_mut(child_entity) {
                *child_global_tran = child_global_tran
                    .compute_transform()
                    .with_rotation(child_tran.rotation)
                    .with_translation(parent_tran.translation + child_tran.translation)
                    .into();
            }
        }
    }
}
