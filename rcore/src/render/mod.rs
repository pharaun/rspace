use bevy::dev_tools::fps_overlay::FpsOverlayConfig;
use bevy::dev_tools::fps_overlay::FpsOverlayPlugin;
use bevy::dev_tools::fps_overlay::FrameTimeGraphConfig;
use bevy::prelude::*;
use bevy::text::FontSmoothing;
use bevy_prototype_lyon::plugin::BuildShapes;
use bevy_prototype_lyon::prelude::Shape;
use bevy_prototype_lyon::prelude::ShapePlugin;
use avian2d::schedule::PhysicsSystems;

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
use crate::radar::interpolate_arc;
use crate::rotation::interpolate_rotation;
use crate::movement::interpolate_movement;
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
        app.add_systems(Startup, camera_setup)
        // TODO: System ordering:
        // 1. update the input/target/etc in Update (or Fixed Update)
        .add_systems(
            Update, (
                render_camera_focus,
                update_camera_focus,
            )
        )
        // 2. PostUpdate (after iterpolotation) do the camera-rig updates
        .add_systems(
            PostUpdate, (
                resolve_follow_mode.run_if(rig_in_follow_mode),
                // constrain_camera,
                apply_camera_rig,
            ).chain()
            .after(interpolate_arc)
            .after(interpolate_movement)
            .after(interpolate_rotation)
            .after(PhysicsSystems::Last)
            .before(TransformSystems::Propagate),
        );
    }
}


// Camera Todo:
// 1. make it so that the camera stays within the area bounds instead of just the target
// 2. add better control (ie pan/drag), maybe edge move, then keyboard - wasd/arrow movement
// 3. Add an ability to follow an target
// 4. make it so that when the camera is about to hit the areana edge, slow down to a stop so its
//    not jarring
// 5. Make follow a target have some slight delay so its not super jarring but also don't want to
//    have large swings/drifts
// 6. figure out how to zoom in/out so that you can zoom out to view the whole arena and into a
//    single ship
// 7. deferred for now (but probs eventually a way to zoom out to a mini-map or a mini-map view)
// 8. make the target invisible or not have a camera target you control, but the whole follow a
//    target idea applies so might still want one just to have that "follow target" thing.

#[derive(Component)]
#[require(Camera2d)]
pub struct CameraRig {
    pub mode: CameraMode,
    config: CameraConfig,
    // The camera target focus
    focus: Vec2,
    // Log-scale zoom. 0 = default, +1 = 2x out, -1 = 2x in
    zoom: f32,
}

#[derive(Debug, PartialEq)]
pub enum CameraMode {
    // Player controlled positioning
    Free,
    // Follow a entity
    Follow(Entity),
    // Additional Modes such as: Auto
    // TODO: Auto mode ideas:
}

#[derive(Debug, Clone, Copy)]
pub struct CameraConfig {
    // Camera target movement factor
    target_speed: f32,
    // Snap to location rate
    decay_rate: f32,
    // Follow deadzone (squared)
    deadzone_radius: f32,
    // Zoom min/max
    zoom_clamp: (f32, f32),
}

pub fn camera_setup(
    mut commands: Commands,
) {
    // Spawn in the main camera rig and put it at 0,0 by default
    commands.spawn(CameraRig {
        mode: CameraMode::Free,
        config: CameraConfig {
            target_speed: 100.0,
            decay_rate: 2.0,
            // TODO: not sure if this is the correct move to have a deadzone
            // on ship follow, set it to 0 for now
            deadzone_radius: 0.0,
            zoom_clamp: (0.25, 2.0),
        },
        focus: Vec2::new(0.0, 0.0),
        zoom: 0.0,
    });
}

fn render_camera_focus(
    mut gizmos: Gizmos,
    query: Query<&CameraRig, With<Camera2d>>,
) {
    for rig in query.iter() {
        gizmos.cross_2d(
            rig.focus,
            12.,
            FUCHSIA,
        );
    }
}

fn update_camera_focus(
    time: Res<Time<Real>>,
    mut camera: Single<&mut CameraRig, With<Camera2d>>,
    key_input: Res<ButtonInput<KeyCode>>,
) {
    let conf = camera.config;
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

    // Update the camera rig only if a key was pressed
    if direction != Vec2::ZERO {
        camera.mode = CameraMode::Free;
        camera.focus += direction.normalize_or_zero() * conf.target_speed * time.delta_secs();
    }
}

// Utility for run_if
fn rig_in_follow_mode(rig: Single<&CameraRig>) -> bool {
    matches!(rig.mode, CameraMode::Follow(_))
}

fn resolve_follow_mode(
    time: Res<Time<Real>>,
    follow: Query<&Transform, Without<CameraRig>>,
    mut camera: Single<&mut CameraRig, With<Camera2d>>,
) {
    let conf = camera.config;

    // Handle finding out where a Follow(entity) is at and update
    // the rig to focus on its current position
    match camera.mode {
        CameraMode::Follow(target) => match follow.get(target) {
            Ok(tran) => {
                let pos = tran.translation.truncate();
                // TODO: Decide how much we want to handle lookahead or not
                if pos.distance_squared(camera.focus) > conf.deadzone_radius {
                    camera.focus = pos;
                }
            },
            Err(_) => {
                // Follow target entity.... is gone, it probs got despawned
                // Go into free-mode here
                camera.mode = CameraMode::Free;
            },
        },
        _ => (),
    }
}

fn apply_camera_rig(
    time: Res<Time>,
    mut camera: Single<(&mut Transform, &CameraRig), With<Camera2d>>,
) {
    let (mut tran, cam) = camera.into_inner();
    let conf = cam.config;
    let cam_z = tran.translation.z;

    // Actually apply the camera rig settings to the viewpoint camera
    tran.translation.smooth_nudge(
        &cam.focus.extend(cam_z),
        conf.decay_rate,
        time.delta_secs(),
    );
}
