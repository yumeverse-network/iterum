use crate::engine::nodes::light::PointLight;
use glam::{Mat4, Vec3};
use smallvec::smallvec;
use std::sync::Arc;
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
    image::{Image, ImageUsage, view::ImageView},
    instance::{Instance, InstanceCreateFlags, InstanceCreateInfo},
    memory::allocator::{AllocationCreateInfo, MemoryTypeFilter, StandardMemoryAllocator},
    pipeline::{
        DynamicState, GraphicsPipeline, Pipeline, PipelineBindPoint, PipelineLayout,
        PipelineShaderStageCreateInfo,
        graphics::{
            GraphicsPipelineCreateInfo,
            color_blend::ColorBlendState,
            input_assembly::InputAssemblyState,
            multisample::MultisampleState,
            rasterization::RasterizationState,
            vertex_input::{Vertex, VertexDefinition},
            viewport::{Viewport, ViewportState},
        },
        layout::PipelineDescriptorSetLayoutCreateInfo,
    },
    render_pass::{Framebuffer, FramebufferCreateInfo, RenderPass},
    swapchain::{self, Surface, Swapchain, SwapchainCreateInfo, SwapchainPresentInfo},
    sync::{self, GpuFuture},
};

use crate::engine::Engine;
use crate::engine::console;
use crate::engine::mesh::{Mesh, MyVertex};
use crate::engine::shaders;
use crate::engine::transform::{Transform, WorldPositionExt};

// Max vertices push per frame. Allocated once, overwritten every frame.
const MAX_VERTICES: u64 = 4096;

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
    vertex_buffer: Option<Subbuffer<[MyVertex]>>,
    elapsed: f32,
    recreate_swapchain: bool,
}

use crate::engine::camera::Camera;
use crate::engine::player::Player;

#[derive(BufferContents, Copy, Clone)]
#[repr(C)]
struct SceneUniform {
    model: [[f32; 4]; 4],
    view: [[f32; 4]; 4],
    projection: [[f32; 4]; 4],

    light_position: [f32; 3],
    _padding1: f32,

    light_color: [f32; 3],
    light_intensity: f32,
    light_range: f32,
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
            vertex_buffer: None,
            elapsed: 0.0,
            recreate_swapchain: false,
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

        let vertex_buffer = Buffer::new_slice::<MyVertex>(
            memory_allocator.clone(),
            BufferCreateInfo {
                usage: BufferUsage::VERTEX_BUFFER,
                ..Default::default()
            },
            AllocationCreateInfo {
                memory_type_filter: MemoryTypeFilter::PREFER_HOST
                    | MemoryTypeFilter::HOST_SEQUENTIAL_WRITE,
                ..Default::default()
            },
            MAX_VERTICES,
        )
        .expect("failed to create vertex buffer");

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
                color: {
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
                },
            },
            pass: {
                color: [color],
                depth_stencil: {depth},
            },
        )
        .unwrap();

        let vertex_stage = PipelineShaderStageCreateInfo::new(vertex_entry.clone());
        let fragment_stage = PipelineShaderStageCreateInfo::new(fragment_entry.clone());

        let stages = [vertex_stage, fragment_stage];

        let layout = PipelineLayout::new(
            device.clone(),
            PipelineDescriptorSetLayoutCreateInfo::from_stages(stages.iter())
                .into_pipeline_layout_create_info(device.clone())
                .unwrap(),
        )
        .unwrap();

        let mut pipeline_info = GraphicsPipelineCreateInfo::layout(layout);

        pipeline_info.stages = stages.into_iter().collect();

        pipeline_info.vertex_input_state =
            Some(MyVertex::per_vertex().definition(&vertex_entry).unwrap());

        pipeline_info.input_assembly_state = Some(InputAssemblyState::default());
        pipeline_info.viewport_state = Some(ViewportState::default());
        pipeline_info.dynamic_state.insert(DynamicState::Viewport);
        pipeline_info.rasterization_state = Some(RasterizationState::default());
        pipeline_info.multisample_state = Some(MultisampleState::default());

        pipeline_info.depth_stencil_state = Some(
            vulkano::pipeline::graphics::depth_stencil::DepthStencilState::simple_depth_test(),
        );

        pipeline_info.color_blend_state = Some(ColorBlendState::with_attachment_states(
            1,
            Default::default(),
        ));

        pipeline_info.subpass = Some(render_pass.clone().first_subpass().into());

        let graphics_pipeline = GraphicsPipeline::new(device.clone(), None, pipeline_info)
            .expect("failed to create graphics pipeline");

        let (swapchain, images) = Swapchain::new(
            device.clone(),
            surface.clone(),
            SwapchainCreateInfo {
                min_image_count: caps.min_image_count + 1,
                image_format,
                image_extent: dimensions.into(),
                image_usage: ImageUsage::COLOR_ATTACHMENT,
                composite_alpha,
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
            .map(|(color_view, depth_view)| {
                Framebuffer::new(
                    render_pass.clone(),
                    FramebufferCreateInfo {
                        attachments: vec![color_view.clone(), depth_view.clone()],
                        ..Default::default()
                    },
                )
                .unwrap()
            })
            .collect::<Vec<_>>();

        logger.log_system(format_args!("Swapchain images: {}", images.len()));
        logger.break_line();

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

        let command_buffer_allocator = self.command_buffer_allocator.as_ref().unwrap();
        let queue = self.queue.as_ref().unwrap();
        let swapchain = self.swapchain.as_ref().unwrap();
        let graphics_pipeline = self.graphics_pipeline.as_ref().unwrap();

        let camera = &engine.camera;

        let dimensions = swapchain.image_extent();

        let position = Vec3::new(
            camera.position.x as f32,
            camera.position.y as f32,
            camera.position.z as f32,
        );

        let forward = camera.rotation * Vec3::NEG_Z;

        let view = glam::camera::rh::view::look_at_mat4(position, position + forward, Vec3::Y);

        let projection = glam::camera::rh::proj::vulkan::perspective(
            camera.fov.to_radians(),
            dimensions[0] as f32 / dimensions[1] as f32,
            camera.near,
            camera.far,
        );

        // World-space light position.
        for (transform, light) in engine
            .world
            .query::<(&Transform, &PointLight)>()
            .iter(&engine.world)
        {
            // light data
        }

        // Convert the light to camera-relative coordinates.
        let (relative_light_position, light_color, light_intensity) = match engine
            .world
            .query::<(&Transform, &PointLight)>()
            .iter(&engine.world)
            .next()
        {
            Some((transform, light)) => (
                transform.position.camera_relative_f32(camera.position),
                light.color,
                light.intensity,
            ),
            None => return,
        };

        let scene_buffer = Buffer::from_data(
            self.memory_allocator.as_ref().unwrap().clone(),
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
                model: Mat4::IDENTITY.to_cols_array_2d(),
                view: view.to_cols_array_2d(),
                projection: projection.to_cols_array_2d(),
                light_position: relative_light_position.to_array(),
                _padding1: 0.0,
                light_color: light_color.to_array(),
                light_intensity,
                light_range: 500.0,
            },
        )
        .unwrap();

        let descriptor_set_allocator = StandardDescriptorSetAllocator::new(
            self.device.as_ref().unwrap().clone(),
            Default::default(),
        );

        let descriptor_set = DescriptorSet::new(
            Arc::new(descriptor_set_allocator),
            graphics_pipeline.layout().set_layouts()[0].clone(),
            [WriteDescriptorSet::buffer(0, scene_buffer.clone())],
            [],
        )
        .unwrap();

        // Clone the Subbuffer
        let vertex_buffer = self.vertex_buffer.as_ref().unwrap().clone();

        // Update delta time
        let dt = engine.get_delta() as f32;
        self.elapsed += dt;

        let mut vertices = Vec::new();
        let mut objects = Vec::new();

        for (transform, mesh) in engine
            .world
            .query::<(&Transform, &Mesh)>()
            .iter(&engine.world)
        {
            let start_vertex = vertices.len();

            vertices.extend_from_slice(&mesh.vertices);

            objects.push((
                start_vertex,
                mesh.vertices.len(),
                transform.position,
                transform.rotation,
                transform.scale,
            ));
        }

        if vertices.is_empty() {
            return;
        }

        let vertex_count = vertices.len() as u32;

        // Write the new vertices into the pre-allocated buffer.
        {
            let mut writer = vertex_buffer.write().unwrap();
            let dst = &mut writer[..vertices.len()];
            dst.copy_from_slice(&vertices);
        }

        let draw_buffer = vertex_buffer.slice(0..vertices.len() as u64);

        // Get the next swapchain image.
        let (image_index, _suboptimal, acquire_future) =
            match swapchain::acquire_next_image(swapchain.clone(), None) {
                Ok(result) => result,

                Err(vulkano::Validated::Error(vulkano::VulkanError::OutOfDate)) => {
                    self.recreate_swapchain = true;
                    return;
                }

                Err(error) => {
                    panic!("Failed to acquire swapchain image: {error}");
                }
            };

        let framebuffer = self.framebuffers[image_index as usize].clone();

        let dimensions = swapchain.image_extent();

        let viewport = Viewport {
            offset: [0.0, 0.0],
            extent: [dimensions[0] as f32, dimensions[1] as f32],
            depth_range: 0.0..=1.0,
        };

        let mut builder = AutoCommandBufferBuilder::primary(
            command_buffer_allocator.clone(),
            queue.queue_family_index(),
            CommandBufferUsage::OneTimeSubmit,
        )
        .unwrap();

        builder
            .begin_render_pass(
                RenderPassBeginInfo {
                    clear_values: vec![
                        Some([0.02, 0.02, 0.02, 1.0].into()),
                        Some(1.0f32.into()), // add
                    ],
                    ..RenderPassBeginInfo::framebuffer(framebuffer)
                },
                SubpassBeginInfo {
                    contents: SubpassContents::Inline,
                    ..Default::default()
                },
            )
            .unwrap()
            .set_viewport(0, smallvec![viewport])
            .unwrap()
            .bind_pipeline_graphics(graphics_pipeline.clone())
            .unwrap()
            .bind_vertex_buffers(0, draw_buffer)
            .unwrap();

        for (start_vertex, vertex_count, object_position, rotation, scale) in objects {
            let relative_position = object_position.camera_relative_f32(camera.position);

            let model = Mat4::from_scale_rotation_translation(scale, rotation, relative_position);

            let scene_buffer = Buffer::from_data(
                self.memory_allocator.as_ref().unwrap().clone(),
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
                    model: model.to_cols_array_2d(),
                    view: view.to_cols_array_2d(),
                    projection: projection.to_cols_array_2d(),
                    light_position: [
                        relative_light_position.x as f32,
                        relative_light_position.y as f32,
                        relative_light_position.z as f32,
                    ],
                    _padding1: 0.0,
                    light_color: [1.0, 1.0, 1.0],
                    light_intensity: 20.0,
                    light_range: 500.0,
                },
            )
            .unwrap();

            let descriptor_set_allocator = StandardDescriptorSetAllocator::new(
                self.device.as_ref().unwrap().clone(),
                Default::default(),
            );

            let descriptor_set = DescriptorSet::new(
                Arc::new(descriptor_set_allocator),
                graphics_pipeline.layout().set_layouts()[0].clone(),
                [WriteDescriptorSet::buffer(0, scene_buffer.clone())],
                [],
            )
            .unwrap();

            builder
                .bind_descriptor_sets(
                    PipelineBindPoint::Graphics,
                    graphics_pipeline.layout().clone(),
                    0,
                    descriptor_set,
                )
                .unwrap();

            unsafe {
                builder
                    .draw(vertex_count as u32, 1, start_vertex as u32, 0)
                    .unwrap();
            }
        }

        builder.end_render_pass(SubpassEndInfo::default()).unwrap();

        let command_buffer = builder.build().unwrap();

        // Submit rendering to GPU, then present the swapchain image.
        let future = acquire_future
            .then_execute(queue.clone(), command_buffer)
            .unwrap()
            .then_swapchain_present(
                queue.clone(),
                SwapchainPresentInfo::swapchain_image_index(swapchain.clone(), image_index),
            );

        let _future = future.then_signal_fence_and_flush().unwrap();
    }

    pub fn request_swapchain_recreation(&mut self) {
        self.recreate_swapchain = true;
    }

    pub fn render_player(&mut self, player: &Player, camera: &Camera) -> (f32, f32) {
        let screen_x = (player.position.x - camera.position.x) as f32;
        let screen_y = (player.position.y - camera.position.y) as f32;
        (screen_x, screen_y)
    }
}
