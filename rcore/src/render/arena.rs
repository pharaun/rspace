use bevy_prototype_lyon::geometry::ShapeBuilderBase as _;
use bevy_prototype_lyon::prelude::Fill;
use bevy_prototype_lyon::prelude::ShapeBuilder;
use bevy_prototype_lyon::prelude::ShapePath;
use bevy_prototype_lyon::prelude::Stroke;

use bevy::prelude::*;
use bevy::sprite_render::Material2d;
use bevy::sprite_render::AlphaMode2d;
use bevy::sprite_render::Material2dPlugin;
use bevy::render::render_resource::ShaderType;
use bevy::render::render_resource::AsBindGroup;
use bevy::shader::ShaderRef;

use crate::math::AbsRot;

#[derive(Component)]
struct ArenaMarker;

fn arena_bounds_setup(mut commands: Commands) {
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

    // Axis Labels (Probs want it on the UI instead) since we can move around.
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

fn arena_grid(mut gizmos: Gizmos) {
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

pub(super) struct ArenaPlugin;
impl Plugin for ArenaPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_plugins(Material2dPlugin::<ArenaMaterial>::default())
            .add_systems(
                Startup,
                arena_bounds_setup,
            )
            .add_systems(
                Update,
                arena_grid,
            );

    }
}

//--------------------------------------------------------------------------------
// Shaders
//--------------------------------------------------------------------------------

#[derive(ShaderType, Clone, Copy, Default)]
struct ArenaUniform {
    pub cam_pos: Vec2,
}

#[derive(Asset, TypePath, AsBindGroup, Clone)]
struct ArenaMaterial {
    #[uniform(0)]
    pub arena: ArenaUniform,
}

impl Material2d for ArenaMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/arena.wgsl".into()
    }

    fn alpha_mode(&self) -> AlphaMode2d {
        AlphaMode2d::Opaque
    }
}

#[derive(Component)]
struct ArenaBackground;





// Shader todo for the Arena background shader:
// input:
//  - list[vec2(x, y)...] - origin? - Ie in a regular game its 0,0 by default, then for each "sun" that is
//  added there's another item in this list 0, 1 ~= 0,0 then 2 and more is 2 pos onward up
//
//  - vec2(x, y) - camera location (to know where we are in the arena for rendering the effect
//  - float - something like camera zoom to match up how zoomed in/out we are with the camera
//
//  - list[vec2(s, e)..] - list of rings (ie start/end distance from the origin, where in like
//  simple case of 1 origin its a stacked ring ie 0 or more ring, where in more than 1 origin case
//  it is merged together smoothly as the zones for eg (or not)
//
//  - list[vec3(rgb)...] - color for each ring i think?
//
//  after that i'm not sure. but for the initial case do the 0/1 star case which is always 1 pos,
//  usually (0,0) but want to allow others by default.
