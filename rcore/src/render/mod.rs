use bevy::diagnostic::DiagnosticsStore;
use bevy::diagnostic::FrameTimeDiagnosticsPlugin;
use bevy::prelude::*;
use bevy::render::view::screenshot::Screenshot;
use bevy::render::view::screenshot::save_to_disk;
use bevy::time::common_conditions::on_timer;

pub mod camera;
mod gizmo;
pub mod grid;
pub mod shape;
mod transform;

use crate::FieldProfile;
use crate::FieldSource;
use crate::assets::ship::Class;
use crate::assets::ship::Mount;
use crate::assets::ship::ShipClass;
use crate::render::transform::TransformSyncPlugin;
use crate::ship::AttachedTo;
use crate::time::TimeMsg;
use crate::weapon::RenderDebugWarhead;
use crate::weapon::RenderDebugWeapon;

// Render plugin to make it easy to keep render segmented off
// TODO: may want to double check/have some system handle the Z axis of transform since sim is on
// position, so the Z axis will be zero, and we want to be able to use Z axis to layer assets, but
// that can probs be handled in render
pub struct RenderPlugin;
impl Plugin for RenderPlugin {
    fn build(&self, app: &mut App) {
        // Sync Sim Position to Render Transform
        app.add_plugins(TransformSyncPlugin)
            // FPS
            .add_plugins(FrameTimeDiagnosticsPlugin::default())
            .add_systems(
                Update,
                render_fps.run_if(on_timer(core::time::Duration::from_millis(100))),
            )
            // Grid systems
            .add_plugins(grid::GridPlugin { debug: true })
            // Startup setup
            .add_systems(
                Startup,
                (arena_axis_setup, fps_ui.spawn(), time_control_ui.spawn()),
            )
            // Ship Class visuals
            .init_resource::<ShipMaterial>()
            .add_systems(Update, (sync_ship_visuals, sync_mount_visuals))
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

#[derive(Component, Default, Clone)]
struct FpsMarker;

fn fps_ui() -> impl Scene {
    bsn! {
        Node {
            position_type: PositionType::Absolute,
            left: Val::Px(5.),
            top: Val::Px(5.),
        }
        Children [
            FpsMarker
            Text::new("FPS: --")
            TextFont { font_size: FontSize::Px(18.) }
            TextColor(Color::srgb(1.0, 0.0, 0.0))
        ]
    }
}

#[expect(clippy::redundant_closure_for_method_calls)]
fn render_fps(diagnostics: Res<DiagnosticsStore>, mut text: Single<&mut Text, With<FpsMarker>>) {
    if let Some(fps) = diagnostics
        .get(&FrameTimeDiagnosticsPlugin::FPS)
        .and_then(|d| d.smoothed())
    {
        text.0 = format!("FPS: {fps:.0}");
    }
}

// Initial attempt of making a Ship Material (for permitting different color for factions)
#[derive(Resource)]
struct ShipMaterial(Handle<ColorMaterial>);

impl FromWorld for ShipMaterial {
    fn from_world(world: &mut World) -> Self {
        Self(world.add_asset(ColorMaterial::default()))
    }
}

// Resync the hull visuals when the class data/asset changes
// (mostly updating the meshes)
#[expect(clippy::type_complexity)]
fn sync_ship_visuals(
    ships: Query<(Entity, &Class), Or<(Changed<Class>, AssetChanged<Class>)>>,
    ship_class: Res<Assets<ShipClass>>,
    material: Res<ShipMaterial>,
    mut commands: Commands,
) {
    for (entity, class) in &ships {
        let Some(class) = ship_class.get(&class.0) else {
            continue;
        };
        update_mesh(&mut commands, entity, class.meshes.hull.as_ref(), &material);
    }
}

// Mounts are things like turret + radar, and parent holds this data.
fn sync_mount_visuals(
    mut events: MessageReader<AssetEvent<ShipClass>>,
    added: Query<(Entity, &Mount, &AttachedTo), Added<Mount>>,
    mounts: Query<(Entity, &Mount, &AttachedTo)>,
    parents: Query<&Class>,
    ship_class: Res<Assets<ShipClass>>,
    material: Res<ShipMaterial>,
    mut commands: Commands,
) {
    // TODO: find a better way than always having to drain the MessageReader all the time...
    let changed = events.read().count() > 0;
    let targets: Box<dyn Iterator<Item = _>> = if changed {
        Box::new(mounts.iter())
    } else {
        Box::new(added.iter())
    };

    for (entity, mount, attached) in targets {
        let Some(class) = parents
            .get(attached.0)
            .ok()
            .and_then(|class| ship_class.get(&class.0))
        else {
            continue;
        };
        update_mesh(
            &mut commands,
            entity,
            class.meshes.mounts.get(&mount.0),
            &material,
        );
    }
}

fn update_mesh(
    commands: &mut Commands,
    entity: Entity,
    mesh: Option<&Handle<Mesh>>,
    material: &ShipMaterial,
) {
    match mesh {
        Some(mesh) => {
            commands
                .entity(entity)
                .insert((Mesh2d(mesh.clone()), MeshMaterial2d(material.0.clone())));
        }
        None => {
            commands
                .entity(entity)
                .remove::<(Mesh2d, MeshMaterial2d<ColorMaterial>)>();
        }
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
            render.origin.as_vec2(),
            render.target.as_vec2(),
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
            Isometry2d::from_translation(render.origin.as_vec2()),
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

// We re-sync the star visuals if:
//  - Changed<FieldSource> - the star got respawned
//  - AssetChanged<FieldSource> - the prf was reloaded (update the visual to match new prf)
#[expect(clippy::type_complexity)]
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
