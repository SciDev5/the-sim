use std::mem::size_of;

use bytemuck::bytes_of;
use naga::ShaderStage;
use wgpu::{
    include_wgsl,
    util::{BufferInitDescriptor, DeviceExt},
    vertex_attr_array, BindGroup, Buffer, BufferUsages, Color, FragmentState, MultisampleState,
    Operations, PipelineLayoutDescriptor, PrimitiveState, RenderPassColorAttachment,
    RenderPassDescriptor, RenderPipeline, RenderPipelineDescriptor, VertexBufferLayout,
    VertexState, VertexStepMode,
};

use crate::{
    engine_base::{EngineBase, Spawner},
    include_glsl, load_img,
};

pub struct TestGPU {
    compute_pipeline: wgpu::ComputePipeline,
    buff_bind_group: BindGroup,
    test_render_pipeline: RenderPipeline,
    vertex_buffer: Buffer,
    cattex_bind_group: BindGroup,
    buff: Buffer,
    bufftex: crate::texture::Texture,
    bufftex_bind_group: BindGroup,
}

impl EngineBase for TestGPU {
    fn title() -> &'static str {
        "test gpu"
    }
    
    fn required_limits() -> wgpu::Limits {
        wgpu::Limits::downlevel_defaults()
    }
    fn required_downlevel_capabilities() -> wgpu::DownlevelCapabilities {
        wgpu::DownlevelCapabilities {
            flags: wgpu::DownlevelFlags::COMPUTE_SHADERS,
            ..Default::default()
        }
    }

    fn init(
        config: &wgpu::SurfaceConfiguration,
        _adapter: &wgpu::Adapter,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
    ) -> Self {
        // let cs_module = device
        //     .create_shader_module(include_glsl!("shaders/compute.comp", ShaderStage::Compute));
        let cs_module = device.create_shader_module(include_wgsl!("shaders/compute.wgsl"));

        let compute_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("compute_bind_group_layout"),
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: false },
                        has_dynamic_offset: false,
                        min_binding_size: wgpu::BufferSize::new(4 * 512 * 512),
                    },
                    count: None,
                }],
            });
        let compute_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("compute_pipeline_layout"),
                bind_group_layouts: &[&compute_bind_group_layout],
                push_constant_ranges: &[],
            });

        let compute_pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("Compute pipeline"),
            layout: Some(&compute_pipeline_layout),
            module: &cs_module,
            entry_point: "main",
        });

        let texture_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("texture_bind_group_layout"),
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            multisampled: false,
                            view_dimension: wgpu::TextureViewDimension::D2,
                            sample_type: wgpu::TextureSampleType::Float { filterable: true },
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

        let buff = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: None,
            contents: &vec![128; 4 * 512 * 512][..],
            usage: wgpu::BufferUsages::COPY_SRC | wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::STORAGE,
        });
        let bufftex = crate::texture::Texture::create_uninit(Some("bufftex"), (512, 512), &device);
        bufftex.write_buffer(
            &buff,
            wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: Some(4 * 512),
                rows_per_image: Some(512),
            },
            &device,
            &queue,
        );
        let bufftex_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("bufftex_bind_group"),
            layout: &texture_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&bufftex.view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&bufftex.sampler),
                },
            ],
        });
        let buff_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("buff_bind_group"),
            layout: &compute_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                    buffer: &buff,
                    offset: 0,
                    size: wgpu::BufferSize::new(4 * 512 * 512),
                }),
            }],
        });

        let cattex = crate::texture::Texture::create(load_img!("cat.jpg").unwrap(), device, queue);
        let cattex_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("cattex_bind_group"),
            layout: &texture_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&cattex.view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&cattex.sampler),
                },
            ],
        });

        // let draw_shader_module = device.create_shader_module(include_wgsl!("draw.wgsl"));
        let fs_module =
            device.create_shader_module(include_glsl!("shaders/draw.frag", ShaderStage::Fragment));
        let vs_module =
            device.create_shader_module(include_glsl!("shaders/draw.vert", ShaderStage::Vertex));

        let test_render_pipeline = device.create_render_pipeline(&RenderPipelineDescriptor {
            label: Some("test_render_pipeline"),
            layout: Some(&device.create_pipeline_layout(&PipelineLayoutDescriptor {
                label: None,
                bind_group_layouts: &[&texture_bind_group_layout, &texture_bind_group_layout],
                push_constant_ranges: &[],
            })),
            vertex: VertexState {
                // module: &draw_shader_module,
                // entry_point: "vert",
                module: &vs_module,
                entry_point: "main",
                buffers: &[VertexBufferLayout {
                    array_stride: size_of::<f32>() as u64 * 2,
                    attributes: &vertex_attr_array![0 => Float32x2],
                    step_mode: VertexStepMode::Vertex,
                }],
            },
            fragment: Some(FragmentState {
                // module: &draw_shader_module,
                // entry_point: "frag",
                module: &fs_module,
                entry_point: "main",
                targets: &[Some(config.view_formats[0].into())],
            }),
            primitive: PrimitiveState::default(),
            multisample: MultisampleState::default(),
            multiview: None,
            depth_stencil: None,
        });

        let vertices = [
            [[0.0, 0.0], [0.0, 1.0], [1.0, 0.0f32]],
            [[1.0, 1.0], [1.0, 0.0], [0.0, 1.0]],
        ];
        let vertex_buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: Some("vertex_buffer"),
            usage: BufferUsages::VERTEX | BufferUsages::COPY_DST,
            contents: bytes_of(&vertices),
        });

        Self {
            test_render_pipeline,
            vertex_buffer,
            cattex_bind_group,
            buff,
            bufftex_bind_group,
            bufftex,
            compute_pipeline,
            buff_bind_group,
        }
    }
    fn resize(
        &mut self,
        _config: &wgpu::SurfaceConfiguration,
        _device: &wgpu::Device,
        _queue: &wgpu::Queue,
    ) {
        // ignore
    }
    fn update(&mut self, _event: winit::event::WindowEvent) {
        // ignore
    }
    fn render(
        &mut self,
        view: &wgpu::TextureView,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        _spawner: &Spawner,
    ) {
        let mut command_encoder =
            device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });

        {
            let mut pass =
                command_encoder.begin_compute_pass(&wgpu::ComputePassDescriptor { label: None });
            pass.set_pipeline(&self.compute_pipeline);
            pass.set_bind_group(0, &self.buff_bind_group, &[]);
            pass.dispatch_workgroups(512 / 16, 512 / 16, 1);
        }

        self.bufftex.write_buffer_commandencoder(
            &self.buff,
            wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: Some(512 * 4),
                rows_per_image: Some(512),
            },
            &mut command_encoder,
        );

        {
            let mut pass = command_encoder.begin_render_pass(&RenderPassDescriptor {
                label: None,
                color_attachments: &[Some(RenderPassColorAttachment {
                    view,
                    resolve_target: None,
                    ops: Operations {
                        load: wgpu::LoadOp::Clear(Color::RED),
                        store: true,
                    },
                })],
                depth_stencil_attachment: None,
            });

            pass.set_pipeline(&self.test_render_pipeline);
            // pass.set_bind_group(0, &self.cattex_bind_group, &[]);
            pass.set_bind_group(0, &self.cattex_bind_group, &[]);
            pass.set_bind_group(1, &self.bufftex_bind_group, &[]);
            pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
            pass.draw(0..6, 0..1)
        }

        queue.submit(Some(command_encoder.finish()));
    }
}
