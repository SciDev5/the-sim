use std::mem::size_of;

use bytemuck::bytes_of;
use naga::ShaderStage;
use wgpu::{
    util::{BufferInitDescriptor, DeviceExt},
    vertex_attr_array, Buffer, BufferUsages, Color, FragmentState, MultisampleState, Operations,
    PipelineLayoutDescriptor, PrimitiveState, RenderPassColorAttachment, RenderPassDescriptor,
    RenderPipeline, RenderPipelineDescriptor, VertexBufferLayout, VertexState, VertexStepMode,
};

use crate::{
    engine_base::{EngineBase, Spawner},
    include_glsl,
};

/// Load the shaders to make sure they compile.
#[cfg(test)]
mod shader_lint_loader {
    #[test]
    fn vert_compiles() {
        vk_shader_macros::include_glsl!("src/test_gpu/shaders/draw.vert");
    }
    #[test]
    fn frag_compiles() {
        vk_shader_macros::include_glsl!("src/test_gpu/shaders/draw.frag");
    }
}

pub struct TestGPU {
    test_render_pipeline: RenderPipeline,
    vertex_buffer: Buffer,
}

impl EngineBase for TestGPU {
    fn title() -> &'static str {
        "test gpu"
    }
    fn init(
        config: &wgpu::SurfaceConfiguration,
        _adapter: &wgpu::Adapter,
        device: &wgpu::Device,
        _queue: &wgpu::Queue,
    ) -> Self {
        // let draw_shader_module = device.create_shader_module(include_wgsl!("draw.wgsl"));
        // let vs_src = include_str!("shaders/draw.vert");
        // // let vs_src = include_glsl!("src/test_gpu/shaders/draw.vert");
        // // let fs_src = include_glsl!("src/test_gpu/shaders/draw.frag");

        // // Parse the given shader code and store its representation.
        // let options = naga::ShaderStage::Vertex.into();
        // // let options = naga::front::spv::Options {
        // //     adjust_coordinate_space: false, // we require NDC_Y_UP feature
        // //     strict_capabilities: true,
        // //     block_ctx_dump_prefix: None,
        // // };
        // let mut parser = naga::front::glsl::Frontend::default();
        // // let parser = naga::front::spv::Frontend::new(vs_src.iter().cloned(), &options);
        // let module = parser.parse(&options, vs_src).unwrap();
        // let vs_module = device.create_shader_module(ShaderModuleDescriptor{
        //     label: None,
        //     source: wgpu::ShaderSource::Naga(Cow::Owned(module))
        // });

        // // let vs_module = device.create_shader_module(wgpu::ShaderSource::Glsl(vs_src.into()));
        // // let fs_module = device.create_shader_module(wgpu::ShaderSource::Glsl(fs_src.into()));
        // // let vs_module = device.create_shader_module(ShaderModuleDescriptor {
        // //     label: None,
        // //     source: wgpu::ShaderSource::SpirV(Cow::Borrowed(vs_src)),
        // // });
        // let fs_module = device.create_shader_module(ShaderModuleDescriptor {
        //     label: None,
        //     source: wgpu::ShaderSource::SpirV(Cow::Borrowed(fs_src)),
        // });

        let fs_module =
            device.create_shader_module(include_glsl!("shaders/draw.frag", ShaderStage::Fragment));
        let vs_module =
            device.create_shader_module(include_glsl!("shaders/draw.vert", ShaderStage::Vertex));

        let test_render_pipeline = device.create_render_pipeline(&RenderPipelineDescriptor {
            label: Some("test_render_pipeline"),
            layout: Some(&device.create_pipeline_layout(&PipelineLayoutDescriptor {
                label: None,
                bind_group_layouts: &[],
                push_constant_ranges: &[],
            })),
            vertex: VertexState {
                // module: &draw_shader_module,
                // module: &device.create_shader_module(ShaderModuleDescriptor { label: None, source: wgpu::ShaderSource::Glsl { shader: Cow::Borrowed(include_str!("draw-vert.glsl")), stage: ShaderStage::Vertex, defines: Default::default() }}),
                module: &vs_module,
                buffers: &[VertexBufferLayout {
                    array_stride: size_of::<f32>() as u64 * 2,
                    attributes: &vertex_attr_array![0 => Float32x2],
                    step_mode: VertexStepMode::Vertex,
                }],
                // entry_point: "vert",
                entry_point: "main",
            },
            fragment: Some(FragmentState {
                // module: &draw_shader_module,
                // module: &device.create_shader_module(ShaderModuleDescriptor { label: None, source: wgpu::ShaderSource::Glsl { shader: Cow::Borrowed(include_str!("draw-frag.glsl")), stage: ShaderStage::Vertex, defines: Default::default() }}),
                module: &fs_module,
                // entry_point: "frag",
                entry_point: "main",
                targets: &[Some(config.view_formats[0].into())],
            }),
            primitive: PrimitiveState::default(),
            multisample: MultisampleState::default(),
            multiview: None,
            depth_stencil: None,
        });

        let vertices = [[0.0f32, 0.0], [0.0, 1.0], [1.0, 0.0]];
        let vertex_buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: Some("triangle verteces"),
            usage: BufferUsages::VERTEX | BufferUsages::COPY_DST,
            contents: bytes_of(&vertices),
        });

        Self {
            test_render_pipeline,
            vertex_buffer,
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
            pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
            pass.draw(0..3, 0..1)
        }

        queue.submit(Some(command_encoder.finish()));
    }
}
