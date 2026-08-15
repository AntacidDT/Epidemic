use epidemic_core::{GamePhase, World};
use wgpu::util::DeviceExt;
use wgpu::SurfaceError;
use winit::{
    application::ApplicationHandler,
    dpi::{PhysicalPosition, PhysicalSize},
    event::{ElementState, MouseButton, WindowEvent},
    event_loop::EventLoop,
    window::Window,
};

use std::sync::Arc;
use std::time::Instant;

mod theme;
mod styles;
use theme::*;

// Backward-compatible color aliases (call theme functions)
macro_rules! theme_colors {
    () => {
        const BLACK: egui::Color32 = egui::Color32::from_rgb(7, 7, 7);
        const WHITE: egui::Color32 = egui::Color32::from_rgb(255, 255, 255);
        const PRIMARY: egui::Color32 = egui::Color32::from_rgb(255, 0, 0);
        const SECONDARY: egui::Color32 = egui::Color32::from_rgb(185, 41, 38);
        const TERTIARY: egui::Color32 = egui::Color32::from_rgb(255, 87, 87);
        const EXTRA: egui::Color32 = egui::Color32::from_rgb(255, 161, 83);
        const BG_DARK: egui::Color32 = egui::Color32::from_rgb(12, 12, 14);
        const BG_PANEL: egui::Color32 = egui::Color32::from_rgb(18, 18, 22);
        const BG_CARD: egui::Color32 = egui::Color32::from_rgb(28, 28, 34);
        const BG_HOVER: egui::Color32 = egui::Color32::from_rgb(38, 38, 46);
        const BORDER: egui::Color32 = egui::Color32::from_rgb(55, 55, 65);
        const TEXT: egui::Color32 = egui::Color32::from_rgb(230, 230, 235);
        const TEXT_DIM: egui::Color32 = egui::Color32::from_rgb(140, 140, 155);
        const SUCCESS: egui::Color32 = egui::Color32::from_rgb(34, 197, 94);
        const DANGER: egui::Color32 = egui::Color32::from_rgb(239, 68, 68);
        const WARNING: egui::Color32 = egui::Color32::from_rgb(245, 158, 11);
        const INFO: egui::Color32 = egui::Color32::from_rgb(59, 130, 246);
    };
}
theme_colors!();

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct Uniforms {
    time: f32,
    map_w: f32,
    map_h: f32,
    hovered_region: f32,
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct RegionGpuData {
    infection_pct: f32,
    death_pct: f32,
    panic: f32,
    fallen: u32,
    healthcare_collapse: u32,
    borders_open: u32,
    newly_infected: u32,
    _pad1: u32,
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct TransportGpuData {
    progress: f32,
    origin_x: f32,
    origin_y: f32,
    dest_x: f32,
    dest_y: f32,
    transport_type: u32,
    _pad0: u32,
    _pad1: u32,
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct BubbleData {
    x: f32,
    y: f32,
    value: f32,
    active: f32,
}

pub struct Renderer {
    surface: wgpu::Surface<'static>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    size: PhysicalSize<u32>,
    pipeline: wgpu::RenderPipeline,
    uniform_buffer: wgpu::Buffer,
    region_buffer: wgpu::Buffer,
    transport_buffer: wgpu::Buffer,
    bubble_buffer: wgpu::Buffer,
    bind_group: wgpu::BindGroup,
    start_time: Instant,
    map_texture: wgpu::Texture,
    logo_texture: Option<egui::TextureHandle>,
    bg_mainmenu: Option<egui::TextureHandle>,
    bg_gamemode: Option<egui::TextureHandle>,
    bg_evolve: Option<egui::TextureHandle>,
    bg_world: Option<egui::TextureHandle>,
    egui_ctx: egui::Context,
    egui_state: egui_winit::State,
    egui_renderer: egui_wgpu::Renderer,
}

// Region center positions for transport line rendering (normalized 0-1)
fn region_center(id: u16) -> (f32, f32) {
    match id {
        1 => (0.15, 0.35),   // US
        2 => (0.15, 0.18),   // Canada
        3 => (0.12, 0.45),   // Mexico
        6 => (0.25, 0.65),   // Brazil
        17 => (0.47, 0.17),  // UK
        19 => (0.48, 0.25),  // France
        20 => (0.50, 0.20),  // Germany
        37 => (0.50, 0.30),  // Russia
        45 => (0.65, 0.15),  // Russia
        48 => (0.58, 0.30),  // Iran
        54 => (0.52, 0.35),  // Saudi Arabia
        55 => (0.52, 0.40),  // Egypt
        60 => (0.48, 0.50),  // Nigeria
        67 => (0.55, 0.45),  // Ethiopia
        81 => (0.52, 0.70),  // South Africa
        92 => (0.65, 0.40),  // India
        97 => (0.75, 0.55),  // Indonesia
        105 => (0.72, 0.30), // China
        106 => (0.80, 0.28), // Japan
        107 => (0.78, 0.30), // South Korea
        110 => (0.80, 0.70), // Australia
        _ => (0.5, 0.5),
    }
}

impl Renderer {
    pub async fn new(window: Arc<Window>, world: &World) -> Self {
        let size = window.inner_size();

        let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            ..Default::default()
        });
        let surface = instance.create_surface(window.clone()).unwrap();
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
            .expect("Failed to find GPU adapter");
        let (device, queue) = adapter
            .request_device(&wgpu::DeviceDescriptor {
                label: Some("epidemic-device"),
                required_limits: wgpu::Limits::downlevel_webgl2_defaults()
                    .using_resolution(adapter.limits()),
                ..Default::default()
            }, None)
            .await
            .expect("Failed to create device");

        let surface_caps = surface.get_capabilities(&adapter);
        let surface_format = surface_caps.formats.iter().find(|f| f.is_srgb()).copied()
            .unwrap_or(surface_caps.formats[0]);
        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: size.width.max(1),
            height: size.height.max(1),
            present_mode: wgpu::PresentMode::AutoVsync,
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };
        surface.configure(&device, &config);

        // Lookup texture (region ID per pixel)
        let map_tex = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("map-lookup"),
            size: wgpu::Extent3d { width: world.lookup_w as u32, height: world.lookup_h as u32, depth_or_array_layers: 1 },
            mip_level_count: 1, sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::R8Unorm,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });
        let map_view = map_tex.create_view(&wgpu::TextureViewDescriptor::default());
        let map_sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Nearest,
            min_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });

        // Upload lookup texture once
        let lookup_bytes: Vec<u8> = world.svg_lookup.iter().map(|&id| id as u8).collect();
        queue.write_texture(
            wgpu::TexelCopyTextureInfo { texture: &map_tex, mip_level: 0, origin: wgpu::Origin3d::ZERO, aspect: wgpu::TextureAspect::All },
            &lookup_bytes,
            wgpu::TexelCopyBufferLayout { offset: 0, bytes_per_row: Some(world.lookup_w as u32), rows_per_image: Some(world.lookup_h as u32) },
            wgpu::Extent3d { width: world.lookup_w as u32, height: world.lookup_h as u32, depth_or_array_layers: 1 },
        );

        // Uniforms
        let uniforms = Uniforms { time: 0.0, map_w: world.lookup_w as f32, map_h: world.lookup_h as f32, hovered_region: 0.0 };
        let uniform_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("uniforms"), contents: bytemuck::cast_slice(&[uniforms]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        // Region data storage buffer (115 entries, index 0 = dummy)
        let region_data = vec![RegionGpuData { infection_pct: 0.0, death_pct: 0.0, panic: 0.0, fallen: 0, healthcare_collapse: 0, borders_open: 1, newly_infected: 0, _pad1: 0 }; 189];
        let region_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("regions"), contents: bytemuck::cast_slice(&region_data),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        // Transport data storage buffer (max 200 transports)
        let transport_data = vec![TransportGpuData { progress: 0.0, origin_x: 0.0, origin_y: 0.0, dest_x: 0.0, dest_y: 0.0, transport_type: 0, _pad0: 0, _pad1: 0 }; 200];
        let transport_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("transports"), contents: bytemuck::cast_slice(&transport_data),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        // Bubble data buffer (max 10 bubbles)
        let bubble_data = vec![BubbleData { x: 0.0, y: 0.0, value: 0.0, active: 0.0 }; 10];
        let bubble_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("bubbles"), contents: bytemuck::cast_slice(&bubble_data),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        // Bind group layout
        let bgl = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("bgl"), entries: &[
                wgpu::BindGroupLayoutEntry { binding: 0, visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                    ty: wgpu::BindingType::Buffer { ty: wgpu::BufferBindingType::Uniform, has_dynamic_offset: false, min_binding_size: None }, count: None },
                wgpu::BindGroupLayoutEntry { binding: 1, visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture { sample_type: wgpu::TextureSampleType::Float { filterable: false }, view_dimension: wgpu::TextureViewDimension::D2, multisampled: false }, count: None },
                wgpu::BindGroupLayoutEntry { binding: 2, visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::NonFiltering), count: None },
                wgpu::BindGroupLayoutEntry { binding: 3, visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer { ty: wgpu::BufferBindingType::Uniform, has_dynamic_offset: false, min_binding_size: None }, count: None },
                wgpu::BindGroupLayoutEntry { binding: 4, visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer { ty: wgpu::BufferBindingType::Uniform, has_dynamic_offset: false, min_binding_size: None }, count: None },
                wgpu::BindGroupLayoutEntry { binding: 5, visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer { ty: wgpu::BufferBindingType::Uniform, has_dynamic_offset: false, min_binding_size: None }, count: None },
            ],
        });

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("bg"), layout: &bgl, entries: &[
                wgpu::BindGroupEntry { binding: 0, resource: uniform_buffer.as_entire_binding() },
                wgpu::BindGroupEntry { binding: 1, resource: wgpu::BindingResource::TextureView(&map_view) },
                wgpu::BindGroupEntry { binding: 2, resource: wgpu::BindingResource::Sampler(&map_sampler) },
                wgpu::BindGroupEntry { binding: 3, resource: region_buffer.as_entire_binding() },
                wgpu::BindGroupEntry { binding: 4, resource: transport_buffer.as_entire_binding() },
                wgpu::BindGroupEntry { binding: 5, resource: bubble_buffer.as_entire_binding() },
            ],
        });

        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("map-shader"), source: wgpu::ShaderSource::Wgsl(include_str!("../shaders/map.wgsl").into()),
        });
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("map-pl"), bind_group_layouts: &[&bgl], push_constant_ranges: &[],
        });
        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("map-pipeline"), layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState { module: &shader, entry_point: Some("vs_main"), buffers: &[], compilation_options: Default::default() },
            fragment: Some(wgpu::FragmentState { module: &shader, entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState { format: surface_format, blend: Some(wgpu::BlendState::REPLACE), write_mask: wgpu::ColorWrites::ALL })],
                compilation_options: Default::default() }),
            primitive: wgpu::PrimitiveState { topology: wgpu::PrimitiveTopology::TriangleList, ..Default::default() },
            depth_stencil: None, multisample: wgpu::MultisampleState::default(), multiview: None, cache: None,
        });

        // egui with custom fonts
        let egui_ctx = egui::Context::default();

        // Load Geist font
        let mut fonts = egui::FontDefinitions::default();
        if let Ok(font_data) = std::fs::read("../Assets/fonts/Geist-Regular.ttf")
            .or_else(|_| std::fs::read("Assets/fonts/Geist-Regular.ttf"))
        {
            fonts.font_data.insert("geist".to_owned(), std::sync::Arc::new(egui::FontData::from_owned(font_data)));
            fonts.families
                .entry(egui::FontFamily::Proportional)
                .or_default()
                .insert(0, "geist".to_owned());
            fonts.families
                .entry(egui::FontFamily::Monospace)
                .or_default()
                .push("geist".to_owned());
        }
        if let Ok(font_data) = std::fs::read("../Assets/fonts/Geist-Bold.ttf")
            .or_else(|_| std::fs::read("Assets/fonts/Geist-Bold.ttf"))
        {
            fonts.font_data.insert("geist-bold".to_owned(), std::sync::Arc::new(egui::FontData::from_owned(font_data)));
        }
        egui_ctx.set_fonts(fonts);

        let egui_state = egui_winit::State::new(egui_ctx.clone(), egui::ViewportId::ROOT, &window, None, None, None);
        let egui_renderer = egui_wgpu::Renderer::new(&device, surface_format, None, 1, false);

        Self {
            surface, device, queue, config, size, pipeline, uniform_buffer,
            region_buffer, transport_buffer, bubble_buffer, bind_group, start_time: Instant::now(),
                        map_texture: map_tex,
            logo_texture: None,
            bg_mainmenu: None,
            bg_gamemode: None,
            bg_evolve: None,
            bg_world: None,
            egui_ctx, egui_state, egui_renderer,
        }
    }

    pub fn resize(&mut self, new_size: PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.size = new_size;
            self.config.width = new_size.width;
            self.config.height = new_size.height;
            self.surface.configure(&self.device, &self.config);
        }
    }

    pub fn handle_event(&mut self, window: &Window, event: &WindowEvent) -> bool {
        let resp = self.egui_state.on_window_event(window, event);
        if matches!(event, WindowEvent::MouseInput { .. }) { return false; }
        resp.consumed
    }

    pub fn render(&mut self, world: &mut World, window: &Window, hovered_region: Option<u16>, show_grid: bool) -> Result<(), SurfaceError> {
        let elapsed = self.start_time.elapsed().as_secs_f32();

        // Load logo on first frame
        if self.logo_texture.is_none() {
            if let Ok(img) = load_logo() {
                let size = [img.width() as usize, img.height() as usize];
                let pixels = img.into_raw();
                let color_image = egui::ColorImage::from_rgba_unmultiplied(size, &pixels);
                self.logo_texture = Some(self.egui_ctx.load_texture("logo", color_image, egui::TextureOptions::LINEAR));
            }
        }

        // Load backgrounds on first frame
        if self.bg_mainmenu.is_none() {
            for (path, slot) in [
                ("mainmenubackground.png", &mut self.bg_mainmenu),
                ("gamemodemenubackground.png", &mut self.bg_gamemode),
                ("evolvebackground.png", &mut self.bg_evolve),
                ("worldmenubackground.png", &mut self.bg_world),
            ] {
                if slot.is_none() {
                    if let Ok(img) = load_asset_image(path) {
                        let size = [img.width() as usize, img.height() as usize];
                        let pixels = img.into_raw();
                        let color_image = egui::ColorImage::from_rgba_unmultiplied(size, &pixels);
                        *slot = Some(self.egui_ctx.load_texture(path, color_image, egui::TextureOptions::LINEAR));
                    }
                }
            }
        }

        // Update uniforms
        let uniforms = Uniforms { time: elapsed, map_w: world.lookup_w as f32, map_h: world.lookup_h as f32, hovered_region: hovered_region.unwrap_or(0) as f32 };
        self.queue.write_buffer(&self.uniform_buffer, 0, bytemuck::cast_slice(&[uniforms]));

        // Update region data buffer
        let mut region_data = vec![RegionGpuData { infection_pct: 0.0, death_pct: 0.0, panic: 0.0, fallen: 0, healthcare_collapse: 0, borders_open: 1, newly_infected: 0, _pad1: 0 }; 189];
        for r in &world.regions {
            if (r.id as usize) < region_data.len() {
                region_data[r.id as usize] = RegionGpuData {
                    infection_pct: r.infection_pct(),
                    death_pct: r.death_pct(),
                    panic: r.panic,
                    fallen: if r.fallen { 1 } else { 0 },
                    healthcare_collapse: if r.healthcare_collapse { 1 } else { 0 },
                    borders_open: if r.borders_open { 1 } else { 0 },
                    newly_infected: if r.newly_infected { 1 } else { 0 },
                    _pad1: 0,
                };
            }
        }
        self.queue.write_buffer(&self.region_buffer, 0, bytemuck::cast_slice(&region_data));

        // Update transport data buffer
        let mut transport_data = vec![TransportGpuData { progress: 0.0, origin_x: 0.0, origin_y: 0.0, dest_x: 0.0, dest_y: 0.0, transport_type: 0, _pad0: 0, _pad1: 0 }; 200];
        for (i, t) in world.transports.iter().enumerate().take(200) {
            let (ox, oy) = region_center(t.origin);
            let (dx, dy) = region_center(t.destination);
            transport_data[i] = TransportGpuData {
                progress: t.progress, origin_x: ox, origin_y: oy, dest_x: dx, dest_y: dy,
                transport_type: match t.transport_type { epidemic_core::TransportType::Flight => 0, epidemic_core::TransportType::CargoShip => 1 },
                _pad0: 0, _pad1: 0,
            };
        }
        self.queue.write_buffer(&self.transport_buffer, 0, bytemuck::cast_slice(&transport_data));

        // Update bubble data
        let mut bubble_data = vec![BubbleData { x: 0.0, y: 0.0, value: 0.0, active: 0.0 }; 10];
        for (i, bubble) in world.dna_bubbles.iter().enumerate().take(10) {
            if !bubble.collected {
                bubble_data[i] = BubbleData {
                    x: bubble.x,
                    y: bubble.y,
                    value: bubble.value as f32,
                    active: 1.0,
                };
            }
        }
        self.queue.write_buffer(&self.bubble_buffer, 0, bytemuck::cast_slice(&bubble_data));

        // egui
        let logo = self.logo_texture.clone();
        let raw_input = self.egui_state.take_egui_input(window);
        let hovered = hovered_region;
        let bg_mm = self.bg_mainmenu.clone();
        let bg_gm = self.bg_gamemode.clone();
        let bg_ev = self.bg_evolve.clone();
        let bg_wm = self.bg_world.clone();
        let full_output = self.egui_ctx.run(raw_input, |ctx| { build_ui(ctx, world, logo.as_ref(), hovered, bg_mm.as_ref(), bg_gm.as_ref(), bg_ev.as_ref(), bg_wm.as_ref(), show_grid); });
        self.egui_state.handle_platform_output(window, full_output.platform_output);
        let paint_jobs = self.egui_ctx.tessellate(full_output.shapes, full_output.pixels_per_point);
        for (id, delta) in &full_output.textures_delta.set { self.egui_renderer.update_texture(&self.device, &self.queue, *id, delta); }

        let output = self.surface.get_current_texture()?;
        let view = output.texture.create_view(&wgpu::TextureViewDescriptor::default());
        let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: Some("encoder") });

        // Pass 1: Map
        {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("map-pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view, resolve_target: None,
                    ops: wgpu::Operations { load: wgpu::LoadOp::Clear(wgpu::Color { r: 0.01, g: 0.02, b: 0.05, a: 1.0 }), store: wgpu::StoreOp::Store },
                })],
                depth_stencil_attachment: None, timestamp_writes: None, occlusion_query_set: None,
            });
            pass.set_pipeline(&self.pipeline);
            pass.set_bind_group(0, &self.bind_group, &[]);
            pass.draw(0..6, 0..1);
        }

        // Pass 2: egui
        {
            let screen_desc = egui_wgpu::ScreenDescriptor { size_in_pixels: [self.config.width, self.config.height], pixels_per_point: window.scale_factor() as f32 };
            self.egui_renderer.update_buffers(&self.device, &self.queue, &mut encoder, &paint_jobs, &screen_desc);
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("egui-pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view, resolve_target: None,
                    ops: wgpu::Operations { load: wgpu::LoadOp::Load, store: wgpu::StoreOp::Store },
                })],
                depth_stencil_attachment: None, timestamp_writes: None, occlusion_query_set: None,
            });
            let pass_mut: &mut wgpu::RenderPass<'static> = unsafe { std::mem::transmute(&mut pass) };
            self.egui_renderer.render(pass_mut, &paint_jobs, &screen_desc);
        }

        self.queue.submit(std::iter::once(encoder.finish()));
        output.present();
        for id in &full_output.textures_delta.free { self.egui_renderer.free_texture(id); }
        Ok(())
    }

    pub fn screen_to_map(&self, pos: PhysicalPosition<f64>, world: &World) -> (usize, usize) {
        let nx = pos.x / self.size.width as f64;
        let ny = pos.y / self.size.height as f64;
        let px = (nx * world.lookup_w as f64) as usize;
        let py = (ny * world.lookup_h as f64) as usize;
        (px.min(world.lookup_w - 1), py.min(world.lookup_h - 1))
    }
}

fn load_logo() -> Result<image::RgbaImage, Box<dyn std::error::Error>> {
    load_asset_image("EPIDEMIC.png")
}

fn load_asset_image(name: &str) -> Result<image::RgbaImage, Box<dyn std::error::Error>> {
    let paths = [
        format!("../Assets/{name}"),
        format!("Assets/{name}"),
        format!("../assets/{name}"),
        format!("assets/{name}"),
    ];
    for path in &paths {
        if let Ok(img) = image::open(path) {
            return Ok(img.to_rgba8());
        }
    }
    Err(format!("Could not find {name}").into())
}

// ─────────────────────────────────────────────────────────────
// egui UI
// ─────────────────────────────────────────────────────────────

fn build_ui(ctx: &egui::Context, world: &mut World, logo: Option<&egui::TextureHandle>, hovered_region: Option<u16>,
    bg_mainmenu: Option<&egui::TextureHandle>, bg_gamemode: Option<&egui::TextureHandle>,
    bg_evolve: Option<&egui::TextureHandle>, bg_world: Option<&egui::TextureHandle>, show_grid: bool) {

    apply_theme(ctx);

    // Debug grid overlay
    if show_grid {
        egui::CentralPanel::default()
            .frame(egui::Frame::new().fill(egui::Color32::TRANSPARENT))
            .show(ctx, |ui| {
                let rect = ui.max_rect();
                let sw = rect.width();
                let sh = rect.height();
                let cols = 80;
                let rows = 80;
                let cell_w = sw / cols as f32;
                let cell_h = sh / rows as f32;
                let painter = ui.painter();

                // Draw grid lines
                for i in 0..=cols {
                    let x = rect.left() + i as f32 * cell_w;
                    let alpha = if i % 10 == 0 { 80 } else { 30 };
                    let color = egui::Color32::from_rgba_premultiplied(255, 255, 255, alpha);
                    painter.line_segment(
                        [egui::pos2(x, rect.top()), egui::pos2(x, rect.bottom())],
                        egui::Stroke::new(1.0, color),
                    );
                }
                for i in 0..=rows {
                    let y = rect.top() + i as f32 * cell_h;
                    let alpha = if i % 10 == 0 { 80 } else { 30 };
                    let color = egui::Color32::from_rgba_premultiplied(255, 255, 255, alpha);
                    painter.line_segment(
                        [egui::pos2(rect.left(), y), egui::pos2(rect.right(), y)],
                        egui::Stroke::new(1.0, color),
                    );
                }

                // Draw coordinate labels at every 10th line
                for i in (0..=cols).step_by(10) {
                    let x = rect.left() + i as f32 * cell_w;
                    let label = format!("{}", i);
                    let galley = painter.layout_no_wrap(
                        label,
                        egui::FontId::proportional(9.0),
                        egui::Color32::from_rgba_premultiplied(255, 200, 200, 160),
                    );
                    painter.galley(egui::pos2(x + 2.0, rect.top() + 2.0), galley, egui::Color32::from_rgba_premultiplied(255, 200, 200, 160));
                }
                for i in (0..=rows).step_by(10) {
                    let y = rect.top() + i as f32 * cell_h;
                    let label = format!("{}", i);
                    let galley = painter.layout_no_wrap(
                        label,
                        egui::FontId::proportional(9.0),
                        egui::Color32::from_rgba_premultiplied(200, 255, 200, 160),
                    );
                    painter.galley(egui::pos2(rect.left() + 2.0, y + 2.0), galley, egui::Color32::from_rgba_premultiplied(200, 255, 200, 160));
                }
            });
    }

    match world.phase {
        GamePhase::SplashScreen => build_splash_screen(ctx, logo),
        GamePhase::TitleScreen => build_title_screen(ctx, world, logo, bg_mainmenu),
        GamePhase::PathogenSelect => build_game_type_select(ctx, world, bg_gamemode),
        GamePhase::DifficultySelect => build_pathogen_select(ctx, world, bg_gamemode), // reuse gamemode bg
        GamePhase::SelectOrigin => {
            build_country_select(ctx, world, bg_world, hovered_region);
        }
        GamePhase::Playing => {
            build_gameplay_hud(ctx, world, hovered_region);
            build_hover_tooltip(ctx, world, hovered_region);
            build_country_detail(ctx, world);
            if world.show_evolution {
                build_evolution_menu(ctx, world, bg_evolve);
            }
        }
        GamePhase::Won | GamePhase::Lost => {
            build_gameplay_hud(ctx, world, hovered_region);
            build_endgame_overlay(ctx, world, bg_mainmenu);
        }
    }
}

// ─── Splash Screen ───
fn build_splash_screen(ctx: &egui::Context, logo: Option<&egui::TextureHandle>) {
    egui::CentralPanel::default()
        .frame(egui::Frame::new().fill(egui::Color32::from_rgb(7, 7, 7)))
        .show(ctx, |ui| {
            let full_rect = ui.max_rect();

            // Center the logo
            if let Some(tex) = logo {
                let logo_size = 300.0;
                let logo_rect = egui::Rect::from_center_size(
                    full_rect.center(),
                    egui::vec2(logo_size, logo_size),
                );
                ui.painter().image(
                    tex.id(),
                    logo_rect,
                    egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)),
                    egui::Color32::WHITE,
                );
            }

            // "Loading..." text below logo
            let loading_y = full_rect.center().y + 180.0;
            let galley = ui.painter().layout_no_wrap(
                "Loading...".to_string(),
                egui::FontId::proportional(16.0),
                egui::Color32::from_rgb(100, 100, 100),
            );
            ui.painter().galley(
                egui::pos2(full_rect.center().x - galley.size().x * 0.5, loading_y),
                galley,
                egui::Color32::from_rgb(100, 100, 100),
            );
        });
}

// ─── Title Screen ───
// ─── Title Screen ───
fn build_title_screen(ctx: &egui::Context, world: &mut World, _logo: Option<&egui::TextureHandle>,
    bg_image: Option<&egui::TextureHandle>) {

    egui::CentralPanel::default()
        .frame(egui::Frame::new().fill(BLACK))
        .show(ctx, |ui| {
            let full_rect = ui.max_rect();
            let sw = full_rect.width();
            let sh = full_rect.height();

            // Background — fill entire window, no gaps
            if let Some(tex) = bg_image {
                let img = egui::Image::new(tex)
                    .fit_to_exact_size(full_rect.size())
                    .uv(egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)));
                ui.put(full_rect, img);
            }

            // ── Belowbrick (sidebar) ──
            let sidebar_w = sw / 3.0;
            let sidebar_rect = egui::Rect::from_min_size(
                egui::pos2(full_rect.min.x, full_rect.min.y),
                egui::vec2(sidebar_w, sh),
            );
            styles::draw_belowbrick(ui, sidebar_rect);

            // ── Text Buttons ──
            let btn_width = sidebar_w * 0.65;
            let btn_height = 38.0;
            let btn_x = sidebar_w * 0.5 - btn_width * 0.5;

            let buttons = [
                ("PLAY", 0.30), ("LOAD", 0.40), ("ENCYCLOPEDIA", 0.50),
                ("CREDITS", 0.60), ("SETTINGS", 0.70),
            ];

            for (label, y_frac) in &buttons {
                let btn_y = sh * y_frac;
                let btn_rect = egui::Rect::from_min_size(
                    egui::pos2(btn_x, btn_y),
                    egui::vec2(btn_width, btn_height),
                );
                let is_hovered = ui.rect_contains_pointer(btn_rect);
                if styles::draw_text_button(ui, btn_rect, label, is_hovered) {
                    if *label == "PLAY" { world.phase = GamePhase::PathogenSelect; }
                }
            }

            // ── Title + Subtitle (design system) ──
            // Right side content area: from sidebar_w to sw
            // Center = sidebar_w + (sw - sidebar_w) / 2 = (sidebar_w + sw) / 2
            let cx = (sidebar_w + sw) * 0.5;
            styles::draw_title(ui, egui::pos2(cx, sh * 0.15), "EPIDEMIC");
            styles::draw_subtitle(ui, egui::pos2(cx, sh * 0.28), "NATURAL STRATEGIES");

            // ── Text (tagline) ──
            styles::draw_text(ui, egui::pos2(cx, sh * 0.38), "open source pandemic strategy game", 33.0);
            styles::draw_text(ui, egui::pos2(cx, sh * 0.44), "thats meant for fun :)", 33.0);

            // ── Disclaimer (text module) ──
            let disc = [
                "WARNING:",
                "This game does not encourage the production of real Biological",
                "hazards. Its purely simulation and absolutely not meant to be used",
                "for harmful purposes.",
            ];
            let dy = sh * 0.80;
            for (i, line) in disc.iter().enumerate() {
                styles::draw_text(ui, egui::pos2(cx, dy + i as f32 * 28.0), line, 22.5);
            }
        });
}

fn muted_to_color32(brightness: f32) -> egui::Color32 {
    let v = (brightness * 255.0) as u8;
    egui::Color32::from_rgb(v, v, v)
}

// ─── Game Type Select ───
fn build_game_type_select(ctx: &egui::Context, world: &mut World,
    bg_image: Option<&egui::TextureHandle>) {

    // Color palette
    let header_text = egui::Color32::from_rgb(128, 24, 24);      // #801818
    let coral_red = egui::Color32::from_rgb(255, 102, 102);      // #FF6666
    let coral_outline = egui::Color32::from_rgb(185, 41, 38);    // #b92926
    let sky_blue = egui::Color32::from_rgb(51, 153, 255);        // #3399FF
    let blue_outline = egui::Color32::from_rgb(86, 137, 231);    // #5689e7
    let lavender = egui::Color32::from_rgb(153, 170, 204);       // #99AACC
    let dark_maroon = egui::Color32::from_rgb(58, 11, 14);       // #3A0B0E
    let footer_fill = egui::Color32::from_rgb(80, 15, 15);       // dark red
    let btn_text_dark = egui::Color32::from_rgb(0, 0, 0);        // black

    egui::CentralPanel::default()
        .frame(egui::Frame::new().fill(BLACK))
        .show(ctx, |ui| {
            let full_rect = ui.max_rect();
            let sw = full_rect.width();
            let sh = full_rect.height();

            // Background image
            if let Some(tex) = bg_image {
                let img = egui::Image::new(tex).fit_to_exact_size(full_rect.size()).uv(egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0))); ui.put(full_rect, img);
            }

            // ── A. Header Banner (Y: 0.12–0.23) ──
            let header_rect = egui::Rect::from_min_size(
                egui::pos2(0.0, sh * 0.12),
                egui::vec2(sw, sh * 0.11),
            );
            ui.painter().rect_filled(header_rect, 0.0, egui::Color32::from_rgba_premultiplied(20, 20, 30, 180));

            // "CHOOSE WISELY" with outline
            let header_y = sh * 0.175;
            let header_x = sw * 0.5;
            draw_outlined_text(ui, "CHOOSE WISELY", egui::pos2(header_x, header_y), 36.0, header_text);

            // ── B. Mode Card 1: Outbreak (Left) ──
            let card_w = sw * 0.20;
            let card_h = sh * 0.33;
            let card1_x = sw * 0.08;
            let card1_y = sh * 0.25;
            let card1_rect = egui::Rect::from_min_size(
                egui::pos2(card1_x, card1_y),
                egui::vec2(card_w, card_h),
            );

            // Drop shadow
            ui.painter().rect_filled(
                egui::Rect::from_min_size(
                    egui::pos2(card1_x + 4.0, card1_y + 4.0),
                    egui::vec2(card_w, card_h),
                ),
                6.0,
                dark_maroon,
            );
            // Card fill
            ui.painter().rect_filled(card1_rect, 6.0, coral_red);
            // Card border
            ui.painter().rect_stroke(card1_rect, 6.0, egui::Stroke::new(3.0, dark_maroon), egui::StrokeKind::Outside);

            // Title "Outbreak" centered at (0.18, 0.35)
            draw_outlined_text(ui, "Outbreak", egui::pos2(sw * 0.18, sh * 0.35), 28.0, coral_red);

            // Description at (0.18, 0.47)
            draw_outlined_text(ui, "cause a outbreak and wipe out humanity", egui::pos2(sw * 0.18, sh * 0.47), 12.0, coral_red);

            // Click detection for Outbreak card
            let response = ui.allocate_rect(card1_rect, egui::Sense::click());
            if response.clicked() {
                world.game_type = epidemic_core::GameType::Campaign;
                world.phase = GamePhase::DifficultySelect;
            }

            // ── C. Mode Card 2: Cure (Right) ──
            let card2_x = sw * 0.63;
            let card2_y = sh * 0.25;
            let card2_rect = egui::Rect::from_min_size(
                egui::pos2(card2_x, card2_y),
                egui::vec2(card_w, card_h),
            );

            // Drop shadow
            ui.painter().rect_filled(
                egui::Rect::from_min_size(
                    egui::pos2(card2_x + 4.0, card2_y + 4.0),
                    egui::vec2(card_w, card_h),
                ),
                6.0,
                dark_maroon,
            );
            // Card fill
            ui.painter().rect_filled(card2_rect, 6.0, lavender);
            // Card border
            ui.painter().rect_stroke(card2_rect, 6.0, egui::Stroke::new(3.0, dark_maroon), egui::StrokeKind::Outside);

            // Title "Cure" centered at (0.73, 0.35)
            draw_outlined_text(ui, "Cure", egui::pos2(sw * 0.73, sh * 0.35), 28.0, sky_blue);

            // Description at (0.73, 0.47)
            draw_outlined_text(ui, "cure the world from a virus and save humanity", egui::pos2(sw * 0.73, sh * 0.47), 12.0, sky_blue);

            // Click detection for Cure card
            let response = ui.allocate_rect(card2_rect, egui::Sense::click());
            if response.clicked() {
                // TODO: Cure mode
                world.game_type = epidemic_core::GameType::FreePlay;
                world.phase = GamePhase::DifficultySelect;
            }

            // ── D. Central Subtitle (Between Cards) ──
            let sub_y = sh * 0.42;
            let sub_x = sw * 0.50;

            // "will you be the one to attack" in coral red
            let attack_text = "will you be the one to attack";
            let attack_galley = ui.painter().layout_no_wrap(
                attack_text.to_string(),
                egui::FontId::proportional(16.0),
                coral_red,
            );
            let save_text = "or the one to save?";
            let save_galley = ui.painter().layout_no_wrap(
                save_text.to_string(),
                egui::FontId::proportional(16.0),
                sky_blue,
            );

            let total_w = attack_galley.size().x + 8.0 + save_galley.size().x;
            let start_x = sub_x - total_w * 0.5;

            // Draw attack text with outline
            draw_outlined_text(
                ui,
                attack_text,
                egui::pos2(start_x + attack_galley.size().x * 0.5, sub_y),
                16.0,
                coral_red,
            );

            // Draw save text with outline
            draw_outlined_text(
                ui,
                save_text,
                egui::pos2(start_x + attack_galley.size().x + 8.0 + save_galley.size().x * 0.5, sub_y),
                16.0,
                sky_blue,
            );

            // ── E. Footer Banner (Y: 0.78–1.0) ──
            let footer_rect = egui::Rect::from_min_size(
                egui::pos2(0.0, sh * 0.78),
                egui::vec2(sw, sh * 0.22),
            );
            ui.painter().rect_filled(footer_rect, 0.0, footer_fill);
        });
}

// ─── Game Mode Select (continued) ───// ─── Pathogen Select ───
fn build_pathogen_select(ctx: &egui::Context, world: &mut World, bg_image: Option<&egui::TextureHandle>) {
    egui::CentralPanel::default().frame(egui::Frame::new().fill(BG_DARK).inner_margin(egui::Margin::same(40))).show(ctx, |ui| {
        // Background image
        if let Some(tex) = bg_image {
            let rect = ui.max_rect();
            let img = egui::Image::new(tex).fit_to_exact_size(rect.size()).uv(egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0))); ui.put(rect, img);
            // Dark overlay for readability
            ui.painter().rect_filled(rect, 0.0, egui::Color32::from_rgba_premultiplied(0, 0, 0, 180));
        }

        ui.vertical_centered(|ui| {
            ui.label(egui::RichText::new("SELECT PATHOGEN").size(28.0).strong().color(WHITE));
            ui.add_space(8.0);
            ui.label(egui::RichText::new(format!("Game: {}", world.game_type.name())).size(13.0).color(TEXT_DIM));
            ui.add_space(16.0);

            // Disease naming
            ui.label(egui::RichText::new("Name your disease:").size(13.0).color(TEXT_DIM));
            ui.add_space(4.0);
            ui.horizontal(|ui| {
                let text_edit = egui::TextEdit::singleline(&mut world.disease_name_input)
                    .desired_width(200.0)
                    .font(egui::TextStyle::Heading);
                ui.add(text_edit);
            });

            ui.add_space(24.0);

            let pathogens = [
                (epidemic_core::PathogenType::Bacteria, "Standard pathogen. Cheap to devolve.", SUCCESS, "Beginner"),
                (epidemic_core::PathogenType::Virus, "Random mutations. Uncontrollable.", PRIMARY, "Intermediate"),
                (epidemic_core::PathogenType::Fungus, "Slow spread. Launch spores.", egui::Color32::from_rgb(180, 120, 60), "Hard"),
                (epidemic_core::PathogenType::Parasite, "Stealth. Low severity.", egui::Color32::from_rgb(80, 180, 80), "Hard"),
                (epidemic_core::PathogenType::Prion, "Slow infection. Slows cure.", egui::Color32::from_rgb(140, 100, 200), "Hard"),
                (epidemic_core::PathogenType::NanoVirus, "Cure starts immediately.", INFO, "Expert"),
                (epidemic_core::PathogenType::BioWeapon, "Innate lethality. Suppress it.", PRIMARY, "Expert"),
            ];

            egui::Grid::new("pathogen_grid").num_columns(2).spacing([16.0, 12.0]).show(ui, |ui| {
                for (i, (ptype, desc, color, diff_tag)) in pathogens.iter().enumerate() {
                    egui::Frame::new().fill(BG_CARD).corner_radius(egui::CornerRadius::same(10))
                        .stroke(egui::Stroke::new(1.0, BORDER)).inner_margin(egui::Margin::same(16))
                        .show(ui, |ui| {
                            ui.set_min_width(280.0);
                            ui.horizontal(|ui| {
                                ui.label(egui::RichText::new(ptype.name()).size(16.0).strong().color(*color));
                                egui::Frame::new().fill(color.linear_multiply(0.15)).corner_radius(egui::CornerRadius::same(4))
                                    .inner_margin(egui::Margin::symmetric(6, 2))
                                    .show(ui, |ui| { ui.label(egui::RichText::new(*diff_tag).size(9.0).strong().color(*color)); });
                            });
                            ui.add_space(4.0);
                            ui.label(egui::RichText::new(*desc).size(12.0).color(TEXT_DIM));
                            ui.add_space(8.0);
                            let btn = egui::Button::new(egui::RichText::new("SELECT").size(12.0).strong().color(WHITE))
                                .fill(*color).corner_radius(egui::CornerRadius::same(8));
                            if ui.add(btn).clicked() {
                                let name = if world.disease_name_input.is_empty() {
                                    "Epidemic".to_string()
                                } else {
                                    world.disease_name_input.clone()
                                };
                                world.init_disease(&name, *ptype);
                                world.phase = GamePhase::DifficultySelect;
                            }
                        });
                    if (i + 1) % 2 == 0 { ui.end_row(); }
                }
            });

            ui.add_space(24.0);
            // Difficulty selector inline
            ui.label(egui::RichText::new("DIFFICULTY").size(16.0).strong().color(WHITE));
            ui.add_space(12.0);
            let diffs = [
                (epidemic_core::Difficulty::Casual, "Casual", SUCCESS),
                (epidemic_core::Difficulty::Normal, "Normal", INFO),
                (epidemic_core::Difficulty::Brutal, "Brutal", EXTRA),
                (epidemic_core::Difficulty::MegaBrutal, "Mega Brutal", PRIMARY),
            ];
            ui.horizontal(|ui| {
                for (diff, name, color) in diffs {
                    let active = world.difficulty == diff;
                    let btn = egui::Button::new(egui::RichText::new(name).size(12.0).strong().color(if active { WHITE } else { TEXT }))
                        .fill(if active { color } else { BG_CARD }).corner_radius(egui::CornerRadius::same(8))
                        .stroke(egui::Stroke::new(1.0, if active { color } else { BORDER }));
                    if ui.add(btn).clicked() { world.difficulty = diff; }
                }
            });
        });
    });
}

// ─── Country Selection Screen ───
fn build_country_select(ctx: &egui::Context, world: &mut World,
    bg_image: Option<&egui::TextureHandle>, _hovered_region: Option<u16>) {

    // Color palette
    let header_fill = egui::Color32::from_rgb(255, 122, 122);     // #FF7A7A
    let header_outline = egui::Color32::from_rgb(107, 19, 19);    // #6B1313
    let map_container = egui::Color32::from_rgba_premultiplied(74, 14, 14, 153); // #4A0E0E 60% opacity
    let placeholder_text = egui::Color32::from_rgb(209, 110, 110); // #D16E6E
    let tip_title = egui::Color32::from_rgb(232, 130, 130);       // #E88282
    let tip_body = egui::Color32::from_rgb(232, 130, 130);        // #E88282

    egui::CentralPanel::default()
        .frame(egui::Frame::new().fill(BLACK))
        .show(ctx, |ui| {
            let full_rect = ui.max_rect();
            let sw = full_rect.width();
            let sh = full_rect.height();

            // Background image
            if let Some(tex) = bg_image {
                let img = egui::Image::new(tex).fit_to_exact_size(full_rect.size()).uv(egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0))); ui.put(full_rect, img);
            }

            // ── A. Main Header Title ──
            // "What country would you dare" + "to begin?"
            let header_x = sw * 0.50;
            let line1_y = sh * 0.08;
            let line2_y = sh * 0.15;

            draw_outlined_text(ui, "What country would you dare", egui::pos2(header_x, line1_y), 32.0, header_fill);

            draw_outlined_text(ui, "to begin?", egui::pos2(header_x, line2_y), 32.0, header_fill);

            // ── B. Interactive World Map Container ──
            let map_x = sw * 0.14;
            let map_y = sh * 0.26;
            let map_w = sw * 0.72;
            let map_h = sh * 0.62;
            let map_rect = egui::Rect::from_min_size(
                egui::pos2(map_x, map_y),
                egui::vec2(map_w, map_h),
            );

            // Map container background
            ui.painter().rect_filled(map_rect, 8.0, map_container);
            ui.painter().rect_stroke(map_rect, 8.0, egui::Stroke::new(2.0, border_color()), egui::StrokeKind::Outside);

            // Placeholder text
            let placeholder_x = sw * 0.50;
            let placeholder_y = sh * 0.57;
            let placeholder_galley = ui.painter().layout_no_wrap(
                "insert world svg here so player can select a country".to_string(),
                egui::FontId::proportional(14.0),
                placeholder_text,
            );
            ui.painter().galley(
                egui::pos2(placeholder_x - placeholder_galley.size().x * 0.5, placeholder_y),
                placeholder_galley,
                placeholder_text,
            );

            // ── C. Pro Tip Section ──
            let tip_x = sw * 0.50;
            let tip_y = sh * 0.90;

            // "Potentially pro tip:"
            draw_outlined_text(ui, "Potentially pro tip:", egui::pos2(tip_x, tip_y), 14.0, tip_title);

            // Body text
            let body_y = sh * 0.94;
            let body_lines = [
                "select a populated, but not too good of a healthcare country.",
                "honestly its your choice tho, choose whatever is best for you.",
            ];

            for (i, line) in body_lines.iter().enumerate() {
                let y = body_y + i as f32 * 18.0;
                let galley = ui.painter().layout_no_wrap(
                    line.to_string(),
                    egui::FontId::proportional(12.0),
                    tip_body,
                );
                ui.painter().galley(
                    egui::pos2(tip_x - galley.size().x * 0.5, y),
                    galley,
                    tip_body,
                );
            }

            // Note: The actual map rendering happens in the GPU pass
            // This UI layer just provides the frame and click handling
        });
}

fn border_color() -> egui::Color32 {
    egui::Color32::from_rgb(55, 55, 65)
}

// ─── Gameplay HUD ───
fn build_gameplay_hud(ctx: &egui::Context, world: &mut World, hovered_region: Option<u16>) {

    // ─── Left Panel ───
    egui::SidePanel::left("stats").exact_width(220.0)
        .frame(egui::Frame::new().fill(BG_DARK).stroke(egui::Stroke::new(1.0, BORDER)).inner_margin(egui::Margin::same(14)))
        .show(ctx, |ui| {
            // Header
            ui.horizontal(|ui| {
                ui.label(egui::RichText::new("EPIDEMIC").size(16.0).strong().color(PRIMARY));
                ui.label(egui::RichText::new("NS").size(16.0).strong().color(WHITE));
            });
            ui.horizontal(|ui| {
                ui.label(egui::RichText::new(format!("T{}", world.tick)).size(10.0).color(TEXT_DIM));
                ui.label(egui::RichText::new(format!("{}x", world.game_speed)).size(10.0).color(PRIMARY));
                ui.label(egui::RichText::new(world.season.name()).size(10.0).color(TEXT_DIM));
            });

            ui.add_space(6.0);
            ui.separator();
            ui.add_space(6.0);

            // Population card
            card(ui, BG_CARD, 8.0, |ui| {
                label_sm(ui, "POPULATION", TEXT_DIM);
                ui.add_space(2.0);
                row(ui, "Healthy", &fmt_num(world.total_healthy), SUCCESS);
                row(ui, "Infected", &fmt_num(world.total_infected), PRIMARY);
                row(ui, "Dead", &fmt_num(world.total_dead), egui::Color32::from_rgb(100, 100, 110));
            });

            ui.add_space(6.0);

            // DNA card
            card(ui, BG_CARD, 8.0, |ui| {
                label_sm(ui, "DNA POINTS", TEXT_DIM);
                ui.label(egui::RichText::new(format!("{}", world.dna_points)).size(22.0).strong().color(PRIMARY));
            });

            ui.add_space(6.0);

            // Cure card
            card(ui, BG_CARD, 8.0, |ui| {
                label_sm(ui, "CURE", TEXT_DIM);
                let cure_color = match world.cure_phase {
                    epidemic_core::CurePhase::Research => EXTRA,
                    epidemic_core::CurePhase::Trials => egui::Color32::from_rgb(200, 150, 0),
                    epidemic_core::CurePhase::Manufacturing => egui::Color32::from_rgb(180, 180, 0),
                    epidemic_core::CurePhase::Distribution => PRIMARY,
                    epidemic_core::CurePhase::Complete => PRIMARY,
                    _ => INFO,
                };
                ui.add(egui::ProgressBar::new(world.cure_overall / 100.0).fill(cure_color).corner_radius(egui::CornerRadius::same(4)));
                ui.label(egui::RichText::new(format!("{:.0}% {}", world.cure_overall, world.cure_phase.name())).size(11.0).color(cure_color));
            });

            ui.add_space(6.0);

            // Disease card
            card(ui, BG_CARD, 8.0, |ui| {
                label_sm(ui, &world.disease.name.to_uppercase(), TEXT_DIM);
                row(ui, "Infectivity", &format!("{:.1}", world.disease.effective_infectivity()), PRIMARY);
                row(ui, "Severity", &format!("{:.1}", world.disease.effective_severity()), EXTRA);
                row(ui, "Lethality", &format!("{:.1}", world.disease.effective_lethality()), egui::Color32::from_rgb(200, 40, 40));
            });

            ui.add_space(6.0);

            // Panic card
            card(ui, BG_CARD, 8.0, |ui| {
                label_sm(ui, "GLOBAL PANIC", TEXT_DIM);
                ui.add(egui::ProgressBar::new(world.global_panic).fill(EXTRA).corner_radius(egui::CornerRadius::same(4)));
                ui.label(egui::RichText::new(format!("{:.0}%", world.global_panic * 100.0)).size(11.0).color(EXTRA));
            });

            ui.add_space(6.0);

            // Speed buttons
            label_sm(ui, "SPEED", TEXT_DIM);
            ui.horizontal(|ui| {
                for (label, speed) in [("1x", 1), ("2x", 2), ("3x", 3)] {
                    let active = world.game_speed == speed;
                    let btn = egui::Button::new(egui::RichText::new(label).size(11.0).strong().color(if active { WHITE } else { TEXT }))
                        .fill(if active { PRIMARY } else { BG_CARD }).corner_radius(egui::CornerRadius::same(6))
                        .stroke(egui::Stroke::new(1.0, if active { PRIMARY } else { BORDER }));
                    if ui.add(btn).clicked() { world.game_speed = speed; }
                }
            });

            ui.add_space(6.0);

            // Save/Load
            label_sm(ui, "GAME", TEXT_DIM);
            ui.horizontal(|ui| {
                let save_btn = egui::Button::new(egui::RichText::new("Save").size(10.0).strong().color(TEXT))
                    .fill(BG_CARD).corner_radius(egui::CornerRadius::same(6))
                    .stroke(egui::Stroke::new(1.0, BORDER));
                if ui.add(save_btn).clicked() {
                    match epidemic_core::save_game(world, std::path::Path::new("epidemic_save.json")) {
                        Ok(()) => { println!("Game saved!"); }
                        Err(e) => { println!("Save failed: {e}"); }
                    }
                }
                let load_btn = egui::Button::new(egui::RichText::new("Load").size(10.0).strong().color(TEXT))
                    .fill(BG_CARD).corner_radius(egui::CornerRadius::same(6))
                    .stroke(egui::Stroke::new(1.0, BORDER));
                if ui.add(load_btn).clicked() {
                    match epidemic_core::load_game(std::path::Path::new("epidemic_save.json")) {
                        Ok(data) => { data.apply_to_world(world); }
                        Err(e) => { println!("Load failed: {e}"); }
                    }
                }
            });

            ui.add_space(6.0);

            // Evolve button
            let evo_btn = egui::Button::new(egui::RichText::new("EVOLVE [E]").size(12.0).strong().color(WHITE))
                .min_size(egui::vec2(ui.available_width(), 36.0))
                .fill(PRIMARY)
                .corner_radius(egui::CornerRadius::same(8));
            if ui.add(evo_btn).clicked() {
                world.show_evolution = !world.show_evolution;
                world.selected_upgrade = None;
            }

            ui.add_space(10.0);

            // Phase indicator
            match world.phase {
                GamePhase::SelectOrigin => {
                    egui::Frame::new().fill(EXTRA.linear_multiply(0.1)).corner_radius(egui::CornerRadius::same(8))
                        .inner_margin(egui::Margin::same(10)).show(ui, |ui| {
                            ui.label(egui::RichText::new("Click a country to start").size(11.0).color(EXTRA));
                        });
                }
                GamePhase::Playing => {
                    egui::Frame::new().fill(PRIMARY.linear_multiply(0.1)).corner_radius(egui::CornerRadius::same(8))
                        .inner_margin(egui::Margin::same(10)).show(ui, |ui| {
                            ui.label(egui::RichText::new("OUTBREAK ACTIVE").size(11.0).strong().color(PRIMARY));
                        });
                    let unlocked: Vec<&str> = world.synergies.iter().filter(|s| s.unlocked).map(|s| s.name).collect();
                    if !unlocked.is_empty() {
                        ui.add_space(4.0);
                        label_sm(ui, "SYNERGIES", TEXT_DIM);
                        for name in unlocked {
                            ui.label(egui::RichText::new(format!("  {name}")).size(10.0).color(SUCCESS));
                        }
                    }
                }
                GamePhase::Won => { ui.label(egui::RichText::new("VICTORY").size(16.0).strong().color(SUCCESS)); }
                GamePhase::Lost => { ui.label(egui::RichText::new("DEFEATED").size(16.0).strong().color(PRIMARY)); }
                _ => {}
            }
        });

    // ─── Bottom: News Ticker ───
    egui::TopBottomPanel::bottom("news").exact_height(36.0)
        .frame(egui::Frame::new().fill(BG_DARK).stroke(egui::Stroke::new(1.0, BORDER)).inner_margin(egui::Margin::symmetric(16, 8)))
        .show(ctx, |ui| {
            ui.horizontal(|ui| {
                // Show last 3 news items
                let news_count = world.news.len();
                let start = if news_count > 3 { news_count - 3 } else { 0 };
                for (i, msg) in world.news[start..].iter().enumerate() {
                    if i > 0 {
                        ui.separator();
                    }
                    let color = if i == news_count - start - 1 { EXTRA } else { TEXT_DIM };
                    ui.label(egui::RichText::new(msg).size(11.0).color(color));
                }
                if news_count == 0 {
                    ui.label(egui::RichText::new("No active reports").size(11.0).color(TEXT_DIM));
                }
            });
        });

}

// ─── Hover Tooltip ───
fn build_hover_tooltip(ctx: &egui::Context, world: &World, hovered_region: Option<u16>) {
    if let Some(rid) = hovered_region {
        if let Some(region) = world.regions.iter().find(|r| r.id == rid) {
            egui::Area::new(egui::Id::new("hover_tooltip"))
                .anchor(egui::Align2::RIGHT_TOP, egui::vec2(-10.0, 10.0))
                .show(ctx, |ui| {
                    egui::Frame::new().fill(BG_CARD).corner_radius(egui::CornerRadius::same(8))
                        .stroke(egui::Stroke::new(1.0, BORDER)).inner_margin(egui::Margin::same(12))
                        .show(ui, |ui| {
                            ui.set_min_width(180.0);
                            ui.label(egui::RichText::new(&region.name).size(14.0).strong().color(WHITE));
                            ui.add_space(4.0);
                            ui.label(egui::RichText::new(format!("Pop: {}", fmt_num(region.population))).size(11.0).color(TEXT_DIM));
                            if region.infected > 0 {
                                ui.label(egui::RichText::new(format!("Infected: {}", fmt_num(region.infected))).size(11.0).color(PRIMARY));
                                ui.label(egui::RichText::new(format!("Dead: {}", fmt_num(region.dead))).size(11.0).color(egui::Color32::from_rgb(100, 100, 110)));
                                let pct = region.infection_pct() * 100.0;
                                ui.label(egui::RichText::new(format!("{pct:.1}% infected")).size(11.0).color(PRIMARY));
                            } else {
                                ui.label(egui::RichText::new("Healthy").size(11.0).color(SUCCESS));
                            }
                            if !region.borders_open {
                                ui.label(egui::RichText::new("Borders CLOSED").size(10.0).color(egui::Color32::from_rgb(200, 100, 0)));
                            }
                            if region.healthcare_collapse {
                                ui.label(egui::RichText::new("Healthcare COLLAPSED").size(10.0).color(PRIMARY));
                            }
                            if region.fallen {
                                ui.label(egui::RichText::new("FALLEN").size(12.0).strong().color(egui::Color32::from_rgb(80, 80, 80)));
                            }
                        });
                });
        }
    }
}

// ─── Evolution Menu ───
fn build_evolution_menu(ctx: &egui::Context, world: &mut World, bg_image: Option<&egui::TextureHandle>) {

    egui::CentralPanel::default()
        .frame(egui::Frame::new().fill(egui::Color32::from_rgba_premultiplied(0, 0, 0, 180)))
        .show(ctx, |ui| {
            // Background image - scaled to fit
            if let Some(tex) = bg_image {
                let avail = ui.available_size();
                let img = egui::Image::new(tex).fit_to_exact_size(avail).uv(egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)));
                ui.add(img);
            }

            // Main panel
            egui::Frame::new()
                .fill(BG_CARD)
                .corner_radius(egui::CornerRadius::same(12))
                .stroke(egui::Stroke::new(1.0, BORDER))
                .inner_margin(egui::Margin::same(20))
                .show(ui, |ui| {
                    ui.set_min_width(ui.available_width());

                    // Header
                    ui.horizontal(|ui| {
                        ui.label(egui::RichText::new("EVOLUTION").size(20.0).strong().color(WHITE));
                        ui.add_space(16.0);
                        ui.label(egui::RichText::new(format!("DNA: {}", world.dna_points)).size(16.0).strong().color(PRIMARY));
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            let close_btn = egui::Button::new(egui::RichText::new("CLOSE [E]").size(11.0).color(TEXT))
                                .fill(BG_HOVER).corner_radius(egui::CornerRadius::same(6))
                                .stroke(egui::Stroke::new(1.0, BORDER));
                            if ui.add(close_btn).clicked() {
                                world.show_evolution = false;
                                world.selected_upgrade = None;
                            }
                        });
                    });

                    ui.add_space(12.0);

                    // Tabs
                    ui.horizontal(|ui| {
                        for (tab, label, color) in [
                            (epidemic_core::EvoTab::Transmission, "Transmission", SUCCESS),
                            (epidemic_core::EvoTab::Symptoms, "Symptoms", EXTRA),
                            (epidemic_core::EvoTab::Abilities, "Abilities", INFO),
                        ] {
                            let active = world.evo_tab == tab;
                            let btn = egui::Button::new(
                                egui::RichText::new(label).size(13.0).strong().color(if active { WHITE } else { TEXT_DIM })
                            )
                            .fill(if active { color.linear_multiply(0.2) } else { BG_HOVER })
                            .corner_radius(egui::CornerRadius::same(8))
                            .stroke(egui::Stroke::new(1.0, if active { color } else { BORDER }));
                            if ui.add(btn).clicked() {
                                world.evo_tab = tab;
                                world.selected_upgrade = None;
                            }
                        }
                    });

                    ui.add_space(12.0);
                    ui.separator();
                    ui.add_space(12.0);

                    // Content area: upgrade list + detail panel
                    ui.horizontal(|ui| {
                        // Left: upgrade list
                        ui.vertical(|ui| {
                            ui.set_min_width(300.0);

                            let upgrades: Vec<_> = world.upgrades.iter()
                                .filter(|u| u.category == match world.evo_tab {
                                    epidemic_core::EvoTab::Transmission => epidemic_core::UpgradeCategory::Transmission,
                                    epidemic_core::EvoTab::Symptoms => epidemic_core::UpgradeCategory::Symptom,
                                    epidemic_core::EvoTab::Abilities => epidemic_core::UpgradeCategory::Ability,
                                })
                                .collect();

                            for upgrade in &upgrades {
                                let owned = world.disease.has_upgrade(upgrade.id);
                                let can_unlock = world.disease.can_unlock(upgrade);
                                let can_afford = world.dna_points >= upgrade.cost;
                                let available = !owned && can_unlock && can_afford;
                                let locked = !owned && !can_unlock;
                                let selected = world.selected_upgrade.as_deref() == Some(upgrade.id);

                                let color = if owned { SUCCESS }
                                    else if available { TEXT }
                                    else { TEXT_DIM };

                                let bg_color = if selected { PRIMARY.linear_multiply(0.15) }
                                    else if owned { SUCCESS.linear_multiply(0.08) }
                                    else { BG_HOVER };

                                let stroke = if selected { egui::Stroke::new(1.5, PRIMARY) }
                                    else if owned { egui::Stroke::new(1.0, SUCCESS.linear_multiply(0.3)) }
                                    else { egui::Stroke::new(1.0, BORDER) };

                                let card = egui::Frame::new()
                                    .fill(bg_color)
                                    .corner_radius(egui::CornerRadius::same(8))
                                    .stroke(stroke)
                                    .inner_margin(egui::Margin::symmetric(12, 8));

                                card.show(ui, |ui| {
                                    ui.set_min_width(280.0);
                                    ui.horizontal(|ui| {
                                        // Status icon
                                        if owned {
                                            ui.label(egui::RichText::new("\u{2713}").size(14.0).color(SUCCESS));
                                        } else if locked {
                                            ui.label(egui::RichText::new("\u{1F512}").size(14.0).color(TEXT_DIM));
                                        } else {
                                            ui.label(egui::RichText::new("\u{25CB}").size(14.0).color(TEXT_DIM));
                                        }

                                        ui.add_space(4.0);

                                        // Name
                                        ui.label(egui::RichText::new(upgrade.name).size(13.0).strong().color(color));

                                        // Cost
                                        if !owned {
                                            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                                let cost_color = if can_afford { PRIMARY } else { TEXT_DIM };
                                                ui.label(egui::RichText::new(format!("{} DNA", upgrade.cost)).size(11.0).color(cost_color));
                                            });
                                        }
                                    });
                                });

                                // Click to select
                                if ui.interact(ui.min_rect(), egui::Id::new(upgrade.id), egui::Sense::click()).clicked() {
                                    world.selected_upgrade = Some(upgrade.id.to_string());
                                }

                                ui.add_space(4.0);
                            }
                        });

                        ui.add_space(16.0);

                        // Right: detail panel
                        egui::Frame::new()
                            .fill(BG_HOVER)
                            .corner_radius(egui::CornerRadius::same(10))
                            .stroke(egui::Stroke::new(1.0, BORDER))
                            .inner_margin(egui::Margin::same(16))
                            .show(ui, |ui| {
                                ui.set_min_width(240.0);
                                ui.set_min_height(300.0);

                                if let Some(ref sel_id) = world.selected_upgrade.clone() {
                                    if let Some(upgrade) = world.upgrades.iter().find(|u| u.id == sel_id.as_str()) {
                                        let owned = world.disease.has_upgrade(upgrade.id);
                                        let can_unlock = world.disease.can_unlock(upgrade);
                                        let can_afford = world.dna_points >= upgrade.cost;

                                        // Title
                                        ui.label(egui::RichText::new(upgrade.name).size(18.0).strong().color(WHITE));
                                        ui.add_space(8.0);

                                        // Description
                                        ui.label(egui::RichText::new(upgrade.description).size(12.0).color(TEXT));
                                        ui.add_space(12.0);

                                        // Stats
                                        ui.label(egui::RichText::new("EFFECTS").size(11.0).strong().color(TEXT_DIM));
                                        ui.add_space(4.0);
                                        if upgrade.infectivity > 0.0 {
                                            ui.label(egui::RichText::new(format!("+{:.1} Infectivity", upgrade.infectivity)).size(12.0).color(PRIMARY));
                                        }
                                        if upgrade.severity > 0.0 {
                                            ui.label(egui::RichText::new(format!("+{:.1} Severity", upgrade.severity)).size(12.0).color(EXTRA));
                                        }
                                        if upgrade.lethality > 0.0 {
                                            ui.label(egui::RichText::new(format!("+{:.1} Lethality", upgrade.lethality)).size(12.0).color(egui::Color32::from_rgb(200, 40, 40)));
                                        }

                                        ui.add_space(8.0);

                                        // Prerequisites
                                        if !upgrade.requires.is_empty() {
                                            ui.label(egui::RichText::new("REQUIRES").size(11.0).strong().color(TEXT_DIM));
                                            ui.add_space(4.0);
                                            for req in &upgrade.requires {
                                                let req_owned = world.disease.has_upgrade(req);
                                                let req_name = world.upgrades.iter().find(|u| u.id == *req).map(|u| u.name).unwrap_or(req);
                                                let color = if req_owned { SUCCESS } else { PRIMARY };
                                                let icon = if req_owned { "\u{2713}" } else { "\u{2717}" };
                                                ui.label(egui::RichText::new(format!("{icon} {req_name}")).size(12.0).color(color));
                                            }
                                            ui.add_space(8.0);
                                        }

                                        // Cost
                                        ui.label(egui::RichText::new(format!("Cost: {} DNA", upgrade.cost)).size(13.0).strong().color(PRIMARY));

                                        ui.add_space(16.0);

                                        // Upgrade button
                                        if owned {
                                            ui.label(egui::RichText::new("UNLOCKED").size(14.0).strong().color(SUCCESS));
                                        } else if can_unlock && can_afford {
                                            let btn = egui::Button::new(
                                                egui::RichText::new(format!("UPGRADE — {} DNA", upgrade.cost)).size(14.0).strong().color(WHITE)
                                            )
                                            .min_size(egui::vec2(ui.available_width(), 40.0))
                                            .fill(PRIMARY)
                                            .corner_radius(egui::CornerRadius::same(8));
                                            if ui.add(btn).clicked() {
                                                world.dna_points -= upgrade.cost;
                                                world.disease.unlock(upgrade);
                                            }
                                        } else if can_unlock && !can_afford {
                                            ui.label(egui::RichText::new("Not enough DNA").size(12.0).color(TEXT_DIM));
                                        } else {
                                            ui.label(egui::RichText::new("Locked — unlock prerequisites first").size(12.0).color(TEXT_DIM));
                                        }
                                    }
                                } else {
                                    ui.vertical_centered(|ui| {
                                        ui.add_space(80.0);
                                        ui.label(egui::RichText::new("Select an upgrade").size(14.0).color(TEXT_DIM));
                                    });
                                }
                            });
                    });
                });
        });
}

// ─── Country Detail Panel ───
fn build_country_detail(ctx: &egui::Context, world: &mut World) {
    let rid = match world.selected_detail { Some(r) => r, None => return };
    let region = match world.regions.iter().find(|r| r.id == rid) { Some(r) => r, None => return };

    let name = region.name.clone();
    let population = region.population;
    let infected = region.infected;
    let dead = region.dead;
    let healthy = region.healthy();
    let inf_pct = region.infection_pct();
    let death_pct = region.death_pct();
    let panic = region.panic;
    let lockdown = region.lockdown_level;
    let borders_open = region.borders_open;
    let healthcare_collapse = region.healthcare_collapse;
    let fallen = region.fallen;
    let climate = format!("{:?}", region.climate);
    let density = format!("{:?}", region.density);
    let govt = format!("{:?}", region.government_type);
    let is_wealthy = region.is_wealthy;
    let inf_history: Vec<(u64, u64)> = region.infection_history.clone();
    let death_history: Vec<(u64, u64)> = region.death_history.clone();

    egui::Window::new(format!("{} — Details", name))
        .collapsible(true)
        .resizable(true)
        .default_size([300.0, 400.0])
        .frame(egui::Frame::new().fill(BG_CARD).stroke(egui::Stroke::new(1.0, BORDER)).inner_margin(egui::Margin::same(14)))
        .show(ctx, |ui| {
            // Header
            ui.horizontal(|ui| {
                ui.label(egui::RichText::new(&name).size(18.0).strong().color(WHITE));
                if fallen {
                    egui::Frame::new().fill(PRIMARY.linear_multiply(0.2)).corner_radius(egui::CornerRadius::same(4))
                        .inner_margin(egui::Margin::symmetric(6, 2))
                        .show(ui, |ui| { ui.label(egui::RichText::new("FALLEN").size(10.0).strong().color(PRIMARY)); });
                }
            });
            ui.add_space(4.0);
            ui.label(egui::RichText::new(format!("{climate} | {density} | {govt}{}", if is_wealthy { " | Wealthy" } else { "" }))
                .size(11.0).color(TEXT_DIM));
            ui.add_space(8.0);

            // Population stats
            ui.label(egui::RichText::new("POPULATION").size(11.0).strong().color(TEXT_DIM));
            ui.add_space(4.0);
            row(ui, "Total", &fmt_num(population), WHITE);
            row(ui, "Healthy", &fmt_num(healthy), SUCCESS);
            row(ui, "Infected", &fmt_num(infected), PRIMARY);
            row(ui, "Dead", &fmt_num(dead), egui::Color32::from_rgb(100, 100, 110));
            ui.add_space(4.0);
            // Infection bar
            ui.label(egui::RichText::new(format!("Infection: {:.1}%", inf_pct * 100.0)).size(11.0).color(PRIMARY));
            ui.add(egui::ProgressBar::new(inf_pct).fill(PRIMARY).corner_radius(egui::CornerRadius::same(4)));
            ui.label(egui::RichText::new(format!("Death: {:.1}%", death_pct * 100.0)).size(11.0).color(egui::Color32::from_rgb(100, 100, 110)));
            ui.add(egui::ProgressBar::new(death_pct).fill(egui::Color32::from_rgb(100, 100, 110)).corner_radius(egui::CornerRadius::same(4)));

            ui.add_space(8.0);

            // Society
            ui.label(egui::RichText::new("SOCIETY").size(11.0).strong().color(TEXT_DIM));
            ui.add_space(4.0);
            row(ui, "Panic", &format!("{:.0}%", panic * 100.0), EXTRA);
            row(ui, "Lockdown", &format!("{:.0}%", lockdown * 100.0), INFO);
            row(ui, "Borders", if borders_open { "Open" } else { "CLOSED" }, if borders_open { SUCCESS } else { PRIMARY });
            if healthcare_collapse {
                ui.label(egui::RichText::new("HEALTHCARE COLLAPSED").size(11.0).strong().color(PRIMARY));
            }

            ui.add_space(8.0);

            // Infection history graph
            if inf_history.len() > 1 {
                ui.label(egui::RichText::new("INFECTION CURVE").size(11.0).strong().color(TEXT_DIM));
                ui.add_space(4.0);
                let max_val = inf_history.iter().map(|(_, v)| *v).max().unwrap_or(1).max(1) as f32;
                let graph_height = 80.0;
                let (response, painter) = ui.allocate_painter(egui::vec2(ui.available_width(), graph_height), egui::Sense::hover());
                let rect = response.rect;

                // Background
                painter.rect_filled(rect, 2.0, egui::Color32::from_rgb(20, 20, 28));

                // Draw infection line
                let points: Vec<egui::Pos2> = inf_history.iter().enumerate().map(|(i, (_, v))| {
                    let x = rect.left() + (i as f32 / inf_history.len().max(1) as f32) * rect.width();
                    let y = rect.bottom() - (*v as f32 / max_val) * rect.height();
                    egui::pos2(x, y)
                }).collect();

                if points.len() > 1 {
                    for w in points.windows(2) {
                        painter.line_segment([w[0], w[1]], egui::Stroke::new(2.0, PRIMARY));
                    }
                }

                // Draw death line
                let death_points: Vec<egui::Pos2> = death_history.iter().enumerate().map(|(i, (_, v))| {
                    let x = rect.left() + (i as f32 / death_history.len().max(1) as f32) * rect.width();
                    let y = rect.bottom() - (*v as f32 / max_val) * rect.height();
                    egui::pos2(x, y)
                }).collect();

                if death_points.len() > 1 {
                    for w in death_points.windows(2) {
                        painter.line_segment([w[0], w[1]], egui::Stroke::new(2.0, egui::Color32::from_rgb(100, 100, 110)));
                    }
                }

                // Legend
                ui.horizontal(|ui| {
                    ui.label(egui::RichText::new("— Infected").size(9.0).color(PRIMARY));
                    ui.label(egui::RichText::new("— Dead").size(9.0).color(egui::Color32::from_rgb(100, 100, 110)));
                });
            }
        });
}

// ─── Endgame Overlay ───
fn build_endgame_overlay(ctx: &egui::Context, world: &mut World, bg_image: Option<&egui::TextureHandle>) {
    let score = epidemic_core::calculate_score(world);
    let is_win = world.phase == GamePhase::Won;

    egui::Area::new(egui::Id::new("endgame"))
        .anchor(egui::Align2::CENTER_CENTER, egui::vec2(0.0, 0.0))
        .show(ctx, |ui| {
            egui::Frame::new().fill(BG_CARD).corner_radius(egui::CornerRadius::same(16))
                .stroke(egui::Stroke::new(2.0, if is_win { SUCCESS } else { PRIMARY }))
                .inner_margin(egui::Margin::same(32))
                .show(ui, |ui| {
                    ui.set_min_width(360.0);
                    ui.vertical_centered(|ui| {
                        // Title
                        ui.label(egui::RichText::new(if is_win { "VICTORY" } else { "DEFEATED" })
                            .size(36.0).strong().color(if is_win { SUCCESS } else { PRIMARY }));
                        ui.add_space(8.0);
                        ui.label(egui::RichText::new(if is_win {
                            "Humanity has fallen."
                        } else {
                            "The cure was completed."
                        }).size(14.0).color(TEXT_DIM));
                        ui.add_space(20.0);

                        // Score
                        ui.label(egui::RichText::new(format!("{}", score.final_score))
                            .size(48.0).strong().color(WHITE));
                        ui.label(egui::RichText::new("POINTS").size(12.0).color(TEXT_DIM));
                        ui.add_space(12.0);

                        // Biohazards
                        let bio_text = "\u{2620}".repeat(score.biohazards as usize);
                        ui.label(egui::RichText::new(bio_text).size(28.0).color(PRIMARY));
                        ui.label(egui::RichText::new(format!("{}/5 Biohazards", score.biohazards))
                            .size(12.0).color(TEXT_DIM));
                        ui.add_space(20.0);

                        // Stats grid
                        egui::Grid::new("endgame_stats")
                            .num_columns(2)
                            .spacing([20.0, 8.0])
                            .show(ui, |ui| {
                                ui.label(egui::RichText::new("Disease").size(12.0).color(TEXT_DIM));
                                ui.label(egui::RichText::new(&world.disease_name).size(12.0).strong().color(TEXT));
                                ui.end_row();

                                ui.label(egui::RichText::new("Pathogen").size(12.0).color(TEXT_DIM));
                                ui.label(egui::RichText::new(world.disease.pathogen_type.name()).size(12.0).color(TEXT));
                                ui.end_row();

                                ui.label(egui::RichText::new("Difficulty").size(12.0).color(TEXT_DIM));
                                ui.label(egui::RichText::new(world.difficulty.name()).size(12.0).color(TEXT));
                                ui.end_row();

                                ui.label(egui::RichText::new("Time").size(12.0).color(TEXT_DIM));
                                let minutes = world.tick / 1000;
                                let seconds = (world.tick % 1000) * 60 / 1000;
                                ui.label(egui::RichText::new(format!("{}m {}s", minutes, seconds)).size(12.0).color(TEXT));
                                ui.end_row();

                                ui.label(egui::RichText::new("Killed").size(12.0).color(TEXT_DIM));
                                ui.label(egui::RichText::new(fmt_num(world.total_dead)).size(12.0).color(PRIMARY));
                                ui.end_row();

                                ui.label(egui::RichText::new("Cure %").size(12.0).color(TEXT_DIM));
                                ui.label(egui::RichText::new(format!("{:.1}%", world.cure_overall)).size(12.0).color(INFO));
                                ui.end_row();
                            });

                        ui.add_space(24.0);

                        // Score breakdown
                        ui.label(egui::RichText::new("SCORE BREAKDOWN").size(11.0).strong().color(TEXT_DIM));
                        ui.add_space(4.0);
                        egui::Grid::new("score_breakdown")
                            .num_columns(2)
                            .spacing([20.0, 4.0])
                            .show(ui, |ui| {
                                ui.label(egui::RichText::new("Time Bonus").size(11.0).color(TEXT_DIM));
                                ui.label(egui::RichText::new(format!("+{}", score.time_bonus)).size(11.0).color(SUCCESS));
                                ui.end_row();

                                ui.label(egui::RichText::new("Disease Score").size(11.0).color(TEXT_DIM));
                                ui.label(egui::RichText::new(format!("+{}", score.disease_score)).size(11.0).color(SUCCESS));
                                ui.end_row();

                                ui.label(egui::RichText::new("Cure Penalty").size(11.0).color(TEXT_DIM));
                                ui.label(egui::RichText::new(format!("-{}", score.cure_penalty)).size(11.0).color(PRIMARY));
                                ui.end_row();

                                ui.label(egui::RichText::new("Difficulty Multiplier").size(11.0).color(TEXT_DIM));
                                ui.label(egui::RichText::new(format!("x{:.1}", score.diff_mult)).size(11.0).color(PRIMARY));
                                ui.end_row();
                            });

                        ui.add_space(24.0);

                        // Play Again button
                        let btn = egui::Button::new(
                            egui::RichText::new("PLAY AGAIN").size(14.0).strong().color(WHITE)
                        )
                        .min_size(egui::vec2(200.0, 44.0))
                        .fill(PRIMARY)
                        .corner_radius(egui::CornerRadius::same(10));
                        if ui.add(btn).clicked() {
                            *world = World::new(&std::fs::read_to_string("../assets/world.svg")
                                .or_else(|_| std::fs::read_to_string("assets/world.svg"))
                                .unwrap_or_default());
                            world.init_disease(&world.disease_name.clone(), world.disease.pathogen_type);
                        }
                    });
                });
        });
}

// ─── Helpers ───
fn card(ui: &mut egui::Ui, fill: egui::Color32, radius: f32, add_contents: impl FnOnce(&mut egui::Ui)) {
    egui::Frame::new().fill(fill).corner_radius(egui::CornerRadius::same(radius as u8))
        .inner_margin(egui::Margin::same(10)).show(ui, add_contents);
}

fn label_sm(ui: &mut egui::Ui, text: &str, color: egui::Color32) {
    // Outlined small label
    let pos = ui.cursor().min;
    draw_outlined_text(ui, text, pos, 10.0, color);
    let galley = ui.painter().layout_no_wrap(text.to_string(), egui::FontId::proportional(10.0), color);
    ui.advance_cursor_after_rect(egui::Rect::from_min_size(pos, galley.size() + egui::vec2(4.0, 2.0)));
}

fn row(ui: &mut egui::Ui, label: &str, value: &str, color: egui::Color32) {
    // Outlined row
    ui.horizontal(|ui| {
        let pos = ui.cursor().min;
        draw_outlined_text(ui, label, pos, 12.0, egui::Color32::from_rgb(140, 140, 155));
        let galley = ui.painter().layout_no_wrap(label.to_string(), egui::FontId::proportional(12.0), egui::Color32::from_rgb(140, 140, 155));
        ui.advance_cursor_after_rect(egui::Rect::from_min_size(pos, galley.size()));
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            let pos = ui.cursor().min;
            draw_outlined_text(ui, value, pos, 12.0, color);
            let galley = ui.painter().layout_no_wrap(value.to_string(), egui::FontId::proportional(12.0), color);
            ui.advance_cursor_after_rect(egui::Rect::from_min_size(pos, galley.size()));
        });
    });
}

fn stat_row(ui: &mut egui::Ui, label: &str, value: &str, color: egui::Color32) {
    row(ui, label, value, color);
}

fn fmt_num(n: u64) -> String {
    if n >= 1_000_000_000 { format!("{:.1}B", n as f64 / 1_000_000_000.0) }
    else if n >= 0_000_000 { format!("{:.1}M", n as f64 / 1_000_000.0) }
    else if n >= 1_000 { format!("{:.1}K", n as f64 / 1_000.0) }
    else { format!("{n}") }
}

// ─────────────────────────────────────────────────────────────
// App
// ─────────────────────────────────────────────────────────────

struct App {
    window: Option<Arc<Window>>,
    renderer: Option<Renderer>,
    world: World,
    last_sim_tick: Instant,
    sim_interval_ms: u64,
    cursor_pos: PhysicalPosition<f64>,
    hovered_region: Option<u16>,
    selected_detail: Option<u16>,
    last_click_time: Instant,
    splash_start: Option<Instant>,
    show_grid: bool,
}

const BASE_SIM_INTERVAL: u64 = 60;

impl App {
    fn new() -> Self {
        let svg_content = std::fs::read_to_string("../assets/world.svg")
            .or_else(|_| std::fs::read_to_string("assets/world.svg"))
            .expect("Failed to load world.svg");
        let mut world = World::new(&svg_content);
        world.init_disease("Epidemic", epidemic_core::PathogenType::Bacteria);
        Self { window: None, renderer: None, world, last_sim_tick: Instant::now(), sim_interval_ms: BASE_SIM_INTERVAL, cursor_pos: PhysicalPosition::new(0.0, 0.0), hovered_region: None, selected_detail: None, last_click_time: Instant::now(), splash_start: None, show_grid: true }
    }
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        if self.window.is_some() { return; }
        let attrs = Window::default_attributes().with_title("Epidemic NS").with_inner_size(PhysicalSize::new(1280, 720));
        let window = Arc::new(event_loop.create_window(attrs).unwrap());
        let renderer = pollster::block_on(Renderer::new(window.clone(), &self.world));
        self.window = Some(window);
        self.renderer = Some(renderer);
    }

    fn window_event(&mut self, event_loop: &winit::event_loop::ActiveEventLoop, _wid: winit::window::WindowId, event: WindowEvent) {
        if let (Some(r), Some(w)) = (self.renderer.as_mut(), self.window.as_ref()) {
            if r.handle_event(w, &event) { return; }
        }
        match event {
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::Resized(s) => { if let Some(r) = self.renderer.as_mut() { r.resize(s); } }
            WindowEvent::CursorMoved { position, .. } => {
                self.cursor_pos = position;
                if !self.world.show_evolution {
                    if let Some(r) = self.renderer.as_ref() {
                        let (px, py) = r.screen_to_map(self.cursor_pos, &self.world);
                        self.hovered_region = self.world.region_at_pixel(px, py).map(|r| r.id);
                    }
                } else {
                    self.hovered_region = None;
                }
            }
            WindowEvent::MouseInput { state: ElementState::Pressed, button: MouseButton::Left, .. } => {
                let now = Instant::now();
                let is_double_click = now.duration_since(self.last_click_time).as_millis() < 400;
                self.last_click_time = now;

                if let Some(r) = self.renderer.as_ref() {
                    let (px, py) = r.screen_to_map(self.cursor_pos, &self.world);

                    // Check for DNA bubble click first
                    if self.world.phase == GamePhase::Playing {
                        let nx = self.cursor_pos.x / r.size.width as f64;
                        let ny = self.cursor_pos.y / r.size.height as f64;
                        if self.world.collect_bubble(nx as f32, ny as f32) {
                            return; // bubble collected, don't process further
                        }
                    }

                    if let Some(region) = self.world.region_at_pixel(px, py) {
                        let rid = region.id;

                        if self.world.phase == GamePhase::SelectOrigin {
                            let name = region.name.clone();
                            self.world.start_outbreak(rid);
                            if let Some(w) = self.window.as_ref() { w.set_title(&format!("Epidemic NS — {name} outbreak!")); }
                        } else if is_double_click {
                            self.selected_detail = if self.selected_detail == Some(rid) { None } else { Some(rid) };
                            self.world.selected_detail = self.selected_detail;
                        }
                    } else {
                        self.selected_detail = None;
                        self.world.selected_detail = None;
                    }
                }
            }
            WindowEvent::KeyboardInput { event: winit::event::KeyEvent { physical_key: winit::keyboard::PhysicalKey::Code(keycode), state: ElementState::Pressed, .. }, .. } => {
                match keycode {
                    winit::keyboard::KeyCode::Space => { if self.world.phase == GamePhase::Playing { self.sim_interval_ms = if self.sim_interval_ms == u64::MAX { BASE_SIM_INTERVAL * self.world.game_speed as u64 } else { u64::MAX }; } }
                    winit::keyboard::KeyCode::Escape => event_loop.exit(),
                    winit::keyboard::KeyCode::Digit1 => { self.world.game_speed = 1; self.sim_interval_ms = BASE_SIM_INTERVAL; }
                    winit::keyboard::KeyCode::Digit2 => { self.world.game_speed = 2; self.sim_interval_ms = BASE_SIM_INTERVAL / 2; }
                    winit::keyboard::KeyCode::Digit3 => { self.world.game_speed = 3; self.sim_interval_ms = BASE_SIM_INTERVAL / 4; }
                    winit::keyboard::KeyCode::KeyE => {
                        if self.world.phase == GamePhase::Playing {
                            self.world.show_evolution = !self.world.show_evolution;
                            self.world.selected_upgrade = None;
                        }
                    }
                    winit::keyboard::KeyCode::KeyG => {
                        self.show_grid = !self.show_grid;
                    }
                    _ => {}
                }
            }
            WindowEvent::RedrawRequested => {
                // Splash screen auto-transition
                if self.world.phase == GamePhase::SplashScreen {
                    if self.splash_start.is_none() {
                        self.splash_start = Some(Instant::now());
                    }
                    if let Some(start) = self.splash_start {
                        let elapsed_ms = start.elapsed().as_millis() as u64;
                        if elapsed_ms >= crate::theme::splash_duration_ms() {
                            self.world.phase = GamePhase::TitleScreen;
                        }
                    }
                }

                if self.sim_interval_ms != u64::MAX { self.sim_interval_ms = BASE_SIM_INTERVAL / self.world.game_speed as u64; }
                if self.world.phase == GamePhase::Playing && self.sim_interval_ms != u64::MAX && self.last_sim_tick.elapsed().as_millis() >= self.sim_interval_ms as u128 {
                    self.world.advance();
                    self.last_sim_tick = Instant::now();
                    while self.world.news.len() > 5 { self.world.news.remove(0); }
                }
                if let (Some(r), Some(w)) = (self.renderer.as_mut(), self.window.as_ref()) {
                    match r.render(&mut self.world, w, self.hovered_region, self.show_grid) {
                        Ok(_) => {}
                        Err(SurfaceError::Lost) => r.resize(r.size),
                        Err(SurfaceError::OutOfMemory) => event_loop.exit(),
                        Err(e) => log::error!("Render error: {e:?}"),
                    }
                }
                if let Some(w) = self.window.as_ref() { w.request_redraw(); }
            }
            _ => {}
        }
    }
}

pub fn run() {
    // Initialize theme from theme.toml
    epidemic_core::ThemeConfig::load(); // verify it parses
    crate::theme::init_theme();

    let event_loop = EventLoop::new().unwrap();
    event_loop.set_control_flow(winit::event_loop::ControlFlow::Poll);
    let mut app = App::new();
    event_loop.run_app(&mut app).unwrap();
}
