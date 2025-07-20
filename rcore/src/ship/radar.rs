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

    // Units arc - [0 = off, 1 = 1/256th of an arc, max 128]
    pub current_arc: u8,
    pub target_arc: u8,
}

// TODO: split this and setup system ordering but for now.
pub(crate) fn apply_radar(
    mut events: EventWriter<ContactEvent>,
    ships: Query<Entity>,
    mut query: Query<(Entity, &mut Radar, &mut Transform, &Position)>,
    _target_query: Query<(Entity, &Position)>,
) {
    for (self_entity, mut radar, mut transform, _position) in query.iter_mut() {
        // Update radar rotation & arc width
        radar.current = radar.target;
        radar.current_arc = radar.target_arc;

        // Deal with transform
        // TODO: Grab parent rotation and cancel it. This radar type is an AbsRadar for now
        //transform.rotation = radar.current.to_quat();

        // Scan through all target on field, and calculate their distance and angle,
        // if within the arc store it in a list till we know the closest contact
//        for ship in ships.iter_ancestors(self_entity) {
//            println!("Ship: {:?}", ship);
//            //events.write(ContactEvent(self_entity, self_entity));
//        }
    }
}


#[derive(Component)]
pub struct RadarDebug;

pub(crate) fn debug_radar_gitzmos(
    mut gizmos: Gizmos,
    query: Query<(&Transform, &Radar), With<RadarDebug>>
) {
    for (tran, radar) in query.iter() {
        let base = tran.translation.truncate();
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
    }
}
