use bevy::prelude::*;

use crate::math::RelRot;
use crate::math::AbsRot;
use crate::movement::Position;
use crate::rotation::NoRotationPropagation;

use crate::FixedGameSystem;
use crate::ARENA_SCALE;

// TODO: dynamic radar distance, for now fixed
const DISTANCE: i32 = 4000;
const DISTANCE_SQUARED: i32 = DISTANCE.pow(2);

pub struct RadarPlugin;
impl Plugin for RadarPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<ContactEvent>()
            .add_systems(FixedUpdate, (
                apply_arc.in_set(FixedGameSystem::GameLogic),
                apply_radar.in_set(FixedGameSystem::GameLogic).after(apply_arc),
            ))
            .add_systems(RunFixedMainLoop, (
                interpolate_arc.in_set(RunFixedMainLoopSystem::AfterFixedMainLoop),
            ))
            .add_systems(Update, (
                debug_arc_gitzmos,
                debug_radar_gitzmos,
            ));
    }
}

#[derive(Bundle, Clone)]
pub struct RadarBundle {
    pub arc: Arc,
    pub radar: Radar,
    pub noprop: NoRotationPropagation,
}

impl RadarBundle {
    pub fn new(current: AbsRot, target: AbsRot, current_arc: u8, target_arc: u8) -> RadarBundle {
        RadarBundle {
            arc: Arc {
                current,
                target,
                current_arc,
                target_arc,
            },
            radar: Radar,
            noprop: NoRotationPropagation,
        }
    }

    pub fn rotation(&mut self, rotation: AbsRot) {
        self.arc.current = rotation;
        self.arc.target = rotation;
    }

    pub fn arc(&mut self, arc: u8) {
        self.arc.current_arc = arc;
        self.arc.target_arc = arc;
    }
}

// There are other components such as shields that wants to reuse the arc subsystem
// TODO: Reuse the Arc component for defining the Shield generator and shield arc
#[derive(Component, Clone, Copy, Default)]
pub struct Arc {
    // TODO: Can probs pull out the AbsRot and reuse Rotation component
    // But would have to look into the catch when it comes to 1/256th of an arc since
    // right now we cannot, best we can do is 2/256th of an arc
    pub current: AbsRot,
    pub target: AbsRot,

    // Units arc - [0 = off, 1 = 1/256th of an arc, max 128]
    pub current_arc: u8,
    pub target_arc: u8,
}

#[derive(Component, Clone, Copy)]
pub struct ArcDebug;

// TODO:
// - radar rotation system
// - radar arc2length via area rule system?
// - radar detection system -> emits contact events.
// - Script subsystem listen for contact event and act upon it
//
// Radar:
//  TODO: Other types such as fixed radar (missiles?) and rotating radar
//  - Direction + arc-width (boosting detection distance)
//
// Radar detection system
// - check the distance of all contacts
//  * optimization (use kdtree)
//  * optimization (check enemy contacts only)
// - These within a certain distance, are then checked again for their angle
// - This will then be compared to the radar angle (is it within?), if so
// - This final list will be all of the entities that are 'detected' by the radar, we can then deal
// with ECM and any other warfare stuff later
// - This approach is basically "converting" each entities into a polaris coordination from your
// ship/radar
#[derive(Component, Clone, Copy)]
#[require(Arc)]
pub struct Radar;

#[derive(Component, Clone, Copy)]
pub struct RadarDebug;

// Radar contact event,
// 0 - self, 1 - target
#[derive(Event, Copy, Clone, Debug)]
pub struct ContactEvent (pub Entity, pub Entity);

// Radar Contact Result
#[derive(Debug)]
enum RadarContact {
    Contact,
    TooFar,
    SamePosition,
    OutsideArc,
}

// Handles rendering
// Lifted from: https://github.com/Jondolf/bevy_transform_interpolation/tree/main
// Consider: https://github.com/Jondolf/bevy_transform_interpolation/blob/main/src/hermite.rs
// - Since we do have velocity information so we should be able to do better interpolation
pub(crate) fn interpolate_arc(
    mut query: Query<(&mut Transform, &Arc)>,
    fixed_time: Res<Time<Fixed>>
) {
    // How much of a "partial timestep" has accumulated since the last fixed timestep run.
    // Between `0.0` and `1.0`.
    let overstep = fixed_time.overstep_fraction();

    for (mut transform, arc) in &mut query {
        // Note: `slerp` will always take the shortest path, but when the two rotations are more than
        // 180 degrees apart, this can cause visual artifacts as the rotation "flips" to the other side.
        transform.rotation = arc.current.transform_slerp(arc.target, overstep);
    }
}

// Handle arc
pub(crate) fn apply_arc(
    mut query: Query<&mut Arc>
) {
    for mut arc in query.iter_mut() {
        // Update arc rotation & arc width
        arc.current = arc.target;
        arc.current_arc = arc.target_arc;
    }
}

// TODO: split this and setup system ordering but for now.
pub(crate) fn apply_radar(
    mut events: EventWriter<ContactEvent>,
    query: Query<(&Arc, &ChildOf), With<Radar>>,
    ship_query: Query<(Entity, &Position)>,
) {
    for (arc, child_of) in query.iter() {
        // Scan through all target on field, and calculate their distance and angle,
        // if within the arc store it in a list till we know the closest contact
        let mut best_target: Option<(Entity, IVec2)> = None;

        // TODO: abstract this logic to a helper class (gizmo debug wants this too and we will have
        // other radar types)
        let (base_ship, base_position) = ship_query.get(child_of.parent()).unwrap();
        for (target_ship, target_position) in ship_query.iter() {
            if base_ship == target_ship {
                continue;
            }

            match within_radar_arc(
                base_position.0, target_position.0,
                arc.current, arc.current_arc, DISTANCE_SQUARED
            ) {
                RadarContact::Contact => {
                    // Is this contact better than current winner?
                    if let Some((_, best_position)) = best_target {
                        let target_distance = base_position.0.distance_squared(target_position.0);
                        let best_distance = base_position.0.distance_squared(best_position);

                        if target_distance <= best_distance {
                            best_target = Some((target_ship, target_position.0));
                        }
                    } else {
                        best_target = Some((target_ship, target_position.0));
                    }
                },
                _ => (),
            }
        }

        // If there is a best_target, then emit a contact
        if let Some((target_ship, _)) = best_target {
            events.write(ContactEvent(base_ship, target_ship));
        }
    }
}

// TODO: probs can abstract this somewhat or have a companion for shields
fn within_radar_arc(
    base: IVec2, target: IVec2,
    radar_heading: AbsRot, radar_arc: u8, distance_squared: i32,
) -> RadarContact {
    if base.distance_squared(target) > distance_squared {
        return RadarContact::TooFar;
    }

    match AbsRot::from_vec2_angle(base, target) {
        Some(rot) => if radar_heading.between(radar_arc, rot) {
            RadarContact::Contact
        } else {
            RadarContact::OutsideArc
        },
        None => RadarContact::SamePosition,
    }
}

pub(crate) fn debug_arc_gitzmos(
    mut gizmos: Gizmos,
    query: Query<(&Arc, &ChildOf), With<ArcDebug>>,
    parent_query: Query<&Transform>
) {
    for (arc, child_of) in query.iter() {
        // Need the ship translation to position the arc gizmo right
        let base = parent_query.get(child_of.parent()).unwrap().translation.truncate();
        let heading = arc.current;
        let target = arc.target;

        let cw_arc = heading + RelRot((arc.current_arc / 2) as i8);
        let ccw_arc = heading + RelRot(-((arc.current_arc / 2) as i8));

        // Current heading
        gizmos.line_2d(
            base + heading.to_quat().mul_vec3(Vec3::Y * 110.).truncate(),
            base + heading.to_quat().mul_vec3(Vec3::Y * 140.).truncate(),
            bevy::color::palettes::css::RED,
        );

        // Target heading
        gizmos.line_2d(
            base + target.to_quat().mul_vec3(Vec3::Y * 110.).truncate(),
            base + target.to_quat().mul_vec3(Vec3::Y * 130.).truncate(),
            bevy::color::palettes::css::GREEN,
        );

        // Arc - only current for now
        gizmos.line_2d(
            base + cw_arc.to_quat().mul_vec3(Vec3::Y * 130.).truncate(),
            base + cw_arc.to_quat().mul_vec3(Vec3::Y * 140.).truncate(),
            bevy::color::palettes::css::YELLOW,
        );
        gizmos.short_arc_2d_between(
            base,
            base + heading.to_quat().mul_vec3(Vec3::Y * 140.).truncate(),
            base + cw_arc.to_quat().mul_vec3(Vec3::Y * 140.).truncate(),
            bevy::color::palettes::css::YELLOW,
        );
        gizmos.line_2d(
            base + ccw_arc.to_quat().mul_vec3(Vec3::Y * 130.).truncate(),
            base + ccw_arc.to_quat().mul_vec3(Vec3::Y * 140.).truncate(),
            bevy::color::palettes::css::YELLOW,
        );
        gizmos.short_arc_2d_between(
            base,
            base + heading.to_quat().mul_vec3(Vec3::Y * 140.).truncate(),
            base + ccw_arc.to_quat().mul_vec3(Vec3::Y * 140.).truncate(),
            bevy::color::palettes::css::YELLOW,
        );
    }
}

pub(crate) fn debug_radar_gitzmos(
    mut gizmos: Gizmos,
    query: Query<(&Arc, &ChildOf), With<RadarDebug>>,
    parent_query: Query<(&Transform, &Position)>,
) {
    for (arc, child_of) in query.iter() {
        // Need the ship translation to position the radar gizmo right
        let (base, base_pos) = {
            let (base, pos) = parent_query.get(child_of.parent()).unwrap();
            (base.translation.truncate(), pos)
        };

        // Draw distance & contact status
        gizmos.circle_2d(
            Isometry2d::from_translation(base),
            (DISTANCE as f32) / ARENA_SCALE,
            bevy::color::palettes::css::GREEN,
        );

        // Draw line between this ship (owner of this radar) and all target
        // color the target if they register as an contact (on radar)
        for (target_base, target_pos) in parent_query.iter() {
            if base_pos.0 == target_pos.0 {
                continue;
            }

            // Find out if its a contact, if so color the lines
            let color = match within_radar_arc(
                base_pos.0, target_pos.0,
                arc.current, arc.current_arc, DISTANCE_SQUARED
            ) {
                RadarContact::Contact => bevy::color::palettes::css::GREEN,
                RadarContact::OutsideArc => bevy::color::palettes::css::YELLOW,
                RadarContact::TooFar => bevy::color::palettes::css::RED,
                RadarContact::SamePosition => bevy::color::palettes::css::PURPLE,
            };

            gizmos.line_2d(base, target_base.translation.truncate(), color);
        }
    }
}
