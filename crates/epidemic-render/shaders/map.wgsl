// Map shader — GPU-side coloring with region data storage buffer

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
var<storage, read> regions: array<RegionData>;

@group(0) @binding(4)
var<storage, read> transports: array<TransportData>;

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

// Infection heatmap: green -> yellow -> orange -> red
fn infection_color(pct: f32) -> vec3<f32> {
    let healthy = vec3<f32>(0.08, 0.28, 0.12);
    let low = vec3<f32>(0.2, 0.6, 0.1);
    let mid = vec3<f32>(0.9, 0.7, 0.0);
    let high = vec3<f32>(0.95, 0.3, 0.0);
    let critical = vec3<f32>(0.85, 0.05, 0.05);

    if pct < 0.01 {
        return healthy;
    } else if pct < 0.25 {
        return mix(healthy, low, pct / 0.25);
    } else if pct < 0.5 {
        return mix(low, mid, (pct - 0.25) / 0.25);
    } else if pct < 0.75 {
        return mix(mid, high, (pct - 0.5) / 0.25);
    } else {
        return mix(high, critical, (pct - 0.75) / 0.25);
    }
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    // Read region ID from lookup texture
    let tex_color = textureSample(map_texture, map_sampler, in.uv);
    let region_id = u32(tex_color.r * 255.0 + 0.5);

    // Ocean
    if region_id == 0u {
        return vec4<f32>(0.02, 0.04, 0.08, 1.0);
    }

    // Get region data
    let data = regions[region_id];
    let is_hovered = f32(region_id == u32(uniforms.hovered_region));

    // Base color from infection
    var color = infection_color(data.infection_pct);

    // Fallen overlay
    if data.fallen == 1u {
        color = vec3<f32>(0.08, 0.08, 0.08);
    }

    // Healthcare collapse: pulsing red tint
    if data.healthcare_collapse == 1u {
        let pulse = sin(uniforms.time * 4.0) * 0.1 + 0.1;
        color = mix(color, vec3<f32>(0.8, 0.0, 0.0), pulse);
    }

    // Panic: blue tint
    if data.panic > 0.3 {
        let panic_tint = (data.panic - 0.3) * 0.3;
        color = mix(color, vec3<f32>(0.3, 0.3, 0.8), panic_tint);
    }

    // Hover highlight
    if is_hovered > 0.5 {
        color += vec3<f32>(0.15, 0.15, 0.15);
    }

    // Border detection: darken edges
    let px = in.uv.x * uniforms.map_w;
    let py = in.uv.y * uniforms.map_h;
    let fract_x = fract(px);
    let fract_y = fract(py);
    let edge = min(min(fract_x, 1.0 - fract_x), min(fract_y, 1.0 - fract_y));
    let border = smoothstep(0.0, 0.08, edge);
    color = mix(vec3<f32>(0.01, 0.01, 0.02), color, border);

    return vec4<f32>(color, 1.0);
}
