use std::{collections::HashMap, sync::Arc};

use bevy_ecs::entity::Entity;
use bytemuck::Zeroable;
use glam::{Mat4, Vec3, Vec4};
use wgpu::util::DeviceExt;
use winit::window::Window;

use crate::{
    Engine,
    mesh::{Mesh, Vertices},
    nodes::{
        planet::Star,
        sky::{GlobalSky, GlobalSpace},
    },
    physics::PhysicsBody,
    transform::{Transform, WorldPositionExt},
};

const MAX_LIGHTS: usize = 8;
const PHYSICS_MESH_LOAD_DISTANCE: f32 = 10_000.0;
const PHYSICS_MESH_UNLOAD_DISTANCE: f32 = 10_000.0;
const STATIC_MESH_LOAD_DISTANCE: f32 = 100_000_000_000.0;
const STATIC_MESH_UNLOAD_DISTANCE: f32 = 100_000_000_000.0;

pub fn logical_extent(window: &Window) -> [u32; 2] {
    let size = window.inner_size().to_logical::<f64>(window.scale_factor());
    [
        (size.width.round() as u32).max(1),
        (size.height.round() as u32).max(1),
    ]
}

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct GpuLight {
    position: [f32; 4],
    color_power: [f32; 4],
}

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct SceneUniform {
    view_proj: [[f32; 4]; 4],
    inv_view_proj: [[f32; 4]; 4],
    viewport: [f32; 4],
    sky_top: [f32; 4],
    sky_bottom: [f32; 4],
    light_count: [u32; 4],
    lights: [GpuLight; MAX_LIGHTS],
    space: [f32; 4],
}

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct ModelUniform {
    model: [[f32; 4]; 4],
    emissive: [f32; 4],
}

struct MeshGpu {
    vertex: wgpu::Buffer,
    index: wgpu::Buffer,
    lods: Vec<Vec<MeshCluster>>,
}

struct MeshCluster {
    index_start: u32,
    index_count: u32,
    base_vertex: i32,
    center: Vec3,
    radius: f32,
}

struct DrawItem {
    entity: Entity,
    mesh_id: u64,
    lod: usize,
    position: Vec3,
    rotation: glam::Quat,
    scale: Vec3,
    bind_group: wgpu::BindGroup,
}

struct Frustum {
    planes: [Vec4; 6],
}

impl Frustum {
    fn from_view_proj(view_proj: Mat4) -> Self {
        let columns = view_proj.to_cols_array_2d();
        let row0 = Vec4::new(columns[0][0], columns[1][0], columns[2][0], columns[3][0]);
        let row1 = Vec4::new(columns[0][1], columns[1][1], columns[2][1], columns[3][1]);
        let row2 = Vec4::new(columns[0][2], columns[1][2], columns[2][2], columns[3][2]);
        let row3 = Vec4::new(columns[0][3], columns[1][3], columns[2][3], columns[3][3]);
        let planes = [
            row3 + row0,
            row3 - row0,
            row3 + row1,
            row3 - row1,
            row2,
            row3 - row2,
        ]
        .map(|plane| {
            let normal_length = plane.truncate().length();
            if normal_length > f32::EPSILON && normal_length.is_finite() {
                plane / normal_length
            } else {
                Vec4::ZERO
            }
        });
        Self { planes }
    }

    fn sphere_visible(&self, center: Vec3, radius: f32) -> bool {
        self.planes
            .iter()
            .all(|plane| plane.truncate().dot(center) + plane.w >= -radius)
    }
}

pub struct Renderer {
    surface: Option<wgpu::Surface<'static>>,
    surface_config: Option<wgpu::SurfaceConfiguration>,
    device: Option<wgpu::Device>,
    queue: Option<wgpu::Queue>,
    scene_pipeline: Option<wgpu::RenderPipeline>,
    sky_pipeline: Option<wgpu::RenderPipeline>,
    outline_pipeline: Option<wgpu::RenderPipeline>,
    model_bind_group_layout: Option<wgpu::BindGroupLayout>,
    scene_buffer: Option<wgpu::Buffer>,
    scene_bind_group: Option<wgpu::BindGroup>,
    viewport_texture: Option<wgpu::Texture>,
    viewport_view: Option<wgpu::TextureView>,
    viewport_present_pipeline: Option<wgpu::RenderPipeline>,
    viewport_present_layout: Option<wgpu::BindGroupLayout>,
    viewport_present_bind_group: Option<wgpu::BindGroup>,
    viewport_sampler: Option<wgpu::Sampler>,
    depth_texture: Option<wgpu::Texture>,
    depth_view: Option<wgpu::TextureView>,
    viewport_extent: [u32; 2],
    viewport_size_dirty: bool,
    meshes: HashMap<u64, MeshGpu>,
}

impl Renderer {
    pub fn new() -> Self {
        Self {
            surface: None,
            surface_config: None,
            device: None,
            queue: None,
            scene_pipeline: None,
            sky_pipeline: None,
            outline_pipeline: None,
            model_bind_group_layout: None,
            scene_buffer: None,
            scene_bind_group: None,
            viewport_texture: None,
            viewport_view: None,
            viewport_present_pipeline: None,
            viewport_present_layout: None,
            viewport_present_bind_group: None,
            viewport_sampler: None,
            depth_texture: None,
            depth_view: None,
            viewport_extent: [1, 1],
            viewport_size_dirty: true,
            meshes: HashMap::new(),
        }
    }

    pub async fn init(
        &mut self,
        window: Arc<Window>,
        display_handle: winit::event_loop::OwnedDisplayHandle,
    ) -> anyhow::Result<()> {
        let window_size = window.inner_size();
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor::new_with_display_handle(
            Box::new(display_handle),
        ));
        let surface = instance.create_surface(window)?;
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
                apply_limit_buckets: false,
            })
            .await?;
        let adapter_info = adapter.get_info();
    
        let adapter_limits = adapter.limits();
        let mut required_features = wgpu::Features::empty();
        let mut required_limits = wgpu::Limits {
            max_texture_dimension_2d: adapter_limits.max_texture_dimension_2d,
            ..wgpu::Limits::downlevel_defaults()
        };
        if adapter_limits.max_immediate_size > 0 {
            required_features |= wgpu::Features::IMMEDIATES;
            required_limits.max_immediate_size = adapter_limits.max_immediate_size;
        }
        
        let (device, queue) = adapter
            .request_device(&wgpu::DeviceDescriptor {
                label: Some("iterum-renderer"),
                required_features,
                required_limits,
                experimental_features: wgpu::ExperimentalFeatures::disabled(),
                memory_hints: wgpu::MemoryHints::MemoryUsage,
                trace: wgpu::Trace::Off,
            })
            .await
            .map_err(|error| {
                anyhow::anyhow!(
                    "WGPU device creation failed on {} ({:?}): {error}",
                    adapter_info.name,
                    adapter_info.backend
                )
            })?;
    
        let tri_vert = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("triangle.vert"),
            source: wgpu::ShaderSource::Glsl {
                shader: include_str!("../../../assets/shaders/triangle.vert").into(),
                stage: wgpu::naga::ShaderStage::Vertex,
                defines: &[],
            },
        });
        let tri_frag = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("triangle.frag"),
            source: wgpu::ShaderSource::Glsl {
                shader: include_str!("../../../assets/shaders/triangle.frag").into(),
                stage: wgpu::naga::ShaderStage::Fragment,
                defines: &[],
            },
        });
        let sky_vert = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("sky.vert"),
            source: wgpu::ShaderSource::Glsl {
                shader: include_str!("../../../assets/shaders/sky.vert").into(),
                stage: wgpu::naga::ShaderStage::Vertex,
                defines: &[],
            },
        });
        let sky_frag = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("sky.frag"),
            source: wgpu::ShaderSource::Glsl {
                shader: include_str!("../../../assets/shaders/sky.frag").into(),
                stage: wgpu::naga::ShaderStage::Fragment,
                defines: &[],
            },
        });
        let outline_vert = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("outline.vert"),
            source: wgpu::ShaderSource::Glsl {
                shader: include_str!("../../../assets/shaders/outline.vert").into(),
                stage: wgpu::naga::ShaderStage::Vertex,
                defines: &[],
            },
        });
        let outline_frag = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("outline.frag"),
            source: wgpu::ShaderSource::Glsl {
                shader: include_str!("../../../assets/shaders/outline.frag").into(),
                stage: wgpu::naga::ShaderStage::Fragment,
                defines: &[],
            },
        });
        let viewport_present_vert = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("viewport.vert"),
            source: wgpu::ShaderSource::Glsl {
                shader: include_str!("../../../assets/shaders/viewport.vert").into(),
                stage: wgpu::naga::ShaderStage::Vertex,
                defines: &[],
            },
        });
        let viewport_present_frag = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("viewport.frag"),
            source: wgpu::ShaderSource::Glsl {
                shader: include_str!("../../../assets/shaders/viewport.frag").into(),
                stage: wgpu::naga::ShaderStage::Fragment,
                defines: &[],
            },
        });

        let caps = surface.get_capabilities(&adapter);
        let format = caps
            .formats
            .iter()
            .copied()
            .find(|format| format.is_srgb())
            .unwrap_or(caps.formats[0]);
        let present_mode = if caps.present_modes.contains(&wgpu::PresentMode::Fifo) {
            wgpu::PresentMode::Fifo
        } else {
            caps.present_modes[0]
        };
        let mut surface_config = surface
            .get_default_config(
                &adapter,
                window_size.width.max(1),
                window_size.height.max(1),
            )
            .ok_or_else(|| anyhow::anyhow!("the selected adapter cannot present to this window"))?;
        surface_config.format = format;
        surface_config.present_mode = present_mode;

        let max_dim = device.limits().max_texture_dimension_2d;
        surface_config.width = surface_config.width.min(max_dim);
        surface_config.height = surface_config.height.min(max_dim);
        
        surface.configure(&device, &surface_config);

        let viewport_present_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("viewport-present-layout"),
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            sample_type: wgpu::TextureSampleType::Float { filterable: true },
                            view_dimension: wgpu::TextureViewDimension::D2,
                            multisampled: false,
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                        count: None,
                    },
                ],
            });
        let viewport_sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("viewport-present-sampler"),
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            ..Default::default()
        });
        let viewport_present_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("viewport-present-pipeline-layout"),
                bind_group_layouts: &[Some(&viewport_present_layout)],
                immediate_size: 0,
            });
        let viewport_present_pipeline =
            device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("viewport-present-pipeline"),
                layout: Some(&viewport_present_pipeline_layout),
                vertex: wgpu::VertexState {
                    module: &viewport_present_vert,
                    entry_point: Some("main"),
                    compilation_options: Default::default(),
                    buffers: &[],
                },
                primitive: Default::default(),
                depth_stencil: None,
                multisample: Default::default(),
                fragment: Some(wgpu::FragmentState {
                    module: &viewport_present_frag,
                    entry_point: Some("main"),
                    compilation_options: Default::default(),
                    targets: &[Some(wgpu::ColorTargetState {
                        format: surface_config.format,
                        blend: None,
                        write_mask: wgpu::ColorWrites::ALL,
                    })],
                }),
                multiview_mask: None,
                cache: None,
            });

        let scene_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("scene-bind-group-layout"),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
        });
        let model_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("model-bind-group-layout"),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
        });
        let scene_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("scene-uniform"),
            size: std::mem::size_of::<SceneUniform>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let scene_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("scene-bind-group"),
            layout: &scene_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: scene_buffer.as_entire_binding(),
            }],
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("scene-pipeline-layout"),
            bind_group_layouts: &[Some(&scene_layout), Some(&model_layout)],
            immediate_size: 0,
        });
        let sky_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("sky-pipeline-layout"),
            bind_group_layouts: &[Some(&scene_layout)],
            immediate_size: 0,
        });
        let scene_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("scene-pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &tri_vert,
                entry_point: Some("main"),
                compilation_options: Default::default(),
                buffers: &[Some(wgpu::VertexBufferLayout {
                    array_stride: std::mem::size_of::<Vertices>() as u64,
                    step_mode: wgpu::VertexStepMode::Vertex,
                    attributes: &VERTEX_ATTRIBUTES,
                })],
            },
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                cull_mode: Some(wgpu::Face::Back),
                ..Default::default()
            },
            depth_stencil: Some(wgpu::DepthStencilState {
                format: wgpu::TextureFormat::Depth32Float,
                depth_write_enabled: Some(true),
                depth_compare: Some(wgpu::CompareFunction::LessEqual),
                stencil: Default::default(),
                bias: Default::default(),
            }),
            multisample: Default::default(),
            fragment: Some(wgpu::FragmentState {
                module: &tri_frag,
                entry_point: Some("main"),
                compilation_options: Default::default(),
                targets: &[Some(wgpu::ColorTargetState {
                    format: wgpu::TextureFormat::Rgba8UnormSrgb,
                    blend: None,
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            multiview_mask: None,
            cache: None,
        });
        let sky_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("sky-pipeline"),
            layout: Some(&sky_layout),
            vertex: wgpu::VertexState {
                module: &sky_vert,
                entry_point: Some("main"),
                compilation_options: Default::default(),
                buffers: &[],
            },
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                ..Default::default()
            },
            depth_stencil: Some(wgpu::DepthStencilState {
                format: wgpu::TextureFormat::Depth32Float,
                depth_write_enabled: Some(false),
                depth_compare: Some(wgpu::CompareFunction::Always),
                stencil: Default::default(),
                bias: Default::default(),
            }),
            multisample: Default::default(),
            fragment: Some(wgpu::FragmentState {
                module: &sky_frag,
                entry_point: Some("main"),
                compilation_options: Default::default(),
                targets: &[Some(wgpu::ColorTargetState {
                    format: wgpu::TextureFormat::Rgba8UnormSrgb,
                    blend: None,
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            multiview_mask: None,
            cache: None,
        });
        let outline_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("outline-pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &outline_vert,
                entry_point: Some("main"),
                compilation_options: Default::default(),
                buffers: &[Some(wgpu::VertexBufferLayout {
                    array_stride: std::mem::size_of::<Vertices>() as u64,
                    step_mode: wgpu::VertexStepMode::Vertex,
                    attributes: &VERTEX_ATTRIBUTES,
                })],
            },
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                cull_mode: Some(wgpu::Face::Front),
                ..Default::default()
            },
            depth_stencil: Some(wgpu::DepthStencilState {
                format: wgpu::TextureFormat::Depth32Float,
                depth_write_enabled: Some(false),
                depth_compare: Some(wgpu::CompareFunction::LessEqual),
                stencil: Default::default(),
                bias: Default::default(),
            }),
            multisample: Default::default(),
            fragment: Some(wgpu::FragmentState {
                module: &outline_frag,
                entry_point: Some("main"),
                compilation_options: Default::default(),
                targets: &[Some(wgpu::ColorTargetState {
                    format: wgpu::TextureFormat::Rgba8UnormSrgb,
                    blend: None,
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            multiview_mask: None,
            cache: None,
        });

        self.surface = Some(surface);
        self.surface_config = Some(surface_config);
        self.device = Some(device);
        self.queue = Some(queue);
        self.scene_pipeline = Some(scene_pipeline);
        self.sky_pipeline = Some(sky_pipeline);
        self.outline_pipeline = Some(outline_pipeline);
        self.model_bind_group_layout = Some(model_layout);
        self.scene_buffer = Some(scene_buffer);
        self.scene_bind_group = Some(scene_bind_group);
        self.viewport_present_pipeline = Some(viewport_present_pipeline);
        self.viewport_present_layout = Some(viewport_present_layout);
        self.viewport_sampler = Some(viewport_sampler);
        self.recreate_viewport_targets();
        Ok(())
    }

    pub fn device(&self) -> &wgpu::Device {
        self.device.as_ref().unwrap()
    }

    pub fn queue(&self) -> &wgpu::Queue {
        self.queue.as_ref().unwrap()
    }

    pub fn surface(&self) -> &wgpu::Surface<'static> {
        self.surface.as_ref().unwrap()
    }

    pub fn surface_config(&self) -> &wgpu::SurfaceConfiguration {
        self.surface_config.as_ref().unwrap()
    }

    pub fn viewport_view(&self) -> &wgpu::TextureView {
        self.viewport_view.as_ref().unwrap()
    }

    pub fn present_viewport(&self, frame: wgpu::SurfaceTexture) {
        let device = self.device.as_ref().unwrap();
        let queue = self.queue.as_ref().unwrap();
        let view = frame.texture.create_view(&Default::default());
        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("viewport-present-encoder"),
        });
        {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("viewport-present-pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    depth_slice: None,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
                multiview_mask: None,
            });
            pass.set_pipeline(self.viewport_present_pipeline.as_ref().unwrap());
            pass.set_bind_group(0, self.viewport_present_bind_group.as_ref().unwrap(), &[]);
            pass.draw(0..3, 0..1);
        }
        queue.submit([encoder.finish()]);
        queue.present(frame);
    }

    pub fn viewport_tex_id(&self) -> imgui::TextureId {
        imgui::TextureId::new(1)
    }

    pub fn set_viewport_size(&mut self, [width, height]: [u32; 2]) {
        let extent = [width.max(1), height.max(1)];
        if extent != self.viewport_extent {
            self.viewport_extent = extent;
            self.viewport_size_dirty = true;
        }
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        if width == 0 || height == 0 {
            return;
        }
        let max_dim = self.device.as_ref().unwrap().limits().max_texture_dimension_2d;
        if let Some(config) = self.surface_config.as_mut() {
            config.width = width.min(max_dim);
            config.height = height.min(max_dim);
        }
        self.surface.as_ref().unwrap().configure(
            self.device.as_ref().unwrap(),
            self.surface_config.as_ref().unwrap(),
        );
    }

    pub fn request_swapchain_recreation(&mut self) {
        self.viewport_size_dirty = true;
    }

    pub fn render_scene(&mut self, engine: &mut Engine, selected_entity: Option<Entity>) {
        if self.viewport_size_dirty {
            self.recreate_viewport_targets();
        }
        let device = self.device.as_ref().unwrap().clone();
        let queue = self.queue.as_ref().unwrap().clone();
        let camera = &engine.camera;
        let aspect = self.viewport_extent[0] as f32 / self.viewport_extent[1] as f32;
        let projection = glam::camera::rh::proj::directx::perspective(
            camera.fov.to_radians(),
            aspect,
            camera.near,
            camera.far,
        );
        let forward = camera.rotation * Vec3::NEG_Z;
        let up = camera.rotation * Vec3::Y;
        let view = glam::camera::rh::view::look_at_mat4(Vec3::ZERO, forward, up);
        let view_proj = projection * view;
        let frustum = Frustum::from_view_proj(view_proj);
        let inv_view_proj = view_proj.inverse();
        let mut lights = [GpuLight::zeroed(); MAX_LIGHTS];
        let mut light_count = 0;
        for (transform, star) in engine
            .world
            .query::<(&Transform, &Star)>()
            .iter(&engine.world)
        {
            if light_count == MAX_LIGHTS {
                break;
            }
            let position = transform.position.camera_relative_f32(camera.position);
            lights[light_count] = GpuLight {
                position: [position.x, position.y, position.z, 0.0],
                color_power: [star.color.x, star.color.y, star.color.z, star.power],
            };
            light_count += 1;
        }
        let (sky_top, sky_bottom) = engine
            .world
            .query::<&GlobalSky>()
            .iter(&engine.world)
            .next()
            .map(|sky| (sky.sky_top, sky.sky_bottom))
            .unwrap_or((Vec3::new(0.05, 0.15, 0.35), Vec3::new(0.4, 0.6, 0.9)));
        let space = engine
            .world
            .query::<&GlobalSpace>()
            .iter(&engine.world)
            .next()
            .map(|space| [space.density, space.brightness, space.seed, camera.far])
            .unwrap_or([0.0, 0.0, 0.0, camera.far]);
        let uniform = SceneUniform {
            view_proj: view_proj.to_cols_array_2d(),
            inv_view_proj: inv_view_proj.to_cols_array_2d(),
            viewport: [
                self.viewport_extent[0] as f32,
                self.viewport_extent[1] as f32,
                0.0,
                0.0,
            ],
            sky_top: [sky_top.x, sky_top.y, sky_top.z, 0.0],
            sky_bottom: [sky_bottom.x, sky_bottom.y, sky_bottom.z, 0.0],
            light_count: [light_count as u32, 0, 0, 0],
            lights,
            space,
        };
        queue.write_buffer(
            self.scene_buffer.as_ref().unwrap(),
            0,
            bytemuck::bytes_of(&uniform),
        );
        let mut resident = HashMap::<u64, &Mesh>::new();
        for (transform, mesh, physics) in engine
            .world
            .query::<(&Transform, &Mesh, Option<&PhysicsBody>)>()
            .iter(&engine.world)
        {
            let relative = transform.position.camera_relative_f32(camera.position);
            if !relative.is_finite() {
                continue;
            }
            let (load, unload) = if physics.is_some() {
                (PHYSICS_MESH_LOAD_DISTANCE, PHYSICS_MESH_UNLOAD_DISTANCE)
            } else {
                (STATIC_MESH_LOAD_DISTANCE, STATIC_MESH_UNLOAD_DISTANCE)
            };
            let limit = if self.meshes.contains_key(&mesh.id) {
                unload
            } else {
                load
            };
            if relative.length() <= limit {
                resident.entry(mesh.id).or_insert(mesh);
            }
        }
        self.meshes.retain(|id, _| resident.contains_key(id));
        for (&id, mesh) in &resident {
            if !self.meshes.contains_key(&id) {
                self.upload_mesh(mesh);
            }
        }
        drop(resident);

        let mut draw_items = Vec::new();
        for (entity, transform, mesh, star) in engine
            .world
            .query::<(Entity, &Transform, &Mesh, Option<&Star>)>()
            .iter(&engine.world)
        {
            let relative = transform.position.camera_relative_f32(camera.position);
            if !relative.is_finite()
                || !transform.rotation.is_finite()
                || !transform.scale.is_finite()
            {
                continue;
            }
            let distance = relative.length();
            let center = relative + transform.rotation * (mesh.bounds.center * transform.scale);
            let radius = mesh.bounds.radius * transform.scale.abs().max_element();
            if distance > STATIC_MESH_UNLOAD_DISTANCE || !frustum.sphere_visible(center, radius)
            {
                continue;
            }
            let Some(gpu_mesh) = self.meshes.get(&mesh.id) else {
                continue;
            };
            if gpu_mesh.lods.is_empty() {
                continue;
            }
            let projected =
                radius.max(0.001) / (distance.max(0.001) * (camera.fov.to_radians() * 0.5).tan());
            let lod = if projected > 0.25 {
                0
            } else if projected > 0.08 {
                1
            } else if projected > 0.02 {
                2
            } else {
                3
            }
            .min(gpu_mesh.lods.len() - 1);
            let model = Mat4::from_scale_rotation_translation(
                transform.scale,
                transform.rotation,
                relative,
            );
            let uniform = ModelUniform {
                model: model.to_cols_array_2d(),
                emissive: star
                    .map(|star| [star.color.x, star.color.y, star.color.z, 1.0])
                    .unwrap_or([0.0; 4]),
            };
            let buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("model-uniform"),
                contents: bytemuck::bytes_of(&uniform),
                usage: wgpu::BufferUsages::UNIFORM,
            });
            let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("model-bind-group"),
                layout: self.model_bind_group_layout.as_ref().unwrap(),
                entries: &[wgpu::BindGroupEntry {
                    binding: 0,
                    resource: buffer.as_entire_binding(),
                }],
            });
            draw_items.push(DrawItem {
                entity,
                mesh_id: mesh.id,
                lod,
                position: relative,
                rotation: transform.rotation,
                scale: transform.scale,
                bind_group,
            });
        }

        let color_view = self.viewport_view.as_ref().unwrap();
        let depth_view = self.depth_view.as_ref().unwrap();
        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("scene-encoder"),
        });
        {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("scene-pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: color_view,
                    depth_slice: None,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: depth_view,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(1.0),
                        store: wgpu::StoreOp::Store,
                    }),
                    stencil_ops: None,
                }),
                timestamp_writes: None,
                occlusion_query_set: None,
                multiview_mask: None,
            });
            pass.set_pipeline(self.sky_pipeline.as_ref().unwrap());
            pass.set_bind_group(0, self.scene_bind_group.as_ref().unwrap(), &[]);
            pass.draw(0..3, 0..1);
            pass.set_pipeline(self.scene_pipeline.as_ref().unwrap());
            pass.set_bind_group(0, self.scene_bind_group.as_ref().unwrap(), &[]);

            for item in &draw_items {
                let gpu_mesh = self.meshes.get(&item.mesh_id).unwrap();
                pass.set_bind_group(1, &item.bind_group, &[]);
                pass.set_vertex_buffer(0, gpu_mesh.vertex.slice(..));
                pass.set_index_buffer(gpu_mesh.index.slice(..), wgpu::IndexFormat::Uint16);
                for cluster in &gpu_mesh.lods[item.lod] {
                    let center = item.position + item.rotation * (cluster.center * item.scale);
                    let radius = cluster.radius * item.scale.abs().max_element();
                    if !frustum.sphere_visible(center, radius) {
                        continue;
                    }
                    pass.draw_indexed(
                        cluster.index_start..cluster.index_start + cluster.index_count,
                        cluster.base_vertex,
                        0..1,
                    );
                }
            }
            if let Some(item) = draw_items
                .iter()
                .find(|item| Some(item.entity) == selected_entity)
            {
                let mesh = self.meshes.get(&item.mesh_id).unwrap();
                pass.set_pipeline(self.outline_pipeline.as_ref().unwrap());
                pass.set_bind_group(0, self.scene_bind_group.as_ref().unwrap(), &[]);
                pass.set_bind_group(1, &item.bind_group, &[]);
                pass.set_vertex_buffer(0, mesh.vertex.slice(..));
                pass.set_index_buffer(mesh.index.slice(..), wgpu::IndexFormat::Uint16);
                if let Some(clusters) = mesh.lods.first() {
                    for cluster in clusters {
                        let center = item.position + item.rotation * (cluster.center * item.scale);
                        let radius = cluster.radius * item.scale.abs().max_element();
                        if !frustum.sphere_visible(center, radius) {
                            continue;
                        }
                        pass.draw_indexed(
                            cluster.index_start..cluster.index_start + cluster.index_count,
                            cluster.base_vertex,
                            0..1,
                        );
                    }
                }
            }
        }
        queue.submit([encoder.finish()]);
    }

    pub fn upload_mesh(&mut self, mesh: &Mesh) {
        let device = self.device.as_ref().unwrap().clone();
        let positions: Vec<[f32; 3]> = mesh.vertices.iter().map(|vertex| vertex.position).collect();
        let mut workspace = meshoptimizer_rs::Workspace::default();
        let mut vertices = Vec::new();
        let mut indices_u16 = Vec::new();
        let mut lods = Vec::with_capacity(mesh.lods.len() + 1);
        let index_sets = std::iter::once(mesh.indices.as_slice())
            .chain(mesh.lods.iter().map(|lod| lod.indices.as_slice()));

        for indices in index_sets {
            let mut clusters = Vec::new();
            if !indices.is_empty() {
                let meshlets = match meshoptimizer_rs::build_meshlets(
                    indices,
                    meshoptimizer_rs::Positions::from_packed(&positions),
                    meshoptimizer_rs::MeshletSettings::default(),
                    0.25,
                    &mut workspace,
                ) {
                    Ok(meshlets) => meshlets,
                    Err(error) => {
                        eprintln!("meshlet build failed for mesh {}: {error}", mesh.id);
                        return;
                    }
                };

                for meshlet in meshlets.meshlets {
                    let vertex_start = meshlet.vertex_offset as usize;
                    let vertex_end = vertex_start + meshlet.vertex_count as usize;
                    let source_vertices = &meshlets.vertices[vertex_start..vertex_end];
                    let base_vertex = vertices.len() as i32;
                    let cluster_start = vertices.len();
                    vertices.extend(
                        source_vertices
                            .iter()
                            .map(|&index| mesh.vertices[index as usize]),
                    );
                    let bounds = crate::mesh::Bounds::from_vertices(&vertices[cluster_start..]);
                    let index_start = indices_u16.len() as u32;
                    let triangle_start = meshlet.triangle_offset as usize;
                    let triangle_end = triangle_start + meshlet.triangle_count as usize * 3;
                    indices_u16.extend(
                        meshlets.triangles[triangle_start..triangle_end]
                            .iter()
                            .map(|&index| index as u16),
                    );
                    clusters.push(MeshCluster {
                        index_start,
                        index_count: meshlet.triangle_count * 3,
                        base_vertex,
                        center: bounds.center,
                        radius: bounds.radius,
                    });
                }
            }
            lods.push(clusters);
        }

        let vertex = if vertices.is_empty() {
            device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("mesh-vertices"),
                size: 4,
                usage: wgpu::BufferUsages::VERTEX,
                mapped_at_creation: false,
            })
        } else {
            device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("mesh-vertices"),
                contents: bytemuck::cast_slice(&vertices),
                usage: wgpu::BufferUsages::VERTEX,
            })
        };
        let index = if indices_u16.is_empty() {
            device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("mesh-indices"),
                size: 4,
                usage: wgpu::BufferUsages::INDEX,
                mapped_at_creation: false,
            })
        } else {
            device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("mesh-indices"),
                contents: bytemuck::cast_slice(&indices_u16),
                usage: wgpu::BufferUsages::INDEX,
            })
        };
        self.meshes.insert(mesh.id, MeshGpu { vertex, index, lods });
    }

    pub fn unload_mesh(&mut self, id: u64) {
        self.meshes.remove(&id);
    }

    fn recreate_viewport_targets(&mut self) {
        let Some(device) = self.device.as_ref() else {
            return;
        };
        let extent = wgpu::Extent3d {
            width: self.viewport_extent[0],
            height: self.viewport_extent[1],
            depth_or_array_layers: 1,
        };
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("scene-viewport"),
            size: extent,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });
        let view = texture.create_view(&Default::default());
        let depth = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("scene-depth"),
            size: extent,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Depth32Float,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            view_formats: &[],
        });
        let depth_view = depth.create_view(&Default::default());
        let present_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("viewport-present-bind-group"),
            layout: self.viewport_present_layout.as_ref().unwrap(),
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(
                        self.viewport_sampler.as_ref().unwrap(),
                    ),
                },
            ],
        });
        self.viewport_texture = Some(texture);
        self.viewport_view = Some(view);
        self.viewport_present_bind_group = Some(present_bind_group);
        self.depth_texture = Some(depth);
        self.depth_view = Some(depth_view);
        self.viewport_size_dirty = false;
    }
}

const VERTEX_ATTRIBUTES: [wgpu::VertexAttribute; 2] =
    wgpu::vertex_attr_array![0 => Float32x3, 1 => Float32x3];