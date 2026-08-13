use bevy_prototype_lyon::geometry::ShapeBuilderBase as _;
use bevy_prototype_lyon::prelude::ShapeBuilder;
use bevy_prototype_lyon::prelude::ShapePath;
use bevy_prototype_lyon::prelude::Stroke;

use bevy::dev_tools::fps_overlay::FpsOverlayConfig;
use bevy::dev_tools::fps_overlay::FpsOverlayPlugin;
use bevy::dev_tools::fps_overlay::FrameTimeGraphConfig;
use bevy::prelude::*;
use bevy::render::view::screenshot::Screenshot;
use bevy::render::view::screenshot::save_to_disk;
use bevy::text::FontSmoothing;
use bevy_prototype_lyon::plugin::BuildShapes;
use bevy_prototype_lyon::prelude::Shape;
use bevy_prototype_lyon::prelude::ShapePlugin;

pub mod camera;
mod gizmo;
pub mod grid;
mod shape;

use shape::get_radar;
use shape::get_ship;

use crate::AbsRot;
use crate::FieldProfile;
use crate::FieldSource;
use crate::radar::Radar;
use crate::ship::Ship;
use crate::time::TimeMsg;
use crate::weapon::RenderDebugWarhead;
use crate::weapon::RenderDebugWeapon;

// Render plugin to make it easy to keep render segmented off
pub struct RenderPlugin;
impl Plugin for RenderPlugin {
    fn build(&self, app: &mut App) {
        // Graphics (lyon)
        app.add_plugins(ShapePlugin)
            // FPS
            .add_plugins(FpsOverlayPlugin {
                config: FpsOverlayConfig {
                    text_config: TextFont {
                        // Here we define size of our overlay
                        font_size: FontSize::Px(18.0),
                        // If we want, we can use a custom font
                        font: default(),
                        // We could also disable font smoothing,
                        font_smoothing: FontSmoothing::default(),
                        ..default()
                    },
                    // We can also change color of the overlay
                    text_color: Color::srgb(1.0, 0.0, 0.0),
                    // We can also set the refresh interval for the FPS counter
                    refresh_interval: core::time::Duration::from_millis(100),
                    enabled: true,
                    frame_time_graph_config: FrameTimeGraphConfig {
                        enabled: false,
                        // The minimum acceptable fps
                        min_fps: 30.0,
                        // The target fps
                        target_fps: 60.0,
                    },
                },
            })
            // Grid systems
            .add_plugins(grid::GridPlugin { debug: true })
            // Startup setup
            .add_systems(
                Startup,
                (
                    arena_compass_setup,
                    arena_axis_setup,
                    time_control_ui.spawn(),
                ),
            )
            // Handle assigning a lyon shape to entities
            .add_systems(
                PostUpdate,
                (apply_ship_shape, apply_radar_shape).before(BuildShapes),
            )
            // Gizmos
            .add_systems(
                Update,
                (
                    gizmo::movement,
                    gizmo::radar,
                    gizmo::arc,
                    gizmo::rotation,
                    gizmo::health,
                    gizmo::shield_health,
                    render_time_control,
                ),
            )
            // Add stars
            .add_systems(Update, sync_star_visuals)
            // Time controls
            .add_systems(PreUpdate, (manage_time_control, manage_screenshots))
            // Temporary weapon render via gizmos
            .add_systems(
                RunFixedMainLoop,
                (
                    render_debug_weapon.in_set(RunFixedMainLoopSystems::AfterFixedMainLoop),
                    render_debug_warhead.in_set(RunFixedMainLoopSystems::AfterFixedMainLoop),
                ),
            );
    }
}

fn manage_screenshots(
    key_input: Res<ButtonInput<KeyCode>>,
    mut counter: Local<u32>,
    mut commands: Commands,
) {
    if key_input.just_pressed(KeyCode::Backslash) {
        let path = format!("./screenshot-{}.png", *counter);
        *counter += 1;
        commands
            .spawn(Screenshot::primary_window())
            .observe(save_to_disk(path));
    }
}

fn manage_time_control(
    key_input: Res<ButtonInput<KeyCode>>,
    time: Res<Time<Virtual>>,
    mut message: MessageWriter<TimeMsg>,
) {
    #[expect(clippy::cast_possible_truncation)]
    let exp = time.relative_speed().log2().round() as i8;

    if key_input.just_pressed(KeyCode::Space) {
        message.write(TimeMsg::Pause(!time.is_paused()));
    }
    if key_input.just_pressed(KeyCode::BracketLeft) {
        message.write(TimeMsg::Speed(exp - 1));
    }
    if key_input.just_pressed(KeyCode::BracketRight) {
        message.write(TimeMsg::Speed(exp + 1));
    }
    if key_input.just_pressed(KeyCode::Period) {
        message.write(TimeMsg::Step(1));
    }
}

#[derive(Component, Default, Clone)]
struct TimeControlMarker;

fn time_control_ui() -> impl Scene {
    bsn! {
        Node {
            position_type: PositionType::Absolute,
            left: Val::Px(20.),
            bottom: Val::Px(20.),
        }
        Children [
            TimeControlMarker
            Text::new("   1x -  Running")
            TextFont { font_size: FontSize::Px(18.) }
        ]
    }
}

fn render_time_control(
    time: Res<Time<Virtual>>,
    mut text: Single<&mut Text, With<TimeControlMarker>>,
) {
    #[expect(clippy::cast_possible_truncation)]
    let exp = match time.relative_speed().log2().round() as i8 {
        -4 => "1/16x",
        -3 => " 1/8x",
        -2 => " 1/4x",
        -1 => " 1/2x",
        0 => "   1x",
        1 => "   2x",
        2 => "   4x",
        3 => "   8x",
        4 => "  16x",
        _ => " Err ",
    };
    let pause = if time.is_paused() {
        "Paused "
    } else {
        "Running"
    };
    text.0 = format!("{exp} - {pause}");
}

fn apply_ship_shape(query: Query<(Entity, &Ship), Without<Shape>>, mut commands: Commands) {
    for (entity, ship) in query.iter() {
        commands.entity(entity).insert(get_ship(
            ship.0,
            bevy::color::palettes::css::GREEN,
            bevy::color::palettes::css::BLACK,
        ));
    }
}

fn apply_radar_shape(query: Query<(Entity, &Radar), Without<Shape>>, mut commands: Commands) {
    for (entity, _) in query.iter() {
        commands
            .entity(entity)
            .insert(get_radar(bevy::color::palettes::css::MAROON));
    }
}

#[expect(clippy::needless_pass_by_value)]
fn render_debug_weapon(
    mut gizmos: Gizmos,
    mut commands: Commands,
    mut query: Query<(Entity, &mut RenderDebugWeapon)>,
    time: Res<Time>,
) {
    for (entity, mut render) in query.iter_mut() {
        render.fade.tick(time.delta());

        // TODO: render the beam thicker
        gizmos.line_2d(
            render.origin,
            render.target,
            bevy::color::palettes::css::RED,
        );

        // Check if fade has expired?
        // if so, despawn
        if render.fade.is_finished() {
            commands.entity(entity).despawn();
        }
    }
}

#[expect(clippy::needless_pass_by_value)]
fn render_debug_warhead(
    mut gizmos: Gizmos,
    mut commands: Commands,
    mut query: Query<(Entity, &mut RenderDebugWarhead)>,
    time: Res<Time>,
) {
    for (entity, mut render) in query.iter_mut() {
        render.fade.tick(time.delta());

        // TODO: render the beam thicker
        gizmos.circle_2d(
            Isometry2d::from_translation(render.origin),
            crate::weapon::DISTANCE as f32,
            bevy::color::palettes::css::RED,
        );

        // Check if fade has expired?
        // if so, despawn
        if render.fade.is_finished() {
            commands.entity(entity).despawn();
        }
    }
}

fn arena_axis_setup(mut commands: Commands) {
    let display = Vec2::new(10240., 6400.);
    commands.spawn((
        Text2d::new("+X"),
        Transform::from_xyz(display.x / 2.0 + 150., 0., -0.9).with_scale(Vec3::splat(10.)),
    ));
    commands.spawn((
        Text2d::new("-X"),
        Transform::from_xyz(-(display.x / 2.0 + 150.), 0., -0.9).with_scale(Vec3::splat(10.)),
    ));
    commands.spawn((
        Text2d::new("+Y"),
        Transform::from_xyz(0., display.y / 2.0 + 150., -0.9).with_scale(Vec3::splat(10.)),
    ));
    commands.spawn((
        Text2d::new("-Y"),
        Transform::from_xyz(0., -(display.y / 2.0 + 150.), -0.9).with_scale(Vec3::splat(10.)),
    ));
}

fn arena_compass_setup(mut commands: Commands) {
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
        ))
        .with_children(|parent| {
            for angle in [0, 64, 128, 192] {
                let hdr = AbsRot(angle).to_quat().mul_vec3(Vec3::Y * 400.);

                parent.spawn((
                    Text2d::new(format!("{angle}")),
                    Transform::from_translation(hdr).with_scale(Vec3::splat(10.)),
                ));
            }
        });
}

// We re-sync the star visuals if:
//  - Changed<FieldSource> - the star got respawned
//  - AssetChanged<FieldSource> - the prf was reloaded (update the visual to match new prf)
fn sync_star_visuals(
    stars: Query<(Entity, &FieldSource), Or<(Changed<FieldSource>, AssetChanged<FieldSource>)>>,
    profiles: Res<Assets<FieldProfile>>,
    mut commands: Commands,
) {
    for (entity, field_source) in &stars {
        let Some(prof) = profiles.get(&field_source.0) else {
            continue;
        };

        // Insert will replace the Sprite already on the star with a new one if it already had one
        commands.entity(entity).insert(Sprite::from_color(
            prof.color,
            Vec2::splat(prof.body_radius * 2.0),
        ));
    }
}
