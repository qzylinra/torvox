use std::sync::Arc;
use thiserror::Error;
use wgpu::util::DeviceExt;

fn log_gpu_error(error: &wgpu::Error) {
    log::error!("GPU_UNCAPTURED_ERROR: {error:#?}");
}

#[derive(Debug, Error)]
pub enum GpuError {
    #[error("wgpu request adapter failed")]
    NoAdapter,
    #[error("wgpu request device failed: {0}")]
    DeviceRequest(String),
    #[error("surface creation failed: {0}")]
    Surface(String),
    #[error("shader compilation failed: {0}")]
    Shader(String),
    #[error("buffer creation failed: {0}")]
    Buffer(String),
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct CellInstance {
    pub cell_pos: [f32; 2],
    pub atlas_offset: [f32; 2],
    pub atlas_size: [f32; 2],
    pub fg_color: [f32; 4],
    pub bg_color: [f32; 4],
    pub flags: f32,
    pub _pad: [f32; 3],
}

impl CellInstance {
    pub const ATTRIBS: [wgpu::VertexAttribute; 6] = wgpu::vertex_attr_array![
        1 => Float32x2,
        2 => Float32x2,
        3 => Float32x2,
        4 => Float32x4,
        5 => Float32x4,
        6 => Float32,
    ];

    pub fn buffer_layout() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<CellInstance>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: &Self::ATTRIBS,
        }
    }
}

const QUAD_CORNERS: &[[f32; 2]; 6] = &[
    [-1.0, -1.0],
    [1.0, -1.0],
    [-1.0, 1.0],
    [-1.0, 1.0],
    [1.0, -1.0],
    [1.0, 1.0],
];

fn quad_corner_buffer_layout() -> wgpu::VertexBufferLayout<'static> {
    wgpu::VertexBufferLayout {
        array_stride: std::mem::size_of::<[f32; 2]>() as wgpu::BufferAddress,
        step_mode: wgpu::VertexStepMode::Vertex,
        attributes: &[wgpu::VertexAttribute {
            format: wgpu::VertexFormat::Float32x2,
            offset: 0,
            shader_location: 0,
        }],
    }
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct CursorInstance {
    pub cursor_pos: [f32; 2],
    pub cursor_size: [f32; 2],
    pub color: [f32; 4],
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct GpuUniforms {
    pub projection: [[f32; 4]; 4],
    pub cell_size: [f32; 2],
    pub atlas_size: [f32; 2],
}

/// Safe wrapper for raw window pointer that implements Send/Sync
/// The raw pointer is only used on the render thread and is guaranteed
/// to outlive the GpuContext.
#[derive(Clone, Copy)]
#[expect(dead_code)]
struct RawWindowPtr(std::ptr::NonNull<std::ffi::c_void>);
unsafe impl Send for RawWindowPtr {}
unsafe impl Sync for RawWindowPtr {}

pub struct GpuContext {
    pub instance: wgpu::Instance,
    pub adapter: wgpu::Adapter,
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    pub surface: Option<wgpu::Surface<'static>>,
    pub surface_config: Option<wgpu::SurfaceConfiguration>,
    pub cell_pipeline: Option<wgpu::RenderPipeline>,
    pub quad_vertex_buffer: wgpu::Buffer,
    pub cell_bind_group: Option<wgpu::BindGroup>,
    pub cell_uniform_buffer: Option<wgpu::Buffer>,
    pub instance_buffer: Option<wgpu::Buffer>,
    pub atlas_texture: Option<wgpu::Texture>,
    pub atlas_view: Option<wgpu::TextureView>,
    pub atlas_sampler: Option<wgpu::Sampler>,
    // Raw ANativeWindow pointer for direct rendering fallback
    raw_window: Option<RawWindowPtr>,
}

impl GpuContext {
    async fn init_instance_adapter_device()
    -> Result<(wgpu::Instance, wgpu::Adapter, wgpu::Device, wgpu::Queue), GpuError> {
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::GL,
            flags: wgpu::InstanceFlags::DISCARD_HAL_LABELS,
            memory_budget_thresholds: wgpu::MemoryBudgetThresholds::default(),
            backend_options: wgpu::BackendOptions::default(),
            display: None,
        });

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                compatible_surface: None,
                force_fallback_adapter: false,
            })
            .await
            .map_err(|_| GpuError::NoAdapter)?;

        let (device, queue) = adapter
            .request_device(&wgpu::DeviceDescriptor {
                label: Some("Torvox Device"),
                required_features: wgpu::Features::empty(),
                required_limits: adapter.limits(),
                ..Default::default()
            })
            .await
            .map_err(|e| GpuError::DeviceRequest(e.to_string()))?;

        device.on_uncaptured_error(Arc::new(|error| {
            log_gpu_error(&error);
        }));

        Ok((instance, adapter, device, queue))
    }

    fn create_cell_pipeline(
        device: &wgpu::Device,
        format: wgpu::TextureFormat,
    ) -> wgpu::RenderPipeline {
        let wgsl_source = include_str!("../shaders/cell.wgsl");
        let cell_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Cell Shader"),
            source: wgpu::ShaderSource::Wgsl(std::borrow::Cow::Borrowed(wgsl_source)),
        });

        let cell_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("Cell Bind Group Layout"),
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
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
                            sample_type: wgpu::TextureSampleType::Float { filterable: true },
                            view_dimension: wgpu::TextureViewDimension::D2,
                            multisampled: false,
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

        let cell_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Cell Pipeline Layout"),
            bind_group_layouts: &[Some(&cell_bind_group_layout)],
            immediate_size: 0,
        });

        device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Cell Pipeline"),
            layout: Some(&cell_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &cell_shader,
                entry_point: Some("vs_main"),
                buffers: &[quad_corner_buffer_layout(), CellInstance::buffer_layout()],
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &cell_shader,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: Default::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: None,
                polygon_mode: wgpu::PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false,
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview_mask: None,
            cache: None,
        })
    }

    pub async fn new() -> Result<Self, GpuError> {
        let (instance, adapter, device, queue) = Self::init_instance_adapter_device().await?;
        let quad_vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Quad Vertex Buffer"),
            contents: bytemuck::cast_slice(QUAD_CORNERS),
            usage: wgpu::BufferUsages::VERTEX,
        });

        Ok(Self {
            instance,
            adapter,
            device,
            queue,
            surface: None,
            surface_config: None,
            cell_pipeline: None,
            quad_vertex_buffer,
            cell_bind_group: None,
            cell_uniform_buffer: None,
            instance_buffer: None,
            atlas_texture: None,
            atlas_view: None,
            atlas_sampler: None,
            raw_window: None,
        })
    }

    pub fn new_with_no_surface() -> Self {
        let (instance, adapter, device, queue) =
            futures::executor::block_on(Self::init_instance_adapter_device())
                .expect("no GPU adapter/device found");
        let quad_vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Quad Vertex Buffer"),
            contents: bytemuck::cast_slice(QUAD_CORNERS),
            usage: wgpu::BufferUsages::VERTEX,
        });

        Self {
            instance,
            adapter,
            device,
            queue,
            surface: None,
            surface_config: None,
            cell_pipeline: None,
            quad_vertex_buffer,
            cell_bind_group: None,
            cell_uniform_buffer: None,
            instance_buffer: None,
            atlas_texture: None,
            atlas_view: None,
            atlas_sampler: None,
            raw_window: None,
        }
    }

    pub fn set_surface_from_native_window(
        &mut self,
        window_ptr: *mut std::ffi::c_void,
        initial_width: u32,
        initial_height: u32,
    ) -> Result<(), GpuError> {
        use raw_window_handle::{AndroidDisplayHandle, AndroidNdkWindowHandle};

        let non_null = std::ptr::NonNull::new(window_ptr)
            .ok_or_else(|| GpuError::Surface("null window pointer".to_string()))?;

        let android_handle = AndroidNdkWindowHandle::new(non_null);
        let display_handle = AndroidDisplayHandle::new();

        let raw_win_handle = raw_window_handle::RawWindowHandle::AndroidNdk(android_handle);
        let raw_display_handle = raw_window_handle::RawDisplayHandle::Android(display_handle);

        let surface = unsafe {
            self.instance
                .create_surface_unsafe(wgpu::SurfaceTargetUnsafe::RawHandle {
                    raw_window_handle: raw_win_handle,
                    raw_display_handle: Some(raw_display_handle),
                })
        }
        .map_err(|e| GpuError::Surface(e.to_string()))?;

        let caps = surface.get_capabilities(&self.adapter);
        // Use Rgba8Unorm (non-sRGB). GL backend on Android doesn't expose sRGB formats.
        // With Rgba8Unorm, the shader outputs sRGB-byte/255 values which the display
        // correctly interprets as sRGB → correct physical brightness.
        let format = caps
            .formats
            .iter()
            .copied()
            .find(|f| {
                matches!(
                    f,
                    wgpu::TextureFormat::Rgba8Unorm | wgpu::TextureFormat::Bgra8Unorm
                )
            })
            .or_else(|| caps.formats.first().copied())
            .unwrap_or(wgpu::TextureFormat::Rgba8Unorm);

        log::info!(
            "Surface formats available: {:?} (chose: {:?})",
            caps.formats,
            format
        );
        log::info!("Present modes available: {:?}", caps.present_modes);

        let alpha_mode = wgpu::CompositeAlphaMode::Opaque;
        let present_mode = if caps.present_modes.contains(&wgpu::PresentMode::AutoNoVsync) {
            wgpu::PresentMode::AutoNoVsync
        } else if caps.present_modes.contains(&wgpu::PresentMode::Fifo) {
            wgpu::PresentMode::Fifo
        } else {
            caps.present_modes
                .first()
                .copied()
                .unwrap_or(wgpu::PresentMode::AutoNoVsync)
        };
        log::info!(
            "Present mode selected: {:?} (available: {:?})",
            present_mode,
            caps.present_modes
        );

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format,
            width: initial_width,
            height: initial_height,
            present_mode,
            alpha_mode,
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };

        surface.configure(&self.device, &config);
        log::info!(
            "Surface configured: {}x{}, alpha={:?}, present={:?}",
            initial_width,
            initial_height,
            alpha_mode,
            present_mode
        );

        self.cell_pipeline = Some(Self::create_cell_pipeline(&self.device, format));
        self.surface = Some(surface);
        self.surface_config = Some(config);
        self.raw_window = std::ptr::NonNull::new(window_ptr).map(RawWindowPtr);

        Ok(())
    }

    #[cfg(feature = "winit")]
    pub fn create_surface(
        &mut self,
        window: std::sync::Arc<winit::window::Window>,
    ) -> Result<(), GpuError> {
        let size = window.inner_size();
        let surface = self
            .instance
            .create_surface(window)
            .map_err(|e| GpuError::Surface(e.to_string()))?;

        let caps = surface.get_capabilities(&self.adapter);
        let format = caps
            .formats
            .first()
            .copied()
            .unwrap_or(wgpu::TextureFormat::Bgra8UnormSrgb);

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format,
            width: size.width.max(1),
            height: size.height.max(1),
            present_mode: wgpu::PresentMode::Fifo,
            alpha_mode: caps
                .alpha_modes
                .first()
                .copied()
                .unwrap_or(wgpu::CompositeAlphaMode::Auto),
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };

        surface.configure(&self.device, &config);

        self.cell_pipeline = Some(Self::create_cell_pipeline(&self.device, format));
        self.surface = Some(surface);
        self.surface_config = Some(config);

        Ok(())
    }

    /// 创建 GPU 图集纹理。
    ///
    /// 格式：`Rgba8UnormSrgb` — RGBA 字节布局与 font.rs CPU 位图输出匹配。
    /// "sRGB" 后缀表示 GPU 在采样时应用 sRGB 伽马校正，
    /// 这对字体渲染是正确的（光栅化器产生线性 alpha，GPU 应用伽马）。
    pub fn create_atlas_texture(&mut self, width: u32, height: u32) {
        let texture = self.device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Atlas Texture"),
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

        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        let sampler = self.device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Nearest,
            min_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });

        self.atlas_texture = Some(texture);
        self.atlas_view = Some(view);
        self.atlas_sampler = Some(sampler);
    }

    pub fn upload_atlas(&self, data: &[u8], width: u32, height: u32) {
        if let Some(texture) = &self.atlas_texture {
            self.queue.write_texture(
                wgpu::TexelCopyTextureInfo {
                    texture,
                    mip_level: 0,
                    origin: wgpu::Origin3d::ZERO,
                    aspect: wgpu::TextureAspect::All,
                },
                data,
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
    }

    pub fn update_bind_group(
        &mut self,
        atlas_width: f32,
        atlas_height: f32,
        cell_width: f32,
        cell_height: f32,
    ) {
        let config = match self.surface_config.as_ref() {
            Some(c) => c,
            None => return,
        };
        let pipeline = match self.cell_pipeline.as_ref() {
            Some(p) => p,
            None => return,
        };

        if self.cell_uniform_buffer.is_none() {
            self.cell_uniform_buffer = Some(self.device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("Cell Uniform Buffer"),
                size: std::mem::size_of::<GpuUniforms>() as u64,
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            }));
        }

        let proj = orthographic_projection(config.width as f32, config.height as f32);

        let uniforms = GpuUniforms {
            projection: proj,
            cell_size: [cell_width, cell_height],
            atlas_size: [atlas_width, atlas_height],
        };

        let uniform_buffer = match self.cell_uniform_buffer.as_ref() {
            Some(buf) => buf,
            None => return,
        };
        self.queue
            .write_buffer(uniform_buffer, 0, bytemuck::cast_slice(&[uniforms]));

        let atlas_view = match self.atlas_view.as_ref() {
            Some(v) => v,
            None => return,
        };
        let atlas_sampler = match self.atlas_sampler.as_ref() {
            Some(s) => s,
            None => return,
        };

        self.cell_bind_group = Some(self.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Cell Bind Group"),
            layout: &pipeline.get_bind_group_layout(0),
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: uniform_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(atlas_view),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::Sampler(atlas_sampler),
                },
            ],
        }));
    }

    pub fn resize_surface(&mut self, width: u32, height: u32) {
        if let Some(config) = &mut self.surface_config {
            config.width = width;
            config.height = height;
            if let Some(surface) = &self.surface {
                surface.configure(&self.device, config);
            }
        }
    }

    pub fn has_surface(&self) -> bool {
        self.surface.is_some()
    }

    pub fn warmup(&self) {
        let surface = match self.surface.as_ref() {
            Some(s) => s,
            None => return,
        };

        // Catch panics from get_current_texture (e.g. SwiftShader Vulkan config issues)
        let output = match std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            surface.get_current_texture()
        })) {
            Ok(
                wgpu::CurrentSurfaceTexture::Success(tex)
                | wgpu::CurrentSurfaceTexture::Suboptimal(tex),
            ) => tex,
            Ok(_) => return,
            Err(_) => {
                log::warn!("warmup: get_current_texture panicked (SwiftShader compat)");
                return;
            }
        };
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Warmup Encoder"),
            });
        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());
        {
            let _render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Warmup Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.0,
                            g: 0.0,
                            b: 0.0,
                            a: 1.0,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                    depth_slice: None,
                })],
                depth_stencil_attachment: None,
                ..Default::default()
            });
        }
        self.queue.submit(std::iter::once(encoder.finish()));
        output.present();
    }

    /// GPU readback test: render to a small off-screen texture and read to CPU
    pub fn gpu_readback_test(&self) {
        log::info!("GPU_READBACK_TEST_START");

        let pipeline = match self.cell_pipeline.as_ref() {
            Some(p) => p,
            None => {
                log::error!("GPU_READBACK_TEST: no pipeline");
                return;
            }
        };

        // Create a 4x4 off-screen texture
        let test_texture = self.device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Readback Test Texture"),
            size: wgpu::Extent3d {
                width: 4,
                height: 4,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8Unorm,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
            view_formats: &[],
        });

        let test_view = test_texture.create_view(&wgpu::TextureViewDescriptor::default());

        // Create staging buffer for readback
        let readback_buffer = self.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Readback Buffer"),
            size: 4 * 4 * 4, // 4x4 pixels, 4 bytes each
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
            mapped_at_creation: false,
        });

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Readback Encoder"),
            });

        // Render pass: clear to red, draw green quad
        {
            let mut rp = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Readback Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &test_view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.0130,
                            g: 0.0130,
                            b: 0.0273,
                            a: 1.0,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                    depth_slice: None,
                })],
                depth_stencil_attachment: None,
                ..Default::default()
            });

            rp.set_pipeline(pipeline);
            rp.set_viewport(0.0, 0.0, 4.0, 4.0, 0.0, 1.0);
            rp.set_scissor_rect(0, 0, 4, 4);

            if let Some(bind_group) = &self.cell_bind_group {
                rp.set_bind_group(0, bind_group, &[]);
            }

            rp.set_vertex_buffer(0, self.quad_vertex_buffer.slice(..));
            rp.draw(0..6, 0..1);
        }

        // Copy texture to staging buffer
        let texel_size = 4u32; // Rgba8Unorm
        let bytes_per_row = 4 * texel_size;
        encoder.copy_texture_to_buffer(
            wgpu::TexelCopyTextureInfo {
                texture: &test_texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            wgpu::TexelCopyBufferInfo {
                buffer: &readback_buffer,
                layout: wgpu::TexelCopyBufferLayout {
                    offset: 0,
                    bytes_per_row: Some(bytes_per_row),
                    rows_per_image: Some(4),
                },
            },
            wgpu::Extent3d {
                width: 4,
                height: 4,
                depth_or_array_layers: 1,
            },
        );

        self.queue.submit(std::iter::once(encoder.finish()));

        // Map buffer and read back (synchronous via channel)
        let (tx, rx) = std::sync::mpsc::channel();
        let buffer_slice = readback_buffer.slice(..);
        buffer_slice.map_async(wgpu::MapMode::Read, move |result| {
            let _ = tx.send(result.is_ok());
        });
        self.device.poll(wgpu::PollType::wait_indefinitely()).ok();
        let map_ok = rx
            .recv_timeout(std::time::Duration::from_secs(10))
            .unwrap_or(false);
        if !map_ok {
            log::error!("GPU_READBACK: map_async failed");
            return;
        }

        let data = buffer_slice.get_mapped_range();
        for y in 0..4 {
            let row_start = (y * bytes_per_row) as usize;
            let row = &data[row_start..row_start + 16];
            log::info!(
                "GPU_READBACK[{}]: {:02x} {:02x} {:02x} {:02x} | {:02x} {:02x} {:02x} {:02x} | {:02x} {:02x} {:02x} {:02x} | {:02x} {:02x} {:02x} {:02x}",
                y,
                row[0],
                row[1],
                row[2],
                row[3],
                row[4],
                row[5],
                row[6],
                row[7],
                row[8],
                row[9],
                row[10],
                row[11],
                row[12],
                row[13],
                row[14],
                row[15],
            );
        }
        drop(data);
        readback_buffer.unmap();
        log::info!("GPU_READBACK_TEST_END");
    }

    pub fn render_frame(&mut self, instances: &[CellInstance]) -> Result<(), GpuError> {
        let surface = self
            .surface
            .as_ref()
            .ok_or(GpuError::Surface("No surface configured".to_string()))?;
        let pipeline = self
            .cell_pipeline
            .as_ref()
            .ok_or(GpuError::Surface("No render pipeline".to_string()))?;
        let config = self
            .surface_config
            .as_ref()
            .ok_or(GpuError::Surface("No surface config".to_string()))?;

        log::trace!(
            "render_frame: {} instances, surface={}, pipeline={}, bind_group={}",
            instances.len(),
            self.surface.is_some(),
            self.cell_pipeline.is_some(),
            self.cell_bind_group.is_some(),
        );

        let output = match std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            surface.get_current_texture()
        })) {
            Ok(
                wgpu::CurrentSurfaceTexture::Success(tex)
                | wgpu::CurrentSurfaceTexture::Suboptimal(tex),
            ) => tex,
            Ok(wgpu::CurrentSurfaceTexture::Lost) => {
                if let Some(config) = &self.surface_config {
                    surface.configure(&self.device, config);
                }
                return Ok(());
            }
            Ok(_) => return Ok(()),
            Err(_) => {
                log::warn!("render_frame: get_current_texture panicked (SwiftShader)");
                return Ok(());
            }
        };

        let tex_size = output.texture.size();
        if tex_size.width != config.width || tex_size.height != config.height {
            log::warn!(
                "render_frame: size mismatch! config={}x{} texture={}x{}",
                config.width,
                config.height,
                tex_size.width,
                tex_size.height
            );
        }

        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Frame Encoder"),
            });

        // Upload instance data
        if !instances.is_empty() {
            let instance_data = bytemuck::cast_slice(instances);
            let needed_size = instance_data.len() as u64;
            let resize_buffer = self
                .instance_buffer
                .as_ref()
                .is_none_or(|buf| buf.size() < needed_size);
            if resize_buffer {
                self.instance_buffer = Some(self.device.create_buffer(&wgpu::BufferDescriptor {
                    label: Some("Instance Buffer"),
                    size: needed_size.max(64),
                    usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
                    mapped_at_creation: false,
                }));
            }
            if let Some(ref buf) = self.instance_buffer {
                self.queue.write_buffer(buf, 0, instance_data);
            }
        }

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Cell Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.0,
                            g: 0.0,
                            b: 0.0,
                            a: 1.0,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                    depth_slice: None,
                })],
                depth_stencil_attachment: None,
                ..Default::default()
            });

            let w = config.width as f32;
            let h = config.height as f32;
            render_pass.set_pipeline(pipeline);
            render_pass.set_viewport(0.0, 0.0, w, h, 0.0, 1.0);
            render_pass.set_scissor_rect(0, 0, config.width, config.height);

            if let Some(bind_group) = &self.cell_bind_group {
                render_pass.set_bind_group(0, bind_group, &[]);

                if !instances.is_empty() {
                    render_pass.set_vertex_buffer(0, self.quad_vertex_buffer.slice(..));
                    if let Some(ref instance_buffer) = self.instance_buffer {
                        render_pass.set_vertex_buffer(1, instance_buffer.slice(..));
                    }
                    render_pass.draw(0..6, 0..instances.len() as u32);
                }
            }
        }

        self.queue.submit(std::iter::once(encoder.finish()));
        output.present();
        log::debug!("render_frame: presented {} instances", instances.len());

        Ok(())
    }
}

pub fn orthographic_projection(width: f32, height: f32) -> [[f32; 4]; 4] {
    [
        [2.0 / width, 0.0, 0.0, -1.0],
        [0.0, -2.0 / height, 0.0, 1.0],
        [0.0, 0.0, 1.0, 0.0],
        [0.0, 0.0, 0.0, 1.0],
    ]
}

/// Convert an sRGB byte value (0-255) divided by 255.0 to linear color space.
#[allow(dead_code)]
pub fn build_cell_instances(
    grid: &dyn torvox_core::grid::GridSnapshot,
    font_pipeline: &mut crate::font::FontPipeline,
    _cell_width: f32,
    _cell_height: f32,
    atlas_width: f32,
    atlas_height: f32,
) -> Vec<CellInstance> {
    let rows = grid.rows();
    let cols = grid.cols();
    let mut instances = Vec::with_capacity((rows * cols) as usize);

    for row in 0..rows {
        if let Some(line) = grid.get(row) {
            for col in 0..cols {
                if let Some(cell) = line.get(col) {
                    let fg = cell.fg;
                    let bg = cell.bg;

                    if cell.char == ' ' {
                        instances.push(CellInstance {
                            cell_pos: [col as f32, row as f32],
                            atlas_offset: [0.0, 0.0],
                            atlas_size: [0.0, 0.0],
                            fg_color: [0.0, 0.0, 0.0, 0.0],
                            bg_color: [
                                bg.r as f32 / 255.0,
                                bg.g as f32 / 255.0,
                                bg.b as f32 / 255.0,
                                1.0,
                            ],
                            flags: 0.0,
                            _pad: [0.0; 3],
                        });
                    } else if let Some(info) = font_pipeline.glyph_info(cell.char) {
                        let uv_x = info.atlas_x as f32 / atlas_width;
                        let uv_y = info.atlas_y as f32 / atlas_height;
                        let uv_w = info.width as f32 / atlas_width;
                        let uv_h = info.height as f32 / atlas_height;

                        let flags = if cell.attrs.bold { 1.0 } else { 0.0 }
                            + if cell.attrs.italic { 2.0 } else { 0.0 }
                            + if cell.attrs.reverse { 4.0 } else { 0.0 }
                            + if cell.attrs.underline { 8.0 } else { 0.0 }
                            + if cell.attrs.dim { 16.0 } else { 0.0 }
                            + if cell.attrs.hidden { 32.0 } else { 0.0 }
                            + if cell.attrs.strikethrough { 64.0 } else { 0.0 }
                            + if cell.attrs.overline { 128.0 } else { 0.0 };

                        instances.push(CellInstance {
                            cell_pos: [col as f32, row as f32],
                            atlas_offset: [uv_x, uv_y],
                            atlas_size: [uv_w, uv_h],
                            fg_color: [
                                fg.r as f32 / 255.0,
                                fg.g as f32 / 255.0,
                                fg.b as f32 / 255.0,
                                1.0,
                            ],
                            bg_color: [
                                bg.r as f32 / 255.0,
                                bg.g as f32 / 255.0,
                                bg.b as f32 / 255.0,
                                1.0,
                            ],
                            flags,
                            _pad: [0.0; 3],
                        });
                    } else {
                        instances.push(CellInstance {
                            cell_pos: [col as f32, row as f32],
                            atlas_offset: [0.0, 0.0],
                            atlas_size: [0.0, 0.0],
                            fg_color: [0.8, 0.8, 0.8, 1.0],
                            bg_color: [
                                bg.r as f32 / 255.0,
                                bg.g as f32 / 255.0,
                                bg.b as f32 / 255.0,
                                1.0,
                            ],
                            flags: 0.0,
                            _pad: [0.0; 3],
                        });
                    }
                }
            }
        }
    }
    instances
}

/// 扁平网格数据，用于构建单元格实例而无需 GridSnapshot。
/// 由 Ghostty VT 集成使用，终端状态在外部管理。
pub struct FlatGrid {
    pub rows: u32,
    pub cols: u32,
    pub chars: Vec<char>,
    pub fg: Vec<[f32; 4]>,
    pub bg: Vec<[f32; 4]>,
}

impl FlatGrid {
    pub fn new(rows: u32, cols: u32) -> Self {
        let len = (rows * cols) as usize;
        Self {
            rows,
            cols,
            chars: vec![' '; len],
            fg: vec![[1.0, 1.0, 1.0, 1.0]; len],
            bg: vec![[0.0, 0.0, 0.0, 1.0]; len],
        }
    }

    pub fn set_cell(&mut self, row: u32, col: u32, ch: char, fg: [f32; 4], bg: [f32; 4]) {
        let idx = (row * self.cols + col) as usize;
        if idx < self.chars.len() {
            self.chars[idx] = ch;
            self.fg[idx] = fg;
            self.bg[idx] = bg;
        }
    }

    pub fn cell(&self, row: u32, col: u32) -> Option<(char, [f32; 4], [f32; 4])> {
        if row >= self.rows || col >= self.cols {
            return None;
        }
        let idx = (row * self.cols + col) as usize;
        if idx < self.chars.len() {
            Some((self.chars[idx], self.fg[idx], self.bg[idx]))
        } else {
            None
        }
    }
}

#[allow(dead_code)]
pub fn build_cell_instances_from_flat(
    flat: &FlatGrid,
    font_pipeline: &mut crate::font::FontPipeline,
    atlas_width: f32,
    atlas_height: f32,
) -> Vec<CellInstance> {
    let mut instances = Vec::with_capacity((flat.rows * flat.cols) as usize);

    for row in 0..flat.rows {
        for col in 0..flat.cols {
            if let Some((ch, fg, bg)) = flat.cell(row, col) {
                if ch == ' ' {
                    instances.push(CellInstance {
                        cell_pos: [col as f32, row as f32],
                        atlas_offset: [0.0, 0.0],
                        atlas_size: [0.0, 0.0],
                        fg_color: [0.0, 0.0, 0.0, 0.0],
                        bg_color: bg,
                        flags: 0.0,
                        _pad: [0.0; 3],
                    });
                } else if let Some(info) = font_pipeline.glyph_info(ch) {
                    let uv_x = info.atlas_x as f32 / atlas_width;
                    let uv_y = info.atlas_y as f32 / atlas_height;
                    let uv_w = info.width as f32 / atlas_width;
                    let uv_h = info.height as f32 / atlas_height;

                    instances.push(CellInstance {
                        cell_pos: [col as f32, row as f32],
                        atlas_offset: [uv_x, uv_y],
                        atlas_size: [uv_w, uv_h],
                        fg_color: fg,
                        bg_color: bg,
                        flags: 0.0,
                        _pad: [0.0; 3],
                    });
                }
            }
        }
    }

    instances
}

#[allow(clippy::collapsible_if)]
pub fn build_cell_instances_from_snapshot(
    snapshot: &torvox_terminal::ghostty_terminal::GridSnapshot,
    font_pipeline: &mut crate::font::FontPipeline,
    atlas_width: f32,
    atlas_height: f32,
    dirty_rows: Option<&[bool]>,
) -> Vec<CellInstance> {
    let rows = snapshot.rows;
    let cols = snapshot.cols;
    let mut instances = Vec::with_capacity((rows * cols) as usize);

    let mut glyph_found = 0u64;
    let mut glyph_not_found = 0u64;
    for row in 0..rows {
        if let Some(dirty) = dirty_rows {
            if row < dirty.len() as u32 && !dirty[row as usize] {
                for col in 0..cols {
                    instances.push(CellInstance {
                        cell_pos: [col as f32, row as f32],
                        atlas_offset: [0.0; 2],
                        atlas_size: [0.0; 2],
                        fg_color: [0.0; 4],
                        bg_color: [0.0; 4],
                        flags: 0.0,
                        _pad: [0.0; 3],
                    });
                }
                continue;
            }
        }
        for col in 0..cols {
            let idx = (row * cols + col) as usize;
            let cell = &snapshot.cells[idx];

            if cell.codepoint == 0 || cell.codepoint == 0x20 {
                instances.push(CellInstance {
                    cell_pos: [col as f32, row as f32],
                    atlas_offset: [0.0, 0.0],
                    atlas_size: [0.0, 0.0],
                    fg_color: [0.0, 0.0, 0.0, 0.0],
                    bg_color: cell.bg,
                    flags: 0.0,
                    _pad: [0.0; 3],
                });
                continue;
            }

            let ch = char::from_u32(cell.codepoint).unwrap_or('\u{FFFD}');
            let flags = if cell.bold { 1.0 } else { 0.0 }
                + if cell.italic { 2.0 } else { 0.0 }
                + if cell.reverse { 4.0 } else { 0.0 }
                + if cell.underline { 8.0 } else { 0.0 }
                + if cell.uri.is_some() { 16.0 } else { 0.0 };

            let (fg, bg) = if cell.reverse {
                (cell.bg, cell.fg)
            } else {
                (cell.fg, cell.bg)
            };

            if let Some(info) = font_pipeline.glyph_info(ch) {
                glyph_found += 1;
                let uv_x = info.atlas_x as f32 / atlas_width;
                let uv_y = info.atlas_y as f32 / atlas_height;
                let uv_w = info.width as f32 / atlas_width;
                let uv_h = info.height as f32 / atlas_height;

                instances.push(CellInstance {
                    cell_pos: [col as f32, row as f32],
                    atlas_offset: [uv_x, uv_y],
                    atlas_size: [uv_w, uv_h],
                    fg_color: fg,
                    bg_color: bg,
                    flags,
                    _pad: [0.0; 3],
                });
            } else {
                glyph_not_found += 1;
                // Fallback: render full-cell fg block (visible but blurry)
                instances.push(CellInstance {
                    cell_pos: [col as f32, row as f32],
                    atlas_offset: [0.0; 2],
                    atlas_size: [1.0 / atlas_width, 1.0 / atlas_height],
                    fg_color: fg,
                    bg_color: bg,
                    flags,
                    _pad: [0.0; 3],
                });
            }
        }
    }
    if glyph_found + glyph_not_found > 0 {
        log::info!(
            "build_cell_instances: glyph_found={} glyph_not_found={} total={}",
            glyph_found,
            glyph_not_found,
            glyph_found + glyph_not_found
        );
    }
    instances
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cell_instance_size() {
        assert_eq!(std::mem::size_of::<CellInstance>(), 72);
    }

    #[test]
    fn orthographic_projection_identity() {
        let proj = orthographic_projection(800.0, 600.0);
        assert!((proj[0][0] - 2.0 / 800.0).abs() < f32::EPSILON);
        assert!((proj[1][1] - (-2.0 / 600.0)).abs() < f32::EPSILON);
    }

    #[test]
    fn cell_instance_buffer_layout() {
        let layout = CellInstance::buffer_layout();
        assert_eq!(layout.step_mode, wgpu::VertexStepMode::Instance);
        assert!(layout.array_stride > 0);
    }

    #[test]
    fn flat_grid_new() {
        let grid = FlatGrid::new(10, 20);
        assert_eq!(grid.rows, 10);
        assert_eq!(grid.cols, 20);
        assert_eq!(grid.chars.len(), 200);
        assert!(grid.chars.iter().all(|&c| c == ' '));
    }

    #[test]
    fn flat_grid_set_and_get_cell() {
        let mut grid = FlatGrid::new(5, 5);
        let fg = [1.0, 0.0, 0.0, 1.0];
        let bg = [0.0, 0.0, 0.0, 1.0];
        grid.set_cell(2, 3, 'A', fg, bg);

        let (ch, fg_out, bg_out) = grid.cell(2, 3).unwrap();
        assert_eq!(ch, 'A');
        assert_eq!(fg_out, fg);
        assert_eq!(bg_out, bg);
    }

    #[test]
    fn flat_grid_out_of_bounds() {
        let grid = FlatGrid::new(3, 3);
        assert!(grid.cell(3, 0).is_none());
        assert!(grid.cell(0, 3).is_none());
    }

    #[test]
    fn build_cell_instances_from_flat_basic() {
        let mut grid = FlatGrid::new(1, 4);
        grid.set_cell(0, 0, 'A', [1.0, 1.0, 1.0, 1.0], [0.0, 0.0, 0.0, 1.0]);
        grid.set_cell(0, 1, ' ', [0.0, 0.0, 0.0, 0.0], [0.5, 0.5, 0.5, 1.0]);
        grid.set_cell(0, 2, 'B', [0.0, 1.0, 0.0, 1.0], [0.2, 0.2, 0.2, 1.0]);
        grid.set_cell(0, 3, 'C', [1.0, 0.0, 1.0, 1.0], [0.3, 0.3, 0.3, 1.0]);

        let mut font_pipeline = crate::font::FontPipeline::new(2048, 2048, 14.0);
        font_pipeline.rasterize_ascii();

        let instances = build_cell_instances_from_flat(&grid, &mut font_pipeline, 2048.0, 2048.0);
        // 所有 4 个单元格都有实例（空格只有背景，其他有字形）
        assert_eq!(instances.len(), 4);

        let cell0 = &instances[0];
        assert_eq!(cell0.cell_pos, [0.0, 0.0]);
        assert_eq!(cell0.bg_color, [0.0, 0.0, 0.0, 1.0]);

        let cell1 = &instances[1];
        assert_eq!(cell1.cell_pos, [1.0, 0.0]);
        assert_eq!(cell1.bg_color, [0.5, 0.5, 0.5, 1.0]);
    }

    #[test]
    fn cell_instance_pod_roundtrip() {
        let c = CellInstance {
            cell_pos: [1.0, 2.0],
            atlas_offset: [0.5, 0.5],
            atlas_size: [0.1, 0.1],
            fg_color: [1.0, 1.0, 1.0, 1.0],
            bg_color: [0.0, 0.0, 0.0, 1.0],
            flags: 5.0,
            _pad: [0.0; 3],
        };
        let bytes = bytemuck::bytes_of(&c);
        let back: &CellInstance = bytemuck::from_bytes(bytes);
        assert_eq!(back.cell_pos, [1.0, 2.0]);
        assert_eq!(back.flags, 5.0);
    }

    #[test]
    fn cell_instance_zeroable() {
        let c: CellInstance = bytemuck::Zeroable::zeroed();
        assert_eq!(c.cell_pos, [0.0, 0.0]);
        assert_eq!(c.fg_color, [0.0, 0.0, 0.0, 0.0]);
        assert_eq!(c.flags, 0.0);
    }

    #[test]
    fn cursor_instance_size() {
        // 仅验证它是 pod 且可序列化
        let c = CursorInstance {
            cursor_pos: [0.0, 0.0],
            cursor_size: [10.0, 20.0],
            color: [1.0, 1.0, 1.0, 1.0],
        };
        let bytes = bytemuck::bytes_of(&c);
        let back: &CursorInstance = bytemuck::from_bytes(bytes);
        assert_eq!(back.cursor_size, [10.0, 20.0]);
    }

    #[test]
    fn gpu_uniforms_size() {
        // 4x4 mat (16 floats) + 2 floats cell_size + 2 floats atlas_size = 80 bytes
        assert_eq!(std::mem::size_of::<GpuUniforms>(), 80);
    }

    #[test]
    fn orthographic_projection_basic() {
        let proj = orthographic_projection(100.0, 100.0);
        // [0][0] = 2/width
        assert!((proj[0][0] - 0.02).abs() < f32::EPSILON);
        // [1][1] = -2/height
        assert!((proj[1][1] - (-0.02)).abs() < f32::EPSILON);
        // [3][3] = 1
        assert!((proj[3][3] - 1.0).abs() < f32::EPSILON);
    }

    #[test]
    fn flat_grid_zero_size() {
        let grid = FlatGrid::new(0, 0);
        assert_eq!(grid.chars.len(), 0);
        assert_eq!(grid.fg.len(), 0);
    }

    #[test]
    fn flat_grid_set_out_of_bounds_no_panic() {
        let mut grid = FlatGrid::new(2, 2);
        grid.set_cell(100, 100, 'X', [1.0; 4], [0.0; 4]); // out of bounds
        // 不应 panic，值未存储
        assert_eq!(grid.chars.len(), 4);
    }

    #[test]
    fn flat_grid_default_chars_are_spaces() {
        let grid = FlatGrid::new(3, 3);
        assert!(grid.chars.iter().all(|&c| c == ' '));
    }

    #[test]
    fn flat_grid_default_fg_is_white() {
        let grid = FlatGrid::new(2, 2);
        for f in &grid.fg {
            assert_eq!(*f, [1.0, 1.0, 1.0, 1.0]);
        }
    }

    #[test]
    fn flat_grid_default_bg_is_black() {
        let grid = FlatGrid::new(2, 2);
        for b in &grid.bg {
            assert_eq!(*b, [0.0, 0.0, 0.0, 1.0]);
        }
    }

    #[test]
    fn flat_grid_cell_after_set() {
        let mut grid = FlatGrid::new(2, 2);
        let fg = [0.5, 0.6, 0.7, 1.0];
        let bg = [0.1, 0.2, 0.3, 1.0];
        grid.set_cell(0, 0, 'H', fg, bg);
        let (ch, f, b) = grid.cell(0, 0).unwrap();
        assert_eq!(ch, 'H');
        assert_eq!(f, fg);
        assert_eq!(b, bg);
    }

    #[test]
    fn build_cell_instances_from_flat_empty() {
        let grid = FlatGrid::new(0, 0);
        let mut font = crate::font::FontPipeline::new(2048, 2048, 14.0);
        font.rasterize_ascii();
        let instances = build_cell_instances_from_flat(&grid, &mut font, 2048.0, 2048.0);
        assert!(instances.is_empty());
    }

    #[test]
    fn build_cell_instances_from_flat_space_only() {
        let grid = FlatGrid::new(1, 5);
        let mut font = crate::font::FontPipeline::new(2048, 2048, 14.0);
        font.rasterize_ascii();
        let instances = build_cell_instances_from_flat(&grid, &mut font, 2048.0, 2048.0);
        assert_eq!(instances.len(), 5);
        // 全是空格，atlas_size 应为 0
        for inst in &instances {
            assert_eq!(inst.atlas_size, [0.0, 0.0]);
        }
    }

    #[test]
    fn build_cell_instances_from_flat_unicode_cjk() {
        let mut grid = FlatGrid::new(1, 3);
        grid.set_cell(0, 0, '中', [1.0; 4], [0.0; 4]);
        grid.set_cell(0, 1, '文', [1.0; 4], [0.0; 4]);
        let mut font = crate::font::FontPipeline::new(2048, 2048, 14.0);
        font.rasterize_ascii();
        let instances = build_cell_instances_from_flat(&grid, &mut font, 2048.0, 2048.0);
        // 3 个单元格（CJK 可能未被光栅化，可能或可能不产生实例）
        // 仅验证无 panic
        assert_eq!(instances.len(), 3);
    }

    #[test]
    fn cell_instance_attribs_locations() {
        let attribs = CellInstance::ATTRIBS;
        assert_eq!(attribs.len(), 6);
        assert_eq!(attribs[0].shader_location, 1);
        assert_eq!(attribs[1].shader_location, 2);
        assert_eq!(attribs[5].shader_location, 6);
    }

    #[test]
    fn orthographic_projection_zero_size() {
        let proj = orthographic_projection(0.0, 0.0);
        // 2.0 / 0.0 = inf
        assert!(proj[0][0].is_infinite());
    }
}
