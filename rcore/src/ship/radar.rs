use bevy::prelude::*;

use crate::math::RelRot;
use crate::math::AbsRot;
use crate::ship::movement::Position;

// TODO:
// - radar rotation system
// - radar arc2length via area rule system?
// - radar detection system -> emits contact events.
// - Script subsystem listen for contact event and act upon it
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

// Radar contact event,
// 0 - self, 1 - target
#[derive(Event, Copy, Clone, Debug)]
pub struct ContactEvent (pub Entity, pub Entity);

// Radar:
//  TODO: Other types such as fixed radar (missiles?) and rotating radar
//  - Direction + arc-width (boosting detection distance)
//  - Add rendering iterpolation
#[derive(Component)]
pub struct Radar {
    pub current: AbsRot,
    pub target: AbsRot,

    // This should be private to this and the rotation system
    pub offset: Quat,

    // Units arc - [0 = off, 1 = 1/256th of an arc, max 128]
    pub current_arc: u8,
    pub target_arc: u8,
}

// TODO: split this and setup system ordering but for now.
pub(crate) fn apply_radar(
    mut events: EventWriter<ContactEvent>,
    mut query: Query<(&mut Radar, &ChildOf)>,
    ship_query: Query<(Entity, &Position)>,
) {
    for (mut radar, child_of) in query.iter_mut() {
        // Update radar rotation & arc width
        radar.current = radar.target;
        radar.current_arc = radar.target_arc;

        // Deal with transform
        // Offset transform (so that ship rotation system can compsenate)
        radar.offset = radar.current.to_quat();

        // Scan through all target on field, and calculate their distance and angle,
        // if within the arc store it in a list till we know the closest contact
        let mut best_target: Option<(Entity, IVec2)> = None;

        // TODO: abstract this logic to a helper class (gizmo debug wants this too and we will have
        // other radar types)
        let (base_ship, base_position) = ship_query.get(child_of.parent()).unwrap();
        for (target_ship, target_position) in ship_query.iter() {
            if base_ship == target_ship {
                println!("SKIP - Same Ship");
                continue;
            }

            // TODO: dynamic radar distance, for now fixed
            let distance: i32 = 5000_i32.pow(2);
            let calc_distance: i32 = base_position.0.distance_squared(target_position.0);
            println!("dist: {:?}, calc: {:?}", distance, calc_distance);
            if calc_distance > distance {
                println!("SKIP - Too Far");
                continue;
            }

            // Is distance better than current winner?
            if let Some((_, best_position)) = best_target {
                let target_distance = base_position.0.distance_squared(target_position.0);
                let best_distance = base_position.0.distance_squared(best_position);

                if best_distance >= target_distance {
                    println!("SKIP - Not Closer Than Best Match");
                    continue;
                }
            }

            // Validate Angle
            match AbsRot::from_vec2_angle(base_position.0, target_position.0) {
                Some(rot) => {
                    // There is an angle, validate that its within radar arcA
                    // TODO: calculcate the arc + heading
                    if radar.current.between(radar.current_arc, rot) {
                        // Yes it is, store it as new winner of best_target
                        best_target = Some((target_ship, target_position.0));
                    }
                },
                None => {
                    println!("SKIP - No Angle");
                },
            }
        }

        // If there is a best_target, then emit a contact
        if let Some((target_ship, _)) = best_target {
            println!("CONTACT");
            events.write(ContactEvent(base_ship, target_ship));
        }
    }
}


#[derive(Component)]
pub struct RadarDebug;

pub(crate) fn debug_radar_gitzmos(
    mut gizmos: Gizmos,
    query: Query<(&Radar, &ChildOf), With<RadarDebug>>,
    parent_query: Query<(&Transform, &Position)>,
) {
    for (radar, child_of) in query.iter() {
        // Need the ship translation to position the radar gizmo right
        let base = parent_query.get(child_of.parent()).unwrap().0.translation.truncate();
        let heading = radar.current;
        let target = radar.target;

        let cw_arc = heading + RelRot((radar.current_arc / 2) as i8);
        let ccw_arc = heading + RelRot(-((radar.current_arc / 2) as i8));

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

        // Radar arc - only current for now
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

        // Draw line between this ship (owner of this radar) and all target
        // color the target if they register as an contact (on radar)
        for (target_base, _) in parent_query.iter() {
            // Same ship
            if base.abs_diff_eq(
                target_base.translation.truncate(),
                0.1,
            ) {
                continue;
            }

            // If within distance (yellow) if within radar arc (green), if not (red)
            // TODO: dynamic radar distance, for now fixed
            let distance: f32 = 3500_f32.powf(2.0);
            let color = if base.distance_squared(target_base.translation.truncate()) < distance {
                match AbsRot::from_vec2_angle(base.as_ivec2(), target_base.translation.truncate().as_ivec2()) {
                    Some(rot) => {
                        if heading.between(radar.current_arc, rot) {
                            bevy::color::palettes::css::GREEN
                        } else {
                            bevy::color::palettes::css::YELLOW
                        }
                    },
                    None => bevy::color::palettes::css::PURPLE,
                }
            } else {
                bevy::color::palettes::css::RED
            };

            gizmos.line_2d(base, target_base.translation.truncate(), color);
        }
    }
}
