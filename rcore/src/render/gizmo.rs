use bevy::prelude::*;

use crate::ARENA_SCALE;

use crate::math::AbsRot;
use crate::math::vec_scale;

use crate::movement::Position;
use crate::movement::Velocity;
use crate::movement::MovDebug;

use crate::rotation::RotDebug;
use crate::rotation::TargetRotation;

use crate::weapon::Shield;
use crate::weapon::ShieldHealthDebug;
use crate::weapon::Health;
use crate::weapon::HealthDebug;

use crate::radar::Arc;
use crate::radar::ArcDebug;
use crate::radar::RadarDebug;
use crate::radar::RadarContact;

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
            position + Vec2::new(-(width / 2.), 5. - v_off as f32),
            position + Vec2::new(bar_offset, 5. - v_off as f32),
            bar_color,
        );
    }
    gizmos.rect_2d(
        Isometry2d::from_translation(position),
        Vec2::new(width, 10.),
        bevy::color::palettes::css::RED,
    );
}

pub(super) fn movement(
    query: Query<(&Transform, &Velocity), With<MovDebug>>,
    mut gizmos: Gizmos,
) {
    for (tran, vel) in query.iter() {
        let base = tran.translation.truncate();
        let heading = tran.rotation;
        let velocity = vel.velocity;
        let acceleration = heading
            .mul_vec3(Vec3::Y * (vel.acceleration as f32))
            .truncate();

        // Current heading
        gizmos.line_2d(
            base + heading.mul_vec3(Vec3::Y * 30.).truncate(),
            base + heading.mul_vec3(Vec3::Y * 60.).truncate(),
            bevy::color::palettes::css::RED,
        );

        // Velocity direction
        gizmos.line_2d(
            base + vec_scale(velocity, 1.).normalize() * 30.,
            base + vec_scale(velocity, 1.).normalize() * 50.,
            bevy::color::palettes::css::GREEN,
        );

        // Acceleration direction
        if vel.acceleration > 0 {
            gizmos.line_2d(
                base + acceleration.normalize() * 30.,
                base + acceleration.normalize() * 40.,
                bevy::color::palettes::css::YELLOW,
            );
        }
    }
}

#[expect(clippy::similar_names)]
pub(super) fn arc(
    mut gizmos: Gizmos,
    query: Query<(&Arc, &ChildOf), With<ArcDebug>>,
    parent_query: Query<&Transform>,
) {
    for (arc, child_of) in query.iter() {
        // Need the ship translation to position the arc gizmo right
        let base = parent_query
            .get(child_of.parent())
            .expect("child")
            .translation
            .truncate();
        let heading = arc.current;
        let target = arc.target;

        let cw_arc = heading.cw_edge(arc.current_arc);
        let ccw_arc = heading.ccw_edge(arc.current_arc);

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

pub(super) fn radar(
    mut gizmos: Gizmos,
    query: Query<(&Arc, &ChildOf), With<RadarDebug>>,
    parent_query: Query<(&Transform, &Position)>,
) {
    for (arc, child_of) in query.iter() {
        // Need the ship translation to position the radar gizmo right
        let (base, base_pos) = {
            let (base, pos) = parent_query.get(child_of.parent()).expect("child");
            (base.translation.truncate(), pos)
        };

        // Draw distance & contact status
        gizmos.circle_2d(
            Isometry2d::from_translation(base),
            (crate::radar::DISTANCE as f32) / ARENA_SCALE,
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
                base_pos.0,
                target_pos.0,
                arc.current,
                arc.current_arc,
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
    query: Query<(&Transform, &TargetRotation), With<RotDebug>>,
) {
    for (tran, target) in query.iter() {
        let base = tran.translation.truncate();
        let heading = tran.rotation;
        let qtarget = target.target.to_quat();

        let cw_limit = heading * AbsRot(target.limit).to_quat();
        let ccw_limit = heading * AbsRot(255 - target.limit).to_quat();

        // Current heading
        gizmos.line_2d(
            base + heading.mul_vec3(Vec3::Y * 70.).truncate(),
            base + heading.mul_vec3(Vec3::Y * 100.).truncate(),
            bevy::color::palettes::css::RED,
        );

        // Target heading
        gizmos.line_2d(
            base + qtarget.mul_vec3(Vec3::Y * 70.).truncate(),
            base + qtarget.mul_vec3(Vec3::Y * 90.).truncate(),
            bevy::color::palettes::css::GREEN,
        );
        gizmos.short_arc_2d_between(
            base,
            base + heading.mul_vec3(Vec3::Y * 80.).truncate(),
            base + qtarget.mul_vec3(Vec3::Y * 80.).truncate(),
            bevy::color::palettes::css::GREEN,
        );

        // Limit + Arcs for rotation direction
        gizmos.line_2d(
            base + cw_limit.mul_vec3(Vec3::Y * 70.).truncate(),
            base + cw_limit.mul_vec3(Vec3::Y * 80.).truncate(),
            bevy::color::palettes::css::YELLOW,
        );
        gizmos.short_arc_2d_between(
            base,
            base + heading.mul_vec3(Vec3::Y * 70.).truncate(),
            base + cw_limit.mul_vec3(Vec3::Y * 70.).truncate(),
            bevy::color::palettes::css::YELLOW,
        );
        gizmos.line_2d(
            base + ccw_limit.mul_vec3(Vec3::Y * 70.).truncate(),
            base + ccw_limit.mul_vec3(Vec3::Y * 80.).truncate(),
            bevy::color::palettes::css::YELLOW,
        );
        gizmos.short_arc_2d_between(
            base,
            base + heading.mul_vec3(Vec3::Y * 70.).truncate(),
            base + ccw_limit.mul_vec3(Vec3::Y * 70.).truncate(),
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
            base + Vec2::new(0., -25.),
            35.,
            f32::from(health.current) / f32::from(health.maximum),
            bevy::color::palettes::css::GREEN,
        );
    }
}

#[expect(clippy::type_complexity)]
pub(super) fn shield_health(
    mut gizmos: Gizmos,
    query: Query<(&Health, &ChildOf), (With<ShieldHealthDebug>, With<Shield>)>,
    ship_query: Query<&Transform>,
) {
    for (health, child_of) in query.iter() {
        let base = ship_query
            .get(child_of.parent())
            .expect("child")
            .translation
            .truncate();

        render_bar_gizmos(
            &mut gizmos,
            base + Vec2::new(0., -35.),
            35.,
            f32::from(health.current) / f32::from(health.maximum),
            bevy::color::palettes::css::BLUE,
        );
    }
}
