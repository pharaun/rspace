use bevy::prelude::*;

// Radar:
//  TODO: Other types such as fixed radar (missiles?) and rotating radar
//  - Direction + arc-width (boosting detection distance)
#[derive(Component)]
pub struct Radar {
    pub limit: f32, // Per second?
    pub arc: f32, // Radian the width of the arc
    pub target: Quat, // Direction the radar should be pointing in
}

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
pub(crate) fn apply_radar_rotation(
    _time: Res<Time<Fixed>>,
    mut query: Query<(&Radar, &mut Transform)>,
) {
    for (_radar, _tran) in query.iter_mut() {
    }
}


#[derive(Component)]
pub struct RadarDebug;

pub(crate) fn debug_radar_gitzmos(
    _gizmos: Gizmos,
    query: Query<(&Transform, &Radar), With<RadarDebug>>
) {
    for (_tran, _radar) in query.iter() {
    }
}
