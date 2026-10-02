use crate::engine::nodes::planet::Star;
use crate::engine::nodes::sky::{GlobalSky, GlobalSpace};
use glam::{DVec3, Mat4, Vec3};
use smallvec::smallvec;
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
            viewport::{Viewport, ViewportState},
        },
        layout::{PipelineDescriptorSetLayoutCreateInfo, PushConstantRange},
    },
    render_pass::{Framebuffer, FramebufferCreateInfo, RenderPass},
    shader::ShaderStages,
    swapchain::{self, Surface, Swapchain, SwapchainCreateInfo, SwapchainPresentInfo},
    sync::{self, GpuFuture},
};

use crate::engine::Engine;
use crate::engine::console;
use crate::engine::mesh::{Mesh, Vertices};
use crate::engine::shaders;
use crate::engine::transform::{Transform, WorldPositionExt};

const MAX_VERTICES: u64 = 16_000_000;
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
    start_vertex: usize,
    vertex_count: usize,
    position: DVec3,
    rotation: glam::Quat,
    scale: Vec3,
    emissive: [f32; 4],
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
    msaa_samples: SampleCount,
    pending_msaa_samples: Option<SampleCount>,
    descriptor_set_allocator: Option<Arc<StandardDescriptorSetAllocator>>,
    sky_pipeline: Option<Arc<GraphicsPipeline>>,
    vertex_buffer: Option<Subbuffer<[Vertices]>>,
    staging_buffers: Vec<Subbuffer<[Vertices]>>,
    uploaded_vertex_count: usize,
    scene_uniforms: Vec<Subbuffer<SceneUniform>>,
    scene_descriptors: Vec<Arc<DescriptorSet>>,
    fences: Vec<Arc<Fence>>,
    image_available: Vec<Arc<Semaphore>>,
    render_finished: Vec<Arc<Semaphore>>,
    frame_index: usize,
    draw_list: Vec<DrawObject>,
    in_flight: Vec<Option<FenceSignalFuture<Box<dyn GpuFuture>>>>,
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
            msaa_samples: SampleCount::Sample1,
            pending_msaa_samples: None,
            descriptor_set_allocator: None,
            sky_pipeline: None,
            vertex_buffer: None,
            staging_buffers: Vec::new(),
            uploaded_vertex_count: 0,
            scene_uniforms: Vec::new(),
            scene_descriptors: Vec::new(),
            fences: Vec::new(),
            image_available: Vec::new(),
            render_finished: Vec::new(),
            frame_index: 0,
            draw_list: Vec::new(),
            in_flight: (0..FRAMES_IN_FLIGHT).map(|_| None).collect(),
        }
    }

    /*pub fn set_msaa_samples(&mut self, samples: SampleCount) {
        if samples != self.msaa_samples {
            self.pending_msaa_samples = Some(samples);
        }
    }*/

    /*pub fn set_msaa_enabled(&mut self, enabled: bool) {
        self.set_msaa_samples(if enabled {
            SampleCount::Sample1
        } else {
            SampleCount::Sample1
        });
    }*/

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

    pub fn init(&mut self, window: &sdl3::video::Window) {
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

        //let data: i32 = 12;
        let iter = (0..128).map(|_| 5u8);
        let buffer = Buffer::from_iter(
            //from iter, from data
            memory_allocator.clone(),
            BufferCreateInfo {
                usage: BufferUsage::UNIFORM_BUFFER,
                ..Default::default()
            },
            AllocationCreateInfo {
                memory_type_filter: MemoryTypeFilter::PREFER_DEVICE
                    | MemoryTypeFilter::HOST_SEQUENTIAL_WRITE,
                ..Default::default()
            },
            iter, // data
        )
        .expect("failed to create buffer");

        let mut content = buffer.write().unwrap();
        content[12] *= 2;
        content[7] = 9;

        let source_content = 0..64;
        let source = Buffer::from_iter(
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
            source_content,
        )
        .expect("failed to create source buffer");

        let destination_content = (0..64).map(|_| 0);
        let destination = Buffer::from_iter(
            memory_allocator.clone(),
            BufferCreateInfo {
                usage: BufferUsage::TRANSFER_DST,
                ..Default::default()
            },
            AllocationCreateInfo {
                memory_type_filter: MemoryTypeFilter::PREFER_HOST
                    | MemoryTypeFilter::HOST_RANDOM_ACCESS,
                ..Default::default()
            },
            destination_content,
        )
        .expect("failed to create destination buffer");

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

        let mut builder = AutoCommandBufferBuilder::primary(
            command_buffer_allocator.clone(),
            queue_family_index,
            CommandBufferUsage::OneTimeSubmit,
        )
        .unwrap();

        builder
            .copy_buffer(CopyBufferInfo::buffers(source.clone(), destination.clone()))
            .unwrap();

        let command_buffer = builder.build().unwrap();

        let future = sync::now(device.clone())
            .then_execute(queue.clone(), command_buffer)
            .unwrap()
            .then_signal_fence_and_flush()
            .unwrap();

        future.wait(None).unwrap();

        let src_content = source.read().unwrap();
        let destination_content = destination.read().unwrap();
        assert_eq!(&*src_content, &*destination_content);

        logger.log_system("Surface initialisation succeeded.");

        let caps = physical_device
            .surface_capabilities(&surface, Default::default())
            .expect("failed to get surface capabilities");

        let dimensions = window.size();

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

        let render_pass = vulkano::single_pass_renderpass!(
            device.clone(),
            attachments: {
                /*color: {
                    format: image_format,
                    samples: 4, // here
                    load_op: Clear,
                    store_op: Store,
                },*/
                /*multisampled_color: {
                    format: image_format,
                    samples: 1,
                    load_op: Clear,
                    store_op: DontCare,
                },*/
                swapchain_color: { // this
                    format: image_format,
                    samples: 1,
                    load_op: Clear,
                    store_op: Store,
                },
                depth: {
                    format: Format::D32_SFLOAT,
                    samples: 1, // here
                    load_op: Clear,
                    store_op: DontCare,
                },
            },
            pass: {
                //color: [multisampled_color], // color
                //color_resolve: [swapchain_color], // that
                color: [swapchain_color],
                depth_stencil: {depth},
            },
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
                present_mode: PresentMode::Immediate, // Immediate (No Vsync), Mailbox (Vsync- Non blocking), Fifo (Vsync- Blocking), FifoRelaxed (Adaptive Vsync?)
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
                        extent: [dimensions.0, dimensions.1, 1],
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
        self.descriptor_set_allocator = Some(descriptor_set_allocator);
        self.staging_buffers = staging_buffers;
        self.scene_uniforms = scene_uniforms;
        self.scene_descriptors = scene_descriptors;
        self.fences = fences;
        self.image_available = image_available;
        self.render_finished = render_finished;
        self.sky_pipeline = Some(sky_pipeline);
    }

    pub fn render(&mut self, window: &sdl3::video::Window, engine: &mut Engine) {
        if self.recreate_swapchain {
            let dimensions = window.size();
            if dimensions.0 == 0 || dimensions.1 == 0 {
                return;
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
                            extent: [dimensions.0, dimensions.1, 1],
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
            self.recreate_swapchain = false;
        }

        if let Some(samples) = self.pending_msaa_samples.take() {
            self.msaa_samples = samples;
            self.rebuild_render_targets(window);
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
        let view = glam::camera::rh::view::look_at_mat4(Vec3::ZERO, forward, Vec3::Y);
        let projection = glam::camera::rh::proj::vulkan::perspective(
            camera.fov.to_radians(),
            dimensions[0] as f32 / dimensions[1] as f32,
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

        let mut vertices: Vec<Vertices> = Vec::new();
        self.draw_list.clear();

        for (transform, mesh, star) in engine
            .world
            .query::<(&Transform, &Mesh, Option<&Star>)>()
            .iter(&engine.world)
        {
            let start_vertex = vertices.len();
            vertices.extend_from_slice(&mesh.vertices);
            self.draw_list.push(DrawObject {
                start_vertex,
                vertex_count: mesh.vertices.len(),
                position: transform.position,
                rotation: transform.rotation,
                scale: transform.scale,
                emissive: star
                    .map(|s| [s.color.x, s.color.y, s.color.z, 1.0])
                    .unwrap_or([0.0; 4]),
            });
        }

        if vertices.is_empty() {
            return;
        }
        if vertices.len() as u64 > MAX_VERTICES {
            eprintln!(
                "Vertex buffer overflow: {} > {}",
                vertices.len(),
                MAX_VERTICES
            );
            return;
        }

        {
            let mut writer = self.staging_buffers[frame].write().unwrap();
            writer[..vertices.len()].copy_from_slice(&vertices);
        }
        let vertex_count = vertices.len();

        {
            let mut copy_builder = AutoCommandBufferBuilder::primary(
                self.command_buffer_allocator.as_ref().unwrap().clone(),
                queue.queue_family_index(),
                CommandBufferUsage::OneTimeSubmit,
            )
            .unwrap();

            copy_builder
                .copy_buffer(CopyBufferInfo::buffers(
                    self.staging_buffers[frame]
                        .clone()
                        .slice(0..vertex_count as u64),
                    self.vertex_buffer
                        .as_ref()
                        .unwrap()
                        .clone()
                        .slice(0..vertex_count as u64),
                ))
                .unwrap();

            let copy_cb = copy_builder.build().unwrap();

            sync::now(self.device.as_ref().unwrap().clone())
                .then_execute(queue.clone(), copy_cb)
                .unwrap()
                .then_signal_fence_and_flush()
                .unwrap()
                .wait(None)
                .unwrap();
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

        let view_proj = projection * view;
        let inv_view_proj = view_proj.inverse();

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

        {
            let mut w = self.scene_uniforms[frame].write().unwrap();
            *w = SceneUniform {
                view: view.to_cols_array_2d(),
                projection: projection.to_cols_array_2d(),
                inv_view_proj: inv_view_proj.to_cols_array_2d(),
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

        builder
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
            builder
                .push_constants(sky_pipeline.layout().clone(), 0, sky_push)
                .unwrap()
                .draw(3, 1, 0, 0)
                .unwrap();
        }

        let draw_buffer = self
            .vertex_buffer
            .as_ref()
            .unwrap()
            .clone()
            .slice(0..vertex_count as u64);

        builder
            .bind_pipeline_graphics(graphics_pipeline.clone())
            .unwrap()
            .bind_vertex_buffers(0, draw_buffer)
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
            let model =
                Mat4::from_scale_rotation_translation(obj.scale, obj.rotation, relative_position);
            unsafe {
                builder
                    .push_constants(
                        graphics_pipeline.layout().clone(),
                        0,
                        ModelPush {
                            model: model.to_cols_array_2d(),
                            emissive: obj.emissive,
                        },
                    )
                    .unwrap();
                builder
                    .draw(obj.vertex_count as u32, 1, obj.start_vertex as u32, 0)
                    .unwrap();
            }
        }

        builder.end_render_pass(SubpassEndInfo::default()).unwrap();
        let command_buffer = builder.build().unwrap();

        let boxed = acquire_future
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

    pub fn request_swapchain_recreation(&mut self) {
        self.recreate_swapchain = true;
    }

    fn rebuild_render_targets(&mut self, _window: &sdl3::video::Window) {
        self.recreate_swapchain = true;
    }
}
