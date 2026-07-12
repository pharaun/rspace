use avian2d::schedule::PhysicsSystems;
use bevy::prelude::*;

// Input for camera
use bevy::color::palettes::css::FUCHSIA;
use bevy::input::ButtonInput;
use bevy::input::keyboard::KeyCode;
use bevy::input::mouse::{AccumulatedMouseScroll, MouseScrollUnit};
use bevy::window::PrimaryWindow;

use crate::movement::interpolate_movement;
use crate::radar::interpolate_arc;
use crate::rotation::interpolate_rotation;

// Camera plugin to keep the camera system segmented off from render for now, might fold it in
pub struct CameraPlugin;
impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, camera_setup)
            // TODO: System ordering:
            // 1. update the input/target/etc in Update (or Fixed Update)
            .add_systems(Update, (render_camera_focus, update_camera_focus))
            // 2. PostUpdate (after iterpolotation) do the camera-rig updates
            .add_systems(
                PostUpdate,
                (
                    resolve_follow_mode.run_if(rig_in_follow_mode),
                    // constrain_camera,
                    apply_camera_rig,
                )
                    .chain()
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
    // Mouse can return scroll in terms of line or pixels...
    zoom_step_per_line: f32,
    zoom_step_per_pixel: f32,
    zoom_decay_rate: f32,
    // Edge panning config
    // The margin around the edge of the screen for panning, if
    // camera is barely into it, move slowly but if its far into it, move faster
    edge_margin: Vec2,
    edge_speed: f32,
    edge_speed_max: f32,
}

pub fn camera_setup(mut commands: Commands) {
    // Spawn in the main camera rig and put it at 0,0 by default
    commands.spawn(CameraRig {
        mode: CameraMode::Free,
        config: CameraConfig {
            target_speed: 100.0,
            decay_rate: 2.0,
            // TODO: not sure if this is the correct move to have a deadzone
            // on ship follow, set it to 0 for now
            deadzone_radius: 0.0,
            // Tuning:
            //  - You want ~12-24 ticks from min->max zoom
            //      * zoom_step_per_line ~= (zoom_max - zoom_min) / (~12-24)
            //  - 0.14 ~ 0.38 for 1.1x to 1.3x steps seem to be a sweet spot?
            //      2^0.25 ~= 1.19
            //      animation (via zoom decay can affect the feeling of this too)
            zoom_clamp: (-2.0, 2.0),
            // 0.25 per step ~= 16 ticks with a (-2 <-> 2) zoom range (0.25 to 4x)
            zoom_step_per_line: 0.25,
            // Seems like firefox uses ~38px, chrome ~= 50px for this
            zoom_step_per_pixel: 0.25 / 45.0,
            // 4 ~= supcom floaty, 8 ~= smooth, 12 ~= snappy but has some animation, 15 ~= crisp
            zoom_decay_rate: 12.0,
            // Margin around the edge of the window
            edge_margin: Vec2::splat(30.0),
            edge_speed: 0.05,
            edge_speed_max: 100.0,
        },
        focus: Vec2::new(0.0, 0.0),
        zoom: 0.0,
    });
}

fn render_camera_focus(mut gizmos: Gizmos, query: Query<&CameraRig, With<Camera2d>>) {
    for rig in query.iter() {
        gizmos.cross_2d(rig.focus, 12., FUCHSIA);
    }
}

fn update_camera_focus(
    time: Res<Time<Real>>,
    mut camera: Single<&mut CameraRig, With<Camera2d>>,
    key_input: Res<ButtonInput<KeyCode>>,
    mouse_wheel_input: Res<AccumulatedMouseScroll>,
    window_input: Single<&Window, With<PrimaryWindow>>,
) {
    let conf = camera.config;

    // Deal with zoom from the mouse (and ideally a keyboard too)
    if mouse_wheel_input.delta != Vec2::ZERO {
        let step = match mouse_wheel_input.unit {
            MouseScrollUnit::Line => mouse_wheel_input.delta.y * conf.zoom_step_per_line,
            MouseScrollUnit::Pixel => mouse_wheel_input.delta.y * conf.zoom_step_per_pixel,
        };

        camera.zoom = (camera.zoom - step).clamp(conf.zoom_clamp.0, conf.zoom_clamp.1);
    }

    // Deal with direction
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
    } else {
        // keyboard was not moved, check the cursor for edge panning
        if let Some(cursor_position) = window_input.cursor_position() {
            // Coordination:
            // Top Left - (0,0)
            // Bottom Right - window.size()
            let mut direction = Vec2::ZERO;
            let mut rel_speed: f32 = 1.0;
            let size = window_input.size();

            if cursor_position.x <= conf.edge_margin.x {
                // Left
                let ratio = cursor_position.x / conf.edge_margin.x;
                rel_speed = rel_speed.min(ratio);
                direction.x -= 1.0;
            }
            if cursor_position.x >= size.x - conf.edge_margin.x {
                // Right
                let offset = size.x - cursor_position.x;
                let ratio = offset / conf.edge_margin.x;
                rel_speed = rel_speed.min(ratio);
                direction.x += 1.0;
            }
            if cursor_position.y <= conf.edge_margin.y {
                // Up
                let ratio = cursor_position.y / conf.edge_margin.y;
                rel_speed = rel_speed.min(ratio);
                direction.y += 1.0;
            }
            if cursor_position.y >= size.y - conf.edge_margin.y {
                // Down
                let offset = size.y - cursor_position.y;
                let ratio = offset / conf.edge_margin.y;
                rel_speed = rel_speed.min(ratio);
                direction.y -= 1.0;
            }

            // Apply motion
            if direction != Vec2::ZERO && camera.mode == CameraMode::Free {
                let speed = conf.edge_speed_max / rel_speed.max(conf.edge_speed);
                camera.focus += direction.normalize_or_zero() * speed * time.delta_secs();
            }
        }
    }
}

// Utility for run_if
fn rig_in_follow_mode(rig: Single<&CameraRig>) -> bool {
    matches!(rig.mode, CameraMode::Follow(_))
}

fn resolve_follow_mode(
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
            }
            Err(_) => {
                // Follow target entity.... is gone, it probs got despawned
                // Go into free-mode here
                camera.mode = CameraMode::Free;
            }
        },
        _ => (),
    }
}

fn apply_camera_rig(
    time: Res<Time>,
    camera: Single<(&mut Transform, &mut Projection, &CameraRig), With<Camera2d>>,
) {
    let (mut tran, mut proj, cam) = camera.into_inner();
    let conf = cam.config;
    let cam_z = tran.translation.z;

    // Actually apply the camera rig settings to the viewpoint camera
    tran.translation
        .smooth_nudge(&cam.focus.extend(cam_z), conf.decay_rate, time.delta_secs());

    // Apply camera zoom
    if let Projection::Orthographic(ref mut ortho) = *proj {
        // Probs can do this better?
        let mut old_scale = ortho.scale.log2();
        old_scale.smooth_nudge(&cam.zoom, conf.zoom_decay_rate, time.delta_secs());
        ortho.scale = old_scale.exp2();
    }
}
