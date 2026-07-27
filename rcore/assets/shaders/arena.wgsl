#import bevy_sprite::{
    mesh2d_vertex_output::VertexOutput,
    mesh2d_view_bindings::view,
}

// Defs via mesh2d pipeline based off the camera. Details: Material2dPipeline::specialize.
#ifdef TONEMAP_IN_SHADER
#import bevy_core_pipeline::tonemapping
#endif
#ifdef SRGB_OUTPUT
#import bevy_render::color_operations::linear_to_srgb
#endif
#ifdef OKLAB_OUTPUT
#import bevy_render::color_operations::linear_rgb_to_oklab
#endif

// NOTE: always pad to 16 bytes
// TODO: may not need this (via built in bevy uniforms)
struct CameraUniform {
    pos: vec2<f32>,
    zoom: f32,
    _pad: f32,
}

// NOTE: If need any color, prefer to get it rust-side via
//  - Color::srgb(..).to_linear() and pass in via uniform
struct GridUniform {
    x_axis: vec4<f32>,
    y_axis: vec4<f32>,
    major: vec4<f32>,
    minor: vec4<f32>,
}

@group(#{MATERIAL_BIND_GROUP}) @binding(0) var<uniform> camera: CameraUniform;
@group(#{MATERIAL_BIND_GROUP}) @binding(1) var<uniform> grid: GridUniform;

fn axis_line(sd: f32, thickness: f32) -> f32 {
    let upp = max(fwidth(sd), 1e-8);
    let d_px = abs(sd) / upp;
    let half = thickness * 0.5;
    return 1.0 - smoothstep(half - 0.5, half + 0.5, d_px);
}

fn grid_line(sd: f32, spacing: f32, thickness: f32) -> f32 {
    let upp = max(fwidth(sd), 1e-8);
    let d_px = abs(sd - round(sd / spacing) * spacing);
    let half = thickness * 0.5;
    return 1.0 - smoothstep(half - 0.5, half + 0.5, d_px / upp);
}

@fragment
fn fragment(in: VertexOutput) -> @location(0) vec4<f32> {
    // Background bits here
    var col = vec4(0.15, 0.15, 0.15, 1.0);

    // Grid bits on top of background
    // in.position.xy = (0.5, 0.5) -> (0..width) and (0..height)
    let p = in.world_position.xy;

    let a_y_axis = axis_line(p.x, 2.0);
    let a_x_axis = axis_line(p.y, 2.0);

    // minor/major
    let major_y_axis = grid_line(p.x, 1000.0, 2.0);

    col = mix(col, grid.minor, major_y_axis);
    col = mix(col, grid.x_axis, a_x_axis);
    col = mix(col, grid.y_axis, a_y_axis);

    // Source: color_material.wgsl (in bevy)
#ifdef TONEMAP_IN_SHADER
    col = tonemapping::tone_mapping(col, view.color_grading);
#endif
#ifdef SRGB_OUTPUT
    col = vec4(linear_to_srgb(col.rgb), col.a);
#endif
#ifdef OKLAB_OUTPUT
    col = vec4(linear_rgb_to_oklab(col.rgb), col.a);
#endif
    return col;
}
