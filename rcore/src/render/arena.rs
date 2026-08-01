use bevy_prototype_lyon::geometry::ShapeBuilderBase as _;
use bevy_prototype_lyon::prelude::ShapeBuilder;
use bevy_prototype_lyon::prelude::ShapePath;
use bevy_prototype_lyon::prelude::Stroke;

use bevy::camera::CameraUpdateSystems;
use bevy::material::descriptor::RenderPipelineDescriptor;
use bevy::material::specialize::SpecializedMeshPipelineError;
use bevy::math::DVec2;
use bevy::mesh::MeshVertexBufferLayoutRef;
use bevy::prelude::*;
use bevy::render::render_resource::AsBindGroup;
use bevy::render::render_resource::ShaderType;
use bevy::shader::ShaderRef;
use bevy::sprite_render::AlphaMode2d;
use bevy::sprite_render::Material2d;
use bevy::sprite_render::Material2dKey;
use bevy::sprite_render::Material2dPlugin;

use crate::math::AbsRot;

#[derive(Component)]
struct ArenaMarker;

fn arena_axis_setup(mut commands: Commands) {
    let display = Vec2::new(10240., 6400.);

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

pub(super) struct ArenaPlugin;
impl Plugin for ArenaPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(Material2dPlugin::<ArenaMaterial>::default())
            .add_systems(
                Startup,
                (
                    arena_compass_setup,
                    arena_axis_setup,
                    setup_grid_control,
                    load_util_shaders,
                    sprite_test,
                ),
            )
            .add_systems(PostStartup, spawn_background)
            .add_systems(Update, (debug_shader_control, render_grid_control))
            // Get projection from camera (CameraUpdateSystem), but apply it
            // before transform propagation for the scale to take.
            .add_systems(
                PostUpdate,
                update_background
                    .after(CameraUpdateSystems)
                    .before(TransformSystems::Propagate),
            );
    }
}

//--------------------------------------------------------------------------------
// Shaders
//--------------------------------------------------------------------------------

// NOTE: always pad to 16 bytes
// NOTE: If need any color, prefer to get it rust-side via
//  - Color::srgb(..).to_linear() and pass in via uniform
#[derive(ShaderType, Clone, Copy, Default)]
struct GridUniform {
    // Grid line colors
    pub axis: Vec4,
    pub major: Vec4,
    pub minor: Vec4,

    // Grid structure
    // - ratio == "cells between each major line"
    // - levels == "how many nested levels there are"
    pub ratio: u32,
    pub levels: u32,

    // Line render constraints in pixels
    pub half_width: f32,
    pub cell: f32,
    pub fade_min: f32,
    pub fade_max: f32,

    // Camera pos modulo coarse tier spacing.
    pub phase: Vec2,
    pub spacing: f32,

    // Debugging bits
    pub mode: u32,
    pub magic: u32,
}

#[derive(Asset, TypePath, AsBindGroup, Clone)]
struct ArenaMaterial {
    #[uniform(0)]
    pub grid: GridUniform,
}

impl Material2d for ArenaMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/arena.wgsl".into()
    }

    fn alpha_mode(&self) -> AlphaMode2d {
        AlphaMode2d::Opaque
    }

    fn specialize(
        descriptor: &mut RenderPipelineDescriptor,
        _layout: &MeshVertexBufferLayoutRef,
        _key: Material2dKey<Self>,
    ) -> Result<(), SpecializedMeshPipelineError> {
        if cfg!(debug_assertions) || cfg!(feature = "shader-debug") {
            descriptor.vertex.shader_defs.push("SHADER_DEBUG".into());
            if let Some(fragment) = &mut descriptor.fragment {
                fragment.shader_defs.push("SHADER_DEBUG".into());
            }
        }
        Ok(())
    }
}

#[expect(unused)]
#[derive(Resource)]
struct ShaderUtil(Handle<Shader>);

// This is to load up a util shader and hold on it, because if it is unloaded
// the other shaders that depends on it will error out at run time
// TODO: replace with 'load_shader_library!(app, "util.wgsl");' but that
// bakes it into the binary so don't need it rn.
fn load_util_shaders(asset_server: Res<AssetServer>, mut commands: Commands) {
    commands.insert_resource(ShaderUtil(asset_server.load("shaders/util.wgsl")));
}

#[derive(Component)]
struct ArenaBackground;

fn spawn_background(
    mut mesh: ResMut<Assets<Mesh>>,
    mut material: ResMut<Assets<ArenaMaterial>>,
    camera: Single<Entity, With<Camera2d>>,
    mut commands: Commands,
) {
    // This background rectangle is a child of camera so that
    // the global transform propagation takes care of moving it
    // with the camera. We do need to handle resizing it though.
    commands.spawn((
        ArenaBackground,
        Mesh2d(mesh.add(Rectangle::new(1.0, 1.0))),
        MeshMaterial2d(
            material.add(ArenaMaterial {
                grid: GridUniform {
                    //axis: Color::oklcha(0.4847, 0.1249, 253.08, 1.0).to_linear().to_vec4(),
                    axis: Color::oklcha(0.5232, 0.1404, 13.84, 1.0)
                        .to_linear()
                        .to_vec4(),
                    major: Color::srgb(0.4, 0.4, 0.4).to_linear().to_vec4(),
                    minor: Color::srgb(0.25, 0.25, 0.25).to_linear().to_vec4(),
                    // Grid Config
                    ratio: 10,
                    levels: 3,
                    // Line render
                    half_width: 1.5,
                    cell: 10.0,
                    fade_min: 10.0,
                    fade_max: 100.0, // fade_min * ratio
                    // Camera (zero value rn, will be recomputed)
                    phase: Vec2::ZERO,
                    spacing: 0.0,
                    // Debugging
                    mode: 0,
                    magic: 0xDEAD_C0DE,
                },
            }),
        ),
        // Place this as close to the back as possible
        Transform::from_xyz(0.0, 0.0, -999.0),
        ChildOf(*camera),
    ));
}

// Use Transform because this is updated post camera-rig, but we want freshest transform
// for shader code + background
#[expect(clippy::cast_possible_truncation)]
fn update_background(
    camera: Query<(&Camera, &Projection, &Transform), With<Camera2d>>,
    mut back: Query<
        (&mut Transform, &MeshMaterial2d<ArenaMaterial>),
        (With<ArenaBackground>, Without<Camera2d>),
    >,
    mut material: ResMut<Assets<ArenaMaterial>>,
) -> Result<()> {
    let (cam, projection, cam_tran) = camera.single()?;
    let Projection::Orthographic(ortho) = projection else {
        return Ok(());
    };
    let Some(viewport_px) = cam.physical_viewport_size() else {
        return Ok(());
    };

    // Backdrop sizing + margin
    let (mut background, material_handle) = back.single_mut()?;
    background.scale = (ortho.area.size()).extend(1.0);

    // World unit per pixel
    let wupp = ortho.area.width() / viewport_px.x as f32;

    // Material fetch
    // TODO: optimize this to only update when it actually changes
    let Some(mut mat) = material.get_mut(&material_handle.0) else {
        return Ok(());
    };

    // We have 3 levels by default, pick which level to use and aim for each cell to be ~32px
    let base_spacing = 10_000.0;
    let ideal = f64::from(mat.grid.cell * wupp);
    let k = (ideal / base_spacing).log(f64::from(mat.grid.ratio)).ceil() as i32;
    let k = k.clamp(-6, 6);

    let spacing = if k >= 0 {
        base_spacing * f64::from(mat.grid.ratio.pow(k.cast_unsigned()))
    } else {
        base_spacing / f64::from(mat.grid.ratio.pow((-k).cast_unsigned()))
    };
    mat.grid.spacing = spacing as f32;

    // Update the camera phase to pass in to the shader based off the camera
    // position + coarse spacing.
    let cam_pos = cam_tran.translation.truncate().as_dvec2();
    let coarse = f64::from(mat.grid.spacing) * f64::from(mat.grid.ratio.pow(mat.grid.levels - 1));
    mat.grid.phase = cam_pos.rem_euclid(DVec2::splat(coarse)).as_vec2();
    Ok(())
}

fn debug_shader_control(
    key_input: Res<ButtonInput<KeyCode>>,
    back: Query<&MeshMaterial2d<ArenaMaterial>, With<ArenaBackground>>,
    mut material: ResMut<Assets<ArenaMaterial>>,
) -> Result<()> {
    let material_handle = back.single()?;
    let Some(mut mat) = material.get_mut(&material_handle.0) else {
        return Ok(());
    };

    if key_input.just_pressed(KeyCode::Digit0) {
        mat.grid.mode = 0;
    }
    if key_input.just_pressed(KeyCode::Digit1) {
        mat.grid.mode = 1;
    }
    if key_input.just_pressed(KeyCode::Digit2) {
        mat.grid.mode = 2;
    }
    if key_input.just_pressed(KeyCode::Digit3) {
        mat.grid.mode = 3;
    }
    if key_input.just_pressed(KeyCode::Digit4) {
        mat.grid.mode = 4;
    }
    if key_input.just_pressed(KeyCode::Digit5) {
        mat.grid.mode = 5;
    }

    // adjust target px cell
    if key_input.just_pressed(KeyCode::KeyO) {
        mat.grid.cell -= 1.0;
        mat.grid.cell = mat.grid.cell.clamp(1.0, 32.0);

        // Calculate the rest
        mat.grid.fade_min = mat.grid.cell;
        mat.grid.fade_max = mat.grid.cell * (mat.grid.ratio as f32);
    }
    if key_input.just_pressed(KeyCode::KeyP) {
        mat.grid.cell += 1.0;
        mat.grid.cell = mat.grid.cell.clamp(1.0, 32.0);

        // Calculate the rest
        mat.grid.fade_min = mat.grid.cell;
        mat.grid.fade_max = mat.grid.cell * (mat.grid.ratio as f32);
    }

    Ok(())
}

#[derive(Component)]
struct GridControlMarker;

fn setup_grid_control(mut commands: Commands) {
    commands
        .spawn(Node {
            position_type: PositionType::Absolute,
            right: Val::Px(20.),
            bottom: Val::Px(20.),
            ..default()
        })
        .with_children(|parent| {
            parent.spawn((
                Text::new("-- Cell"),
                TextFont {
                    font_size: FontSize::Px(18.),
                    ..default()
                },
                GridControlMarker,
            ));
        });
}

fn render_grid_control(
    back: Query<&MeshMaterial2d<ArenaMaterial>, With<ArenaBackground>>,
    material: Res<Assets<ArenaMaterial>>,
    mut text: Single<&mut Text, With<GridControlMarker>>,
) -> Result<()> {
    let material_handle = back.single()?;
    let Some(mat) = material.get(&material_handle.0) else {
        return Ok(());
    };
    let cell = mat.grid.cell;
    text.0 = format!(" {cell:>2.0} Cell");

    Ok(())
}

fn sprite_test(mut commands: Commands) {
    for (x, y) in [
        (0.0, 0.0),
        (10_000.0, 0.0),
        (0.0, 10_000.0),
        (10_000.0, 10_000.0),
    ] {
        commands.spawn((
            Sprite::from_color(Color::srgb(0.0, 1.0, 1.0), Vec2::splat(200.0)),
            Transform::from_xyz(x, y, 10.0),
        ));
    }
}

#[test]
fn test_uniform_sizing() {
    <GridUniform as ShaderType>::assert_uniform_compat();
}
