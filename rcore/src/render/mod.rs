use bevy::dev_tools::fps_overlay::FpsOverlayConfig;
use bevy::dev_tools::fps_overlay::FpsOverlayPlugin;
use bevy::dev_tools::fps_overlay::FrameTimeGraphConfig;
use bevy::prelude::*;
use bevy::text::FontSmoothing;
use bevy_prototype_lyon::plugin::BuildShapes;
use bevy_prototype_lyon::prelude::Shape;
use bevy_prototype_lyon::prelude::ShapePlugin;

// Input for camera
use bevy::input::keyboard::KeyCode;
use bevy::input::mouse::{AccumulatedMouseScroll, MouseButton, MouseScrollUnit};
use bevy::input::ButtonInput;
use bevy::picking::events::{Drag, DragEnd, DragStart, Pointer};
use bevy::color::palettes::css::FUCHSIA;

mod arena;
mod gizmo;
mod shape;

use arena::arena_bounds_setup;
use shape::get_radar;
use shape::get_ship;

use crate::ARENA;
use crate::ARENA_SCALE;
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
            .add_systems(Startup, arena_bounds_setup)
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
                    // Arena gizmo
                    arena::arena_grid,
                ),
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
            crate::weapon::DISTANCE as f32 / ARENA_SCALE,
            bevy::color::palettes::css::RED,
        );

        // Check if fade has expired?
        // if so, despawn
        if render.fade.is_finished() {
            commands.entity(entity).despawn();
        }
    }
}

// Camera plugin to keep the camera system segmented off from render for now, might fold it in
pub struct CameraPlugin;
impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, |mut commands: Commands| {
            commands.spawn((
                Camera2d,
                MainCamera {
                    move_speed: 1.0,
                    max_speed: 2.0,
                },
            ));

            commands.spawn((
                Transform::default(),
                CameraTarget,
            ));
        })
        .add_systems(
            Update,
            render_camera_target,
        )
        .add_systems(
            Update,
            (update_camera_target, update_main_camera).chain()
        );
    }
}

// Camera target movement factor
const TARGET_SPEED: f32 = 100.;

// Snap to location rate
const CAMERA_DECAY_RATE: f32 = 2.;

#[derive(Component)]
struct MainCamera {
    pub move_speed: f32,
    pub max_speed: f32,
}

#[derive(Component)]
struct CameraTarget;

fn render_camera_target(
    mut gizmos: Gizmos,
    query: Query<&Transform, With<CameraTarget>>,
) {
    for tran in query.iter() {
        gizmos.cross_2d(
            tran.translation.truncate(),
            12.,
            FUCHSIA,
        );
    }
}

fn update_camera_target(
    time: Res<Time<Real>>,
    mut target: Single<&mut Transform, With<CameraTarget>>,
    key_input: Res<ButtonInput<KeyCode>>,
) {
    let mut direction = Vec2::ZERO;

    if key_input.pressed(KeyCode::KeyW) {
        direction.y += 1.;
    }
    if key_input.pressed(KeyCode::KeyS) {
        direction.y -= 1.;
    }
    if key_input.pressed(KeyCode::KeyA) {
        direction.x -= 1.;
    }
    if key_input.pressed(KeyCode::KeyD) {
        direction.x += 1.;
    }

    let move_delta = direction.normalize_or_zero() * TARGET_SPEED * time.delta_secs();
    target.translation += move_delta.extend(0.);

    // Check if it will exceed any of the arena boundaries, if so, clamp it.
    let z = target.translation.z;

    target.translation = target.translation.clamp(
        ((ARENA * IVec2::NEG_ONE).as_vec2() / (ARENA_SCALE * 2.)).extend(-z - 1.),
        (ARENA.as_vec2() / (ARENA_SCALE * 2.)).extend(z + 1.),
    );
}

fn update_main_camera(
    time: Res<Time>,
    target: Single<&Transform, (With<CameraTarget>, Without<Camera>)>,
    mut camera: Single<(&mut Transform, &MainCamera), (With<Camera>, Without<CameraTarget>)>,
) {
    let Vec3 { x, y, .. } = target.translation;
    let (mut tran, cam) = camera.into_inner();
    let direction = Vec3::new(x, y, tran.translation.z);
    tran.translation.smooth_nudge(&direction, CAMERA_DECAY_RATE, time.delta_secs());
}
