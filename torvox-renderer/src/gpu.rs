// @wgpu render pipeline, IMPL_REND_001, impl, [REQ_REND_001]
// @need-ids: REQ_REND_001, REQ_REND_002
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
    pub quad_origin: [f32; 2],
    pub atlas_offset: [f32; 2],
    pub atlas_size: [f32; 2],
    pub fg_color: [f32; 4],
    pub bg_color: [f32; 4],
    pub quad_size: [f32; 2],
    pub flags: f32,
    /// Pixel offset from cell top-left to glyph bitmap top-left within the cell.
    /// Used for bearing-aware fragment shader UV mapping.
    pub bearing: [f32; 2],
    /// The glyph's advance width in pixels (unscaled). Used by the fragment shader
    /// to stretch narrow glyphs to fill the cell, matching Termux's canvas.scale() behavior.
    pub glyph_advance_width: f32,
}

impl CellInstance {
    pub const ATTRIBS: [wgpu::VertexAttribute; 9] = wgpu::vertex_attr_array![
        1 => Float32x2,
        2 => Float32x2,
        3 => Float32x2,
        4 => Float32x4,
        5 => Float32x4,
        6 => Float32x2,
        7 => Float32,
        8 => Float32x2,
        9 => Float32,
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
    pub atlas_size: [f32; 2],
    pub _padding: [f32; 2],
}

pub const CATPPUCCIN_MOCHA_BG: wgpu::Color = wgpu::Color {
    r: 30.0 / 255.0,
    g: 30.0 / 255.0,
    b: 46.0 / 255.0,
    a: 1.0,
};

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
    pub pipeline_format: wgpu::TextureFormat,
    projection_width: u32,
    projection_height: u32,
    readback_texture: Option<wgpu::Texture>,
    readback_buffer: Option<wgpu::Buffer>,
    bg_color: wgpu::Color,
}

impl GpuContext {
    async fn init_instance_adapter_device()
    -> Result<(wgpu::Instance, wgpu::Adapter, wgpu::Device, wgpu::Queue), GpuError> {
        // Vulkan priority, GLES fallback on Android.
        // wgpu selects Vulkan first; if emulator lacks Vulkan driver,
        // GLES is used automatically. The user mandated pure GPU Vulkan
        // rendering; GLES remains as emulator-compatibility fallback.
        #[cfg(target_os = "android")]
        let backends = wgpu::Backends::VULKAN | wgpu::Backends::GL;
        #[cfg(not(target_os = "android"))]
        let backends = wgpu::Backends::PRIMARY;
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends,
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

        let adapter_info = adapter.get_info();
        log::info!(
            "GPU adapter: {} (backend={:?}, type={:?})",
            adapter_info.name,
            adapter_info.backend,
            adapter_info.device_type,
        );

        let device_descriptor = wgpu::DeviceDescriptor {
            label: Some("Torvox Device"),
            required_features: wgpu::Features::empty(),
            required_limits: adapter.limits(),
            ..Default::default()
        };

        let (device, queue) = adapter
            .request_device(&device_descriptor)
            .await
            .map_err(|e| GpuError::DeviceRequest(e.to_string()))?;

        device.on_uncaptured_error(Arc::new(|error| {
            log_gpu_error(&error);
        }));

        log::info!("GPU device created, queue ok");
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
            pipeline_format: wgpu::TextureFormat::Rgba8Unorm,
            projection_width: 0,
            projection_height: 0,
            readback_texture: None,
            readback_buffer: None,
            bg_color: CATPPUCCIN_MOCHA_BG,
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
            pipeline_format: wgpu::TextureFormat::Rgba8Unorm,
            projection_width: 0,
            projection_height: 0,
            readback_texture: None,
            readback_buffer: None,
            bg_color: CATPPUCCIN_MOCHA_BG,
        }
    }

    pub fn set_bg_color(&mut self, bg: [u8; 3]) {
        self.bg_color = wgpu::Color {
            r: bg[0] as f64 / 255.0,
            g: bg[1] as f64 / 255.0,
            b: bg[2] as f64 / 255.0,
            a: 1.0,
        };
    }

    pub fn set_surface_from_native_window(
        &mut self,
        window_ptr: *mut std::ffi::c_void,
        initial_width: u32,
        initial_height: u32,
        configure_surface: bool,
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

        // Request a new adapter compatible with this surface.
        // The initial adapter in new_with_no_surface was created without a
        // compatible_surface, which may select an adapter that does not
        // support the Android NDK native window surface (SwiftShader / real GPU).
        let adapter = futures::executor::block_on(self.instance.request_adapter(
            &wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            },
        ))
        .map_err(|_| GpuError::NoAdapter)?;

        let adapter_info = adapter.get_info();
        log::info!(
            "GPU adapter (surface-compatible): {} (backend={:?}, type={:?})",
            adapter_info.name,
            adapter_info.backend,
            adapter_info.device_type,
        );

        let (device, queue) =
            futures::executor::block_on(adapter.request_device(&wgpu::DeviceDescriptor {
                label: Some("Torvox Device"),
                required_features: wgpu::Features::empty(),
                required_limits: adapter.limits(),
                ..Default::default()
            }))
            .map_err(|e| GpuError::DeviceRequest(e.to_string()))?;

        device.on_uncaptured_error(Arc::new(|error| {
            log_gpu_error(&error);
        }));

        self.adapter = adapter;
        self.device = device;
        self.queue = queue;

        // Re-create device-dependent resources (old ones were tied to old device)
        self.quad_vertex_buffer =
            self.device
                .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("Quad Vertex Buffer"),
                    contents: bytemuck::cast_slice(QUAD_CORNERS),
                    usage: wgpu::BufferUsages::VERTEX,
                });

        let caps = surface.get_capabilities(&self.adapter);
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

        // Prefer Opaque alpha mode.  With `setZOrderOnTop(true)` (SurfaceView
        // as overlay layer), Opaque alpha mode avoids SurfaceFlinger alpha
        // compositing — the pixel content is directly visible.  On real
        // Adreno/Mali GPUs, Opaque is universally supported by Vulkan WSI.
        // PreMultiplied is fine as a fallback (our shader outputs alpha=1.0).
        // Inherit is last resort — it inherits alpha=0 from the overlay plane
        // on SwiftShader/emulator, making GPU swapchain output invisible.
        let has_opaque = caps.alpha_modes.contains(&wgpu::CompositeAlphaMode::Opaque);
        let has_premultiplied = caps
            .alpha_modes
            .contains(&wgpu::CompositeAlphaMode::PreMultiplied);
        let alpha_mode = if has_opaque {
            wgpu::CompositeAlphaMode::Opaque
        } else if has_premultiplied {
            wgpu::CompositeAlphaMode::PreMultiplied
        } else if caps
            .alpha_modes
            .contains(&wgpu::CompositeAlphaMode::Inherit)
        {
            wgpu::CompositeAlphaMode::Inherit
        } else {
            caps.alpha_modes
                .first()
                .copied()
                .unwrap_or(wgpu::CompositeAlphaMode::Opaque)
        };
        log::info!(
            "Alpha mode selected: {:?} (available: {:?})",
            alpha_mode,
            caps.alpha_modes,
        );
        // Prefer low-latency present modes across all backends
        let present_mode = if caps.present_modes.contains(&wgpu::PresentMode::Immediate) {
            wgpu::PresentMode::Immediate
        } else if caps.present_modes.contains(&wgpu::PresentMode::AutoNoVsync) {
            wgpu::PresentMode::AutoNoVsync
        } else if caps.present_modes.contains(&wgpu::PresentMode::Mailbox) {
            wgpu::PresentMode::Mailbox
        } else {
            wgpu::PresentMode::Fifo
        };
        log::info!(
            "Present mode selected: {:?} (available: {:?})",
            present_mode,
            caps.present_modes
        );

        self.cell_pipeline = Some(Self::create_cell_pipeline(&self.device, format));
        self.pipeline_format = format;
        self.surface = Some(surface);

        if configure_surface {
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

            if let Some(ref configured_surface) = self.surface {
                configured_surface.configure(&self.device, &config);
            }
            self.surface_config = Some(config);
            log::info!(
                "Surface configured: {}x{}, alpha={:?}, present={:?} ({})",
                initial_width,
                initial_height,
                alpha_mode,
                present_mode,
                if cfg!(target_os = "android") {
                    "android"
                } else {
                    "desktop"
                },
            );
        } else {
            log::info!(
                "Surface created (pipeline only, no config): {}x{}, format={:?} (android offscreen)",
                initial_width,
                initial_height,
                format,
            );
        }

        Ok(())
    }

    /// Create GPU atlas texture.
    ///
    /// Format: `R8Unorm` — single-channel byte layout matches font.rs CPU bitmap output.
    /// The "sRGB" suffix means the GPU applies sRGB gamma correction during sampling,
    /// which is correct for font rendering (rasterizer produces linear alpha, GPU applies gamma).
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
            format: wgpu::TextureFormat::R8Unorm,
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
                    bytes_per_row: Some(width),
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

    pub fn update_bind_group(&mut self, atlas_width: f32, atlas_height: f32) {
        log::debug!("DIAG_BIND_GROUP: enter update_bind_group");
        let pipeline = match self.cell_pipeline.as_ref() {
            Some(p) => p,
            None => {
                log::warn!("DIAG_BIND_GROUP: cell_pipeline is None");
                return;
            }
        };

        if self.cell_uniform_buffer.is_none() {
            self.cell_uniform_buffer = Some(self.device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("Cell Uniform Buffer"),
                size: std::mem::size_of::<GpuUniforms>() as u64,
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            }));
        }

        let (proj_w, proj_h) = match self.surface_config.as_ref() {
            Some(c) => (c.width as f32, c.height as f32),
            None => (self.projection_width as f32, self.projection_height as f32),
        };
        let proj = orthographic_projection(proj_w, proj_h);

        let uniforms = GpuUniforms {
            projection: proj,
            atlas_size: [atlas_width, atlas_height],
            _padding: [0.0; 2],
        };

        let uniform_buffer = match self.cell_uniform_buffer.as_ref() {
            Some(buf) => buf,
            None => return,
        };
        self.queue
            .write_buffer(uniform_buffer, 0, bytemuck::cast_slice(&[uniforms]));

        let atlas_view = match self.atlas_view.as_ref() {
            Some(v) => v,
            None => {
                log::warn!("DIAG_BIND_GROUP: atlas_view is None");
                return;
            }
        };
        let atlas_sampler = match self.atlas_sampler.as_ref() {
            Some(s) => s,
            None => {
                log::warn!("DIAG_BIND_GROUP: atlas_sampler is None");
                return;
            }
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
        log::info!("DIAG_BIND_GROUP: bind_group created successfully");
    }

    /// Initialize the cell pipeline and bind group without a wgpu surface.
    /// Used on Android where create_surface_unsafe conflicts with
    /// ANativeWindow_lock (the Vulkan surface locks the buffer queue).
    /// Pipeline is created with Rgba8Unorm for offscreen rendering.
    pub fn init_pipeline_and_bind_group(
        &mut self,
        atlas_width: u32,
        atlas_height: u32,
        surface_width: u32,
        surface_height: u32,
    ) {
        let format = self
            .surface_config
            .as_ref()
            .map(|c| c.format)
            .unwrap_or(wgpu::TextureFormat::Rgba8Unorm);
        self.pipeline_format = format;
        self.cell_pipeline = Some(Self::create_cell_pipeline(&self.device, format));

        self.projection_width = surface_width;
        self.projection_height = surface_height;
        self.create_atlas_texture(atlas_width, atlas_height);

        let proj = orthographic_projection(surface_width as f32, surface_height as f32);
        let uniforms = GpuUniforms {
            projection: proj,
            atlas_size: [atlas_width as f32, atlas_height as f32],
            _padding: [0.0; 2],
        };

        if self.cell_uniform_buffer.is_none() {
            self.cell_uniform_buffer = Some(self.device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("Cell Uniform Buffer"),
                size: std::mem::size_of::<GpuUniforms>() as u64,
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            }));
        }

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
        let pipeline = match self.cell_pipeline.as_ref() {
            Some(p) => p,
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

        log::info!(
            "init_pipeline_and_bind_group: pipeline={} atlas={}x{} surf={}x{}",
            self.cell_pipeline.is_some(),
            atlas_width,
            atlas_height,
            surface_width,
            surface_height,
        );
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

    pub fn release_gpu_surface(&mut self) {
        self.surface = None;
        self.surface_config = None;
    }

    pub fn has_surface(&self) -> bool {
        self.surface.is_some()
    }

    pub fn has_pipeline(&self) -> bool {
        self.cell_pipeline.is_some()
    }

    /// Create a wgpu Surface from an Android ANativeWindow and configure
    /// it with the existing device+queue.  Does NOT request a new adapter
    /// or create a new device — call after `init_pipeline_and_bind_group`.
    pub fn configure_android_surface(
        &mut self,
        window_ptr: *mut std::ffi::c_void,
        width: u32,
        height: u32,
    ) -> Result<(), GpuError> {
        // Drop old surface FIRST to release ANativeWindow before creating a new one.
        // This allows multiple GpuContexts to share the same ANativeWindow sequentially
        // (e.g., when switching between sessions each with their own bridge).
        self.surface = None;
        self.surface_config = None;

        let non_null = std::ptr::NonNull::new(window_ptr)
            .ok_or_else(|| GpuError::Surface("null window pointer".to_string()))?;
        let android_handle = raw_window_handle::AndroidNdkWindowHandle::new(non_null);
        let display_handle = raw_window_handle::AndroidDisplayHandle::new();

        let surface = unsafe {
            self.instance
                .create_surface_unsafe(wgpu::SurfaceTargetUnsafe::RawHandle {
                    raw_window_handle: raw_window_handle::RawWindowHandle::AndroidNdk(
                        android_handle,
                    ),
                    raw_display_handle: Some(raw_window_handle::RawDisplayHandle::Android(
                        display_handle,
                    )),
                })
        }
        .map_err(|e| GpuError::Surface(e.to_string()))?;

        let caps = surface.get_capabilities(&self.adapter);
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

        let alpha_mode = if caps.alpha_modes.contains(&wgpu::CompositeAlphaMode::Opaque) {
            wgpu::CompositeAlphaMode::Opaque
        } else if caps
            .alpha_modes
            .contains(&wgpu::CompositeAlphaMode::PreMultiplied)
        {
            wgpu::CompositeAlphaMode::PreMultiplied
        } else if caps.alpha_modes.contains(&wgpu::CompositeAlphaMode::Auto) {
            wgpu::CompositeAlphaMode::Auto
        } else {
            // SwiftShader on some configs only exposes Inherit
            wgpu::CompositeAlphaMode::Inherit
        };

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format,
            width: width.max(1),
            height: height.max(1),
            present_mode: if caps.present_modes.contains(&wgpu::PresentMode::Immediate) {
                wgpu::PresentMode::Immediate
            } else if caps.present_modes.contains(&wgpu::PresentMode::AutoNoVsync) {
                wgpu::PresentMode::AutoNoVsync
            } else if caps.present_modes.contains(&wgpu::PresentMode::Mailbox) {
                wgpu::PresentMode::Mailbox
            } else {
                wgpu::PresentMode::Fifo
            },
            alpha_mode,
            view_formats: vec![],
            desired_maximum_frame_latency: 1,
        };
        surface.configure(&self.device, &config);
        self.surface = Some(surface);

        log::info!(
            "configure_android_surface: {}x{} format={:?} alpha={:?} present={:?}",
            config.width,
            config.height,
            format,
            alpha_mode,
            config.present_mode,
        );

        self.surface_config = Some(config);
        Ok(())
    }

    /// Reconfigure the existing swapchain with new dimensions and update
    /// the projection matrix in the uniform buffer.  Called on surface
    /// resize (IME keyboard show/hide, window config change).
    #[cfg(target_os = "android")]
    pub fn reconfigure_swapchain(&mut self, width: u32, height: u32) {
        let (surface, config) = match (self.surface.as_ref(), self.surface_config.as_mut()) {
            (Some(s), Some(c)) => (s, c),
            _ => return,
        };
        if config.width == width && config.height == height {
            return;
        }
        config.width = width.max(1);
        config.height = height.max(1);
        surface.configure(&self.device, config);

        let aw = self.atlas_texture.as_ref().map_or(0, |t| t.width());
        let ah = self.atlas_texture.as_ref().map_or(0, |t| t.height());
        let proj = orthographic_projection(width as f32, height as f32);
        let uniforms = GpuUniforms {
            projection: proj,
            atlas_size: [aw as f32, ah as f32],
            _padding: [0.0; 2],
        };
        if let Some(buf) = &self.cell_uniform_buffer {
            self.queue
                .write_buffer(buf, 0, bytemuck::cast_slice(&[uniforms]));
        }
        log::info!("RECONFIGURE_SWAPCHAIN: {}x{}", width, height);
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

    /// Acquire a texture from the surface, retrying once on panic.
    /// Returns `None` if the texture could not be acquired (surface Lost, etc.).
    fn acquire_texture(
        &self,
        surface: &wgpu::Surface<'static>,
        _cfg_width: u32,
        _cfg_height: u32,
    ) -> Option<wgpu::SurfaceTexture> {
        match std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            surface.get_current_texture()
        })) {
            Ok(
                wgpu::CurrentSurfaceTexture::Success(tex)
                | wgpu::CurrentSurfaceTexture::Suboptimal(tex),
            ) => Some(tex),
            Ok(wgpu::CurrentSurfaceTexture::Lost) => {
                if let Some(config) = &self.surface_config {
                    surface.configure(&self.device, config);
                }
                None
            }
            Ok(_) => None,
            Err(_) => {
                log::warn!(
                    "render_frame: get_current_texture panicked (unconfigured surface), reconfiguring"
                );
                if let Some(config) = &self.surface_config {
                    surface.configure(&self.device, config);
                }
                // Retry once after reconfigure
                match std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                    surface.get_current_texture()
                })) {
                    Ok(
                        wgpu::CurrentSurfaceTexture::Success(tex)
                        | wgpu::CurrentSurfaceTexture::Suboptimal(tex),
                    ) => Some(tex),
                    _ => None,
                }
            }
        }
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

        let mut cfg_width = self
            .surface_config
            .as_ref()
            .map(|c| c.width)
            .ok_or(GpuError::Surface("No surface config".to_string()))?;
        let mut cfg_height = self
            .surface_config
            .as_ref()
            .map(|c| c.height)
            .ok_or(GpuError::Surface("No surface config".to_string()))?;

        log::trace!(
            "render_frame: {} instances, surface={}, pipeline={}, bind_group={}",
            instances.len(),
            self.surface.is_some(),
            self.cell_pipeline.is_some(),
            self.cell_bind_group.is_some(),
        );

        let output = self.acquire_texture(surface, cfg_width, cfg_height);

        let output = match output {
            Some(tex) => tex,
            None => return Ok(()),
        };

        let tex_size = output.texture.size();
        log::info!(
            "RENDER_FRAME: config={}x{} tex={}x{} instances={}",
            cfg_width,
            cfg_height,
            tex_size.width,
            tex_size.height,
            instances.len(),
        );
        if tex_size.width != cfg_width || tex_size.height != cfg_height {
            log::warn!(
                "render_frame: size mismatch! config={}x{} texture={}x{}",
                cfg_width,
                cfg_height,
                tex_size.width,
                tex_size.height
            );
            cfg_width = tex_size.width;
            cfg_height = tex_size.height;
            let mut new_config = self.surface_config.as_ref().unwrap().clone();
            new_config.width = cfg_width;
            new_config.height = cfg_height;
            surface.configure(&self.device, &new_config);
            self.surface_config = Some(new_config);

            let aw = self.atlas_texture.as_ref().map_or(0, |t| t.width());
            let ah = self.atlas_texture.as_ref().map_or(0, |t| t.height());
            let proj = orthographic_projection(cfg_width as f32, cfg_height as f32);
            let uniforms = GpuUniforms {
                projection: proj,
                atlas_size: [aw as f32, ah as f32],
                _padding: [0.0; 2],
            };
            if let Some(buf) = &self.cell_uniform_buffer {
                self.queue
                    .write_buffer(buf, 0, bytemuck::cast_slice(&[uniforms]));
            }
            log::info!("RENDER_FRAME_RECONFIGURE: {}x{}", cfg_width, cfg_height);
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
                        load: wgpu::LoadOp::Clear(self.bg_color),
                        store: wgpu::StoreOp::Store,
                    },
                    depth_slice: None,
                })],
                depth_stencil_attachment: None,
                ..Default::default()
            });

            let viewport_width = cfg_width as f32;
            let viewport_height = cfg_height as f32;
            render_pass.set_pipeline(pipeline);
            render_pass.set_viewport(0.0, 0.0, viewport_width, viewport_height, 0.0, 1.0);
            render_pass.set_scissor_rect(0, 0, cfg_width, cfg_height);

            let has_bind_group = self.cell_bind_group.is_some();
            let has_instance_buffer = self.instance_buffer.is_some();
            log::debug!(
                "DIAG_DRAW: bind_group={} instances={} instance_buffer={}",
                has_bind_group,
                instances.len(),
                has_instance_buffer,
            );

            if let Some(bind_group) = &self.cell_bind_group {
                render_pass.set_pipeline(pipeline);
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

    /// Render the current frame to an offscreen buffer and return raw RGBA pixels.
    /// Creates a dedicated offscreen texture if one does not exist or has mismatched size.
    /// This is a test-only path — NOT used for display.
    pub fn render_to_buffer(&mut self, instances: &[CellInstance]) -> Result<Vec<u8>, GpuError> {
        let (w, h) = self
            .surface_config
            .as_ref()
            .map_or((0, 0), |c| (c.width, c.height));
        if w == 0 || h == 0 {
            return Err(GpuError::Surface("No surface config".to_string()));
        }

        let tex_size = wgpu::Extent3d {
            width: w,
            height: h,
            depth_or_array_layers: 1,
        };
        let needs_new = match &self.readback_texture {
            Some(t) => t.width() != w || t.height() != h,
            None => true,
        };
        if needs_new {
            self.readback_texture = Some(self.device.create_texture(&wgpu::TextureDescriptor {
                label: Some("Readback Texture"),
                size: tex_size,
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: wgpu::TextureFormat::Rgba8Unorm,
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
                view_formats: &[],
            }));
        }
        let texture = self.readback_texture.as_ref().unwrap();
        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());

        let buf_size = (w * h * 4) as u64;
        let needs_buf_new = match &self.readback_buffer {
            Some(b) => b.size() < buf_size,
            None => true,
        };
        if needs_buf_new {
            self.readback_buffer = Some(self.device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("Readback Buffer"),
                size: buf_size,
                usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
                mapped_at_creation: false,
            }));
        }

        let pipeline = self
            .cell_pipeline
            .as_ref()
            .ok_or_else(|| GpuError::Surface("No render pipeline".to_string()))?;

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Readback Encoder"),
            });

        // Upload instance data
        if !instances.is_empty() {
            let instance_data = bytemuck::cast_slice(instances);
            let needed_size = instance_data.len() as u64;
            let resize = self
                .instance_buffer
                .as_ref()
                .is_none_or(|b| b.size() < needed_size);
            if resize {
                self.instance_buffer = Some(self.device.create_buffer(&wgpu::BufferDescriptor {
                    label: Some("Instance Buffer (readback)"),
                    size: needed_size.max(64),
                    usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
                    mapped_at_creation: false,
                }));
            }
            if let Some(ref buf) = self.instance_buffer {
                self.queue.write_buffer(buf, 0, instance_data);
            }
        }

        // Render pass → offscreen texture
        {
            let mut rp = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Readback Render Pass"),
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
            let wf = w as f32;
            let hf = h as f32;
            rp.set_pipeline(pipeline);
            rp.set_viewport(0.0, 0.0, wf, hf, 0.0, 1.0);
            rp.set_scissor_rect(0, 0, w, h);
            if let Some(bind_group) = &self.cell_bind_group {
                rp.set_bind_group(0, bind_group, &[]);
                if !instances.is_empty() {
                    rp.set_vertex_buffer(0, self.quad_vertex_buffer.slice(..));
                    if let Some(ref ib) = self.instance_buffer {
                        rp.set_vertex_buffer(1, ib.slice(..));
                    }
                    rp.draw(0..6, 0..instances.len() as u32);
                }
            }
        }

        // Copy texture → staging buffer
        let dst = self.readback_buffer.as_ref().unwrap();
        let bytes_per_row = w * 4;
        encoder.copy_texture_to_buffer(
            wgpu::TexelCopyTextureInfo {
                texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            wgpu::TexelCopyBufferInfo {
                buffer: dst,
                layout: wgpu::TexelCopyBufferLayout {
                    offset: 0,
                    bytes_per_row: Some(bytes_per_row),
                    rows_per_image: Some(h),
                },
            },
            tex_size,
        );

        self.queue.submit(std::iter::once(encoder.finish()));

        // Wait for GPU to finish
        let _ = self.device.poll(wgpu::PollType::Wait {
            submission_index: None,
            timeout: None,
        });

        // Map buffer
        let slice = dst.slice(..);
        slice.map_async(wgpu::MapMode::Read, |r| {
            if let Err(e) = r {
                log::error!("readback map failed: {e:?}");
            }
        });
        let _ = self.device.poll(wgpu::PollType::Wait {
            submission_index: None,
            timeout: None,
        });
        let data = slice.get_mapped_range().to_vec();
        dst.unmap();

        Ok(data)
    }
}

pub fn orthographic_projection(width: f32, height: f32) -> [[f32; 4]; 4] {
    // wgpu NDC: clip_y=-1 = top, clip_y=+1 = bottom.
    // Terminal: row 0 = top of screen.
    //
    // GLES swapchain on Android presents with (0,0) at bottom-left (GL convention),
    // but ANativeWindow expects (0,0) at top-left. This causes an unwanted Y-flip.
    // To compensate, we flip the projection Y so the swapchain's double-flip
    // produces the correct orientation:
    //   clip_y = -2 * world_y / height + 1
    //   world_y=0 → clip_y=+1 (after presentation flip → screen top)
    //   world_y=h → clip_y=-1 (after presentation flip → screen bottom)
    //
    // Rust stores row-major but WGSL reads column-major.
    // With M*vec4, result[i] = sum_j Rust[j][i] * vec[j].
    // Translation components must be in Rust[3][0] and Rust[3][1]
    // so they act on the w component of the column-major columns.
    [
        [2.0 / width, 0.0, 0.0, 0.0],
        [0.0, -2.0 / height, 0.0, 0.0],
        [0.0, 0.0, 1.0, 0.0],
        [-1.0, 1.0, 0.0, 1.0],
    ]
}

/// Flat grid data for building cell instances without a GridSnapshot.
/// Used by Ghostty VT integration where terminal state is managed externally.
#[cfg(test)]
pub struct FlatGrid {
    pub rows: u32,
    pub cols: u32,
    pub chars: Vec<char>,
    pub fg: Vec<[f32; 4]>,
    pub bg: Vec<[f32; 4]>,
}

#[cfg(test)]
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

#[cfg(test)]
pub fn build_cell_instances_from_flat(
    flat: &FlatGrid,
    font_pipeline: &mut crate::font::FontPipeline,
    atlas_width: f32,
    atlas_height: f32,
) -> Vec<CellInstance> {
    let (cell_w, cell_h) = font_pipeline.cell_metrics();
    let ascent_px = font_pipeline.ascent_px();
    let mut instances = Vec::with_capacity((flat.rows * flat.cols) as usize);

    for row in 0..flat.rows {
        for col in 0..flat.cols {
            if let Some((ch, fg, bg)) = flat.cell(row, col) {
                if ch == ' ' {
                    instances.push(CellInstance {
                        quad_origin: [col as f32 * cell_w, row as f32 * cell_h],
                        atlas_offset: [0.0, 0.0],
                        atlas_size: [0.0, 0.0],
                        fg_color: [0.0, 0.0, 0.0, 0.0],
                        bg_color: bg,
                        quad_size: [cell_w, cell_h],
                        flags: 0.0,
                        bearing: [0.0; 2],
                        glyph_advance_width: 0.0,
                    });
                } else if let Some(info) = font_pipeline.glyph_info(ch) {
                    let uv_x = info.atlas_x as f32 / atlas_width;
                    let uv_y = info.atlas_y as f32 / atlas_height;
                    let uv_w = info.width as f32 / atlas_width;
                    let uv_h = info.height as f32 / atlas_height;

                    let bearing_x = info.placement.left as f32;
                    let bearing_y = ascent_px - info.placement.top as f32;

                    instances.push(CellInstance {
                        quad_origin: [col as f32 * cell_w, row as f32 * cell_h],
                        atlas_offset: [uv_x, uv_y],
                        atlas_size: [uv_w, uv_h],
                        fg_color: fg,
                        bg_color: bg,
                        quad_size: [cell_w, cell_h],
                        flags: 0.0,
                        bearing: [bearing_x, bearing_y],
                        glyph_advance_width: info.advance_width,
                    });
                }
            }
        }
    }
    instances
}

#[allow(clippy::collapsible_if)]
#[derive(Debug, Clone, Copy, Default)]
pub struct SelectionRange {
    pub start_row: i32,
    pub start_col: i32,
    pub end_row: i32,
    pub end_col: i32,
    pub active: bool,
}

impl SelectionRange {
    pub fn contains(&self, row: u32, col: u32) -> bool {
        if !self.active {
            return false;
        }
        let row = row as i32;
        let col = col as i32;
        let (lo_row, lo_col, hi_row, hi_col) = if self.start_row < self.end_row
            || (self.start_row == self.end_row && self.start_col <= self.end_col)
        {
            (self.start_row, self.start_col, self.end_row, self.end_col)
        } else {
            (self.end_row, self.end_col, self.start_row, self.start_col)
        };
        if row < lo_row || row > hi_row {
            return false;
        }
        if lo_row == hi_row {
            col >= lo_col && col <= hi_col
        } else if row == lo_row {
            col >= lo_col
        } else if row == hi_row {
            col <= hi_col
        } else {
            true
        }
    }
}

pub fn build_cell_instances_from_snapshot(
    snapshot: &torvox_terminal::ghostty_terminal::GridSnapshot,
    font_pipeline: &mut crate::font::FontPipeline,
    atlas_width: f32,
    atlas_height: f32,
    dirty_rows: Option<&[bool]>,
    selection: Option<SelectionRange>,
    cursor_color: Option<[f32; 4]>,
) -> Vec<CellInstance> {
    let rows = snapshot.rows;
    let cols = snapshot.cols;
    let (cell_w, cell_h) = font_pipeline.cell_metrics();
    let ascent_px = font_pipeline.ascent_px();
    let mut instances = Vec::with_capacity((rows * cols) as usize);

    let cursor_row = snapshot.cursor_row;
    let cursor_col = snapshot.cursor_col;
    let cursor_visible = snapshot.cursor_visible;

    let mut glyph_found = 0u64;
    let mut glyph_not_found = 0u64;
    for row in 0..rows {
        if let Some(dirty) = dirty_rows
            && row < dirty.len() as u32
            && !dirty[row as usize]
        {
            for col in 0..cols {
                let is_cursor = cursor_visible && row == cursor_row && col == cursor_col;
                let (fg, bg) = if is_cursor {
                    let cursor_bg = cursor_color.unwrap_or([1.0, 1.0, 1.0, 1.0]);
                    ([0.0, 0.0, 0.0, 1.0], cursor_bg)
                } else {
                    ([0.0, 0.0, 0.0, 0.0], [0.0, 0.0, 0.0, 0.0])
                };
                instances.push(CellInstance {
                    quad_origin: [col as f32 * cell_w, row as f32 * cell_h],
                    atlas_offset: [0.0; 2],
                    atlas_size: [0.0; 2],
                    fg_color: fg,
                    bg_color: bg,
                    quad_size: [cell_w, cell_h],
                    flags: 0.0,
                    bearing: [0.0; 2],
                    glyph_advance_width: 0.0,
                });
            }
            continue;
        }
        let mut skip_cols = 0u32;
        for col in 0..cols {
            if skip_cols > 0 {
                skip_cols -= 1;
                continue;
            }

            let idx = (row * cols + col) as usize;
            let cell = &snapshot.cells[idx];
            let cell_span = cell.width.max(1) as f32;
            let is_cursor = cursor_visible && row == cursor_row && col == cursor_col;

            if cell.codepoint == 0 || cell.codepoint == 0x20 {
                let (mut fg, mut bg) = if selection.unwrap_or_default().contains(row, col) {
                    (cell.bg, cell.fg)
                } else {
                    ([0.0, 0.0, 0.0, 0.0], cell.bg)
                };
                if is_cursor {
                    let cursor_bg = cursor_color.unwrap_or([1.0, 1.0, 1.0, 1.0]);
                    fg = bg;
                    bg = cursor_bg;
                }
                instances.push(CellInstance {
                    quad_origin: [col as f32 * cell_w, row as f32 * cell_h],
                    atlas_offset: [0.0; 2],
                    atlas_size: [0.0; 2],
                    fg_color: fg,
                    bg_color: bg,
                    quad_size: [cell_w, cell_h],
                    flags: 0.0,
                    bearing: [0.0; 2],
                    glyph_advance_width: 0.0,
                });
                continue;
            }

            let ch = char::from_u32(cell.codepoint).unwrap_or('\u{FFFD}');
            let flags = if cell.bold { 1.0 } else { 0.0 }
                + if cell.italic { 2.0 } else { 0.0 }
                + if cell.reverse { 4.0 } else { 0.0 }
                + if cell.underline { 8.0 } else { 0.0 }
                + if cell.uri.is_some() { 16.0 } else { 0.0 };

            let (mut fg, mut bg) = if cell.reverse {
                (cell.bg, cell.fg)
            } else {
                (cell.fg, cell.bg)
            };

            if selection.unwrap_or_default().contains(row, col) {
                std::mem::swap(&mut fg, &mut bg);
            }

            let (fg, bg) = if is_cursor {
                // Standard terminal behavior: invert fg/bg on cursor cell
                // so the character is visible against the cursor background.
                let cursor_bg = cursor_color.unwrap_or([1.0, 1.0, 1.0, 1.0]);
                (bg, cursor_bg)
            } else {
                (fg, bg)
            };

            if let Some(info) = font_pipeline.glyph_info(ch) {
                glyph_found += 1;
                let uv_x = info.atlas_x as f32 / atlas_width;
                let uv_y = info.atlas_y as f32 / atlas_height;
                let uv_w = info.width as f32 / atlas_width;
                let uv_h = info.height as f32 / atlas_height;

                // bearing_x = font's left side bearing (x offset from advance origin to glyph left)
                let bearing_x = info.placement.left as f32;
                // bearing_y positions the glyph within the cell.
                //
                // For ASCII: bearing_y = ascent_px - placement.top (natural baseline).
                // For CJK fallback: placement.top > ascent_px (different font metrics),
                //   making raw bearing_y negative and the glyph extend above the cell.
                //   When the glyph is TALLER than the cell (glyph_h > cell_h), we center
                //   it vertically instead — this prevents the flat/squished look that
                //   clipping causes. Matches Kitty/Ghostty: oversized fallback glyphs are
                //   centered, not baseline-positioned, when they exceed the cell.
                let glyph_h = info.height as f32;
                let raw_bearing_y = ascent_px - info.placement.top as f32;
                let bearing_y = if glyph_h > cell_h {
                    // Center oversized glyph (CJK fallback with different metrics)
                    (cell_h - glyph_h) / 2.0
                } else {
                    // Normal baseline positioning for glyphs that fit
                    raw_bearing_y
                };

                instances.push(CellInstance {
                    quad_origin: [col as f32 * cell_w, row as f32 * cell_h],
                    atlas_offset: [uv_x, uv_y],
                    atlas_size: [uv_w, uv_h],
                    fg_color: fg,
                    bg_color: bg,
                    quad_size: [cell_w * cell_span, cell_h],
                    flags,
                    bearing: [bearing_x, bearing_y],
                    glyph_advance_width: info.advance_width,
                });
                if cell_span > 1.0 {
                    skip_cols = (cell_span as u32) - 1;
                }
            } else {
                glyph_not_found += 1;
                instances.push(CellInstance {
                    quad_origin: [col as f32 * cell_w, row as f32 * cell_h],
                    atlas_offset: [0.0; 2],
                    atlas_size: [1.0 / atlas_width, 1.0 / atlas_height],
                    fg_color: fg,
                    bg_color: bg,
                    quad_size: [cell_w * cell_span, cell_h],
                    flags,
                    bearing: [0.0; 2],
                    glyph_advance_width: 0.0,
                });
                if cell_span > 1.0 {
                    skip_cols = (cell_span as u32) - 1;
                }
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
        assert_eq!(std::mem::size_of::<CellInstance>(), 80);
    }

    #[test]
    fn orthographic_projection_identity() {
        let proj = orthographic_projection(800.0, 600.0);
        assert!((proj[0][0] - 2.0 / 800.0).abs() < f32::EPSILON);
        assert!((proj[1][1] + 2.0 / 600.0).abs() < f32::EPSILON);
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
        let (cell_w, _cell_h) = font_pipeline.cell_metrics();

        let instances = build_cell_instances_from_flat(&grid, &mut font_pipeline, 2048.0, 2048.0);
        assert_eq!(instances.len(), 4);

        let cell0 = &instances[0];
        assert_eq!(cell0.quad_origin, [0.0, 0.0]);
        assert_eq!(cell0.bg_color, [0.0, 0.0, 0.0, 1.0]);

        let cell1 = &instances[1];
        assert_eq!(cell1.quad_origin, [cell_w, 0.0]);
        assert_eq!(cell1.bg_color, [0.5, 0.5, 0.5, 1.0]);
    }

    #[test]
    fn cell_instance_pod_roundtrip() {
        let c = CellInstance {
            quad_origin: [1.0, 2.0],
            atlas_offset: [0.5, 0.5],
            atlas_size: [0.1, 0.1],
            fg_color: [1.0, 1.0, 1.0, 1.0],
            bg_color: [0.0, 0.0, 0.0, 1.0],
            quad_size: [3.0, 4.0],
            flags: 5.0,
            bearing: [0.0; 2],
            glyph_advance_width: 8.0,
        };
        let bytes = bytemuck::bytes_of(&c);
        let back: &CellInstance = bytemuck::from_bytes(bytes);
        assert_eq!(back.quad_origin, [1.0, 2.0]);
        assert_eq!(back.flags, 5.0);
        assert_eq!(back.glyph_advance_width, 8.0);
    }

    #[test]
    fn cell_instance_zeroable() {
        let c: CellInstance = bytemuck::Zeroable::zeroed();
        assert_eq!(c.quad_origin, [0.0, 0.0]);
        assert_eq!(c.fg_color, [0.0, 0.0, 0.0, 0.0]);
        assert_eq!(c.flags, 0.0);
        assert_eq!(c.bearing, [0.0, 0.0]);
    }

    #[test]
    fn cursor_instance_size() {
        // Verify it is pod and serializable
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
        assert_eq!(std::mem::size_of::<GpuUniforms>(), 80);
    }

    #[test]
    fn orthographic_projection_basic() {
        let proj = orthographic_projection(100.0, 100.0);
        // [0][0] = 2/width
        assert!((proj[0][0] - 0.02).abs() < f32::EPSILON);
        // [1][1] = -2/height (flipped Y for GLES swapchain)
        assert!((proj[1][1] + 0.02).abs() < f32::EPSILON);
        assert!((proj[3][1] - 1.0).abs() < f32::EPSILON, "translation Y");
        // [3][3] = 1 (result.w=1 for all vertices via Rust row-major→WGSL column-major)
        assert!((proj[3][3] - 1.0).abs() < f32::EPSILON);
        // Rust row-major → WGSL column-major: result[0]=sum_j Rust[j][0]*v[j]
        // For v=(1,1,0,1):
        //   result[0]=2/w*1 + 0*1 + 0*0 + (-1)*1 = 0.02-1 = -0.98
        //   result[1]=0*1 + (-2/h)*1 + 0*0 + 1*1 = -0.02+1 = 0.98
        //   result[3]=0*1 + 0*1 + 0*0 + 1*1 = 1
        let v = [1.0_f32, 1.0, 0.0, 1.0];
        let mut result = [0.0_f32; 4];
        for i in 0..4 {
            result[i] =
                proj[0][i] * v[0] + proj[1][i] * v[1] + proj[2][i] * v[2] + proj[3][i] * v[3];
        }
        assert!(
            (result[0] - (-0.98)).abs() < 1e-6,
            "result[0]={}",
            result[0]
        );
        assert!((result[1] - (0.98)).abs() < 1e-6, "result[1]={}", result[1]);
        assert!((result[3] - 1.0).abs() < 1e-6, "result[3]={}", result[3]);
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
        // Should not panic, value not stored
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
        // All spaces, atlas_size should be 0
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
        // 3 cells (CJK may or may not be rasterized, may or may not produce instances)
        // Verify no panic
        assert_eq!(instances.len(), 3);
    }

    #[test]
    fn cell_instance_attribs_locations() {
        let attribs = CellInstance::ATTRIBS;
        assert_eq!(attribs.len(), 9);
        assert_eq!(attribs[0].shader_location, 1);
        assert_eq!(attribs[1].shader_location, 2);
        assert_eq!(attribs[7].shader_location, 8);
        assert_eq!(attribs[8].shader_location, 9);
    }

    #[test]
    fn orthographic_projection_zero_size() {
        let proj = orthographic_projection(0.0, 0.0);
        // 2.0 / 0.0 = inf
        assert!(proj[0][0].is_infinite());
    }

    /// Helper: create a wgpu instance + adapter + device for testing.
    /// Returns None when no suitable GPU is available.
    fn create_test_device() -> Option<(wgpu::Instance, wgpu::Adapter, wgpu::Device, wgpu::Queue)> {
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor::new_without_display_handle());
        let adapter =
            futures::executor::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
                compatible_surface: None,
                power_preference: wgpu::PowerPreference::LowPower,
                force_fallback_adapter: false,
            }))
            .ok()?;
        let (device, queue) =
            futures::executor::block_on(adapter.request_device(&wgpu::DeviceDescriptor {
                label: Some("gpu_test_device"),
                required_features: wgpu::Features::empty(),
                required_limits: wgpu::Limits::default(),
                ..Default::default()
            }))
            .ok()?;
        Some((instance, adapter, device, queue))
    }

    fn write_storage_buffer_shader() -> wgpu::ShaderModuleDescriptor<'static> {
        wgpu::ShaderModuleDescriptor {
            label: Some("write_color"),
            source: wgpu::ShaderSource::Wgsl(std::borrow::Cow::Borrowed(
                "@group(0) @binding(0) var<storage, read_write> output: array<vec4<f32>>;
                @compute @workgroup_size(1, 1, 1)
                fn cs_main(@builtin(global_invocation_id) gid: vec3<u32>) {
                    output[gid.x] = vec4<f32>(0.157, 0.165, 0.212, 1.0);
                }",
            )),
        }
    }

    fn blend_shader() -> wgpu::ShaderModuleDescriptor<'static> {
        wgpu::ShaderModuleDescriptor {
            label: Some("blend"),
            source: wgpu::ShaderSource::Wgsl(std::borrow::Cow::Borrowed(
                "@group(0) @binding(0) var<storage, read_write> output: array<vec4<f32>>;
                @compute @workgroup_size(1, 1, 1)
                fn cs_main(@builtin(global_invocation_id) gid: vec3<u32>) {
                    let bg = vec4<f32>(40.0/255.0, 42.0/255.0, 54.0/255.0, 1.0);
                    let fg = vec4<f32>(1.0, 1.0, 1.0, 1.0);
                    let alpha = 0.5;
                    output[gid.x] = mix(bg, fg, vec4<f32>(alpha, alpha, alpha, alpha));
                }",
            )),
        }
    }

    fn run_compute_and_capture(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        shader_desc: wgpu::ShaderModuleDescriptor,
    ) -> [u8; 4] {
        let shader = device.create_shader_module(shader_desc);

        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("bind_group_layout"),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::COMPUTE,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Storage { read_only: false },
                    has_dynamic_offset: false,
                    min_binding_size: Some(std::num::NonZeroU64::new(16).unwrap()),
                },
                count: None,
            }],
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("compute_layout"),
            bind_group_layouts: &[Some(&bind_group_layout)],
            immediate_size: 0,
        });

        let compute_pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("compute_pipeline"),
            layout: Some(&pipeline_layout),
            module: &shader,
            entry_point: Some("cs_main"),
            compilation_options: wgpu::PipelineCompilationOptions::default(),
            cache: None,
        });

        // Create storage buffer (1 vec4<f32> = 16 bytes)
        let storage_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("storage_buffer"),
            size: 16,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC,
            mapped_at_creation: false,
        });

        // Create staging buffer for readback
        let staging_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("staging_buffer"),
            size: 16,
            usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("bind_group"),
            layout: &bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: storage_buffer.as_entire_binding(),
            }],
        });

        // Dispatch compute
        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("compute_encoder"),
        });
        {
            let mut cpass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some("compute_pass"),
                timestamp_writes: None,
            });
            cpass.set_pipeline(&compute_pipeline);
            cpass.set_bind_group(0, &bind_group, &[]);
            cpass.dispatch_workgroups(1, 1, 1);
        }

        // Copy storage to staging
        encoder.copy_buffer_to_buffer(&storage_buffer, 0, &staging_buffer, 0, 16);

        queue.submit(Some(encoder.finish()));

        // Map read
        let (tx, rx) = std::sync::mpsc::channel();
        staging_buffer
            .slice(..)
            .map_async(wgpu::MapMode::Read, move |result| {
                tx.send(result).ok();
            });
        let _ = device.poll(wgpu::PollType::Wait {
            submission_index: None,
            timeout: None,
        });

        if let Ok(Ok(())) = rx.recv() {
            let view = staging_buffer.slice(..).get_mapped_range();
            let actual: [u8; 4] = [view[0], view[1], view[2], view[3]];
            drop(view);
            staging_buffer.unmap();
            actual
        } else {
            panic!("buffer map_async failed");
        }
    }

    #[test]
    fn gpu_compute_write_color() {
        let Some((_instance, _adapter, device, queue)) = create_test_device() else {
            panic!("requires GPU adapter but none available");
        };
        let output1 = run_compute_and_capture(&device, &queue, write_storage_buffer_shader());
        assert_ne!(
            output1,
            [0, 0, 0, 0],
            "write_color: shader produced all zeros — pipeline broken"
        );

        let output2 = run_compute_and_capture(&device, &queue, write_storage_buffer_shader());
        assert_eq!(
            output1, output2,
            "write_color: non-deterministic output: first={:?} second={:?}",
            output1, output2
        );

        let input = [255u8; 4];
        assert_ne!(
            output1, input,
            "write_color: output identical to input — shader is a no-op"
        );
    }

    #[test]
    fn gpu_compute_blend() {
        let Some((_instance, _adapter, device, queue)) = create_test_device() else {
            panic!("requires GPU adapter but none available");
        };
        let output1 = run_compute_and_capture(&device, &queue, blend_shader());
        assert_ne!(
            output1,
            [0, 0, 0, 0],
            "blend: shader produced all zeros — pipeline broken"
        );

        let output2 = run_compute_and_capture(&device, &queue, blend_shader());
        assert_eq!(
            output1, output2,
            "blend: non-deterministic output: first={:?} second={:?}",
            output1, output2
        );
    }

    #[test]
    fn orthographic_projection_resize_changes_mapping() {
        let proj_wide = orthographic_projection(800.0, 400.0);
        let proj_small = orthographic_projection(800.0, 200.0);

        // Same world Y should map to different clip Y when height changes.
        let world_y = 100.0;
        let clip_y_400 = -2.0 * world_y / 400.0 + 1.0;
        let clip_y_200 = -2.0 * world_y / 200.0 + 1.0;
        assert!(
            clip_y_200 < clip_y_400,
            "smaller height -> same world Y maps lower in clip space"
        );

        // Verify using matrix directly (Rust row-major → WGSL column-major):
        // clip_y = proj[1][1] * world_y + proj[3][1]
        let result_400: f32 = proj_wide[1][1] * world_y + proj_wide[3][1];
        let result_200: f32 = proj_small[1][1] * world_y + proj_small[3][1];
        assert!(
            result_200 < result_400,
            "matrix clip_y at h=200 ({}) < h=400 ({})",
            result_200,
            result_400
        );
    }

    #[test]
    fn orthographic_projection_resize_full_range() {
        // Compute WGSL clip_y = proj[0][1]*v[0] + proj[1][1]*v[1] + proj[2][1]*v[2] + proj[3][1]*v[3]
        // (Rust row-major → WGSL column-major multiplication)
        let clip_y = |proj: &[[f32; 4]; 4], y: f32| -> f32 {
            proj[0][1] * 0.0 + proj[1][1] * y + proj[2][1] * 0.0 + proj[3][1] * 1.0
        };

        // At 800x400 the entire surface height maps to clip [-1, 1].
        let proj = orthographic_projection(800.0, 400.0);
        let clip_y_top = clip_y(&proj, 0.0);
        let clip_y_bot = clip_y(&proj, 400.0);

        assert!(
            (clip_y_top - 1.0).abs() < 1e-6,
            "top of surface (y=0) -> clip_y=+1, got {}",
            clip_y_top
        );
        assert!(
            (clip_y_bot + 1.0).abs() < 1e-6,
            "bottom of surface (y=400) -> clip_y=-1, got {}",
            clip_y_bot
        );

        // At smaller height 800x250, bottom should still map to clip_y=-1.
        let proj_250 = orthographic_projection(800.0, 250.0);
        let clip_y_bot_250 = clip_y(&proj_250, 250.0);
        assert!(
            (clip_y_bot_250 + 1.0).abs() < 1e-6,
            "bottom of smaller surface (y=250) -> clip_y=-1, got {}",
            clip_y_bot_250
        );
    }

    #[test]
    fn orthographic_projection_resize_gpu_uniforms() {
        let uniforms_800 = GpuUniforms {
            projection: orthographic_projection(800.0, 600.0),
            atlas_size: [1024.0, 1024.0],
            _padding: [0.0; 2],
        };
        let uniforms_400 = GpuUniforms {
            projection: orthographic_projection(800.0, 400.0),
            atlas_size: [1024.0, 1024.0],
            _padding: [0.0; 2],
        };

        // Same projection/atlas layout, different height
        assert_eq!(
            std::mem::size_of_val(&uniforms_800),
            std::mem::size_of_val(&uniforms_400)
        );
        // Projection matrix Y-scale differs
        assert!(
            (uniforms_800.projection[1][1] - uniforms_400.projection[1][1]).abs() > 0.001,
            "Y-scale should differ: 800={} 400={}",
            uniforms_800.projection[1][1],
            uniforms_400.projection[1][1]
        );
        // Write/readback via bytemuck
        let bytes = bytemuck::bytes_of(&uniforms_400);
        let back: &GpuUniforms = bytemuck::from_bytes(bytes);
        assert_eq!(back.projection[1][1], uniforms_400.projection[1][1]);
    }

    #[test]
    fn cursor_rendering_on_visible_cursor() {
        use torvox_terminal::ghostty_terminal::{CellSnapshot, GridSnapshot};
        let mut font_pipeline = crate::font::FontPipeline::new(2048, 2048, 14.0);
        font_pipeline.rasterize_ascii();
        let cells = vec![
            CellSnapshot {
                codepoint: 'A' as u32,
                ..Default::default()
            },
            CellSnapshot {
                codepoint: 0,
                ..Default::default()
            },
        ];
        let snapshot = GridSnapshot {
            rows: 1,
            cols: 2,
            cursor_row: 0,
            cursor_col: 0,
            cursor_visible: true,
            cells,
        };
        let cursor_color = Some([1.0, 1.0, 1.0, 1.0]);
        let instances = build_cell_instances_from_snapshot(
            &snapshot,
            &mut font_pipeline,
            2048.0,
            2048.0,
            None,
            None,
            cursor_color,
        );
        assert_eq!(instances.len(), 2);
        let cursor_cell = &instances[0];
        assert_eq!(
            cursor_cell.bg_color,
            [1.0, 1.0, 1.0, 1.0],
            "cursor cell bg should be white when cursor_visible=true"
        );
        let non_cursor_cell = &instances[1];
        assert_ne!(
            non_cursor_cell.bg_color,
            [1.0, 1.0, 1.0, 1.0],
            "non-cursor cell bg should NOT be white"
        );
    }

    #[test]
    fn cursor_not_rendered_when_invisible() {
        use torvox_terminal::ghostty_terminal::{CellSnapshot, GridSnapshot};
        let mut font_pipeline = crate::font::FontPipeline::new(2048, 2048, 14.0);
        font_pipeline.rasterize_ascii();
        let cells = vec![CellSnapshot {
            codepoint: 'A' as u32,
            ..Default::default()
        }];
        let snapshot = GridSnapshot {
            rows: 1,
            cols: 1,
            cursor_row: 0,
            cursor_col: 0,
            cursor_visible: false,
            cells,
        };
        let instances = build_cell_instances_from_snapshot(
            &snapshot,
            &mut font_pipeline,
            2048.0,
            2048.0,
            None,
            None,
            Some([1.0, 1.0, 1.0, 1.0]),
        );
        assert_eq!(instances.len(), 1);
        let cell = &instances[0];
        assert_ne!(
            cell.bg_color,
            [1.0, 1.0, 1.0, 1.0],
            "cursor cell should not have white bg when cursor_visible=false"
        );
    }

    #[test]
    fn selection_swaps_fg_bg() {
        use super::SelectionRange;
        use torvox_terminal::ghostty_terminal::{CellSnapshot, GridSnapshot};
        let mut font_pipeline = crate::font::FontPipeline::new(2048, 2048, 14.0);
        font_pipeline.rasterize_ascii();
        let cells = vec![CellSnapshot {
            codepoint: 'X' as u32,
            fg: [1.0, 0.0, 0.0, 1.0],
            bg: [0.0, 0.0, 0.0, 1.0],
            ..Default::default()
        }];
        let snapshot = GridSnapshot {
            rows: 1,
            cols: 1,
            cursor_row: 0,
            cursor_col: 0,
            cursor_visible: false,
            cells,
        };
        let selection = Some(SelectionRange {
            start_row: 0,
            end_row: 0,
            start_col: 0,
            end_col: 0,
            active: true,
        });
        let instances = build_cell_instances_from_snapshot(
            &snapshot,
            &mut font_pipeline,
            2048.0,
            2048.0,
            None,
            selection,
            None,
        );
        assert_eq!(instances.len(), 1);
        let cell = &instances[0];
        assert_eq!(
            cell.fg_color,
            [0.0, 0.0, 0.0, 1.0],
            "selected cell fg should be original bg (swap)"
        );
        assert_eq!(
            cell.bg_color,
            [1.0, 0.0, 0.0, 1.0],
            "selected cell bg should be original fg (swap)"
        );
    }

    // ── Bearing correctness: Termux-aligned font metrics ──────────────

    /// Verify bearing_y uses font baseline, not centering.
    /// build_cell_instances_from_flat uses raw bearing_y = ascent_px - placement.top
    /// (no centering, no clamping — the raw font baseline offset).
    #[test]
    fn bearing_y_uses_font_baseline_not_centering() {
        let mut font_pipeline = crate::font::FontPipeline::new(2048, 2048, 14.0);
        font_pipeline.rasterize_ascii();
        let ascent_px = font_pipeline.ascent_px();

        let chars = ['A', 'g', 'p', '.', ','];
        for ch in chars {
            let info = font_pipeline.glyph_info(ch).expect("glyph exists");
            let expected_bearing_y = ascent_px - info.placement.top as f32;

            let mut grid = FlatGrid::new(1, 1);
            grid.set_cell(0, 0, ch, [1.0; 4], [0.0; 4]);
            let instances =
                build_cell_instances_from_flat(&grid, &mut font_pipeline, 2048.0, 2048.0);
            let cell = &instances[0];

            assert!(
                (cell.bearing[1] - expected_bearing_y).abs() < 1.0,
                "'{ch}' bearing_y={} should be font baseline {} (not centered)",
                cell.bearing[1],
                expected_bearing_y
            );
        }
    }

    /// Verify bearing_x uses font's natural left side bearing, not centering.
    #[test]
    fn bearing_x_uses_font_natural_bearing() {
        let mut font_pipeline = crate::font::FontPipeline::new(2048, 2048, 14.0);
        font_pipeline.rasterize_ascii();

        let chars = ['A', 'i', 'l', 'W', 'M'];
        for ch in chars {
            let info = font_pipeline.glyph_info(ch).expect("glyph exists");
            let expected_bearing_x = info.placement.left as f32;

            let mut grid = FlatGrid::new(1, 1);
            grid.set_cell(0, 0, ch, [1.0; 4], [0.0; 4]);
            let instances =
                build_cell_instances_from_flat(&grid, &mut font_pipeline, 2048.0, 2048.0);
            let cell = &instances[0];

            assert!(
                (cell.bearing[0] - expected_bearing_x).abs() < 1.0,
                "'{ch}' bearing_x={} should be font natural bearing {}",
                cell.bearing[0],
                expected_bearing_x
            );
        }
    }

    /// Verify all characters in a row share the same bearing_y baseline.
    /// This ensures no vertical misalignment between characters.
    #[test]
    fn all_chars_share_same_baseline_y() {
        let mut font_pipeline = crate::font::FontPipeline::new(2048, 2048, 14.0);
        font_pipeline.rasterize_ascii();
        let (cell_w, cell_h) = font_pipeline.cell_metrics();

        let chars = ['A', 'B', 'C', 'x', 'y', 'z', '0', '1', '9'];
        let mut grid = FlatGrid::new(1, chars.len() as u32);
        for (i, &ch) in chars.iter().enumerate() {
            grid.set_cell(0, i as u32, ch, [1.0; 4], [0.0; 4]);
        }
        let instances = build_cell_instances_from_flat(&grid, &mut font_pipeline, 2048.0, 2048.0);
        assert_eq!(instances.len(), chars.len());

        // All cells should have the same quad_size (cell dimensions)
        for (i, inst) in instances.iter().enumerate() {
            assert!(
                (inst.quad_size[0] - cell_w).abs() < 0.1,
                "cell[{i}] width={} should be {cell_w}",
                inst.quad_size[0]
            );
            assert!(
                (inst.quad_size[1] - cell_h).abs() < 0.1,
                "cell[{i}] height={} should be {cell_h}",
                inst.quad_size[1]
            );
        }
    }

    /// Verify CJK bearing_y is not centered.
    /// build_cell_instances_from_flat uses raw bearing_y = ascent_px - placement.top.
    #[test]
    fn cjk_bearing_y_not_centered() {
        let mut font_pipeline = crate::font::FontPipeline::new(2048, 2048, 14.0);
        let ascent_px = font_pipeline.ascent_px();

        let cjk_chars = ['中', '文', '好'];
        for ch in cjk_chars {
            if let Some(info) = font_pipeline.glyph_info(ch) {
                let expected = ascent_px - info.placement.top as f32;

                let mut grid = FlatGrid::new(1, 2);
                grid.set_cell(0, 0, ch, [1.0; 4], [0.0; 4]);
                let instances =
                    build_cell_instances_from_flat(&grid, &mut font_pipeline, 2048.0, 2048.0);
                let cell = &instances[0];

                assert!(
                    (cell.bearing[1] - expected).abs() < 1.0,
                    "'{ch}' bearing_y={} should be font baseline {} (not centered)",
                    cell.bearing[1],
                    expected
                );
            }
        }
    }
}
