#import bevy_sprite::mesh2d_vertex_output::VertexOutput

struct ArenaUniform {
    cam_pos: vec2<f32>,
}

@group(#{MATERIAL_BIND_GROUP}) @binding(0) var<uniform> arena: ArenaUniform;

@fragment
fn fragmentMain(in: VertexOutput) -> @location(0) vec4<f32> {
    return vec4(in.uv.xyx, 1.0);
}
