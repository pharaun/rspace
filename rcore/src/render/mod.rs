use bevy::dev_tools::fps_overlay::FpsOverlayConfig;
use bevy::dev_tools::fps_overlay::FpsOverlayPlugin;
use bevy::dev_tools::fps_overlay::FrameTimeGraphConfig;
use bevy::prelude::*;
use bevy::text::FontSmoothing;
use bevy_prototype_lyon::plugin::BuildShapes;
use bevy_prototype_lyon::prelude::Shape;
use bevy_prototype_lyon::prelude::ShapePlugin;

mod arena;
pub mod camera;
mod gizmo;
mod shape;

use arena::arena_bounds_setup;
use shape::get_radar;
use shape::get_ship;

use crate::time::TimeMsg;
use crate::radar::Radar;
use crate::ship::Ship;
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
            // Startup setup (ie arena)
            .add_systems(
                Startup,
                (arena_bounds_setup, setup_time_control),
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
                    arena::arena_grid,
                    render_time_control,
                ),
            )
            // Time controls
            .add_systems(
                PreUpdate,
                manage_time_control,
            )
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

fn manage_time_control(
    key_input: Res<ButtonInput<KeyCode>>,
    time: Res<Time<Virtual>>,
    mut message: MessageWriter<TimeMsg>,
) {
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

#[derive(Component)]
struct TimeControlMarker;

fn setup_time_control(
    mut commands: Commands,
) {
    let display = Vec2::new(10240., 6400.);

    commands.spawn((
        Text2d::new("1x"),
        Transform::from_xyz(
            -(display.x / 2.0 + 400.),
            -(display.y / 2.0 + 180.),
            -0.9
        ).with_scale(Vec3::splat(8.)),
        TimeControlMarker,
    ));
}

fn render_time_control(
    time: Res<Time<Virtual>>,
    mut text: Single<&mut Text2d, With<TimeControlMarker>>,
) {
    let exp = match time.relative_speed().log2().round() as i8 {
        -4 => "1/16x",
        -3 => " 1/8x",
        -2 => " 1/4x",
        -1 => " 1/2x",
        0  => "   1x",
        1  => "   2x",
        2  => "   4x",
        3  => "   8x",
        4  => "  16x",
        _  => " Err ",
    };
    let pause = if time.is_paused() {
        " Paused "
    } else {
        " Running"
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
