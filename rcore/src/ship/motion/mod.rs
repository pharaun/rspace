use bevy::prelude::*;

pub mod movement;
pub use crate::ship::motion::movement::Velocity;
pub use crate::ship::motion::movement::Position;
pub use crate::ship::motion::movement::PreviousPosition;
pub use crate::ship::motion::movement::MovDebug;

use crate::ship::motion::movement::apply_movement;
use crate::ship::motion::movement::interpolate_movement;
use crate::ship::motion::movement::debug_movement_gitzmos;

pub mod rotation;
pub use crate::ship::motion::rotation::TargetRotation;
pub use crate::ship::motion::rotation::Rotation;
pub use crate::ship::motion::rotation::PreviousRotation;
pub use crate::ship::motion::rotation::RotDebug;

use crate::ship::motion::rotation::apply_rotation;
use crate::ship::motion::rotation::interpolate_rotation;
use crate::ship::motion::rotation::debug_rotation_gitzmos;

// Plugin for motion related code:
// - Velocity, Position, PreviousPosition
// - TargetRotation, Rotation, PreviousRotion
pub struct MotionPlugin;
impl Plugin for MotionPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(FixedUpdate, (
                apply_movement,
                apply_rotation,
            ))
            .add_systems(RunFixedMainLoop, (
                interpolate_movement.in_set(RunFixedMainLoopSystem::AfterFixedMainLoop),
                interpolate_rotation.in_set(RunFixedMainLoopSystem::AfterFixedMainLoop),
            ))
            .add_systems(Update, (
                debug_movement_gitzmos,
                debug_rotation_gitzmos,
            ));
    }
}
