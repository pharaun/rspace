use avian2d::interpolation::RotationInterpolation;
use avian2d::interpolation::TranslationInterpolation;
use avian2d::prelude::Position;
use bevy::prelude::*;

use crate::attach::AttachedTo;
use crate::math::AbsRot;
use crate::rotation::Heading;
use crate::rotation::TargetHeading;
use crate::rotation::apply_rotation;

use crate::FixedGameSystem;

// TODO: dynamic radar distance, for now fixed
pub const DISTANCE: i32 = 4000;
pub const DISTANCE_SQUARED: i32 = DISTANCE.pow(2);

pub struct RadarPlugin;
impl Plugin for RadarPlugin {
    fn build(&self, app: &mut App) {
        app.add_message::<ContactMessage>().add_systems(
            FixedUpdate,
            (apply_arc_width, apply_radar)
                .chain()
                .after(apply_rotation)
                .in_set(FixedGameSystem::GameLogic),
        );
    }
}

// TODO: look at some sort of arc_iterpolation for growing/shrinking the arc render wise
#[derive(Bundle, Clone)]
pub struct RadarBundle {
    pub radar: Radar,
    pub arc: ArcWidth,
    // Rotation
    pub heading: Heading,
    pub target: TargetHeading,
    pub rotation_interpolation: RotationInterpolation,
    pub translation_interpolation: TranslationInterpolation,
}

impl RadarBundle {
    pub fn new(current: AbsRot, target: AbsRot, current_arc: u8, target_arc: u8) -> Self {
        Self {
            radar: Radar,
            arc: ArcWidth {
                current: current_arc,
                target: target_arc,
            },
            // Rotation system
            heading: Heading(current),
            target: TargetHeading {
                // NOTE: Insta rotation, can adjust later
                limit: u16::MAX,
                target,
                carry: 0,
            },
            rotation_interpolation: RotationInterpolation,
            translation_interpolation: TranslationInterpolation,
        }
    }

    pub fn rotation(&mut self, rotation: AbsRot) {
        self.heading.0 = rotation;
        self.target.target = rotation;
        self.target.carry = 0;
    }

    pub fn arc(&mut self, arc: u8) {
        self.arc.current = arc;
        self.arc.target = arc;
    }
}

// Arc Width:
// TODO: move to a better location because shield uses this as well
//
// Half-arc:
// - 0 == 1/256th of an arc
// - 1 == 3/256th of an arc
// - 127 = 255/256th of an arc
#[derive(Component, Clone, Copy, Default)]
pub struct ArcWidth {
    pub current: u8,
    pub target: u8,
}

#[derive(Component, Clone, Copy)]
pub struct ArcDebug;

// TODO:
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
#[require(ArcWidth)]
pub struct Radar;

#[derive(Component, Clone, Copy)]
pub struct RadarDebug;

// Radar contact event,
// 0 - self, 1 - target
#[derive(Message, Copy, Clone, Debug)]
pub struct ContactMessage(pub Entity, pub Entity);

// Radar Contact Result
#[derive(Debug)]
pub enum RadarContact {
    TooFar,
    Contact,
    SamePosition,
    OutsideArc,
}

#[derive(Debug)]
pub enum ArcCheck {
    InsideArc,
    OutsideArc,
    SamePosition,
}

// Handle arc width changes:
// - Instant (like headings right now)A
pub(crate) fn apply_arc_width(mut query: Query<&mut ArcWidth>) {
    for mut arc in query.iter_mut() {
        arc.current = arc.target;
    }
}

// TODO: split this and setup system ordering but for now.
pub(crate) fn apply_radar(
    mut message: MessageWriter<ContactMessage>,
    query: Query<(&Heading, &ArcWidth, &AttachedTo), With<Radar>>,
    ship_query: Query<(Entity, &Position)>,
) {
    for (heading, arc, attached_to) in query.iter() {
        // Scan through all target on field, and calculate their distance and angle,
        // if within the arc store it in a list till we know the closest contact
        let mut best_target: Option<(Entity, IVec2)> = None;

        // TODO: abstract this logic to a helper class (gizmo debug wants this too and we will have
        // other radar types)
        let (base_ship, base_position) = ship_query.get(attached_to.0).expect("attached");
        for (target_ship, target_position) in ship_query.iter() {
            if base_ship == target_ship {
                continue;
            }

            if matches!(
                within_radar(
                    base_position.0.as_ivec2(),
                    target_position.0.as_ivec2(),
                    heading.0,
                    arc.current,
                    DISTANCE_SQUARED
                ),
                RadarContact::Contact
            ) {
                // Is this contact better than current winner?
                if let Some((_, best_position)) = best_target {
                    let target_distance = base_position
                        .0
                        .as_ivec2()
                        .distance_squared(target_position.0.as_ivec2());
                    let best_distance = base_position.0.as_ivec2().distance_squared(best_position);

                    if target_distance <= best_distance {
                        best_target = Some((target_ship, target_position.0.as_ivec2()));
                    }
                } else {
                    best_target = Some((target_ship, target_position.0.as_ivec2()));
                }
            }
        }

        // If there is a best_target, then emit a contact
        if let Some((target_ship, _)) = best_target {
            message.write(ContactMessage(base_ship, target_ship));
        }
    }
}

pub fn within_radar(
    base: IVec2,
    target: IVec2,
    radar_heading: AbsRot,
    radar_arc: u8,
    distance_squared: i32,
) -> RadarContact {
    if base.distance_squared(target) > distance_squared {
        return RadarContact::TooFar;
    }
    match within_arc(base, target, radar_heading, radar_arc) {
        ArcCheck::InsideArc => RadarContact::Contact,
        ArcCheck::OutsideArc => RadarContact::OutsideArc,
        ArcCheck::SamePosition => RadarContact::SamePosition,
    }
}

pub fn within_arc(base: IVec2, target: IVec2, heading: AbsRot, arc: u8) -> ArcCheck {
    match AbsRot::from_vec2_angle(base, target) {
        Some(rot) => {
            if heading.within(arc, rot) {
                ArcCheck::InsideArc
            } else {
                ArcCheck::OutsideArc
            }
        }
        None => ArcCheck::SamePosition,
    }
}
