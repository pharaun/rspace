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
    cell: f32,
    fade_min: f32,
    fade_max: f32,
    // Camera pos/grid-spacing
    phase: vec2<f32>,
    spacing: f32,
    // Debugging bits
    mode: u32,
    magic: u32,
}

@group(#{MATERIAL_BIND_GROUP}) @binding(0) var<uniform> grid: GridUniform;

fn over(src: vec4<f32>, dst: vec4<f32>) -> vec4<f32> {
    let a = src.a + dst.a * (1.0 - src.a);
    if a <= 0.0 {
        return vec4(0.0);
    }
    let rgb = (src.rgb * src.a + dst.rgb * dst.a * (1.0 - src.a)) / a;
    return vec4(rgb, a);
}

fn line_coverage(coord: vec2<f32>, deriv: vec2<f32>, half_width_px: f32) -> f32 {
    let dist_cells = abs(fract(coord - 0.5) - 0.5);
    let dist_px = dist_cells / max(deriv, vec2(1e-20));
    return 1.0 - smoothstep(0.0, half_width_px, min(dist_px.x, dist_px.y));
}

@fragment
fn fragment(in: VertexOutput) -> @location(0) vec4<f32> {
    // Self checks
#ifdef SHADER_DEBUG
    if grid.magic != 0xDEADC0DEu {
        // Magenta - structure is wrong
        return vec4(1.0, 0.0, 1.0, 1.0);
    }
    if !util::is_orthographic() {
        // Yellow - non-ortho camera
        return vec4(1.0, 1.0, 0.0, 1.0);
    }
#endif

    // Camera relative world-pos
    let rel = util::world_offset(in.position);
    let drel = fwidth(rel);

    // Self checks
#ifdef SHADER_DEBUG
    if grid.mode == 0x1u {
        // red-green gradient - black top-left
        return vec4(util::screen_uv(in.position), 0.0, 1.0);
    }
    if grid.mode == 0x2u {
        // White disc - stay circle on resize
        let p = util::screen_centered(in.position);
        return vec4(vec3(step(length(p), 0.5)), 1.0);
    }
    if grid.mode == 0x3u {
        // per grid-cell gradient - pan to check for swimming
        let p0 = util::mod_euclid(grid.phase, grid.spacing);
        return vec4(fract((rel + p0) / grid.spacing), 0.0, 1.0);
    }
    if grid.mode == 0x4u {
        // grey gradient (0..256 px per cell)
        let ppc = grid.spacing / max(max(drel.x, drel.y), 1e-20);
        return vec4(vec3(ppc / 256.0), 1.0);
    }
    if grid.mode == 0x5u {
        // camera-relative offset divided by 1k
        return vec4(abs(rel) / 1000.0, 0.0, 1.0);
    }
#endif

    var color = vec4(0.0, 0.0, 0.0, 1.0);

    // Grid, one loop for each tier of grid, and then we have the coarse
    // tier compose over top so that it overwrites the thinner lines
    for (var i: u32 = 0u; i < grid.levels; i = i + 1u) {
        let s = grid.spacing * pow(f32(grid.ratio), f32(i));

        // Phase so we know where we are roughly + the relative camera coord
        let p = util::mod_euclid(grid.phase, s);

        let coord = (rel + p) / s;
        let deriv = drel / s;

        // Fade a tier once its smaller than fade_min.
        let px_per_cell = 1.0 / max(max(deriv.x, deriv.y), 1e-20);
        let lod = smoothstep(grid.fade_min, grid.fade_max, px_per_cell);

        let cov = line_coverage(coord, deriv, grid.half_width);

        let t = f32(i) / max(f32(grid.levels) - 1.0, 1.0);
        let tint = mix(grid.minor, grid.major, t);

        color = over(vec4(tint.rgb, tint.a * cov * lod), color);
    }

    // World axis
    let w = util::world_pos(in.position);
    let axis_px = abs(w) / max(drel, vec2(1e-20));
    let axis_cov = 1.0 - smoothstep(
        0.0,
        grid.half_width * 1.5,
        min(axis_px.x, axis_px.y),
    );
    color = over(
        vec4(grid.axis.rgb, grid.axis.a * axis_cov),
        color,
    );

    return util::adjust_color(color);
}
