#import bevy_sprite::{
    mesh2d_vertex_output::VertexOutput,
    mesh2d_view_bindings::view,
}
#import rcore::util

struct GridUniform {
    // Grid line colors
    axis: vec4<f32>,
    major: vec4<f32>,
    minor: vec4<f32>,
    // Grid Structure
    ratio: u32,
    levels: u32,
    // Line render constraints in pixels
    half_width: f32,
    fade_min: f32,
    fade_max: f32,
    // Camera pos/grid-spacing
    phase: vec2<f32>,
    spacing: f32,
}

@group(#{MATERIAL_BIND_GROUP}) @binding(0) var<uniform> grid: GridUniform;

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
    col = mix(col, grid.axis, a_x_axis);
    col = mix(col, grid.axis, a_y_axis);

    return util::adjust_color(col);
}
