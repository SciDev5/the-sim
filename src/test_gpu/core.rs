use naga::ShaderStage;
use wgpu::{
    include_wgsl, vertex_attr_array, BindGroup, Color, FragmentState, MultisampleState, Operations,
    PipelineLayoutDescriptor, PrimitiveState, RenderPassColorAttachment, RenderPassDescriptor,
    RenderPipeline, RenderPipelineDescriptor, VertexState, VertexStepMode,
};

use crate::{
    bind_group::TypedBindGroupGen,
    buffer::{Buffer1d, BufferInfo1d},
    engine_base::{EngineBase, Spawner},
    include_glsl, load_img,
    texture::{Tex2dFragBindGroup, Tex2dFragBindGroupInit, Texture2D},
};

pub struct TestGPU {
    compute_pipeline: wgpu::ComputePipeline,
    buff_bind_group: BindGroup,
    test_render_pipeline: RenderPipeline,
    vertex_buffer: Buffer1d<[f32; 2]>,
    cattex_bind_group: BindGroup,
    buff: Buffer1d<[u8; 4]>,
    _bufftex: crate::texture::Texture2D,
    bufftex_bind_group: BindGroup,
    reset_tex: bool,
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
        let buf_info: BufferInfo1d<[u8; 4]> = BufferInfo1d::new(
            1,
            wgpu::ShaderStages::COMPUTE,
            wgpu::BufferBindingType::Storage { read_only: false },
        );
        let vertbuf_info: BufferInfo1d<[f32; 2]> = BufferInfo1d::new(
            3,
            wgpu::ShaderStages::VERTEX,
            wgpu::BufferBindingType::default(),
        );

        // let cs_module = device
        //     .create_shader_module(include_glsl!("shaders/compute.comp", ShaderStage::Compute));
        let cs_module = device.create_shader_module(include_wgsl!("shaders/compute.wgsl"));

        let compute_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("compute_bind_group_layout"),
                entries: &[buf_info.layout_entry(0), Texture2D::layout_entry_compute(1, wgpu::StorageTextureAccess::WriteOnly)],
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

        let texture_bind_group_layout = Tex2dFragBindGroup::new(&device, Tex2dFragBindGroupInit);

        let buff = buf_info.create(&device, &[[128, 128, 128, 128]; 512 * 512][..]);
        let bufftex =
            crate::texture::Texture2D::create_uninit("bufftex", true, (512, 512), &device);
        let bufftex_bind_group = texture_bind_group_layout.create(&bufftex);
        let buff_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("buff_bind_group"),
            layout: &compute_bind_group_layout,
            entries: &[buff.bind_group_entry(0), bufftex.bind_group_entry(1)],
        });

        let cattex =
            crate::texture::Texture2D::create(load_img!("cat.jpg").unwrap(), false, device, queue);
        let cattex_bind_group = texture_bind_group_layout.create(&cattex);

        // let draw_shader_module = device.create_shader_module(include_wgsl!("draw.wgsl"));
        let fs_module =
            device.create_shader_module(include_glsl!("shaders/draw.frag", ShaderStage::Fragment));
        let vs_module =
            device.create_shader_module(include_glsl!("shaders/draw.vert", ShaderStage::Vertex));

        let test_render_pipeline = device.create_render_pipeline(&RenderPipelineDescriptor {
            label: Some("test_render_pipeline"),
            layout: Some(&device.create_pipeline_layout(&PipelineLayoutDescriptor {
                label: None,
                bind_group_layouts: &[
                    texture_bind_group_layout.layout(),
                    texture_bind_group_layout.layout(),
                ],
                push_constant_ranges: &[],
            })),
            vertex: VertexState {
                // module: &draw_shader_module,
                // entry_point: "vert",
                module: &vs_module,
                entry_point: "main",
                buffers: &[vertbuf_info
                    .layout_vertex(&vertex_attr_array![0 => Float32x2], VertexStepMode::Vertex)],
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

        let vertex_buffer = vertbuf_info.create(
            &device,
            &[
                [0.0, 0.0],
                [0.0, 1.0],
                [1.0, 0.0f32],
                [1.0, 1.0],
                [1.0, 0.0],
                [0.0, 1.0],
            ],
        );

        Self {
            test_render_pipeline,
            vertex_buffer,
            cattex_bind_group,
            buff,
            bufftex_bind_group,
            _bufftex: bufftex,
            compute_pipeline,
            buff_bind_group,
            reset_tex: false,
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
    fn update(&mut self, event: winit::event::WindowEvent) {
        // ignore
        match event {
            winit::event::WindowEvent::KeyboardInput { input, .. } => match input.virtual_keycode {
                Some(winit::event::VirtualKeyCode::Space) => {
                    self.reset_tex = true;
                }
                _ => {}
            },
            _ => {}
        }
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

        if self.reset_tex {
            self.reset_tex = false;
            self.buff.write(0, &[[128; 4]; 512 * 512], queue);
        }

        {
            let mut pass =
                command_encoder.begin_compute_pass(&wgpu::ComputePassDescriptor { label: None });
            pass.set_pipeline(&self.compute_pipeline);
            pass.set_bind_group(0, &self.buff_bind_group, &[]);
            pass.dispatch_workgroups(512 / 16, 512 / 16, 1);
        }

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
