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
// NOTE: If need any color, prefer to get it rust-side via
//  - Color::srgb(..).to_linear() and pass in via uniform
struct ArenaUniform {
    cam_pos: vec2<f32>,
    _pad: vec2<f32>,
}

@group(#{MATERIAL_BIND_GROUP}) @binding(0) var<uniform> arena: ArenaUniform;

@fragment
fn fragment(in: VertexOutput) -> @location(0) vec4<f32> {
    var col = vec4(in.uv.xyx, 1.0);

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
