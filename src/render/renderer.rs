use crate::nodes::planet::Star;
use crate::nodes::sky::{GlobalSky, GlobalSpace};
use glam::{DVec3, Mat4, Vec3};
use smallvec::smallvec;
use vulkano::image::sampler::Sampler;
use std::sync::Arc;
use vulkano::pipeline::graphics::depth_stencil::{CompareOp, DepthStencilState};
use vulkano::pipeline::graphics::vertex_input::VertexInputState;
use vulkano::swapchain::PresentMode;
use vulkano::sync::fence::{Fence, FenceCreateFlags, FenceCreateInfo};
use vulkano::sync::future::FenceSignalFuture;
use vulkano::sync::semaphore::{Semaphore, SemaphoreCreateInfo};
use vulkano::{
    VulkanLibrary,
    buffer::{Buffer, BufferContents, BufferCreateInfo, BufferUsage, Subbuffer},
    command_buffer::{
        AutoCommandBufferBuilder, CommandBufferUsage, CopyBufferInfo, RenderPassBeginInfo,
        SubpassBeginInfo, SubpassContents, SubpassEndInfo,
        allocator::{StandardCommandBufferAllocator, StandardCommandBufferAllocatorCreateInfo},
    },
    descriptor_set::{
        DescriptorSet, WriteDescriptorSet, allocator::StandardDescriptorSetAllocator,
    },
    device::{
        Device, DeviceCreateInfo, DeviceExtensions, Queue, QueueCreateInfo, QueueFlags,
        physical::{PhysicalDevice, PhysicalDeviceType},
    },
    format::Format,
    image::{Image, ImageUsage, SampleCount, view::ImageView},
    instance::{Instance, InstanceCreateFlags, InstanceCreateInfo},
    memory::allocator::{AllocationCreateInfo, MemoryTypeFilter, StandardMemoryAllocator},
    pipeline::{
        DynamicState, GraphicsPipeline, Pipeline, PipelineBindPoint, PipelineLayout,
        PipelineShaderStageCreateInfo,
        graphics::{
            GraphicsPipelineCreateInfo,
            color_blend::ColorBlendState,
            depth_stencil::DepthState,
            input_assembly::InputAssemblyState,
            multisample::MultisampleState,
            rasterization::{CullMode, FrontFace, RasterizationState},
            vertex_input::{Vertex, VertexDefinition},
            viewport::{Scissor, Viewport, ViewportState},
        },
        layout::{PipelineDescriptorSetLayoutCreateInfo, PushConstantRange},
    },
    render_pass::{Framebuffer, FramebufferCreateInfo, RenderPass},
    shader::ShaderStages,
    swapchain::{self, Surface, Swapchain, SwapchainCreateInfo, SwapchainPresentInfo},
    sync::{self, GpuFuture},
};

use crate::Engine;
use crate::console;
use crate::mesh::{Mesh, Vertices};
use crate::render::shaders;
use crate::transform::{Transform, WorldPositionExt};

const MAX_VERTICES: u64 = 16_000_000;
const MAX_INDICES: u64 = 48_000_000;
const MAX_LIGHTS: usize = 8;
const FRAMES_IN_FLIGHT: usize = 2;

#[derive(BufferContents, Copy, Clone)]
#[repr(C)]
struct GpuLight {
    position: [f32; 3],
    _pad0: f32,
    color: [f32; 3],
    power: f32,
}

struct DrawObject {
    base_vertex: u32,
    base_index: u32,
    index_count: u32,
    position: DVec3,
    rotation: glam::Quat,
    scale: Vec3,
    emissive: [f32; 4],
}

struct Frustum {
    planes: [glam::Vec4; 6],
}

impl Frustum {
    fn from_view_proj(m: Mat4, view: Mat4) -> Self {
        let m = m.transpose();

        let mut planes = [
            m.w_axis + m.x_axis,
            m.w_axis - m.x_axis,
            m.w_axis + m.y_axis,
            m.w_axis - m.y_axis,
            m.z_axis,
            m.w_axis - m.z_axis,
        ];

        let inv_view = view.transpose();
        let test_point_world = inv_view.transform_point3(glam::Vec3::new(0.0, 0.0, -1.0));

        for p in &mut planes {
            let len = p.truncate().length();
            if len > 0.0 {
                *p /= len;
            }

            if p.truncate().dot(test_point_world) + p.w < 0.0 {
                *p = -*p;
            }
        }

        Self { planes }
    }

    fn sphere_visible(&self, center: Vec3, radius: f32) -> bool {
        self.planes
            .iter()
            .all(|p| p.truncate().dot(center) + p.w >= -(radius + 0.1))
    }
}

pub fn logical_extent(window: &winit::window::Window) -> [u32; 2] {
    let s = window
        .inner_size()
        .to_logical::<f64>(window.scale_factor());
    [(s.width.round() as u32).max(1), (s.height.round() as u32).max(1)]
}

pub struct Renderer {
    device: Option<Arc<Device>>,
    queue: Option<Arc<Queue>>,
    surface: Option<Arc<Surface>>,
    swapchain: Option<Arc<Swapchain>>,
    instance: Option<Arc<Instance>>,
    memory_allocator: Option<Arc<StandardMemoryAllocator>>,
    render_pass: Option<Arc<RenderPass>>,
    framebuffers: Vec<Arc<Framebuffer>>,
    command_buffer_allocator: Option<Arc<StandardCommandBufferAllocator>>,
    graphics_pipeline: Option<Arc<GraphicsPipeline>>,
    elapsed: f32,
    recreate_swapchain: bool,
    startup_nudge: bool,
    pending_msaa_samples: Option<SampleCount>,
    descriptor_set_allocator: Option<Arc<StandardDescriptorSetAllocator>>,
    sky_pipeline: Option<Arc<GraphicsPipeline>>,
    vertex_buffer: Option<Subbuffer<[Vertices]>>,
    index_buffer: Option<Subbuffer<[u32]>>,
    staging_buffers: Vec<Subbuffer<[Vertices]>>,
    index_staging_buffers: Vec<Subbuffer<[u32]>>,
    mesh_allocs: std::collections::HashMap<u64, (u32, u32)>,
    index_allocs: std::collections::HashMap<u64, (u32, u32)>,
    next_free_vertex: u32,
    next_free_index: u32,
    scene_uniforms: Vec<Subbuffer<SceneUniform>>,
    scene_descriptors: Vec<Arc<DescriptorSet>>,
    fences: Vec<Arc<Fence>>,
    image_available: Vec<Arc<Semaphore>>,
    render_finished: Vec<Arc<Semaphore>>,
    frame_index: usize,
    draw_list: Vec<DrawObject>,
    in_flight: Vec<Option<FenceSignalFuture<Box<dyn GpuFuture>>>>,
    ui_pipeline: Option<Arc<GraphicsPipeline>>,
    ui_font_set: Option<Arc<DescriptorSet>>,
    swapchain_format: Format,
    viewport_color_view: Option<Arc<ImageView>>,
    viewport_depth_view: Option<Arc<ImageView>>,
    viewport_framebuffer: Option<Arc<Framebuffer>>,
    viewport_extent: [u32; 2],
    viewport_size_dirty: bool,
    viewport_sampler: Option<Arc<Sampler>>,
    viewport_descriptor: Option<Arc<DescriptorSet>>,
    viewport_tex_id: imgui::TextureId,
}

#[derive(BufferContents, Copy, Clone)]
#[repr(C)]
struct ModelPush {
    model: [[f32; 4]; 4],
    emissive: [f32; 4],
}

#[derive(BufferContents, Copy, Clone)]
#[repr(C)]
struct SkyPush {
    sun_direction: [f32; 3],
    sun_intensity: f32,
    sun_color: [f32; 3],
    _pad0: f32,
    sky_top: [f32; 3],
    _pad1: f32,
    sky_bottom: [f32; 3],
    _pad2: f32,
    _pad3: [f32; 4],
    star_density: f32,
    star_brightness: f32,
    star_seed: f32,
    _pad4: f32,
}

#[derive(BufferContents, Copy, Clone)]
#[repr(C)]
struct SceneUniform {
    view: [[f32; 4]; 4],
    projection: [[f32; 4]; 4],
    inv_view_proj: [[f32; 4]; 4],
    camera_position: [f32; 3],
    _pad0: f32,
    light_count: u32,
    _pad1: [u32; 3],
    lights: [GpuLight; MAX_LIGHTS],
    sky_top: [f32; 3],
    _pad2: f32,
    sky_bottom: [f32; 3],
    _pad3: f32,
}

#[derive(BufferContents, Vertex, Clone, Copy)]
#[repr(C)]
struct UiVertex {
    #[format(R32G32_SFLOAT)] pos: [f32; 2],
    #[format(R32G32_SFLOAT)] uv: [f32; 2],
    #[format(R8G8B8A8_UNORM)] col: [u8; 4],
}

impl Renderer {
    pub fn new() -> Self {
        Self {
            device: None,
            queue: None,
            surface: None,
            swapchain: None,
            instance: None,
            memory_allocator: None,
            render_pass: None,
            framebuffers: Vec::new(),
            command_buffer_allocator: None,
            graphics_pipeline: None,
            elapsed: 0.0,
            recreate_swapchain: false,
            startup_nudge: true,
            pending_msaa_samples: None,
            descriptor_set_allocator: None,
            sky_pipeline: None,
            vertex_buffer: None,
            index_buffer: None,
            staging_buffers: Vec::new(),
            index_staging_buffers: Vec::new(),
            mesh_allocs: std::collections::HashMap::new(),
            index_allocs: std::collections::HashMap::new(),
            next_free_vertex: 0,
            next_free_index: 0,
            scene_uniforms: Vec::new(),
            scene_descriptors: Vec::new(),
            fences: Vec::new(),
            image_available: Vec::new(),
            render_finished: Vec::new(),
            frame_index: 0,
            draw_list: Vec::new(),
            in_flight: (0..FRAMES_IN_FLIGHT).map(|_| None).collect(),
            ui_pipeline: None,
            ui_font_set: None,
            swapchain_format: Format::B8G8R8A8_SRGB,
            viewport_color_view: None,
            viewport_depth_view: None,
            viewport_framebuffer: None,
            viewport_extent: [1280, 720],
            viewport_size_dirty: false,
            viewport_sampler: None,
            viewport_descriptor: None,
            viewport_tex_id: imgui::TextureId::new(1),
        }
    }

    fn select_physical_device(
        instance: &Arc<Instance>,
        surface: &Arc<Surface>,
        device_extensions: &DeviceExtensions,
    ) -> (Arc<PhysicalDevice>, u32) {
        instance
            .enumerate_physical_devices()
            .expect("could not enumerate devices")
            .filter(|p| p.supported_extensions().contains(&device_extensions))
            .filter_map(|p| {
                p.queue_family_properties()
                    .iter()
                    .enumerate()
                    .position(|(i, q)| {
                        q.queue_flags.contains(QueueFlags::GRAPHICS)
                            && p.surface_support(i as u32, &surface).unwrap_or(false)
                    })
                    .map(|q| (p, q as u32))
            })
            .min_by_key(|(p, _)| match p.properties().device_type {
                PhysicalDeviceType::DiscreteGpu => 0,
                PhysicalDeviceType::IntegratedGpu => 1,
                PhysicalDeviceType::VirtualGpu => 2,
                PhysicalDeviceType::Cpu => 3,
                _ => 4,
            })
            .expect("no device available")
    }

    fn create_viewport_targets(
        &self,
        format: Format,
        extent: [u32; 2],
    ) -> (Arc<ImageView>, Arc<ImageView>, Arc<Framebuffer>) {
        let ma = self.memory_allocator.as_ref().unwrap().clone();
        let color_image = Image::new(
            ma.clone(),
            vulkano::image::ImageCreateInfo {
                format,
                extent: [extent[0].max(1), extent[1].max(1), 1],
                usage: ImageUsage::COLOR_ATTACHMENT | ImageUsage::SAMPLED,
                ..Default::default()
            },
            AllocationCreateInfo::default(),
        )
        .unwrap();
        let depth_image = Image::new(
            ma,
            vulkano::image::ImageCreateInfo {
                format: Format::D32_SFLOAT,
                extent: [extent[0].max(1), extent[1].max(1), 1],
                usage: ImageUsage::DEPTH_STENCIL_ATTACHMENT,
                ..Default::default()
            },
            AllocationCreateInfo::default(),
        )
        .unwrap();
        let color_view = ImageView::new_default(color_image).unwrap();
        let depth_view = ImageView::new_default(depth_image).unwrap();
        let fb = Framebuffer::new(
            self.render_pass.as_ref().unwrap().clone(),
            FramebufferCreateInfo {
                attachments: vec![color_view.clone(), depth_view.clone()],
                ..Default::default()
            },
        )
        .unwrap();
        (color_view, depth_view, fb)
    }

    pub fn init(&mut self, window: &winit::window::Window) {
        #[cfg(target_os = "macos")]
        unsafe {
            std::env::remove_var("VK_ICD_FILENAMES");
        }

        let logger = console::Console::new();

        let library = VulkanLibrary::new().expect("no local Vulkan library/DLL");

        let required_extensions = Surface::required_extensions(window)
            .expect("Failed to get the required Vulkan extensions");

        let instance = Instance::new(
            library,
            InstanceCreateInfo {
                flags: InstanceCreateFlags::ENUMERATE_PORTABILITY,
                enabled_extensions: required_extensions,
                ..Default::default()
            },
        )
        .expect("failed to create instance");

        let surface = unsafe { Surface::from_window_ref(instance.clone(), window) }
            .expect("Failed to create Vulkan surface");

        let device_extensions = DeviceExtensions {
            khr_swapchain: true,
            ..DeviceExtensions::empty()
        };

        let (physical_device, queue_family_index) =
            Self::select_physical_device(&instance, &surface, &device_extensions);

        let (device, mut queues) = Device::new(
            physical_device.clone(),
            DeviceCreateInfo {
                queue_create_infos: vec![QueueCreateInfo {
                    queue_family_index,
                    ..Default::default()
                }],
                enabled_extensions: device_extensions,
                ..Default::default()
            },
        )
        .expect("Failed to create device");

        let queue = queues.next().unwrap();

        let vertex_shader =
            shaders::vertex::load(device.clone()).expect("failed to load vertext shader");
        let fragment_shader =
            shaders::fragment::load(device.clone()).expect("failed to load fragment shader");

        let vertex_entry = vertex_shader.entry_point("main").unwrap();
        let fragment_entry = fragment_shader.entry_point("main").unwrap();

        let memory_allocator = Arc::new(StandardMemoryAllocator::new_default(device.clone()));

        let command_buffer_allocator = Arc::new(StandardCommandBufferAllocator::new(
            device.clone(),
            StandardCommandBufferAllocatorCreateInfo::default(),
        ));

        let vertex_buffer = Buffer::new_slice::<Vertices>(
            memory_allocator.clone(),
            BufferCreateInfo {
                usage: BufferUsage::VERTEX_BUFFER | BufferUsage::TRANSFER_DST,
                ..Default::default()
            },
            AllocationCreateInfo {
                memory_type_filter: MemoryTypeFilter::PREFER_DEVICE,
                ..Default::default()
            },
            MAX_VERTICES,
        )
        .expect("failed to create vertex buffer");

        let index_buffer = Buffer::new_slice::<u32>(
            memory_allocator.clone(),
            BufferCreateInfo {
                usage: BufferUsage::INDEX_BUFFER | BufferUsage::TRANSFER_DST,
                ..Default::default()
            },
            AllocationCreateInfo {
                memory_type_filter: MemoryTypeFilter::PREFER_DEVICE,
                ..Default::default()
            },
            MAX_INDICES,
        )
        .expect("failed to create index buffer");

        let staging_buffers: Vec<_> = (0..FRAMES_IN_FLIGHT)
            .map(|_| {
                Buffer::new_slice::<Vertices>(
                    memory_allocator.clone(),
                    BufferCreateInfo {
                        usage: BufferUsage::TRANSFER_SRC,
                        ..Default::default()
                    },
                    AllocationCreateInfo {
                        memory_type_filter: MemoryTypeFilter::PREFER_HOST
                            | MemoryTypeFilter::HOST_SEQUENTIAL_WRITE,
                        ..Default::default()
                    },
                    MAX_VERTICES,
                )
                .unwrap()
            })
            .collect();

        let index_staging_buffers: Vec<_> = (0..FRAMES_IN_FLIGHT)
            .map(|_| {
                Buffer::new_slice::<u32>(
                    memory_allocator.clone(),
                    BufferCreateInfo {
                        usage: BufferUsage::TRANSFER_SRC,
                        ..Default::default()
                    },
                    AllocationCreateInfo {
                        memory_type_filter: MemoryTypeFilter::PREFER_HOST
                            | MemoryTypeFilter::HOST_SEQUENTIAL_WRITE,
                        ..Default::default()
                    },
                    MAX_INDICES,
                )
                .unwrap()
            })
            .collect();

        logger.log_system("Surface initialisation succeeded.");

        let caps = physical_device
            .surface_capabilities(&surface, Default::default())
            .expect("failed to get surface capabilities");

        let dimensions = logical_extent(window);

        let composite_alpha = caps
            .supported_composite_alpha
            .into_iter()
            .next()
            .expect("surface has no supported composite alpha mode");

        let image_format = physical_device
            .surface_formats(&surface, Default::default())
            .expect("failed to get surface formats")
            .into_iter()
            .next()
            .expect("surface has no supported formats")
            .0;

        let render_pass = vulkano::ordered_passes_renderpass!(
            device.clone(),
            attachments: {
                swapchain_color: {
                    format: image_format,
                    samples: 1,
                    load_op: Clear,
                    store_op: Store,
                },
                depth: {
                    format: Format::D32_SFLOAT,
                    samples: 1,
                    load_op: Clear,
                    store_op: DontCare,
                }
            },
            passes: [
                {
                    color: [swapchain_color],
                    depth_stencil: {depth},
                    input: []
                },
                {
                    color: [swapchain_color],
                    depth_stencil: {},
                    input: []
                }
            ]
        )
        .unwrap();

        let vertex_stage = PipelineShaderStageCreateInfo::new(vertex_entry.clone());
        let fragment_stage = PipelineShaderStageCreateInfo::new(fragment_entry.clone());

        let stages = [vertex_stage, fragment_stage];

        let mut layout_info = PipelineDescriptorSetLayoutCreateInfo::from_stages(stages.iter())
            .into_pipeline_layout_create_info(device.clone())
            .unwrap();

        layout_info.push_constant_ranges = vec![PushConstantRange {
            stages: ShaderStages::VERTEX | ShaderStages::FRAGMENT,
            offset: 0,
            size: 96,
        }];

        let layout = PipelineLayout::new(device.clone(), layout_info).unwrap();

        let mut pipeline_info = GraphicsPipelineCreateInfo::layout(layout.clone());

        pipeline_info.stages = stages.into_iter().collect();

        pipeline_info.vertex_input_state =
            Some(Vertices::per_vertex().definition(&vertex_entry).unwrap());

        pipeline_info.input_assembly_state = Some(InputAssemblyState::default());
        pipeline_info.viewport_state = Some(ViewportState::default());
        pipeline_info.dynamic_state.insert(DynamicState::Viewport);
        pipeline_info.rasterization_state = Some(RasterizationState {
            cull_mode: CullMode::Back,
            front_face: FrontFace::CounterClockwise,
            ..Default::default()
        });
        pipeline_info.multisample_state = Some(MultisampleState::default());

        pipeline_info.depth_stencil_state = Some(
            vulkano::pipeline::graphics::depth_stencil::DepthStencilState {
                depth: Some(DepthState::simple()),
                ..Default::default()
            },
        );

        pipeline_info.color_blend_state = Some(ColorBlendState::with_attachment_states(
            1,
            Default::default(),
        ));

        pipeline_info.subpass = Some(render_pass.clone().first_subpass().into());

        let graphics_pipeline = GraphicsPipeline::new(device.clone(), None, pipeline_info)
            .expect("failed to create graphics pipeline");

        let sky_vertex_shader = shaders::sky_vertex::load(device.clone()).unwrap();
        let sky_fragment_shader = shaders::sky_fragment::load(device.clone()).unwrap();
        let sky_vert_entry = sky_vertex_shader.entry_point("main").unwrap();
        let sky_frag_entry = sky_fragment_shader.entry_point("main").unwrap();

        let sky_stages = [
            PipelineShaderStageCreateInfo::new(sky_vert_entry),
            PipelineShaderStageCreateInfo::new(sky_frag_entry),
        ];

        let mut sky_info = GraphicsPipelineCreateInfo::layout(layout);
        sky_info.stages = sky_stages.into_iter().collect();
        sky_info.vertex_input_state = Some(VertexInputState::default());
        sky_info.input_assembly_state = Some(InputAssemblyState::default());
        sky_info.viewport_state = Some(ViewportState::default());
        sky_info.dynamic_state.insert(DynamicState::Viewport);
        sky_info.rasterization_state = Some(RasterizationState {
            cull_mode: CullMode::None,
            front_face: FrontFace::CounterClockwise,
            ..Default::default()
        });
        sky_info.multisample_state = Some(MultisampleState::default());
        sky_info.depth_stencil_state = Some(DepthStencilState {
            depth: Some(DepthState {
                write_enable: false,
                compare_op: CompareOp::Always,
                ..Default::default()
            }),
            ..Default::default()
        });
        sky_info.color_blend_state = Some(ColorBlendState::with_attachment_states(
            1,
            Default::default(),
        ));
        sky_info.subpass = Some(render_pass.clone().first_subpass().into());

        let sky_pipeline = GraphicsPipeline::new(device.clone(), None, sky_info)
            .expect("failed to create sky pipeline");

        let (swapchain, images) = Swapchain::new(
            device.clone(),
            surface.clone(),
            SwapchainCreateInfo {
                min_image_count: caps.min_image_count + 1,
                image_format,
                image_extent: dimensions.into(),
                image_usage: ImageUsage::COLOR_ATTACHMENT | ImageUsage::TRANSFER_DST,
                composite_alpha,
                present_mode: PresentMode::Immediate,
                ..Default::default()
            },
        )
        .expect("failed to create swapchain");

        let image_views = images
            .iter()
            .map(|image| ImageView::new_default(image.clone()).unwrap())
            .collect::<Vec<_>>();

        let depth_views = images
            .iter()
            .map(|_| {
                let depth_image = Image::new(
                    memory_allocator.clone(),
                    vulkano::image::ImageCreateInfo {
                        format: Format::D32_SFLOAT,
                        extent: [dimensions[0], dimensions[1], 1],
                        usage: ImageUsage::DEPTH_STENCIL_ATTACHMENT,
                        ..Default::default()
                    },
                    AllocationCreateInfo::default(),
                )
                .unwrap();

                ImageView::new_default(depth_image).unwrap()
            })
            .collect::<Vec<_>>();

        let framebuffers = image_views
            .iter()
            .zip(depth_views.iter())
            .map(|(swapchain_view, depth_view)| {
                Framebuffer::new(
                    render_pass.clone(),
                    FramebufferCreateInfo {
                        attachments: vec![swapchain_view.clone(), depth_view.clone()],
                        ..Default::default()
                    },
                )
                .unwrap()
            })
            .collect::<Vec<_>>();

        logger.log_system(format_args!("Swapchain images: {}", images.len()));
        logger.break_line();

        let descriptor_set_allocator = Arc::new(StandardDescriptorSetAllocator::new(
            device.clone(),
            Default::default(),
        ));

        let scene_uniforms: Vec<_> = (0..FRAMES_IN_FLIGHT)
            .map(|_| {
                Buffer::from_data(
                    memory_allocator.clone(),
                    BufferCreateInfo {
                        usage: BufferUsage::UNIFORM_BUFFER,
                        ..Default::default()
                    },
                    AllocationCreateInfo {
                        memory_type_filter: MemoryTypeFilter::PREFER_HOST
                            | MemoryTypeFilter::HOST_SEQUENTIAL_WRITE,
                        ..Default::default()
                    },
                    SceneUniform {
                        view: [[0.0; 4]; 4],
                        projection: [[0.0; 4]; 4],
                        inv_view_proj: [[0.0; 4]; 4],
                        camera_position: [0.0; 3],
                        _pad0: 0.0,
                        light_count: 0,
                        _pad1: [0; 3],
                        lights: [GpuLight {
                            position: [0.0; 3],
                            _pad0: 0.0,
                            color: [0.0; 3],
                            power: 0.0,
                        }; MAX_LIGHTS],
                        sky_top: [0.0; 3],
                        _pad2: 0.0,
                        sky_bottom: [0.0; 3],
                        _pad3: 0.0,
                    },
                )
                .unwrap()
            })
            .collect();

        let scene_descriptors: Vec<_> = scene_uniforms
            .iter()
            .map(|buf| {
                DescriptorSet::new(
                    descriptor_set_allocator.clone(),
                    graphics_pipeline.layout().set_layouts()[0].clone(),
                    [WriteDescriptorSet::buffer(0, buf.clone())],
                    [],
                )
                .unwrap()
            })
            .collect();

        let fences: Vec<_> = (0..FRAMES_IN_FLIGHT)
            .map(|_| {
                Arc::new(
                    Fence::new(
                        device.clone(),
                        FenceCreateInfo {
                            flags: FenceCreateFlags::SIGNALED,
                            ..Default::default()
                        },
                    )
                    .unwrap(),
                )
            })
            .collect();

        let image_available: Vec<_> = (0..FRAMES_IN_FLIGHT)
            .map(|_| {
                Arc::new(Semaphore::new(device.clone(), SemaphoreCreateInfo::default()).unwrap())
            })
            .collect();

        let render_finished: Vec<_> = (0..FRAMES_IN_FLIGHT)
            .map(|_| {
                Arc::new(Semaphore::new(device.clone(), SemaphoreCreateInfo::default()).unwrap())
            })
            .collect();

        self.instance = Some(instance);
        self.device = Some(device);
        self.queue = Some(queue);
        self.surface = Some(surface);
        self.swapchain = Some(swapchain);
        self.memory_allocator = Some(memory_allocator);
        self.render_pass = Some(render_pass);
        self.framebuffers = framebuffers;
        self.command_buffer_allocator = Some(command_buffer_allocator);
        self.graphics_pipeline = Some(graphics_pipeline);
        self.vertex_buffer = Some(vertex_buffer);
        self.index_buffer = Some(index_buffer);
        self.descriptor_set_allocator = Some(descriptor_set_allocator);
        self.staging_buffers = staging_buffers;
        self.index_staging_buffers = index_staging_buffers;
        self.scene_uniforms = scene_uniforms;
        self.scene_descriptors = scene_descriptors;
        self.fences = fences;
        self.image_available = image_available;
        self.render_finished = render_finished;
        self.sky_pipeline = Some(sky_pipeline);
        self.swapchain_format = image_format;

        let (color_view, depth_view, fb) =
            self.create_viewport_targets(image_format, self.viewport_extent);
        self.viewport_color_view = Some(color_view);
        self.viewport_depth_view = Some(depth_view);
        self.viewport_framebuffer = Some(fb);
    }

    pub fn upload_mesh(&mut self, mesh: &Mesh) {
        if self.mesh_allocs.contains_key(&mesh.id) && self.index_allocs.contains_key(&mesh.id) {
            return;
        }

        let vertex_count = mesh.vertices.len() as u32;
        let index_count = mesh.indices.len() as u32;

        if vertex_count == 0 || index_count == 0 {
            self.mesh_allocs.insert(mesh.id, (0, 0));
            self.index_allocs.insert(mesh.id, (0, 0));
            return;
        }

        let base_vertex = self.next_free_vertex;
        let base_index = self.next_free_index;

        assert!(
            base_vertex as u64 + vertex_count as u64 <= MAX_VERTICES,
            "vertex pool overflow: {} + {} > {}",
            base_vertex,
            vertex_count,
            MAX_VERTICES
        );
        assert!(
            base_index as u64 + index_count as u64 <= MAX_INDICES,
            "index pool overflow: {} + {} > {}",
            base_index,
            index_count,
            MAX_INDICES
        );

        {
            let mut w = self.staging_buffers[0].write().unwrap();
            w[..vertex_count as usize].copy_from_slice(&mesh.vertices);
        }
        {
            let mut w = self.index_staging_buffers[0].write().unwrap();
            w[..index_count as usize].copy_from_slice(&mesh.indices);
        }

        let mut b = AutoCommandBufferBuilder::primary(
            self.command_buffer_allocator.as_ref().unwrap().clone(),
            self.queue.as_ref().unwrap().queue_family_index(),
            CommandBufferUsage::OneTimeSubmit,
        )
        .unwrap();

        b.copy_buffer(CopyBufferInfo::buffers(
            self.staging_buffers[0].clone().slice(0..vertex_count as u64),
            self.vertex_buffer
                .as_ref()
                .unwrap()
                .clone()
                .slice(base_vertex as u64..(base_vertex + vertex_count) as u64),
        ))
        .unwrap();

        b.copy_buffer(CopyBufferInfo::buffers(
            self.index_staging_buffers[0]
                .clone()
                .slice(0..index_count as u64),
            self.index_buffer
                .as_ref()
                .unwrap()
                .clone()
                .slice(base_index as u64..(base_index + index_count) as u64),
        ))
        .unwrap();

        let cb = b.build().unwrap();
        sync::now(self.device.as_ref().unwrap().clone())
            .then_execute(self.queue.as_ref().unwrap().clone(), cb)
            .unwrap()
            .then_signal_fence_and_flush()
            .unwrap()
            .wait(None)
            .unwrap();

        self.mesh_allocs.insert(mesh.id, (base_vertex, vertex_count));
        self.index_allocs.insert(mesh.id, (base_index, index_count));
        self.next_free_vertex += vertex_count;
        self.next_free_index += index_count;
    }

    fn recreate_viewport_targets(&mut self) {
        let (color_view, depth_view, fb) =
            self.create_viewport_targets(self.swapchain_format, self.viewport_extent);
        self.viewport_color_view = Some(color_view);
        self.viewport_depth_view = Some(depth_view);
        self.viewport_framebuffer = Some(fb);

        if let (Some(pipe), Some(sampler)) = (self.ui_pipeline.as_ref(), self.viewport_sampler.as_ref()) {
            let set = DescriptorSet::new(
                self.descriptor_set_allocator.clone().unwrap(),
                pipe.layout().set_layouts()[0].clone(),
                [WriteDescriptorSet::image_view_sampler(
                    0,
                    self.viewport_color_view.clone().unwrap(),
                    sampler.clone(),
                )],
                [],
            )
            .unwrap();
            self.viewport_descriptor = Some(set);
        }
    }

    pub fn set_viewport_size(&mut self, [w, h]: [u32; 2]) {
        let w = w.max(1);
        let h = h.max(1);
        if [w, h] != self.viewport_extent {
            self.viewport_extent = [w, h];
            self.viewport_size_dirty = true;
        }
    }

    pub fn viewport_tex_id(&self) -> imgui::TextureId {
        self.viewport_tex_id
    }

    pub fn render(
        &mut self,
        window: &winit::window::Window,
        engine: &mut Engine,
        ui: Option<&imgui::DrawData>,
    ) {
        if self.recreate_swapchain {
            let mut dimensions = logical_extent(window);
            if dimensions[0] == 0 || dimensions[1] == 0 {
                return;
            }

            let nudged = self.startup_nudge;
            if nudged {
                self.startup_nudge = false;
                dimensions[0] += 1;
            }

            for f in self.in_flight.iter_mut() {
                if let Some(prev) = f.take() {
                    prev.wait(None).unwrap();
                }
            }

            let old_swapchain = self.swapchain.as_ref().unwrap();
            let (new_swapchain, new_images) = old_swapchain
                .recreate(SwapchainCreateInfo {
                    image_extent: dimensions.into(),
                    ..old_swapchain.create_info()
                })
                .unwrap();

            let image_views = new_images
                .iter()
                .map(|image| ImageView::new_default(image.clone()).unwrap())
                .collect::<Vec<_>>();

            let depth_views = new_images
                .iter()
                .map(|_| {
                    let depth_image = Image::new(
                        self.memory_allocator.as_ref().unwrap().clone(),
                        vulkano::image::ImageCreateInfo {
                            format: Format::D32_SFLOAT,
                            extent: [dimensions[0], dimensions[1], 1],
                            usage: ImageUsage::DEPTH_STENCIL_ATTACHMENT,
                            ..Default::default()
                        },
                        AllocationCreateInfo::default(),
                    )
                    .unwrap();
                    ImageView::new_default(depth_image).unwrap()
                })
                .collect::<Vec<_>>();

            self.framebuffers = image_views
                .iter()
                .zip(depth_views.iter())
                .map(|(color, depth)| {
                    Framebuffer::new(
                        self.render_pass.as_ref().unwrap().clone(),
                        FramebufferCreateInfo {
                            attachments: vec![color.clone(), depth.clone()],
                            ..Default::default()
                        },
                    )
                    .unwrap()
                })
                .collect();

            self.swapchain = Some(new_swapchain);
            self.recreate_swapchain = nudged;
        }

        if self.pending_msaa_samples.take().is_some() {
            self.recreate_swapchain = true;
        }

        if self.viewport_size_dirty {
            self.recreate_viewport_targets();
            self.viewport_size_dirty = false;
        }

        let frame = self.frame_index;

        if let Some(prev) = self.in_flight[frame].take() {
            prev.wait(None).unwrap();
        }

        let command_buffer_allocator = self.command_buffer_allocator.as_ref().unwrap().clone();
        let queue = self.queue.as_ref().unwrap().clone();
        let swapchain = self.swapchain.as_ref().unwrap().clone();
        let graphics_pipeline = self.graphics_pipeline.as_ref().unwrap().clone();
        let sky_pipeline = self.sky_pipeline.as_ref().unwrap().clone();

        let camera = &engine.camera;
        let dimensions = swapchain.image_extent();

        let forward = camera.rotation * Vec3::NEG_Z;

        let swapchain_projection = glam::camera::rh::proj::vulkan::perspective(
            camera.fov.to_radians(),
            dimensions[0] as f32 / dimensions[1] as f32,
            camera.near,
            camera.far,
        );

        let viewport_projection = glam::camera::rh::proj::vulkan::perspective(
            camera.fov.to_radians(),
            self.viewport_extent[0] as f32 / self.viewport_extent[1] as f32,
            camera.near,
            camera.far,
        );

        let mut gpu_lights = [GpuLight {
            position: [0.0; 3],
            _pad0: 0.0,
            color: [0.0; 3],
            power: 0.0,
        }; MAX_LIGHTS];
        let mut light_count = 0usize;
        for (transform, star) in engine
            .world
            .query::<(&Transform, &Star)>()
            .iter(&engine.world)
        {
            if light_count >= MAX_LIGHTS {
                break;
            }
            let p = transform.position.camera_relative_f32(camera.position);
            gpu_lights[light_count] = GpuLight {
                position: [p.x as f32, p.y as f32, p.z as f32],
                _pad0: 0.0,
                color: star.color.to_array(),
                power: star.power,
            };
            light_count += 1;
        }

        let dt = engine.get_delta() as f32;
        self.elapsed += dt;

        let cam_up = camera.rotation * Vec3::Y;
        let view_rel = glam::camera::rh::view::look_at_mat4(Vec3::ZERO, forward, cam_up);

        let view_proj = swapchain_projection * view_rel;
        let frustum = Frustum::from_view_proj(view_proj, view_rel);

        for mesh in engine.world.query::<&Mesh>().iter(&engine.world) {
            if !self.mesh_allocs.contains_key(&mesh.id)
                || !self.index_allocs.contains_key(&mesh.id)
            {
                self.upload_mesh(mesh);
            }
        }

        self.draw_list.clear();

        for (transform, mesh, star) in engine
            .world
            .query::<(&Transform, &Mesh, Option<&Star>)>()
            .iter(&engine.world)
        {
            let rel = transform.position.camera_relative_f32(camera.position);
            if !rel.is_finite() || !transform.rotation.is_finite() || !transform.scale.is_finite() {
                continue;
            }

            let world_center = rel + transform.rotation * (mesh.bounds.center * transform.scale);
            let world_radius = mesh.bounds.radius * transform.scale.abs().max_element();

            if !frustum.sphere_visible(world_center, world_radius) {
                continue;
            }

            let Some(&(base_vertex, vertex_count)) = self.mesh_allocs.get(&mesh.id) else {
                continue;
            };
            let Some(&(base_index, index_count)) = self.index_allocs.get(&mesh.id) else {
                continue;
            };

            if vertex_count == 0 || index_count == 0 {
                continue;
            }

            self.draw_list.push(DrawObject {
                base_vertex,
                base_index,
                index_count,
                position: transform.position,
                rotation: transform.rotation,
                scale: transform.scale,
                emissive: star
                    .map(|s| [s.color.x, s.color.y, s.color.z, 1.0])
                    .unwrap_or([0.0; 4]),
            });
        }

        let (image_index, _suboptimal, acquire_future) =
            match swapchain::acquire_next_image(swapchain.clone(), None) {
                Ok(r) => r,
                Err(vulkano::Validated::Error(vulkano::VulkanError::OutOfDate)) => {
                    self.recreate_swapchain = true;
                    return;
                }
                Err(e) => panic!("acquire image: {e}"),
            };

        let framebuffer = self.framebuffers[image_index as usize].clone();

        let viewport = Viewport {
            offset: [0.0, 0.0],
            extent: [dimensions[0] as f32, dimensions[1] as f32],
            depth_range: 0.0..=1.0,
        };

        let (sky_top, sky_bottom) = match engine
            .world
            .query::<&GlobalSky>()
            .iter(&engine.world)
            .next()
        {
            Some(s) => (s.sky_top, s.sky_bottom),
            None => (Vec3::new(0.05, 0.15, 0.35), Vec3::new(0.4, 0.6, 0.9)),
        };

        let (star_density, star_brightness, star_seed) = match engine
            .world
            .query::<&GlobalSpace>()
            .iter(&engine.world)
            .next()
        {
            Some(s) => (s.density, s.brightness, s.seed),
            None => (0.0, 0.0, 0.0),
        };

        let sun_color = if light_count > 0 {
            gpu_lights[0].color
        } else {
            [1.0, 1.0, 1.0]
        };

        let sky_push = SkyPush {
            sun_direction: [0.0, 1.0, 1.0],
            sun_intensity: 1.0,
            sun_color,
            _pad0: 0.0,
            sky_top: sky_top.to_array(),
            _pad1: 0.0,
            sky_bottom: sky_bottom.to_array(),
            _pad2: 0.0,
            _pad3: [0.0; 4],
            star_density,
            star_brightness,
            star_seed,
            _pad4: 0.0,
        };

        let viewport_inv_vp = (viewport_projection * view_rel).inverse();

        {
            let mut w = self.scene_uniforms[frame].write().unwrap();
            *w = SceneUniform {
                view: view_rel.to_cols_array_2d(),
                projection: viewport_projection.to_cols_array_2d(),
                inv_view_proj: viewport_inv_vp.to_cols_array_2d(),
                camera_position: [0.0, 0.0, 0.0],
                _pad0: 0.0,
                light_count: light_count as u32,
                _pad1: [0; 3],
                lights: gpu_lights,
                sky_top: sky_top.to_array(),
                sky_bottom: sky_bottom.to_array(),
                _pad2: 0.0,
                _pad3: 0.0,
            };
        }

        let descriptor_set = self.scene_descriptors[frame].clone();

        let mut scene_builder = AutoCommandBufferBuilder::primary(
            command_buffer_allocator.clone(),
            queue.queue_family_index(),
            CommandBufferUsage::OneTimeSubmit,
        )
        .unwrap();

        scene_builder
            .begin_render_pass(
                RenderPassBeginInfo {
                    clear_values: vec![Some([0.02, 0.02, 0.02, 1.0].into()), Some(1.0f32.into())],
                    ..RenderPassBeginInfo::framebuffer(
                        self.viewport_framebuffer.clone().unwrap(),
                    )
                },
                SubpassBeginInfo {
                    contents: SubpassContents::Inline,
                    ..Default::default()
                },
            )
            .unwrap()
            .set_viewport(
                0,
                smallvec![Viewport {
                    offset: [0.0, 0.0],
                    extent: [
                        self.viewport_extent[0] as f32,
                        self.viewport_extent[1] as f32
                    ],
                    depth_range: 0.0..=1.0,
                }],
            )
            .unwrap();

        scene_builder
            .bind_pipeline_graphics(sky_pipeline.clone())
            .unwrap()
            .bind_descriptor_sets(
                PipelineBindPoint::Graphics,
                sky_pipeline.layout().clone(),
                0,
                descriptor_set.clone(),
            )
            .unwrap();

        unsafe {
            scene_builder
                .push_constants(sky_pipeline.layout().clone(), 0, sky_push)
                .unwrap()
                .draw(3, 1, 0, 0)
                .unwrap();
        }

        if !self.draw_list.is_empty() {
            scene_builder
                .bind_pipeline_graphics(graphics_pipeline.clone())
                .unwrap()
                .bind_vertex_buffers(0, self.vertex_buffer.as_ref().unwrap().clone())
                .unwrap()
                .bind_index_buffer(self.index_buffer.as_ref().unwrap().clone())
                .unwrap()
                .bind_descriptor_sets(
                    PipelineBindPoint::Graphics,
                    graphics_pipeline.layout().clone(),
                    0,
                    descriptor_set,
                )
                .unwrap();

            for obj in &self.draw_list {
                let relative_position = obj.position.camera_relative_f32(camera.position);
                let model = Mat4::from_scale_rotation_translation(
                    obj.scale,
                    obj.rotation,
                    relative_position,
                );
                unsafe {
                    scene_builder
                        .push_constants(
                            graphics_pipeline.layout().clone(),
                            0,
                            ModelPush {
                                model: model.to_cols_array_2d(),
                                emissive: obj.emissive,
                            },
                        )
                        .unwrap();
                    scene_builder
                        .draw_indexed(
                            obj.index_count,
                            1,
                            obj.base_index,
                            obj.base_vertex as i32,
                            0,
                        )
                        .unwrap();
                }
            }
        }

        scene_builder
            .next_subpass(
                SubpassEndInfo::default(),
                SubpassBeginInfo {
                    contents: SubpassContents::Inline,
                    ..Default::default()
                },
            )
            .unwrap();

        scene_builder
            .end_render_pass(SubpassEndInfo::default())
            .unwrap();

        let scene_cb = scene_builder.build().unwrap();

        let mut builder = AutoCommandBufferBuilder::primary(
            command_buffer_allocator,
            queue.queue_family_index(),
            CommandBufferUsage::OneTimeSubmit,
        )
        .unwrap();

        builder
            .begin_render_pass(
                RenderPassBeginInfo {
                    clear_values: vec![Some([0.02, 0.02, 0.02, 1.0].into()), Some(1.0f32.into())],
                    ..RenderPassBeginInfo::framebuffer(framebuffer)
                },
                SubpassBeginInfo {
                    contents: SubpassContents::Inline,
                    ..Default::default()
                },
            )
            .unwrap()
            .set_viewport(0, smallvec![viewport])
            .unwrap();

        builder
            .next_subpass(
                SubpassEndInfo::default(),
                SubpassBeginInfo {
                    contents: SubpassContents::Inline,
                    ..Default::default()
                },
            )
            .unwrap();

        if let Some(dd) = ui.filter(|d| d.total_vtx_count > 0) {
            let [dw, dh] = dd.display_size;
            let [ox, oy] = dd.display_pos;

            let mut verts: Vec<UiVertex> = Vec::with_capacity(dd.total_vtx_count as usize);
            let mut idxs: Vec<u16> = Vec::with_capacity(dd.total_idx_count as usize);
            for l in dd.draw_lists() {
                verts.extend(l.vtx_buffer().iter().map(|v| UiVertex {
                    pos: v.pos,
                    uv: v.uv,
                    col: v.col,
                }));
                idxs.extend_from_slice(l.idx_buffer());
            }

            let ma = self.memory_allocator.clone().unwrap();
            let host = AllocationCreateInfo {
                memory_type_filter: MemoryTypeFilter::PREFER_HOST
                    | MemoryTypeFilter::HOST_SEQUENTIAL_WRITE,
                ..Default::default()
            };
            let vb = Buffer::from_iter(
                ma.clone(),
                BufferCreateInfo {
                    usage: BufferUsage::VERTEX_BUFFER,
                    ..Default::default()
                },
                host.clone(),
                verts,
            )
            .unwrap();
            let ib = Buffer::from_iter(
                ma,
                BufferCreateInfo {
                    usage: BufferUsage::INDEX_BUFFER,
                    ..Default::default()
                },
                host,
                idxs,
            )
            .unwrap();

            let pipe = self.ui_pipeline.clone().unwrap();
            builder
                .bind_pipeline_graphics(pipe.clone())
                .unwrap()
                .set_viewport(
                    0,
                    smallvec![Viewport {
                        offset: [0.0, 0.0],
                        extent: [dw, dh],
                        depth_range: 0.0..=1.0,
                    }],
                )
                .unwrap()
                .bind_vertex_buffers(0, vb)
                .unwrap()
                .bind_index_buffer(ib)
                .unwrap();

            let s = [2.0 / dw, 2.0 / dh];
            let pc: [f32; 4] = [s[0], s[1], -1.0 - ox * s[0], -1.0 - oy * s[1]];
            builder
                .push_constants(pipe.layout().clone(), 0, pc)
                .unwrap();

            let mut voff = 0i32;
            let mut ioff = 0u32;
            for l in dd.draw_lists() {
                for cmd in l.commands() {
                    if let imgui::DrawCmd::Elements { count, cmd_params } = cmd {
                        let c = cmd_params.clip_rect;
                        let x0 = (c[0] - ox).max(0.0);
                        let y0 = (c[1] - oy).max(0.0);
                        let x1 = c[2] - ox;
                        let y1 = c[3] - oy;
                        if x1 <= x0 || y1 <= y0 {
                            continue;
                        }

                        let set = if cmd_params.texture_id.id() == self.viewport_tex_id.id() {
                            self.viewport_descriptor.clone().unwrap()
                        } else {
                            self.ui_font_set.clone().unwrap()
                        };

                        builder
                            .bind_descriptor_sets(
                                PipelineBindPoint::Graphics,
                                pipe.layout().clone(),
                                0,
                                set,
                            )
                            .unwrap()
                            .set_scissor(
                                0,
                                smallvec![Scissor {
                                    offset: [x0 as u32, y0 as u32],
                                    extent: [(x1 - x0) as u32, (y1 - y0) as u32],
                                }],
                            )
                            .unwrap();

                        unsafe {
                            builder
                                .draw_indexed(
                                    count as u32,
                                    1,
                                    ioff + cmd_params.idx_offset as u32,
                                    voff + cmd_params.vtx_offset as i32,
                                    0,
                                )
                                .unwrap();
                        }
                    }
                }
                voff += l.vtx_buffer().len() as i32;
                ioff += l.idx_buffer().len() as u32;
            }
        }

        builder.end_render_pass(SubpassEndInfo::default()).unwrap();
        let command_buffer = builder.build().unwrap();

        let boxed = acquire_future
            .then_execute(queue.clone(), scene_cb)
            .unwrap()
            .then_execute(queue.clone(), command_buffer)
            .unwrap()
            .then_swapchain_present(
                queue.clone(),
                SwapchainPresentInfo::swapchain_image_index(swapchain.clone(), image_index),
            )
            .boxed();

        let future = boxed.then_signal_fence_and_flush().unwrap();

        self.in_flight[frame] = Some(future);
        self.frame_index = (frame + 1) % FRAMES_IN_FLIGHT;
    }

    pub fn init_ui(&mut self, ctx: &mut imgui::Context) {
        use vulkano::{
            command_buffer::CopyBufferToImageInfo,
            image::{
                ImageCreateInfo, ImageType,
                sampler::{Filter, Sampler, SamplerAddressMode, SamplerCreateInfo},
            },
            pipeline::graphics::color_blend::{AttachmentBlend, ColorBlendAttachmentState},
            render_pass::Subpass,
        };

        let device = self.device.clone().unwrap();
        let queue = self.queue.clone().unwrap();
        let ma = self.memory_allocator.clone().unwrap();
        let render_pass = self.render_pass.clone().unwrap();

        let vs = shaders::ui_vertex::load(device.clone())
            .unwrap()
            .entry_point("main")
            .unwrap();
        let fs = shaders::ui_fragment::load(device.clone())
            .unwrap()
            .entry_point("main")
            .unwrap();
        let stages = [
            PipelineShaderStageCreateInfo::new(vs.clone()),
            PipelineShaderStageCreateInfo::new(fs),
        ];

        let mut layout_info = PipelineDescriptorSetLayoutCreateInfo::from_stages(stages.iter())
            .into_pipeline_layout_create_info(device.clone())
            .unwrap();
        layout_info.push_constant_ranges = vec![PushConstantRange {
            stages: ShaderStages::VERTEX,
            offset: 0,
            size: 16,
        }];
        let layout = PipelineLayout::new(device.clone(), layout_info).unwrap();

        let mut info = GraphicsPipelineCreateInfo::layout(layout);
        info.stages = stages.into_iter().collect();
        info.vertex_input_state = Some(UiVertex::per_vertex().definition(&vs).unwrap());
        info.input_assembly_state = Some(InputAssemblyState::default());
        info.viewport_state = Some(ViewportState::default());
        info.dynamic_state.insert(DynamicState::Viewport);
        info.dynamic_state.insert(DynamicState::Scissor);
        info.rasterization_state = Some(RasterizationState {
            cull_mode: CullMode::None,
            ..Default::default()
        });
        info.multisample_state = Some(MultisampleState::default());
        info.color_blend_state = Some(ColorBlendState::with_attachment_states(
            1,
            ColorBlendAttachmentState {
                blend: Some(AttachmentBlend::alpha()),
                ..Default::default()
            },
        ));
        info.subpass = Some(Subpass::from(render_pass, 1).unwrap().into());
        let pipeline = GraphicsPipeline::new(device.clone(), None, info).unwrap();

        let (staging, w, h) = {
            let tex = ctx.fonts().build_rgba32_texture();
            let staging = Buffer::from_iter(
                ma.clone(),
                BufferCreateInfo {
                    usage: BufferUsage::TRANSFER_SRC,
                    ..Default::default()
                },
                AllocationCreateInfo {
                    memory_type_filter: MemoryTypeFilter::PREFER_HOST
                        | MemoryTypeFilter::HOST_SEQUENTIAL_WRITE,
                    ..Default::default()
                },
                tex.data.iter().copied(),
            )
            .unwrap();
            (staging, tex.width, tex.height)
        };

        let image = Image::new(
            ma,
            ImageCreateInfo {
                image_type: ImageType::Dim2d,
                format: Format::R8G8B8A8_UNORM,
                extent: [w, h, 1],
                usage: ImageUsage::SAMPLED | ImageUsage::TRANSFER_DST,
                ..Default::default()
            },
            AllocationCreateInfo::default(),
        )
        .unwrap();

        let mut b = AutoCommandBufferBuilder::primary(
            self.command_buffer_allocator.clone().unwrap(),
            queue.queue_family_index(),
            CommandBufferUsage::OneTimeSubmit,
        )
        .unwrap();
        b.copy_buffer_to_image(CopyBufferToImageInfo::buffer_image(staging, image.clone()))
            .unwrap();
        sync::now(device.clone())
            .then_execute(queue, b.build().unwrap())
            .unwrap()
            .then_signal_fence_and_flush()
            .unwrap()
            .wait(None)
            .unwrap();

        let view = ImageView::new_default(image).unwrap();
        let sampler = Sampler::new(
            device,
            SamplerCreateInfo {
                mag_filter: Filter::Linear,
                min_filter: Filter::Linear,
                address_mode: [SamplerAddressMode::ClampToEdge; 3],
                ..Default::default()
            },
        )
        .unwrap();

        let set = DescriptorSet::new(
            self.descriptor_set_allocator.clone().unwrap(),
            pipeline.layout().set_layouts()[0].clone(),
            [WriteDescriptorSet::image_view_sampler(0, view, sampler)],
            [],
        )
        .unwrap();

        ctx.fonts().tex_id = imgui::TextureId::new(0);
        self.ui_pipeline = Some(pipeline);
        self.ui_font_set = Some(set);

        let viewport_sampler = Sampler::new(
            self.device.clone().unwrap(),
            SamplerCreateInfo {
                mag_filter: Filter::Linear,
                min_filter: Filter::Linear,
                address_mode: [SamplerAddressMode::ClampToEdge; 3],
                ..Default::default()
            },
        )
        .unwrap();

        let viewport_set = DescriptorSet::new(
            self.descriptor_set_allocator.clone().unwrap(),
            self.ui_pipeline
                .as_ref()
                .unwrap()
                .layout()
                .set_layouts()[0]
                .clone(),
            [WriteDescriptorSet::image_view_sampler(
                0,
                self.viewport_color_view.clone().unwrap(),
                viewport_sampler.clone(),
            )],
            [],
        )
        .unwrap();

        self.viewport_sampler = Some(viewport_sampler);
        self.viewport_descriptor = Some(viewport_set);
    }

    pub fn request_swapchain_recreation(&mut self) {
        self.recreate_swapchain = true;
    }
}