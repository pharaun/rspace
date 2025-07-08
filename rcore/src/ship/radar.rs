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
pub(crate) fn apply_radar_rotation(
    time: Res<Time<Fixed>>,
    mut query: Query<(&Radar, &mut Transform, Option<&mut RadarDebug>)>
) {
    for (radar, tran, debug) in query.iter_mut() {
        // Get current rotation vector, get the target rotation vector, do math, and then rotate
        let current = tran.rotation;
        let target = radar.target;
        let limit = Quat::from_rotation_z(radar.limit);

        // DEBUG
        match debug {
            Some(mut dbg) => {
                dbg.rotation_current = current.to_euler(EulerRot::ZYX).0;
                dbg.rotation_target = target.to_euler(EulerRot::ZYX).0;
                dbg.rotation_limit = limit.to_euler(EulerRot::ZYX).0;

                dbg.radar_length = 0f32;
                dbg.radar_arc = radar.arc;
            },
            None => (),
        }
    }
}


#[derive(Component)]
pub struct RadarDebug {
    pub rotation_current: f32,
    pub rotation_target: f32,
    pub rotation_limit: f32,

    pub radar_length: f32,
    pub radar_arc: f32,
}

// Probs a universal debugger that debug rotation + arc2length, and detection?
pub(crate) fn debug_radar_gitzmos(
    gizmos: Gizmos,
    query: Query<(&Transform, &RadarDebug)>
) {
    for (tran, debug) in query.iter() {
    }
}
