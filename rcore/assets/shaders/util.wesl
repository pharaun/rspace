import bevy_sprite_render::mesh2d::view_bindings::view;
import bevy_render::view::{
     frag_coord_to_uv,
     frag_coord_to_ndc,
     direction_clip_to_world,
     position_ndc_to_world,
};

// Defs via mesh2d pipeline based off the camera. Details: Material2dPipeline::specialize.
@if(TONEMAP_IN_SHADER)
import bevy_core_pipeline::tonemapping;
@if(SRGB_OUTPUT)
import bevy_render::color_operations::linear_to_srgb;
@if(OKLAB_OUTPUT)
import bevy_render::color_operations::linear_rgb_to_oklab;

// Source: color_material.wesl (in bevy)
fn adjust_color(color: vec4<f32>) -> vec4<f32> {
    var adjusted = color;
    @if(TONEMAP_IN_SHADER)
    adjusted = tonemapping::tone_mapping(adjusted, view.color_grading);
    @if(SRGB_OUTPUT)
    adjusted = vec4(linear_to_srgb(adjusted.rgb), adjusted.a);
    @if(OKLAB_OUTPUT)
    adjusted = vec4(linear_rgb_to_oklab(adjusted.rgb), adjusted.a);
    return adjusted;
}

fn mod_euclid(value: vec2<f32>, m: f32) -> vec2<f32> {
    /// [0, m) Euclidean.
    return value - floor(value / m) * m;
}

// In view.wgsl
// > `clip_from_view[3][3] == 1.0` is the standard way to check if a projection is orthographic
fn is_orthographic() -> bool {
    return view.clip_from_view[3][3] == 1.0;
}

// NOTE: coordinate functions for all of the "background shader bits"
// details: https://github.com/bevyengine/bevy/blob/release-0.20.0/crates/bevy_render/src/view.wesl
//
// Requoting:
//   NDC   -1..1, (-1,-1) is BOTTOM-LEFT, (1,1) is TOP-RIGHT   -> y is UP
//   UV     0..1, (0,0) is TOP-LEFT, (1,1) is BOTTOM-RIGHT     -> y is DOWN
//   World  +y is up
//
// Conventions:
//  - all function takes `@builtin(position)` vec4 from VertexOutput so all call
//  sites will invoke `util::foo(in.position)`

fn aspect() -> f32 {
    return view.viewport.z / view.viewport.w;
}

// NOTE: `@builtin(position)` is relative to RENDER TARGET, not
// viewport.
fn screen_px(frag_coord: vec4<f32>) -> vec2<f32> {
    return frag_coord.xy - view.viewport.xy;
}
fn screen_uv(frag_coord: vec4<f32>) -> vec2<f32> {
    return frag_coord_to_uv(frag_coord.xy, view.viewport);
}
fn screen_ndc(frag_coord: vec4<f32>) -> vec2<f32> {
    return frag_coord_to_ndc(frag_coord, view.viewport).xy;
}

// NOTE: ShaderToy's conventions (y-up, y [-1, 1], x [-aspect, aspect])
fn screen_centered(frag_coord: vec4<f32>) -> vec2<f32> {
    return screen_ndc(frag_coord) * vec2(aspect(), 1.0);
}

// Relative world position.
//
// This is so that the f32 transform/world position doesn't
// affect the shader, shader program should depend on this.
//
// Exact for ortho projection. (clip w == 1)
fn world_offset(frag_coord: vec4<f32>) -> vec2<f32> {
    return direction_clip_to_world(
        vec4(screen_ndc(frag_coord), 0.0, 0.0),
        view.world_from_clip,
    ).xy;
}

// Absolute world position.
//
// Subject ot f32 ULP degradation, use it only for shader features
// near the world origin. (0,0)
//
// Use `world_offset` + cpu-side phase.
fn world_pos(frag_coord: vec4<f32>) -> vec2<f32> {
    return position_ndc_to_world(
        frag_coord_to_ndc(frag_coord, view.viewport),
        view.world_from_clip,
    ).xy;
}

// World units per physical pixel.
//
// Assumes unrotated ortho camera with uniform scale.
fn world_units_per_px() -> f32 {
    return 2.0 / (view.clip_from_view[0][0] * view.viewport.z);
}

// Same as above, but correct with rotation/non-uniform scaling and perspective.
fn world_units_per_px_dd(world_xy: vec2<f32>) -> f32 {
    return max(length(dpdx(world_xy)), length(dpdy(world_xy)));
}
