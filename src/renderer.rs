use anyhow::{Context, Result};
use std::sync::Arc;
use wgpu::util::DeviceExt;
use winit::window::Window;

use crate::capture::Frame;

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Uniforms {
    pub zoom_level: f32,
    pub flipped: f32,
    pub fit_mode: f32,
    pub shape: f32,
    pub video_aspect: f32,
    pub window_aspect: f32,
    // Menu overlay position/size
    pub menu_x: f32,
    pub menu_y: f32,
    pub menu_w: f32,
    pub menu_h: f32,
    pub surface_w: f32,
    pub surface_h: f32,
}

impl Default for Uniforms {
    fn default() -> Self {
        Self {
            zoom_level: 0.0,
            flipped: 0.0,
            fit_mode: 0.0,
            shape: 0.0,
            video_aspect: 1.0,
            window_aspect: 1.0,
            menu_x: 0.0,
            menu_y: 0.0,
            menu_w: 0.0,
            menu_h: 0.0,
            surface_w: 320.0,
            surface_h: 320.0,
        }
    }
}

pub struct Renderer {
    device: wgpu::Device,
    queue: wgpu::Queue,
    surface: wgpu::Surface<'static>,
    surface_config: wgpu::SurfaceConfiguration,
    // Video pipeline
    video_pipeline: wgpu::RenderPipeline,
    bind_group_layout: wgpu::BindGroupLayout,
    video_bind_group: Option<wgpu::BindGroup>,
    uniform_buffer: wgpu::Buffer,
    pub uniforms: Uniforms,
    video_texture: Option<wgpu::Texture>,
    video_size: (u32, u32),
    sampler: wgpu::Sampler,
    // Menu pipeline
    menu_pipeline: wgpu::RenderPipeline,
    menu_texture: Option<wgpu::Texture>,
    menu_bind_group: Option<wgpu::BindGroup>,
    menu_size: (u32, u32),
    pub menu_visible: bool,
}

impl Renderer {
    pub fn new(window: Arc<Window>) -> Result<Self> {
        let size = window.inner_size();

        let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            ..Default::default()
        });

        let surface = instance
            .create_surface(window.clone())
            .context("Failed to create surface")?;

        let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::LowPower,
            compatible_surface: Some(&surface),
            force_fallback_adapter: false,
        }))
        .context("Failed to find a suitable GPU adapter")?;

        let (device, queue) = pollster::block_on(adapter.request_device(
            &wgpu::DeviceDescriptor {
                label: Some("cam-overlay device"),
                required_features: wgpu::Features::empty(),
                required_limits: wgpu::Limits::default(),
                ..Default::default()
            },
            None,
        ))
        .context("Failed to create device")?;

        let surface_caps = surface.get_capabilities(&adapter);
        let surface_format = surface_caps.formats[0];

        let alpha_mode = if surface_caps
            .alpha_modes
            .contains(&wgpu::CompositeAlphaMode::PreMultiplied)
        {
            wgpu::CompositeAlphaMode::PreMultiplied
        } else if surface_caps
            .alpha_modes
            .contains(&wgpu::CompositeAlphaMode::PostMultiplied)
        {
            wgpu::CompositeAlphaMode::PostMultiplied
        } else {
            surface_caps.alpha_modes[0]
        };

        let surface_config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: size.width.max(1),
            height: size.height.max(1),
            present_mode: wgpu::PresentMode::AutoVsync,
            alpha_mode,
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };
        surface.configure(&device, &surface_config);

        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("shader.wgsl").into()),
        });

        let uniform_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("uniform buffer"),
            contents: bytemuck::cast_slice(&[Uniforms::default()]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            ..Default::default()
        });

        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("bind group layout"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT | wgpu::ShaderStages::VERTEX,
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
                        multisampled: false,
                        view_dimension: wgpu::TextureViewDimension::D2,
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
            ],
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("pipeline layout"),
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });

        let video_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("video pipeline"),
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
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
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

        let menu_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("menu pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_menu"),
                buffers: &[],
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_menu"),
                targets: &[Some(wgpu::ColorTargetState {
                    format: surface_format,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
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

        let mut uniforms = Uniforms::default();
        uniforms.surface_w = size.width.max(1) as f32;
        uniforms.surface_h = size.height.max(1) as f32;

        Ok(Self {
            device,
            queue,
            surface,
            surface_config,
            video_pipeline,
            bind_group_layout,
            video_bind_group: None,
            uniform_buffer,
            uniforms,
            video_texture: None,
            video_size: (0, 0),
            sampler,
            menu_pipeline,
            menu_texture: None,
            menu_bind_group: None,
            menu_size: (0, 0),
            menu_visible: false,
        })
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        if width > 0 && height > 0 {
            self.surface_config.width = width;
            self.surface_config.height = height;
            self.surface.configure(&self.device, &self.surface_config);
            self.uniforms.window_aspect = width as f32 / height as f32;
            self.uniforms.surface_w = width as f32;
            self.uniforms.surface_h = height as f32;
        }
    }

    pub fn upload_frame(&mut self, frame: &Frame) {
        if self.video_size != (frame.width, frame.height) {
            self.video_size = (frame.width, frame.height);
            self.uniforms.video_aspect = frame.width as f32 / frame.height as f32;

            let texture = self.device.create_texture(&wgpu::TextureDescriptor {
                label: Some("video texture"),
                size: wgpu::Extent3d {
                    width: frame.width,
                    height: frame.height,
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: wgpu::TextureFormat::Rgba8UnormSrgb,
                usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
                view_formats: &[],
            });
            self.video_texture = Some(texture);
            self.video_bind_group = None;
        }

        if let Some(texture) = &self.video_texture {
            self.queue.write_texture(
                wgpu::TexelCopyTextureInfo {
                    texture,
                    mip_level: 0,
                    origin: wgpu::Origin3d::ZERO,
                    aspect: wgpu::TextureAspect::All,
                },
                &frame.data,
                wgpu::TexelCopyBufferLayout {
                    offset: 0,
                    bytes_per_row: Some(4 * frame.width),
                    rows_per_image: Some(frame.height),
                },
                wgpu::Extent3d {
                    width: frame.width,
                    height: frame.height,
                    depth_or_array_layers: 1,
                },
            );
        }

        if self.video_bind_group.is_none() {
            if let Some(texture) = &self.video_texture {
                let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
                self.video_bind_group =
                    Some(self.device.create_bind_group(&wgpu::BindGroupDescriptor {
                        label: Some("video bind group"),
                        layout: &self.bind_group_layout,
                        entries: &[
                            wgpu::BindGroupEntry {
                                binding: 0,
                                resource: self.uniform_buffer.as_entire_binding(),
                            },
                            wgpu::BindGroupEntry {
                                binding: 1,
                                resource: wgpu::BindingResource::TextureView(&view),
                            },
                            wgpu::BindGroupEntry {
                                binding: 2,
                                resource: wgpu::BindingResource::Sampler(&self.sampler),
                            },
                        ],
                    }));
            }
        }
    }

    pub fn upload_menu(&mut self, pixels: &[u8], width: u32, height: u32) {
        if self.menu_size != (width, height) {
            self.menu_size = (width, height);
            let texture = self.device.create_texture(&wgpu::TextureDescriptor {
                label: Some("menu texture"),
                size: wgpu::Extent3d {
                    width,
                    height,
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: wgpu::TextureFormat::Rgba8UnormSrgb,
                usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
                view_formats: &[],
            });
            self.menu_texture = Some(texture);
            self.menu_bind_group = None;
        }

        if let Some(texture) = &self.menu_texture {
            self.queue.write_texture(
                wgpu::TexelCopyTextureInfo {
                    texture,
                    mip_level: 0,
                    origin: wgpu::Origin3d::ZERO,
                    aspect: wgpu::TextureAspect::All,
                },
                pixels,
                wgpu::TexelCopyBufferLayout {
                    offset: 0,
                    bytes_per_row: Some(4 * width),
                    rows_per_image: Some(height),
                },
                wgpu::Extent3d {
                    width,
                    height,
                    depth_or_array_layers: 1,
                },
            );
        }

        if self.menu_bind_group.is_none() {
            if let Some(texture) = &self.menu_texture {
                let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
                self.menu_bind_group =
                    Some(self.device.create_bind_group(&wgpu::BindGroupDescriptor {
                        label: Some("menu bind group"),
                        layout: &self.bind_group_layout,
                        entries: &[
                            wgpu::BindGroupEntry {
                                binding: 0,
                                resource: self.uniform_buffer.as_entire_binding(),
                            },
                            wgpu::BindGroupEntry {
                                binding: 1,
                                resource: wgpu::BindingResource::TextureView(&view),
                            },
                            wgpu::BindGroupEntry {
                                binding: 2,
                                resource: wgpu::BindingResource::Sampler(&self.sampler),
                            },
                        ],
                    }));
            }
        }
    }

    pub fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        self.queue.write_buffer(
            &self.uniform_buffer,
            0,
            bytemuck::cast_slice(&[self.uniforms]),
        );

        let output = self.surface.get_current_texture()?;
        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("render encoder"),
            });

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("render pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                ..Default::default()
            });

            // Draw video
            if let Some(bg) = &self.video_bind_group {
                render_pass.set_pipeline(&self.video_pipeline);
                render_pass.set_bind_group(0, bg, &[]);
                render_pass.draw(0..3, 0..1);
            }

            // Draw menu overlay
            if self.menu_visible {
                if let Some(bg) = &self.menu_bind_group {
                    render_pass.set_pipeline(&self.menu_pipeline);
                    render_pass.set_bind_group(0, bg, &[]);
                    render_pass.draw(0..3, 0..1);
                }
            }
        }

        self.queue.submit(std::iter::once(encoder.finish()));
        output.present();
        Ok(())
    }
}
