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

use crate::render::camera::CameraRigSystems;

#[derive(Resource)]
pub struct GridSettings {
    pub axis: Color,
    pub major: Color,
    pub minor: Color,
    // Grid Structure
    // - ratio == "cells between each major line"
    // - levels == "how many nested levels there are"
    ratio: u8,
    levels: u8,
    // Grid Spacing
    pub base_spacing: f64,
    pub z_depth: f32,
    // Grid Render
    half_width_px: f32,
    pub fade_min: f32,
    pub fade_max: f32,
}

impl GridSettings {
    fn init(ratio: u8, levels: u8, half_width_px: f32, cell_px: f32) -> Self {
        assert!((2..=20).contains(&ratio), "Ratio not between [2..=20]");
        assert!((1..=8).contains(&levels), "Level not between [1..=8]");
        assert!(0.5 <= half_width_px, "half width px not (0.5..)");
        assert!(1.0 <= cell_px, "cell not greater than 1 pixel wide");

        Self {
            axis: Color::oklcha(0.5232, 0.1404, 13.84, 1.0),
            major: Color::srgb(0.4, 0.4, 0.4),
            minor: Color::srgb(0.25, 0.25, 0.25),
            // Structure
            ratio,
            levels,
            // Spacing
            base_spacing: 10_000.0,
            z_depth: -999.0,
            // Render
            half_width_px,
            fade_min: cell_px,
            fade_max: cell_px * f32::from(ratio),
        }
    }
}

impl Default for GridSettings {
    fn default() -> Self {
        Self::init(10, 3, 1.0, 10.0)
    }
}

#[derive(Default)]
pub struct GridPlugin {
    pub debug: bool,
}
impl Plugin for GridPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(Material2dPlugin::<GridMaterial>::default())
            .init_resource::<GridSettings>()
            .add_systems(PostStartup, spawn_grid)
            // Get projection from camera (CameraUpdateSystem), but apply it
            // before transform propagation for the scale to take.
            .add_systems(
                PostUpdate,
                update_grid
                    .after(CameraRigSystems)
                    .after(CameraUpdateSystems)
                    .before(TransformSystems::Propagate),
            );
        if self.debug {
            app.add_systems(Startup, grid_control_ui.spawn())
                .add_systems(Update, (debug_shader_control, render_grid_control));
        }
    }
}

#[derive(Component)]
pub struct GridCamera;

#[derive(Component)]
struct GridBackground;

// NOTE: always pad to 16 bytes
#[derive(ShaderType, Clone, Copy, Default)]
struct GridUniform {
    axis: Vec4,
    major: Vec4,
    minor: Vec4,
    // Structure
    ratio: u32,
    levels: u32,
    // Render
    half_width: f32,
    fade_min: f32,
    fade_max: f32,
    // Camera
    phase: Vec2,
    spacing: f32,
    tier_frac: f32,
    // Debugging
    mode: u32,
    magic: u32,
}

#[derive(Asset, TypePath, AsBindGroup, Clone)]
struct GridMaterial {
    #[uniform(0)]
    grid: GridUniform,
}

impl Material2d for GridMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/grid.wgsl".into()
    }

    fn alpha_mode(&self) -> AlphaMode2d {
        AlphaMode2d::Blend
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

fn spawn_grid(
    mut mesh: ResMut<Assets<Mesh>>,
    mut material: ResMut<Assets<GridMaterial>>,
    settings: Res<GridSettings>,
    camera: Single<Entity, With<GridCamera>>,
    mut commands: Commands,
) {
    #[expect(clippy::cast_possible_truncation)]
    let uniform = GridUniform {
        axis: settings.axis.to_linear().to_vec4(),
        major: settings.major.to_linear().to_vec4(),
        minor: settings.minor.to_linear().to_vec4(),
        ratio: u32::from(settings.ratio),
        levels: u32::from(settings.levels),
        half_width: settings.half_width_px,
        fade_min: settings.fade_min,
        fade_max: settings.fade_max,
        // Camera bits
        phase: Vec2::ZERO,
        spacing: settings.base_spacing as f32,
        tier_frac: 0.0,
        // Debug
        mode: 0,
        magic: 0xDEAD_C0DE,
    };

    commands.spawn((
        GridBackground,
        Mesh2d(mesh.add(Rectangle::new(1.0, 1.0))),
        MeshMaterial2d(material.add(GridMaterial { grid: uniform })),
        // Place this as close to the back as possible
        Transform::from_xyz(0.0, 0.0, settings.z_depth),
        ChildOf(*camera),
    ));
}

#[expect(clippy::cast_possible_truncation, clippy::type_complexity)]
fn update_grid(
    camera: Single<(&Camera, &Projection, &Transform), With<GridCamera>>,
    back: Single<
        (&mut Transform, &MeshMaterial2d<GridMaterial>),
        (With<GridBackground>, Without<GridCamera>),
    >,
    settings: Res<GridSettings>,
    mut material: ResMut<Assets<GridMaterial>>,
) {
    let (cam, projection, cam_tran) = camera.into_inner();
    let Projection::Orthographic(ortho) = projection else {
        warn!("GridCamera is not an Orthographic Camera");
        return;
    };
    let Some(viewport_px) = cam.physical_viewport_size() else {
        warn!("Viewport not available");
        return;
    };
    if viewport_px.x == 0 {
        // viewport_px.x can be 0 (if logical size is zero bevy will skip, so skip here too)
        return;
    }
    if cam_tran.rotation != Quat::IDENTITY {
        warn!("GridCamera does not support rotation, may see weird results");
    }

    // Backdrop sizing + margin, Global Transform Propagation will handle moving it.
    let (mut background, material_handle) = back.into_inner();
    background.scale = (ortho.area.size() * 1.01).extend(1.0);

    // World unit per pixel
    let wupp = ortho.area.width() / viewport_px.x as f32;

    // Material fetch
    // TODO: optimize this to only update when it actually changes
    let Some(mut mat) = material.get_mut(&material_handle.0) else {
        warn!("Material not available");
        return;
    };

    // We have 3 levels by default, pick which level to use and aim for each cell to be ~32px
    let ideal = f64::from(mat.grid.fade_min * wupp);
    let kf = (ideal / settings.base_spacing).log(f64::from(mat.grid.ratio));
    let k = kf.ceil();
    let tier_frac = (k - kf) as f32; // 0.0 at switch, -> 1.0 just before
    let k = (k as i32).clamp(-6, 6);

    let spacing = if k >= 0 {
        settings.base_spacing * f64::from(mat.grid.ratio).powi(k)
    } else {
        settings.base_spacing / f64::from(mat.grid.ratio).powi(-k)
    };

    // Update the camera phase to pass in to the shader based off the camera
    // position + coarse spacing. (NOTE: This uses Transform cuz its post camera-rig, to have
    // the freshest transform).
    let cam_pos = cam_tran.translation.truncate().as_dvec2();
    let coarse = f64::from(mat.grid.spacing) * f64::from(mat.grid.ratio.pow(mat.grid.levels - 1));
    let phase = cam_pos.rem_euclid(DVec2::splat(coarse)).as_vec2();

    // Update the the uniform
    mat.grid.spacing = spacing as f32;
    mat.grid.tier_frac = tier_frac;
    mat.grid.phase = phase;
}

#[derive(Component, Default, Clone)]
struct GridControlMarker;

fn grid_control_ui() -> impl Scene {
    bsn! {
        Node {
            position_type: PositionType::Absolute,
            right: Val::Px(20.),
            bottom: Val::Px(20.),
        }
        Children [
            GridControlMarker
            Text::new("-- Cell")
            TextFont { font_size: FontSize::Px(18.) }
        ]
    }
}

fn debug_shader_control(
    key_input: Res<ButtonInput<KeyCode>>,
    back: Single<&MeshMaterial2d<GridMaterial>, With<GridBackground>>,
    mut material: ResMut<Assets<GridMaterial>>,
) {
    let material_handle = back.into_inner();
    let Some(mut mat) = material.get_mut(&material_handle.0) else {
        warn!("Material not available");
        return;
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
        mat.grid.fade_min -= 1.0;
        mat.grid.fade_min = mat.grid.fade_min.clamp(1.0, 32.0);
        mat.grid.fade_max = mat.grid.fade_min * (mat.grid.ratio as f32);
    }
    if key_input.just_pressed(KeyCode::KeyP) {
        mat.grid.fade_min += 1.0;
        mat.grid.fade_min = mat.grid.fade_min.clamp(1.0, 32.0);
        mat.grid.fade_max = mat.grid.fade_min * (mat.grid.ratio as f32);
    }
}

fn render_grid_control(
    back: Single<&MeshMaterial2d<GridMaterial>, With<GridBackground>>,
    material: Res<Assets<GridMaterial>>,
    mut text: Single<&mut Text, With<GridControlMarker>>,
) {
    let material_handle = back.into_inner();
    let Some(mat) = material.get(&material_handle.0) else {
        warn!("Material not available");
        return;
    };
    let cell = mat.grid.fade_min;
    text.0 = format!(" {cell:>2.0} Cell");
}

#[test]
fn test_uniform_sizing() {
    <GridUniform as ShaderType>::assert_uniform_compat();
}
