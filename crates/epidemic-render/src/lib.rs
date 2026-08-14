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

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct Uniforms {
    time: f32,
    map_w: f32,
    map_h: f32,
    _pad: f32,
}

pub struct Renderer {
    surface: wgpu::Surface<'static>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    size: PhysicalSize<u32>,
    pipeline: wgpu::RenderPipeline,
    uniform_buffer: wgpu::Buffer,
    bind_group: wgpu::BindGroup,
    start_time: Instant,
    map_texture: wgpu::Texture,
    logo_texture: Option<egui::TextureHandle>,
    // egui
    egui_ctx: egui::Context,
    egui_state: egui_winit::State,
    egui_renderer: egui_wgpu::Renderer,
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
            .expect("Failed to find a suitable GPU adapter");

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
        let surface_format = surface_caps
            .formats
            .iter()
            .find(|f| f.is_srgb())
            .copied()
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

        // Map texture
        let map_tex = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("map-texture"),
            size: wgpu::Extent3d {
                width: world.lookup_w as u32,
                height: world.lookup_h as u32,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8Unorm,
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

        let uniforms = Uniforms {
            time: 0.0,
            map_w: world.lookup_w as f32,
            map_h: world.lookup_h as f32,
            _pad: 0.0,
        };
        let uniform_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("uniforms"),
            contents: bytemuck::cast_slice(&[uniforms]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let bgl = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("bgl"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: false },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::NonFiltering),
                    count: None,
                },
            ],
        });

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("bg"),
            layout: &bgl,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: uniform_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(&map_view),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::Sampler(&map_sampler),
                },
            ],
        });

        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("map-shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("../shaders/map.wgsl").into()),
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("map-pl"),
            bind_group_layouts: &[&bgl],
            push_constant_ranges: &[],
        });

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("map-pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                buffers: &[],
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format: surface_format,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: Default::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                ..Default::default()
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
            cache: None,
        });

        // egui setup
        let egui_ctx = egui::Context::default();
        let egui_state = egui_winit::State::new(
            egui_ctx.clone(),
            egui::ViewportId::ROOT,
            &window,
            None,
            None,
            None,
        );
        let egui_renderer = egui_wgpu::Renderer::new(&device, surface_format, None, 1, false);

        Self {
            surface,
            device,
            queue,
            config,
            size,
            pipeline,
            uniform_buffer,
            bind_group,
            start_time: Instant::now(),
            map_texture: map_tex,
            logo_texture: None,
            egui_ctx,
            egui_state,
            egui_renderer,
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
        // Don't consume mouse clicks — let them pass through to game logic
        if matches!(event, WindowEvent::MouseInput { .. }) {
            return false;
        }
        resp.consumed
    }

    pub fn render(&mut self, world: &mut World, window: &Window, hovered_region: Option<u16>) -> Result<(), SurfaceError> {
        let elapsed = self.start_time.elapsed().as_secs_f32();

        // Load logo on first frame
        if self.logo_texture.is_none() {
            if let Ok(img) = load_logo() {
                let size = [img.width() as usize, img.height() as usize];
                let pixels = img.into_raw();
                let color_image = egui::ColorImage::from_rgba_unmultiplied(size, &pixels);
                let tex = self.egui_ctx.load_texture(
                    "logo",
                    color_image,
                    egui::TextureOptions::LINEAR,
                );
                self.logo_texture = Some(tex);
            }
        }

        // Update uniforms
        let uniforms = Uniforms {
            time: elapsed,
            map_w: world.lookup_w as f32,
            map_h: world.lookup_h as f32,
            _pad: 0.0,
        };
        self.queue
            .write_buffer(&self.uniform_buffer, 0, bytemuck::cast_slice(&[uniforms]));

        // Update map texture
        let tex_data = build_map_texture(world, hovered_region);
        self.queue.write_texture(
            wgpu::TexelCopyTextureInfo {
                texture: &self.map_texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            &tex_data,
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(world.lookup_w as u32 * 4),
                rows_per_image: Some(world.lookup_h as u32),
            },
            wgpu::Extent3d {
                width: world.lookup_w as u32,
                height: world.lookup_h as u32,
                depth_or_array_layers: 1,
            },
        );

        // egui
        let logo = self.logo_texture.clone();
        let raw_input = self.egui_state.take_egui_input(window);
        let full_output = self.egui_ctx.run(raw_input, |ctx| {
            build_ui(ctx, world, logo.as_ref());
        });
        self.egui_state
            .handle_platform_output(window, full_output.platform_output);

        let paint_jobs = self
            .egui_ctx
            .tessellate(full_output.shapes, full_output.pixels_per_point);

        // Upload egui textures
        for (id, delta) in &full_output.textures_delta.set {
            self.egui_renderer.update_texture(&self.device, &self.queue, *id, delta);
        }

        let output = self.surface.get_current_texture()?;
        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("encoder"),
            });

        // Pass 1: Game map
        {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("map-pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.01,
                            g: 0.02,
                            b: 0.05,
                            a: 1.0,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });

            pass.set_pipeline(&self.pipeline);
            pass.set_bind_group(0, &self.bind_group, &[]);
            pass.draw(0..6, 0..1);
        }

        // Pass 2: egui HUD
        {
            let screen_desc = egui_wgpu::ScreenDescriptor {
                size_in_pixels: [self.config.width, self.config.height],
                pixels_per_point: window.scale_factor() as f32,
            };

            self.egui_renderer.update_buffers(
                &self.device,
                &self.queue,
                &mut encoder,
                &paint_jobs,
                &screen_desc,
            );

            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("egui-pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Load,
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });

            // SAFETY: egui-wgpu requires 'static but the pass only needs to live
            // through this block. We drop the pass before encoder.finish().
            let pass_mut: &mut wgpu::RenderPass<'static> =
                unsafe { std::mem::transmute(&mut pass) };
            self.egui_renderer.render(pass_mut, &paint_jobs, &screen_desc);
        }

        self.queue.submit(std::iter::once(encoder.finish()));
        output.present();

        // Free egui textures
        for id in &full_output.textures_delta.free {
            self.egui_renderer.free_texture(id);
        }

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

fn build_map_texture(world: &World, hovered_region: Option<u16>) -> Vec<u8> {
    let w = world.lookup_w;
    let h = world.lookup_h;
    let mut data = vec![0u8; w * h * 4];

    // Pre-compute border mask
    let mut is_border = vec![false; w * h];
    for y in 0..h {
        for x in 0..w {
            let id = world.svg_lookup[y * w + x];
            if id == 0 {
                continue;
            }
            // Check 4 neighbors (N, S, E, W)
            let neighbors = [
                if y > 0 { Some(world.svg_lookup[(y - 1) * w + x]) } else { None },
                if y + 1 < h { Some(world.svg_lookup[(y + 1) * w + x]) } else { None },
                if x > 0 { Some(world.svg_lookup[y * w + (x - 1)]) } else { None },
                if x + 1 < w { Some(world.svg_lookup[y * w + (x + 1)]) } else { None },
            ];
            for n in neighbors {
                if let Some(nid) = n {
                    if nid != id {
                        is_border[y * w + x] = true;
                        break;
                    }
                }
            }
        }
    }

    for y in 0..h {
        for x in 0..w {
            let region_id = world.svg_lookup[y * w + x];
            let idx = (y * w + x) * 4;
            let pi = y * w + x;

            if region_id == 0 {
                // Ocean
                data[idx] = 5;
                data[idx + 1] = 10;
                data[idx + 2] = 25;
                data[idx + 3] = 255;
            } else if is_border[pi] {
                // Country border — dark outline
                data[idx] = 15;
                data[idx + 1] = 15;
                data[idx + 2] = 20;
                data[idx + 3] = 255;
            } else if let Some(region) = world.regions.iter().find(|r| r.id == region_id) {
                let is_selected = hovered_region == Some(region_id);

                if region.fallen {
                    data[idx] = 30;
                    data[idx + 1] = 30;
                    data[idx + 2] = 30;
                    data[idx + 3] = 255;
                } else if region.infected == 0 {
                    // Healthy land
                    let base_r: u8 = 20;
                    let base_g: u8 = 70;
                    let base_b: u8 = 30;
                    if is_selected {
                        // White tint on hover
                        data[idx] = base_r.saturating_add(60);
                        data[idx + 1] = base_g.saturating_add(60);
                        data[idx + 2] = base_b.saturating_add(60);
                    } else {
                        data[idx] = base_r;
                        data[idx + 1] = base_g;
                        data[idx + 2] = base_b;
                    }
                    data[idx + 3] = 255;
                } else {
                    // Infected — blend green -> red based on infection %
                    let pct = region.infection_pct();
                    let r = (20.0 + pct * 200.0) as u8;
                    let g = (70.0 * (1.0 - pct)) as u8;
                    let b = (30.0 * (1.0 - pct)) as u8;
                    if is_selected {
                        data[idx] = r.saturating_add(50);
                        data[idx + 1] = g.saturating_add(50);
                        data[idx + 2] = b.saturating_add(50);
                    } else {
                        data[idx] = r;
                        data[idx + 1] = g;
                        data[idx + 2] = b;
                    }
                    data[idx + 3] = 255;
                }
            }
        }
    }

    data
}

fn build_ui(ctx: &egui::Context, world: &mut World, logo: Option<&egui::TextureHandle>) {
    // Moo UI dark theme colors
    let bg = egui::Color32::from_rgb(33, 37, 41);
    let surface = egui::Color32::from_rgb(52, 58, 64);
    let border = egui::Color32::from_rgb(73, 80, 87);
    let text = egui::Color32::from_rgb(222, 226, 230);
    let muted = egui::Color32::from_rgb(113, 113, 122);
    let heading = egui::Color32::from_rgb(255, 255, 255);
    let success = egui::Color32::from_rgb(4, 120, 87);
    let danger = egui::Color32::from_rgb(231, 0, 11);
    let info = egui::Color32::from_rgb(3, 105, 161);
    let warning = egui::Color32::from_rgb(180, 83, 9);

    // Apply style
    let mut style = (*ctx.style()).clone();
    style.spacing.item_spacing = egui::vec2(8.0, 6.0);
    style.spacing.button_padding = egui::vec2(12.0, 6.0);
    style.visuals.window_fill = bg;
    style.visuals.panel_fill = bg;
    style.visuals.override_text_color = Some(text);
    ctx.set_style(style);

    match world.phase {
        GamePhase::TitleScreen => build_title_screen(ctx, world, logo, bg, surface, border, text, heading, info),
        GamePhase::PathogenSelect => build_pathogen_select(ctx, world, bg, surface, border, text, muted, heading, success, danger, info, warning),
        GamePhase::DifficultySelect => build_difficulty_select(ctx, world, bg, surface, border, text, heading, info),
        GamePhase::SelectOrigin | GamePhase::Playing | GamePhase::Won | GamePhase::Lost => {
            build_gameplay_hud(ctx, world, bg, surface, border, text, muted, heading, success, danger, info, warning);
        }
    }
}

fn build_title_screen(
    ctx: &egui::Context, world: &mut World, logo: Option<&egui::TextureHandle>,
    bg: egui::Color32, surface: egui::Color32, border: egui::Color32,
    text: egui::Color32, heading: egui::Color32, info: egui::Color32,
) {
    egui::CentralPanel::default()
        .frame(egui::Frame::new().fill(bg))
        .show(ctx, |ui| {
            ui.vertical_centered(|ui| {
                ui.add_space(80.0);

                // Logo
                if let Some(tex) = logo {
                    ui.image(tex);
                } else {
                    ui.label(egui::RichText::new("EPIDEMIC").size(48.0).strong().color(heading));
                    ui.label(egui::RichText::new("Natural Strategies").size(18.0).color(egui::Color32::from_rgb(161, 161, 170)));
                }

                ui.add_space(60.0);

                // New Game button
                let btn = egui::Button::new(
                    egui::RichText::new("NEW GAME").size(18.0).strong().color(heading)
                )
                .min_size(egui::vec2(200.0, 48.0))
                .fill(info)
                .corner_radius(egui::CornerRadius::same(8));
                if ui.add(btn).clicked() {
                    world.phase = GamePhase::PathogenSelect;
                }

                ui.add_space(16.0);

                // Version
                ui.label(egui::RichText::new("v0.1.0 — Open Source").size(11.0).color(egui::Color32::from_rgb(73, 80, 87)));
            });
        });
}

fn build_pathogen_select(
    ctx: &egui::Context, world: &mut World,
    bg: egui::Color32, surface: egui::Color32, border: egui::Color32,
    text: egui::Color32, muted: egui::Color32, heading: egui::Color32,
    success: egui::Color32, danger: egui::Color32, info: egui::Color32, warning: egui::Color32,
) {
    egui::CentralPanel::default()
        .frame(egui::Frame::new().fill(bg).inner_margin(egui::Margin::same(40)))
        .show(ctx, |ui| {
            ui.vertical_centered(|ui| {
                ui.label(egui::RichText::new("SELECT PATHOGEN").size(24.0).strong().color(heading));
                ui.add_space(24.0);

                let pathogens = [
                    (epidemic_core::PathogenType::Bacteria, "Standard pathogen. Cheap to devolve.", success),
                    (epidemic_core::PathogenType::Virus, "Random mutations. Uncontrollable.", danger),
                    (epidemic_core::PathogenType::Fungus, "Slow spread. Launch spores.", egui::Color32::from_rgb(139, 90, 43)),
                    (epidemic_core::PathogenType::Parasite, "Stealth. Low severity.", egui::Color32::from_rgb(100, 140, 100)),
                    (epidemic_core::PathogenType::Prion, "Slow infection. Slows cure.", egui::Color32::from_rgb(120, 80, 160)),
                    (epidemic_core::PathogenType::NanoVirus, "Cure starts immediately.", info),
                    (epidemic_core::PathogenType::BioWeapon, "Innate lethality. Suppress it.", danger),
                ];

                egui::Grid::new("pathogen_grid")
                    .num_columns(2)
                    .spacing([16.0, 12.0])
                    .show(ui, |ui| {
                        for (i, (ptype, desc, color)) in pathogens.iter().enumerate() {
                            let card = egui::Frame::new()
                                .fill(surface)
                                .corner_radius(egui::CornerRadius::same(8))
                                .stroke(egui::Stroke::new(1.0, border))
                                .inner_margin(egui::Margin::same(16));

                            card.show(ui, |ui| {
                                ui.set_min_width(280.0);
                                ui.label(egui::RichText::new(ptype.name()).size(16.0).strong().color(*color));
                                ui.add_space(4.0);
                                ui.label(egui::RichText::new(*desc).size(12.0).color(muted));
                                ui.add_space(8.0);
                                let btn = egui::Button::new(
                                    egui::RichText::new("SELECT").size(12.0).strong().color(heading)
                                )
                                .fill(*color)
                                .corner_radius(egui::CornerRadius::same(6));
                                if ui.add(btn).clicked() {
                                    world.init_disease("Epidemic", *ptype);
                                    world.phase = GamePhase::DifficultySelect;
                                }
                            });

                            if (i + 1) % 2 == 0 {
                                ui.end_row();
                            }
                        }
                    });
            });
        });
}

fn build_difficulty_select(
    ctx: &egui::Context, world: &mut World,
    bg: egui::Color32, surface: egui::Color32, border: egui::Color32,
    text: egui::Color32, heading: egui::Color32, info: egui::Color32,
) {
    egui::CentralPanel::default()
        .frame(egui::Frame::new().fill(bg).inner_margin(egui::Margin::same(40)))
        .show(ctx, |ui| {
            ui.vertical_centered(|ui| {
                ui.label(egui::RichText::new("SELECT DIFFICULTY").size(24.0).strong().color(heading));
                ui.add_space(8.0);
                ui.label(egui::RichText::new(format!("Pathogen: {}", world.disease.pathogen_type.name()))
                    .size(14.0).color(egui::Color32::from_rgb(161, 161, 170)));
                ui.add_space(32.0);

                let difficulties = [
                    (epidemic_core::Difficulty::Casual, "Easy. Slower cure, weaker borders.", egui::Color32::from_rgb(4, 120, 87)),
                    (epidemic_core::Difficulty::Normal, "Standard challenge.", info),
                    (epidemic_core::Difficulty::Brutal, "Faster cure, stronger borders.", egui::Color32::from_rgb(180, 83, 9)),
                    (epidemic_core::Difficulty::MegaBrutal, "Extreme. For masochists.", egui::Color32::from_rgb(231, 0, 11)),
                ];

                for (diff, desc, color) in difficulties {
                    let card = egui::Frame::new()
                        .fill(surface)
                        .corner_radius(egui::CornerRadius::same(8))
                        .stroke(egui::Stroke::new(1.0, border))
                        .inner_margin(egui::Margin::same(16));

                    card.show(ui, |ui| {
                        ui.set_min_width(400.0);
                        ui.horizontal(|ui| {
                            ui.label(egui::RichText::new(diff.name()).size(16.0).strong().color(color));
                            ui.add_space(16.0);
                            ui.label(egui::RichText::new(desc).size(12.0).color(egui::Color32::from_rgb(161, 161, 170)));
                            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                let btn = egui::Button::new(
                                    egui::RichText::new("SELECT").size(12.0).strong().color(heading)
                                )
                                .fill(color)
                                .corner_radius(egui::CornerRadius::same(6));
                                if ui.add(btn).clicked() {
                                    world.difficulty = diff;
                                    world.phase = GamePhase::SelectOrigin;
                                }
                            });
                        });
                    });
                    ui.add_space(8.0);
                }
            });
        });
}

fn build_gameplay_hud(
    ctx: &egui::Context, world: &mut World,
    bg: egui::Color32, surface: egui::Color32, border: egui::Color32,
    text: egui::Color32, muted: egui::Color32, heading: egui::Color32,
    success: egui::Color32, danger: egui::Color32, info: egui::Color32, warning: egui::Color32,
) {
    // Left stats panel
    egui::SidePanel::left("stats")
        .exact_width(240.0)
        .frame(egui::Frame::new().fill(bg).stroke(egui::Stroke::new(1.0, border)).inner_margin(egui::Margin::same(16)))
        .show(ctx, |ui| {
            ui.label(egui::RichText::new("EPIDEMIC NS").size(20.0).strong().color(heading));
            ui.add_space(4.0);
            ui.horizontal(|ui| {
                ui.label(egui::RichText::new(format!("Tick {}", world.tick)).color(muted).size(12.0));
                ui.separator();
                ui.label(egui::RichText::new(format!("{}x", world.game_speed)).color(muted).size(12.0));
            });

            ui.add_space(8.0);
            ui.separator();
            ui.add_space(8.0);

            egui::Frame::new().fill(surface).corner_radius(egui::CornerRadius::same(8)).inner_margin(egui::Margin::same(12)).show(ui, |ui| {
                ui.label(egui::RichText::new("Population").size(11.0).color(muted).strong());
                ui.add_space(4.0);
                stat_row(ui, "Healthy", &format_num(world.total_healthy), success);
                stat_row(ui, "Infected", &format_num(world.total_infected), danger);
                stat_row(ui, "Dead", &format_num(world.total_dead), egui::Color32::from_rgb(108, 117, 125));
            });

            ui.add_space(8.0);

            egui::Frame::new().fill(surface).corner_radius(egui::CornerRadius::same(8)).inner_margin(egui::Margin::same(12)).show(ui, |ui| {
                ui.label(egui::RichText::new("DNA Points").size(11.0).color(muted).strong());
                ui.add_space(2.0);
                ui.label(egui::RichText::new(format!("{}", world.dna_points)).size(24.0).strong().color(info));
            });

            ui.add_space(8.0);

            egui::Frame::new().fill(surface).corner_radius(egui::CornerRadius::same(8)).inner_margin(egui::Margin::same(12)).show(ui, |ui| {
                ui.label(egui::RichText::new("Cure Progress").size(11.0).color(muted).strong());
                ui.add_space(4.0);
                ui.add(egui::ProgressBar::new(world.cure_overall / 100.0).fill(info).corner_radius(egui::CornerRadius::same(4)));
                ui.label(egui::RichText::new(format!("{:.1}% — {}", world.cure_overall, world.cure_phase.name())).size(12.0).color(info));
            });

            ui.add_space(8.0);

            egui::Frame::new().fill(surface).corner_radius(egui::CornerRadius::same(8)).inner_margin(egui::Margin::same(12)).show(ui, |ui| {
                ui.label(egui::RichText::new(format!("Disease: {}", world.disease.name)).size(11.0).color(muted).strong());
                ui.add_space(4.0);
                stat_row(ui, "Infectivity", &format!("{:.1}", world.disease.effective_infectivity()), danger);
                stat_row(ui, "Severity", &format!("{:.1}", world.disease.effective_severity()), warning);
                stat_row(ui, "Lethality", &format!("{:.1}", world.disease.effective_lethality()), egui::Color32::from_rgb(180, 30, 30));
            });

            ui.add_space(8.0);

            ui.label(egui::RichText::new("Speed").size(11.0).color(muted).strong());
            ui.add_space(4.0);
            ui.horizontal(|ui| {
                for (label, speed) in [("1x", 1), ("2x", 2), ("3x", 3)] {
                    let active = world.game_speed == speed;
                    let btn = egui::Button::new(egui::RichText::new(label).size(13.0).color(if active { heading } else { text }))
                        .fill(if active { info } else { surface })
                        .corner_radius(egui::CornerRadius::same(6))
                        .stroke(egui::Stroke::new(1.0, if active { info } else { border }));
                    if ui.add(btn).clicked() { world.game_speed = speed; }
                }
            });

            ui.add_space(12.0);

            match world.phase {
                GamePhase::SelectOrigin => {
                    egui::Frame::new().fill(egui::Color32::from_rgba_premultiplied(180, 83, 9, 30)).corner_radius(egui::CornerRadius::same(8)).inner_margin(egui::Margin::same(12)).show(ui, |ui| {
                        ui.label(egui::RichText::new("Select a country to begin the outbreak").size(12.0).color(warning));
                    });
                }
                GamePhase::Playing => {
                    egui::Frame::new().fill(egui::Color32::from_rgba_premultiplied(231, 0, 11, 20)).corner_radius(egui::CornerRadius::same(8)).inner_margin(egui::Margin::same(12)).show(ui, |ui| {
                        ui.label(egui::RichText::new("OUTBREAK ACTIVE").size(12.0).strong().color(danger));
                    });
                }
                GamePhase::Won => {
                    ui.label(egui::RichText::new("Humanity has fallen.").size(14.0).strong().color(success));
                }
                GamePhase::Lost => {
                    ui.label(egui::RichText::new("Cure completed.").size(14.0).strong().color(info));
                }
                _ => {}
            }
        });

    // Bottom news ticker
    egui::TopBottomPanel::bottom("news")
        .exact_height(36.0)
        .frame(egui::Frame::new().fill(bg).stroke(egui::Stroke::new(1.0, border)).inner_margin(egui::Margin::symmetric(16, 8)))
        .show(ctx, |ui| {
            ui.horizontal_centered(|ui| {
                if let Some(msg) = world.news.last() {
                    ui.label(egui::RichText::new(format!("BREAKING: {msg}")).size(12.0).color(warning));
                } else {
                    ui.label(egui::RichText::new("No active reports").size(12.0).color(muted));
                }
            });
        });

    // Right panel: Evolution menu
    if world.phase == GamePhase::Playing {
        egui::SidePanel::right("evolution")
            .exact_width(260.0)
            .frame(egui::Frame::new().fill(bg).stroke(egui::Stroke::new(1.0, border)).inner_margin(egui::Margin::same(12)))
            .show(ctx, |ui| {
                ui.label(egui::RichText::new("EVOLUTION").size(16.0).strong().color(heading));
                ui.add_space(4.0);
                ui.label(egui::RichText::new(format!("DNA: {}", world.dna_points)).size(13.0).color(info));
                ui.add_space(8.0);

                for (cat_name, cat_color, cat) in [
                    ("Transmission", success, epidemic_core::UpgradeCategory::Transmission),
                    ("Symptoms", warning, epidemic_core::UpgradeCategory::Symptom),
                    ("Abilities", info, epidemic_core::UpgradeCategory::Ability),
                ] {
                    egui::CollapsingHeader::new(egui::RichText::new(cat_name).color(cat_color).strong())
                        .default_open(true).show(ui, |ui| {
                        for upgrade in &world.upgrades {
                            if upgrade.category != cat { continue; }
                            let owned = world.disease.has_upgrade(upgrade.id);
                            let can_buy = world.disease.can_unlock(upgrade) && world.dna_points >= upgrade.cost;
                            let color = if owned { success } else if can_buy { text } else { muted };
                            let prefix = if owned { "\u{2713} " } else { "" };
                            ui.horizontal(|ui| {
                                let label = format!("{prefix}{} ({})", upgrade.name, upgrade.cost);
                                if owned {
                                    ui.label(egui::RichText::new(label).size(12.0).color(color));
                                } else if can_buy {
                                    if ui.button(egui::RichText::new(label).size(12.0).color(color)).clicked() {
                                        world.dna_points -= upgrade.cost;
                                        world.disease.unlock(upgrade);
                                    }
                                } else {
                                    ui.label(egui::RichText::new(label).size(12.0).color(color));
                                }
                            });
                        }
                    });
                    ui.add_space(4.0);
                }
            });
    }
}

fn stat_row(ui: &mut egui::Ui, label: &str, value: &str, color: egui::Color32) {
    ui.horizontal(|ui| {
        ui.label(egui::RichText::new(label).size(13.0).color(egui::Color32::from_rgb(161, 161, 170)));
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            ui.label(egui::RichText::new(value).size(13.0).strong().color(color));
        });
    });
}

fn load_logo() -> Result<image::RgbaImage, Box<dyn std::error::Error>> {
    let img = image::open("../assets/EPIDEMIC.png")
        .or_else(|_| image::open("assets/EPIDEMIC.png"))
        .or_else(|_| image::open("../Assets/EPIDEMIC.png"))
        .or_else(|_| image::open("Assets/EPIDEMIC.png"))?;
    Ok(img.to_rgba8())
}

fn format_num(n: u64) -> String {
    if n >= 1_000_000_000 {
        format!("{:.1}B", n as f64 / 1_000_000_000.0)
    } else if n >= 1_000_000 {
        format!("{:.1}M", n as f64 / 1_000_000.0)
    } else if n >= 1_000 {
        format!("{:.1}K", n as f64 / 1_000.0)
    } else {
        format!("{n}")
    }
}

struct App {
    window: Option<Arc<Window>>,
    renderer: Option<Renderer>,
    world: World,
    last_sim_tick: Instant,
    sim_interval_ms: u64,
    cursor_pos: PhysicalPosition<f64>,
    hovered_region: Option<u16>,
}

const BASE_SIM_INTERVAL: u64 = 60;

impl App {
    fn new() -> Self {
        let svg_content = std::fs::read_to_string("../assets/world.svg")
            .or_else(|_| std::fs::read_to_string("assets/world.svg"))
            .expect("Failed to load world.svg");

        let mut world = World::new(&svg_content);
        // Default to Bacteria for now — disease selection UI comes later
        world.init_disease("Epidemic", epidemic_core::PathogenType::Bacteria);

        Self {
            window: None,
            renderer: None,
            world,
            last_sim_tick: Instant::now(),
            sim_interval_ms: BASE_SIM_INTERVAL,
            cursor_pos: PhysicalPosition::new(0.0, 0.0),
            hovered_region: None,
        }
    }
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        if self.window.is_some() {
            return;
        }
        let attrs = Window::default_attributes()
            .with_title("Epidemic NS")
            .with_inner_size(PhysicalSize::new(1280, 720));
        let window = Arc::new(event_loop.create_window(attrs).unwrap());
        let renderer = pollster::block_on(Renderer::new(window.clone(), &self.world));
        self.window = Some(window);
        self.renderer = Some(renderer);
    }

    fn window_event(
        &mut self,
        event_loop: &winit::event_loop::ActiveEventLoop,
        _window_id: winit::window::WindowId,
        event: WindowEvent,
    ) {
        // Let egui consume events first
        if let (Some(r), Some(w)) = (self.renderer.as_mut(), self.window.as_ref()) {
            if r.handle_event(w, &event) {
                return;
            }
        }

        match event {
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::Resized(physical_size) => {
                if let Some(r) = self.renderer.as_mut() {
                    r.resize(physical_size);
                }
            }

            WindowEvent::CursorMoved { position, .. } => {
                self.cursor_pos = position;
                // Track hovered region
                if let Some(r) = self.renderer.as_ref() {
                    let (px, py) = r.screen_to_map(self.cursor_pos, &self.world);
                    self.hovered_region = self.world.region_at_pixel(px, py).map(|r| r.id);
                }
            }

            WindowEvent::MouseInput {
                state: ElementState::Pressed,
                button: MouseButton::Left,
                ..
            } => {
                if self.world.phase == GamePhase::SelectOrigin {
                    if let Some(r) = self.renderer.as_ref() {
                        let (px, py) = r.screen_to_map(self.cursor_pos, &self.world);
                        if let Some(region) = self.world.region_at_pixel(px, py) {
                            let rid = region.id;
                            let name = region.name.clone();
                            self.world.start_outbreak(rid);
                            if let Some(w) = self.window.as_ref() {
                                w.set_title(&format!("Epidemic NS — {name} outbreak!"));
                            }
                        }
                    }
                }
            }

            WindowEvent::KeyboardInput {
                event:
                    winit::event::KeyEvent {
                        physical_key: winit::keyboard::PhysicalKey::Code(keycode),
                        state: ElementState::Pressed,
                        ..
                    },
                ..
            } => {
                match keycode {
                    winit::keyboard::KeyCode::Space => {
                        if self.world.phase == GamePhase::Playing {
                            self.sim_interval_ms = if self.sim_interval_ms == u64::MAX {
                                BASE_SIM_INTERVAL * self.world.game_speed as u64
                            } else {
                                u64::MAX
                            };
                        }
                    }
                    winit::keyboard::KeyCode::Escape => event_loop.exit(),
                    winit::keyboard::KeyCode::Digit1 => {
                        self.world.game_speed = 1;
                        self.sim_interval_ms = BASE_SIM_INTERVAL;
                    }
                    winit::keyboard::KeyCode::Digit2 => {
                        self.world.game_speed = 2;
                        self.sim_interval_ms = BASE_SIM_INTERVAL / 2;
                    }
                    winit::keyboard::KeyCode::Digit3 => {
                        self.world.game_speed = 3;
                        self.sim_interval_ms = BASE_SIM_INTERVAL / 4;
                    }
                    _ => {}
                }
            }

            WindowEvent::RedrawRequested => {
                // Sync sim interval with game speed from egui buttons
                if self.sim_interval_ms != u64::MAX {
                    self.sim_interval_ms = BASE_SIM_INTERVAL / self.world.game_speed as u64;
                }

                if self.world.phase == GamePhase::Playing
                    && self.sim_interval_ms != u64::MAX
                    && self.last_sim_tick.elapsed().as_millis() >= self.sim_interval_ms as u128
                {
                    self.world.advance();
                    self.last_sim_tick = Instant::now();

                    // Pop news
                    while self.world.news.len() > 5 {
                        self.world.news.remove(0);
                    }
                }

                if let (Some(r), Some(w)) = (self.renderer.as_mut(), self.window.as_ref()) {
                    match r.render(&mut self.world, w, self.hovered_region) {
                        Ok(_) => {}
                        Err(SurfaceError::Lost) => r.resize(r.size),
                        Err(SurfaceError::OutOfMemory) => event_loop.exit(),
                        Err(e) => log::error!("Render error: {e:?}"),
                    }
                }
                if let Some(w) = self.window.as_ref() {
                    w.request_redraw();
                }
            }
            _ => {}
        }
    }
}

pub fn run() {
    let event_loop = EventLoop::new().unwrap();
    event_loop.set_control_flow(winit::event_loop::ControlFlow::Poll);
    let mut app = App::new();
    event_loop.run_app(&mut app).unwrap();
}
