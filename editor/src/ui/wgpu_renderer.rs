use std::mem::size_of;

use imgui::{DrawCmd, DrawData, DrawIdx};
use wgpu::util::DeviceExt;

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct UiVertex {
    position: [f32; 2],
    uv: [f32; 2],
    color: [u8; 4],
}

pub struct UiRenderer {
    pipeline: wgpu::RenderPipeline,
    params: wgpu::Buffer,
    texture_layout: wgpu::BindGroupLayout,
    sampler: wgpu::Sampler,
    textures: imgui::Textures<wgpu::BindGroup>,
}

impl UiRenderer {
    pub fn new(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        format: wgpu::TextureFormat,
        context: &mut imgui::Context,
        viewport: &wgpu::TextureView,
    ) -> Self {
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("imgui-shader"),
            source: wgpu::ShaderSource::Wgsl(UI_SHADER.into()),
        });
        let params_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("imgui-params-layout"),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
        });
        let texture_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("imgui-texture-layout"),
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
        let layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("imgui-pipeline-layout"),
            bind_group_layouts: &[Some(&params_layout), Some(&texture_layout)],
            immediate_size: 0,
        });
        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("imgui-pipeline"),
            layout: Some(&layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                compilation_options: Default::default(),
                buffers: &[Some(wgpu::VertexBufferLayout {
                    array_stride: size_of::<UiVertex>() as u64,
                    step_mode: wgpu::VertexStepMode::Vertex,
                    attributes: &wgpu::vertex_attr_array![0 => Float32x2, 1 => Float32x2, 2 => Unorm8x4],
                })],
            },
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                cull_mode: None,
                ..Default::default()
            },
            depth_stencil: None,
            multisample: Default::default(),
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                compilation_options: Default::default(),
                targets: &[Some(wgpu::ColorTargetState {
                    format,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            multiview_mask: None,
            cache: None,
        });
        let params = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("imgui-params"),
            size: 16,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("imgui-sampler"),
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            ..Default::default()
        });
        let mut textures = imgui::Textures::new();
        let font = context.fonts().build_rgba32_texture();
        let font_texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("imgui-font-atlas"),
            size: wgpu::Extent3d {
                width: font.width,
                height: font.height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8Unorm,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });
        queue.write_texture(
            wgpu::TexelCopyTextureInfo {
                texture: &font_texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            font.data,
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(font.width * 4),
                rows_per_image: Some(font.height),
            },
            wgpu::Extent3d {
                width: font.width,
                height: font.height,
                depth_or_array_layers: 1,
            },
        );
        let font_view = font_texture.create_view(&Default::default());
        let font_group = make_texture_group(device, &texture_layout, &sampler, &font_view);
        let font_id = textures.insert(font_group);
        context.fonts().tex_id = font_id;
        let viewport_group = make_texture_group(device, &texture_layout, &sampler, viewport);
        textures.insert(viewport_group);
        Self {
            pipeline,
            params,
            texture_layout,
            sampler,
            textures,
        }
    }

    pub fn update_viewport(&mut self, device: &wgpu::Device, view: &wgpu::TextureView) {
        let group = make_texture_group(device, &self.texture_layout, &self.sampler, view);
        self.textures.replace(imgui::TextureId::new(1), group);
    }

    pub fn render(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        draw_data: &DrawData,
        target: &wgpu::TextureView,
        extent: [u32; 2],
    ) {
        let width = extent[0].max(1);
        let height = extent[1].max(1);
        let display_width = draw_data.display_size[0].max(1.0);
        let display_height = draw_data.display_size[1].max(1.0);
        let scale = [
            2.0 * draw_data.framebuffer_scale[0] / display_width,
            2.0 * draw_data.framebuffer_scale[1] / display_height,
        ];
        let params = [
            scale[0],
            -scale[1],
            -1.0 - draw_data.display_pos[0] * scale[0],
            1.0 + draw_data.display_pos[1] * scale[1],
        ];
        queue.write_buffer(&self.params, 0, bytemuck::cast_slice(&params));

        let mut vertices = Vec::with_capacity(draw_data.total_vtx_count as usize);
        let mut indices = Vec::<DrawIdx>::with_capacity(draw_data.total_idx_count as usize);
        for list in draw_data.draw_lists() {
            vertices.extend(list.vtx_buffer().iter().map(|vertex| UiVertex {
                position: vertex.pos,
                uv: vertex.uv,
                color: vertex.col,
            }));
            indices.extend_from_slice(list.idx_buffer());
        }
        let vertex_buffer = if vertices.is_empty() {
            device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("imgui-vertices"),
                size: 4,
                usage: wgpu::BufferUsages::VERTEX,
                mapped_at_creation: false,
            })
        } else {
            device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("imgui-vertices"),
                contents: bytemuck::cast_slice(&vertices),
                usage: wgpu::BufferUsages::VERTEX,
            })
        };
        let index_buffer = if indices.is_empty() {
            device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("imgui-indices"),
                size: 4,
                usage: wgpu::BufferUsages::INDEX,
                mapped_at_creation: false,
            })
        } else {
            device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("imgui-indices"),
                contents: bytemuck::cast_slice(&indices),
                usage: wgpu::BufferUsages::INDEX,
            })
        };
        let params_layout = self.pipeline.get_bind_group_layout(0);
        let params_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("imgui-frame-bind-group"),
            layout: &params_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: self.params.as_entire_binding(),
            }],
        });
        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("imgui-encoder"),
        });
        {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("imgui-pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: target,
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
            pass.set_pipeline(&self.pipeline);
            pass.set_bind_group(0, &params_group, &[]);
            pass.set_vertex_buffer(0, vertex_buffer.slice(..));
            pass.set_index_buffer(
                index_buffer.slice(..),
                if size_of::<DrawIdx>() == 2 {
                    wgpu::IndexFormat::Uint16
                } else {
                    wgpu::IndexFormat::Uint32
                },
            );
            let mut vertex_offset = 0i32;
            let mut index_offset = 0u32;
            for list in draw_data.draw_lists() {
                for command in list.commands() {
                    if let DrawCmd::Elements { count, cmd_params } = command {
                        let clip = cmd_params.clip_rect;
                        let x0 = ((clip[0] - draw_data.display_pos[0])
                            * draw_data.framebuffer_scale[0])
                            .max(0.0);
                        let y0 = ((clip[1] - draw_data.display_pos[1])
                            * draw_data.framebuffer_scale[1])
                            .max(0.0);
                        let x1 = ((clip[2] - draw_data.display_pos[0])
                            * draw_data.framebuffer_scale[0])
                            .min(width as f32);
                        let y1 = ((clip[3] - draw_data.display_pos[1])
                            * draw_data.framebuffer_scale[1])
                            .min(height as f32);
                        if x1 <= x0 || y1 <= y0 {
                            continue;
                        }
                        let Some(group) = self.textures.get(cmd_params.texture_id) else {
                            continue;
                        };
                        pass.set_bind_group(1, group, &[]);
                        pass.set_scissor_rect(
                            x0.floor() as u32,
                            y0.floor() as u32,
                            (x1 - x0).ceil() as u32,
                            (y1 - y0).ceil() as u32,
                        );
                        pass.draw_indexed(
                            index_offset + cmd_params.idx_offset as u32
                                ..index_offset + cmd_params.idx_offset as u32 + count as u32,
                            vertex_offset + cmd_params.vtx_offset as i32,
                            0..1,
                        );
                    }
                }
                vertex_offset += list.vtx_buffer().len() as i32;
                index_offset += list.idx_buffer().len() as u32;
            }
        }
        queue.submit([encoder.finish()]);
    }
}

fn make_texture_group(
    device: &wgpu::Device,
    layout: &wgpu::BindGroupLayout,
    sampler: &wgpu::Sampler,
    view: &wgpu::TextureView,
) -> wgpu::BindGroup {
    device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("imgui-texture-bind-group"),
        layout,
        entries: &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::TextureView(view),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: wgpu::BindingResource::Sampler(sampler),
            },
        ],
    })
}

const UI_SHADER: &str = r#"
struct Params {
    transform: vec4<f32>,
}

@group(0) @binding(0) var<uniform> params: Params;
@group(1) @binding(0) var image: texture_2d<f32>;
@group(1) @binding(1) var image_sampler: sampler;

struct VertexInput {
    @location(0) position: vec2<f32>,
    @location(1) uv: vec2<f32>,
    @location(2) color: vec4<f32>,
}

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) uv: vec2<f32>,
    @location(1) color: vec4<f32>,
}

@vertex
fn vs_main(input: VertexInput) -> VertexOutput {
    var output: VertexOutput;
    output.position = vec4<f32>(input.position * params.transform.xy + params.transform.zw, 0.0, 1.0);
    output.uv = input.uv;
    output.color = input.color;
    return output;
}

@fragment
fn fs_main(input: VertexOutput) -> @location(0) vec4<f32> {
    return input.color * textureSample(image, image_sampler, input.uv);
}
"#;
