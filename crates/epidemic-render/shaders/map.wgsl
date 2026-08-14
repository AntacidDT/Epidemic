// Map shader — Epidemic NS with custom color palette

struct Uniforms {
    time: f32,
    map_w: f32,
    map_h: f32,
    hovered_region: f32,
};

struct RegionData {
    infection_pct: f32,
    death_pct: f32,
    panic: f32,
    fallen: u32,
    healthcare_collapse: u32,
    borders_open: u32,
    _pad0: u32,
    _pad1: u32,
};

struct TransportData {
    progress: f32,
    origin_x: f32,
    origin_y: f32,
    dest_x: f32,
    dest_y: f32,
    transport_type: u32,
    _pad0: u32,
    _pad1: u32,
};

@group(0) @binding(0)
var<uniform> uniforms: Uniforms;

@group(0) @binding(1)
var map_texture: texture_2d<f32>;

@group(0) @binding(2)
var map_sampler: sampler;

@group(0) @binding(3)
var<uniform> regions: array<RegionData, 189>;

@group(0) @binding(4)
var<uniform> transports: array<TransportData, 200>;

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) uv: vec2<f32>,
};

@vertex
fn vs_main(@builtin(vertex_index) vertex_idx: u32) -> VertexOutput {
    var positions = array<vec2<f32>, 6>(
        vec2<f32>(-1.0, -1.0),
        vec2<f32>( 1.0, -1.0),
        vec2<f32>( 1.0,  1.0),
        vec2<f32>(-1.0, -1.0),
        vec2<f32>( 1.0,  1.0),
        vec2<f32>(-1.0,  1.0),
    );
    var uvs = array<vec2<f32>, 6>(
        vec2<f32>(0.0, 1.0),
        vec2<f32>(1.0, 1.0),
        vec2<f32>(1.0, 0.0),
        vec2<f32>(0.0, 1.0),
        vec2<f32>(1.0, 0.0),
        vec2<f32>(0.0, 0.0),
    );
    var out: VertexOutput;
    out.position = vec4<f32>(positions[vertex_idx], 0.0, 1.0);
    out.uv = uvs[vertex_idx];
    return out;
}

// Color palette
const OCEAN: vec3<f32> = vec3<f32>(0.157, 0.459, 0.741);       // #2875bd
const HEALTHY: vec3<f32> = vec3<f32>(0.271, 0.573, 0.196);      // #459232
const INFECTED: vec3<f32> = vec3<f32>(1.0, 0.188, 0.165);       // #ff302a
const DEAD: vec3<f32> = vec3<f32>(0.286, 0.188, 0.184);         // #49302f
const BORDER: vec3<f32> = vec3<f32>(0.0, 0.0, 0.0);             // #000000
const HOVER_TINT: vec3<f32> = vec3<f32>(0.15, 0.15, 0.15);      // white tint

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    // Read region ID from lookup texture
    let tex_color = textureSample(map_texture, map_sampler, in.uv);
    let region_id = u32(tex_color.r * 255.0 + 0.5);

    // Ocean
    if region_id == 0u {
        return vec4<f32>(OCEAN, 1.0);
    }

    // Get region data
    let data = regions[region_id];
    let is_hovered = f32(region_id == u32(uniforms.hovered_region));

    // Base color: healthy blue-green -> infected red -> dead dark
    var color: vec3<f32>;

    if data.fallen == 1u {
        color = DEAD;
    } else if data.infection_pct < 0.01 {
        // Healthy
        color = HEALTHY;
    } else {
        // Blend healthy -> infected based on infection %
        let pct = clamp(data.infection_pct, 0.0, 1.0);
        color = mix(HEALTHY, INFECTED, pct);
    }

    // Healthcare collapse: pulse red
    if data.healthcare_collapse == 1u {
        let pulse = sin(uniforms.time * 4.0) * 0.15 + 0.15;
        color = mix(color, vec3<f32>(0.9, 0.05, 0.05), pulse);
    }

    // Hover highlight
    if is_hovered > 0.5 {
        color += HOVER_TINT;
    }

    // Border detection via UV edge smoothing
    let px = in.uv.x * uniforms.map_w;
    let py = in.uv.y * uniforms.map_h;
    let fract_x = fract(px);
    let fract_y = fract(py);
    let edge = min(min(fract_x, 1.0 - fract_x), min(fract_y, 1.0 - fract_y));
    let border = smoothstep(0.0, 0.06, edge);
    color = mix(BORDER, color, border);

    return vec4<f32>(color, 1.0);
}
