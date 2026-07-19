use bevy_prototype_lyon::geometry::ShapeBuilderBase as _;
use bevy_prototype_lyon::prelude::Fill;
use bevy_prototype_lyon::prelude::ShapeBuilder;
use bevy_prototype_lyon::prelude::ShapePath;
use bevy_prototype_lyon::prelude::Stroke;

use bevy::prelude::*;

use crate::math::AbsRot;

// TODO: add an Arena Marker for ships and stuff for objects we want to have warping
// enabled for, versus objects we don't.
#[derive(Component)]
struct ArenaMarker;

pub(super) fn arena_bounds_setup(mut commands: Commands) {
    let display = Vec2::new(10240., 6400.);

    // Arena Bounds
    let path = ShapePath::new()
        .move_to(Vec2::new(-(display.x / 2.0), -(display.y / 2.0)))
        .line_to(Vec2::new(display.x / 2.0, -(display.y / 2.0)))
        .line_to(Vec2::new(display.x / 2.0, display.y / 2.0))
        .line_to(Vec2::new(-(display.x / 2.0), display.y / 2.0))
        .close();

    commands.spawn((
        ShapeBuilder::with(&path)
            .fill(Fill::color(Color::srgb(0.15, 0.15, 0.15)))
            .stroke(Stroke::new(
                Color::Srgba(bevy::color::palettes::css::RED),
                10.0,
            ))
            .build(),
        Transform::from_xyz(0., 0., -1.),
        ArenaMarker,
    ));

    // Arena Zero axis marks
    let axis = ShapePath::new()
        .move_to(Vec2::new(-(display.x / 2.0), 0.0))
        .line_to(Vec2::new(display.x / 2.0, 0.0))
        .move_to(Vec2::new(0.0, -(display.y / 2.0)))
        .line_to(Vec2::new(0.0, display.y / 2.0));

    commands.spawn((
        ShapeBuilder::with(&axis)
            .stroke(Stroke::new(Color::srgb(0.40, 0.40, 0.40), 5.))
            .build(),
        Transform::from_xyz(0., 0., -0.9),
        ArenaMarker,
    ));

    // Axis Labels
    commands.spawn((
        Text2d::new("+X"),
        Transform::from_xyz(display.x / 2.0 + 150., 0., -0.9).with_scale(Vec3::splat(10.)),
        ArenaMarker,
    ));
    commands.spawn((
        Text2d::new("-X"),
        Transform::from_xyz(-(display.x / 2.0 + 150.), 0., -0.9).with_scale(Vec3::splat(10.)),
        ArenaMarker,
    ));
    commands.spawn((
        Text2d::new("+Y"),
        Transform::from_xyz(0., display.y / 2.0 + 150., -0.9).with_scale(Vec3::splat(10.)),
        ArenaMarker,
    ));
    commands.spawn((
        Text2d::new("-Y"),
        Transform::from_xyz(0., -(display.y / 2.0 + 150.), -0.9).with_scale(Vec3::splat(10.)),
        ArenaMarker,
    ));

    // Rotation Angle Compass
    let compass = ShapePath::new()
        .move_to(Vec2::new(-200., 0.))
        .line_to(Vec2::new(200., 0.))
        .move_to(Vec2::new(0.0, -200.))
        .line_to(Vec2::new(0.0, 200.));

    let base = Vec3::new(4250., 2250., -0.8);
    commands
        .spawn((
            ShapeBuilder::with(&compass)
                .stroke(Stroke::new(Color::srgb(0.80, 0.80, 0.80), 5.))
                .build(),
            Transform::from_translation(base),
            ArenaMarker,
        ))
        .with_children(|parent| {
            for angle in [0, 64, 128, 192] {
                let hdr = AbsRot(angle).to_quat().mul_vec3(Vec3::Y * 400.);

                parent.spawn((
                    Text2d::new(format!("{angle}")),
                    Transform::from_translation(hdr).with_scale(Vec3::splat(10.)),
                    ArenaMarker,
                ));
            }
        });
}

pub(super) fn arena_grid(mut gizmos: Gizmos) {
    // Grid so that we have a background for the camera to deal with
    gizmos
        .grid_2d(
            Isometry2d::IDENTITY,
            UVec2::new(16, 16),
            Vec2::new(800., 800.),
            LinearRgba::gray(0.05),
        )
        .outer_edges();
}
