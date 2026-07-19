use bevy::prelude::*;

use crate::math::AbsRot;

use crate::movement::MovDebug;
use crate::movement::Thrust;
use avian2d::prelude::LinearVelocity;
use avian2d::prelude::Position;

use crate::attach::AttachedTo;

use crate::rotation::Heading;
use crate::rotation::RotDebug;
use crate::rotation::TargetHeading;

use crate::weapon::Health;
use crate::weapon::HealthDebug;
use crate::weapon::Shield;
use crate::weapon::ShieldHealthDebug;

use crate::radar::ArcDebug;
use crate::radar::ArcWidth;
use crate::radar::RadarContact;
use crate::radar::RadarDebug;

use crate::radar::within_radar;

// Primitive bar-graph in gizmo form
fn render_bar_gizmos(
    gizmos: &mut Gizmos,
    position: Vec2,
    width: f32,
    percentage_full: f32,
    bar_color: Srgba,
) {
    let bar_offset = (width * percentage_full) - (width / 2.);

    for v_off in 1..10 {
        gizmos.line_2d(
            position + Vec2::new(-(width / 2.), 50. - (v_off as f32 * 10.)),
            position + Vec2::new(bar_offset, 50. - (v_off as f32 * 10.)),
            bar_color,
        );
    }
    gizmos.rect_2d(
        Isometry2d::from_translation(position),
        Vec2::new(width, 100.),
        bevy::color::palettes::css::RED,
    );
}

pub(super) fn movement(
    query: Query<(&Transform, &LinearVelocity, &Thrust), With<MovDebug>>,
    mut gizmos: Gizmos,
) {
    for (tran, vel, thrust) in query.iter() {
        let base = tran.translation.truncate();
        let heading = tran.rotation;
        let velocity = vel.0;
        let acceleration = heading
            .mul_vec3(Vec3::Y * (thrust.acceleration as f32))
            .truncate();

        // Current heading
        gizmos.line_2d(
            base + heading.mul_vec3(Vec3::Y * 300.).truncate(),
            base + heading.mul_vec3(Vec3::Y * 600.).truncate(),
            bevy::color::palettes::css::RED,
        );

        // Velocity direction
        gizmos.line_2d(
            base + velocity.normalize() * 300.,
            base + velocity.normalize() * 500.,
            bevy::color::palettes::css::GREEN,
        );

        // Acceleration direction
        if thrust.acceleration > 0 {
            gizmos.line_2d(
                base + acceleration.normalize() * 300.,
                base + acceleration.normalize() * 400.,
                bevy::color::palettes::css::YELLOW,
            );
        }
    }
}

#[expect(clippy::similar_names)]
pub(super) fn arc(
    mut gizmos: Gizmos,
    query: Query<(&Heading, &TargetHeading, &ArcWidth, &AttachedTo), With<ArcDebug>>,
    parent_query: Query<&Transform>,
) {
    for (heading, target_heading, arc, attached_to) in query.iter() {
        // Need the ship translation to position the arc gizmo right
        let base = parent_query
            .get(attached_to.0)
            .expect("attached")
            .translation
            .truncate();
        let heading = heading.0;
        let target = target_heading.target;

        let cw_arc = heading.cw_edge(arc.current);
        let ccw_arc = heading.ccw_edge(arc.current);

        // Current heading
        gizmos.line_2d(
            base + heading.to_quat().mul_vec3(Vec3::Y * 1100.).truncate(),
            base + heading.to_quat().mul_vec3(Vec3::Y * 1400.).truncate(),
            bevy::color::palettes::css::RED,
        );

        // Target heading
        gizmos.line_2d(
            base + target.to_quat().mul_vec3(Vec3::Y * 1100.).truncate(),
            base + target.to_quat().mul_vec3(Vec3::Y * 1300.).truncate(),
            bevy::color::palettes::css::GREEN,
        );

        // Arc - only current for now
        gizmos.line_2d(
            base + cw_arc.to_quat().mul_vec3(Vec3::Y * 1300.).truncate(),
            base + cw_arc.to_quat().mul_vec3(Vec3::Y * 1400.).truncate(),
            bevy::color::palettes::css::YELLOW,
        );
        gizmos.short_arc_2d_between(
            base,
            base + heading.to_quat().mul_vec3(Vec3::Y * 1400.).truncate(),
            base + cw_arc.to_quat().mul_vec3(Vec3::Y * 1400.).truncate(),
            bevy::color::palettes::css::YELLOW,
        );
        gizmos.line_2d(
            base + ccw_arc.to_quat().mul_vec3(Vec3::Y * 1300.).truncate(),
            base + ccw_arc.to_quat().mul_vec3(Vec3::Y * 1400.).truncate(),
            bevy::color::palettes::css::YELLOW,
        );
        gizmos.short_arc_2d_between(
            base,
            base + heading.to_quat().mul_vec3(Vec3::Y * 1400.).truncate(),
            base + ccw_arc.to_quat().mul_vec3(Vec3::Y * 1400.).truncate(),
            bevy::color::palettes::css::YELLOW,
        );
    }
}

pub(super) fn radar(
    mut gizmos: Gizmos,
    query: Query<(&Heading, &ArcWidth, &AttachedTo), With<RadarDebug>>,
    parent_query: Query<(&Transform, &Position)>,
) {
    for (heading, arc, attached_to) in query.iter() {
        // Need the ship translation to position the radar gizmo right
        let (base, base_pos) = {
            let (base, pos) = parent_query.get(attached_to.0).expect("attached");
            (base.translation.truncate(), pos)
        };

        // Draw distance & contact status
        gizmos.circle_2d(
            Isometry2d::from_translation(base),
            crate::radar::DISTANCE as f32,
            bevy::color::palettes::css::GREEN,
        );

        // Draw line between this ship (owner of this radar) and all target
        // color the target if they register as an contact (on radar)
        for (target_base, target_pos) in parent_query.iter() {
            if base_pos.0 == target_pos.0 {
                continue;
            }

            // Find out if its a contact, if so color the lines
            let color = match within_radar(
                base_pos.0.as_ivec2(),
                target_pos.0.as_ivec2(),
                heading.0,
                arc.current,
                crate::radar::DISTANCE_SQUARED,
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

#[expect(clippy::similar_names)]
pub(super) fn rotation(
    mut gizmos: Gizmos,
    query: Query<(&Transform, &TargetHeading), With<RotDebug>>,
) {
    for (tran, target) in query.iter() {
        let base = tran.translation.truncate();
        let heading = tran.rotation;
        let qtarget = target.target.to_quat();

        let limit = u8::try_from(target.limit).unwrap_or(u8::MAX);
        let cw_limit = heading * AbsRot(limit).to_quat();
        let ccw_limit = heading * AbsRot(255 - limit).to_quat();

        // Current heading
        gizmos.line_2d(
            base + heading.mul_vec3(Vec3::Y * 700.).truncate(),
            base + heading.mul_vec3(Vec3::Y * 1000.).truncate(),
            bevy::color::palettes::css::RED,
        );

        // Target heading
        gizmos.line_2d(
            base + qtarget.mul_vec3(Vec3::Y * 700.).truncate(),
            base + qtarget.mul_vec3(Vec3::Y * 900.).truncate(),
            bevy::color::palettes::css::GREEN,
        );
        gizmos.short_arc_2d_between(
            base,
            base + heading.mul_vec3(Vec3::Y * 800.).truncate(),
            base + qtarget.mul_vec3(Vec3::Y * 800.).truncate(),
            bevy::color::palettes::css::GREEN,
        );

        // Limit + Arcs for rotation direction
        gizmos.line_2d(
            base + cw_limit.mul_vec3(Vec3::Y * 700.).truncate(),
            base + cw_limit.mul_vec3(Vec3::Y * 800.).truncate(),
            bevy::color::palettes::css::YELLOW,
        );
        gizmos.short_arc_2d_between(
            base,
            base + heading.mul_vec3(Vec3::Y * 700.).truncate(),
            base + cw_limit.mul_vec3(Vec3::Y * 700.).truncate(),
            bevy::color::palettes::css::YELLOW,
        );
        gizmos.line_2d(
            base + ccw_limit.mul_vec3(Vec3::Y * 700.).truncate(),
            base + ccw_limit.mul_vec3(Vec3::Y * 800.).truncate(),
            bevy::color::palettes::css::YELLOW,
        );
        gizmos.short_arc_2d_between(
            base,
            base + heading.mul_vec3(Vec3::Y * 700.).truncate(),
            base + ccw_limit.mul_vec3(Vec3::Y * 700.).truncate(),
            bevy::color::palettes::css::YELLOW,
        );
    }
}

#[expect(clippy::type_complexity)]
pub(super) fn health(
    mut gizmos: Gizmos,
    query: Query<(&Health, &Transform), (With<HealthDebug>, Without<Shield>)>,
) {
    for (health, tran) in query.iter() {
        let base = tran.translation.truncate();

        render_bar_gizmos(
            &mut gizmos,
            base + Vec2::new(0., -250.),
            350.,
            f32::from(health.current) / f32::from(health.maximum),
            bevy::color::palettes::css::GREEN,
        );
    }
}

#[expect(clippy::type_complexity)]
pub(super) fn shield_health(
    mut gizmos: Gizmos,
    query: Query<(&Health, &AttachedTo), (With<ShieldHealthDebug>, With<Shield>)>,
    parent_query: Query<&Transform>,
) {
    for (health, attached_to) in query.iter() {
        let base = parent_query
            .get(attached_to.0)
            .expect("attached")
            .translation
            .truncate();

        render_bar_gizmos(
            &mut gizmos,
            base + Vec2::new(0., -350.),
            350.,
            f32::from(health.current) / f32::from(health.maximum),
            bevy::color::palettes::css::BLUE,
        );
    }
}
