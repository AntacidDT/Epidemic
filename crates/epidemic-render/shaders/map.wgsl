// Map shader — Epidemic NS with color palette + gradients

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

struct BubbleData {
    x: f32,
    y: f32,
    value: f32,
    active: f32,
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

@group(0) @binding(5)
var<uniform> bubbles: array<BubbleData, 10>;

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

const OCEAN: vec3<f32> = vec3<f32>(0.157, 0.459, 0.741);
const HEALTHY: vec3<f32> = vec3<f32>(0.271, 0.573, 0.196);
const INFECTED: vec3<f32> = vec3<f32>(1.0, 0.188, 0.165);
const DEAD: vec3<f32> = vec3<f32>(0.286, 0.188, 0.184);
const BORDER: vec3<f32> = vec3<f32>(0.0, 0.0, 0.0);
const HOVER_TINT: vec3<f32> = vec3<f32>(0.15, 0.15, 0.15);

fn hash2(p: vec2<f32>) -> f32 {
    let h = dot(p, vec2<f32>(127.1, 311.7));
    return fract(sin(h) * 43758.5453);
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let tex_color = textureSample(map_texture, map_sampler, in.uv);
    let region_id = u32(tex_color.r * 255.0 + 0.5);

    if region_id == 0u {
        return vec4<f32>(OCEAN, 1.0);
    }

    let data = regions[region_id];
    let is_hovered = f32(region_id == u32(uniforms.hovered_region));

    let noise = hash2(in.uv * uniforms.map_w);
    let grad = noise * 0.06 - 0.03;

    var color: vec3<f32>;

    if data.fallen == 1u {
        color = DEAD + grad;
    } else if data.infection_pct < 0.01 {
        color = HEALTHY + grad;
    } else {
        let pct = clamp(data.infection_pct, 0.0, 1.0);
        color = mix(HEALTHY, INFECTED, pct) + grad;
    }

    if data.healthcare_collapse == 1u {
        let pulse = sin(uniforms.time * 4.0) * 0.15 + 0.15;
        color = mix(color, vec3<f32>(0.9, 0.05, 0.05), pulse);
    }

    if is_hovered > 0.5 {
        color += HOVER_TINT;
    }

    let texel = vec2<f32>(1.0 / uniforms.map_w, 1.0 / uniforms.map_h);
    let left  = u32(textureSample(map_texture, map_sampler, in.uv + vec2<f32>(-texel.x, 0.0)).r * 255.0 + 0.5);
    let right = u32(textureSample(map_texture, map_sampler, in.uv + vec2<f32>( texel.x, 0.0)).r * 255.0 + 0.5);
    let up    = u32(textureSample(map_texture, map_sampler, in.uv + vec2<f32>(0.0, -texel.y)).r * 255.0 + 0.5);
    let down  = u32(textureSample(map_texture, map_sampler, in.uv + vec2<f32>(0.0,  texel.y)).r * 255.0 + 0.5);

    if left != region_id || right != region_id || up != region_id || down != region_id {
        color = color * 0.3;
    }

    // Check for DNA bubbles
    for (var i = 0u; i < 10u; i++) {
        let bubble = bubbles[i];
        if bubble.active > 0.5 {
            let dx = in.uv.x - bubble.x;
            let dy = in.uv.y - bubble.y;
            let dist = sqrt(dx * dx + dy * dy);
            let radius = 0.015 + sin(uniforms.time * 3.0) * 0.003; // pulsing
            if dist < radius {
                // DNA bubble: bright orange/gold
                let intensity = 1.0 - (dist / radius);
                let bubble_color = vec3<f32>(1.0, 0.7, 0.1);
                color = mix(color, bubble_color, intensity * 0.8);
            }
        }
    }

    return vec4<f32>(color, 1.0);
}
