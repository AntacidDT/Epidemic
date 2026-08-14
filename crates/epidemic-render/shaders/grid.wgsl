// Grid cell shader — instanced quads for the world map

struct Uniforms {
    time: f32,
    grid_w: f32,
    grid_h: f32,
    _pad: f32,
};

@group(0) @binding(0)
var<uniform> uniforms: Uniforms;

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) cell_color: vec3<f32>,
    @location(1) uv: vec2<f32>,
};

const POSITIONS = array<vec2<f32>, 6>(
    vec2<f32>(0.0, 0.0),
    vec2<f32>(1.0, 0.0),
    vec2<f32>(1.0, 1.0),
    vec2<f32>(0.0, 0.0),
    vec2<f32>(1.0, 1.0),
    vec2<f32>(0.0, 1.0),
);

@vertex
fn vs_main(
    @builtin(vertex_index) vertex_idx: u32,
    @location(0) cell_pos: vec2<f32>,
    @location(1) cell_state: f32,
) -> VertexOutput {
    let quad_pos = POSITIONS[vertex_idx];

    let cell_w = 2.0 / uniforms.grid_w;
    let cell_h = 2.0 / uniforms.grid_h;

    let x = -1.0 + cell_pos.x * cell_w;
    let y = 1.0 - (cell_pos.y + 1.0) * cell_h;

    var out: VertexOutput;
    out.position = vec4<f32>(
        x + quad_pos.x * cell_w,
        y + quad_pos.y * cell_h,
        0.0,
        1.0
    );
    out.uv = quad_pos;

    // Colors
    let ocean = vec3<f32>(0.02, 0.04, 0.10);
    let healthy = vec3<f32>(0.08, 0.28, 0.12);
    let infected = vec3<f32>(0.85, 0.12, 0.05);
    let dead = vec3<f32>(0.10, 0.10, 0.10);

    if cell_state < -0.5 {
        // Ocean
        out.cell_color = ocean;
    } else if cell_state < 0.25 {
        // Healthy land
        out.cell_color = healthy;
    } else if cell_state < 0.75 {
        // Infected — pulse
        let pulse = sin(uniforms.time * 3.0 + cell_pos.x * 0.4 + cell_pos.y * 0.3) * 0.15 + 0.85;
        out.cell_color = infected * pulse;
    } else {
        // Dead
        out.cell_color = dead;
    }

    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    // Grid lines on land only (skip for ocean which is already dark)
    let border = 0.04;
    let grid_line = step(border, in.uv.x) * step(border, in.uv.y)
                  * step(in.uv.x, 1.0 - border) * step(in.uv.y, 1.0 - border);
    let edge_color = in.cell_color * 0.4;
    let color = mix(edge_color, in.cell_color, grid_line);

    return vec4<f32>(color, 1.0);
}
