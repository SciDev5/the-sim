use bytemuck::{Pod, Zeroable};
use wgpu::{include_wgsl, vertex_attr_array};
use winit::event::VirtualKeyCode;

use crate::{
    bind_group_info,
    engine_base::EngineBase,
    include_glsl,
    new_abstractions::{
        Buff, BuffInfo, TISampler, TIStorageTexture, TITexture, Tex, TexInfo, VertexLayoutInfo,
        ZSTValue, _2D,
    },
};

const SIZE: u32 = 256;
const WORKGROUP_SIZE: u32 = 16;

#[repr(C)]
#[derive(Debug, Clone, Copy, Pod, Zeroable)]
struct WavePoint {
    x: f32,
    v: f32,
}
impl VertexLayoutInfo for WavePoint {
    const ATTRIBUTES: &'static [wgpu::VertexAttribute] = &vertex_attr_array![0 => Float32x2];
}

pub struct Wave {
    compute_pipeline: wgpu::ComputePipeline,
    render_pipeline: wgpu::RenderPipeline,

    wave_data: [Buff<WavePoint>; 2],
    compute_bind_group: [wgpu::BindGroup; 2],

    out_tex_bind_group: wgpu::BindGroup,

    square_verts: Buff<[f32; 2]>,

    frame_num: u8,

    reset: bool,
}

impl EngineBase for Wave {
    fn title() -> &'static str {
        "wave"
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
        _queue: &wgpu::Queue,
    ) -> Self {
        let tex2d_bind_group_info = Tex2DBindGroup::new(&device);
        bind_group_info!(Tex2DBindGroup; wgpu::ShaderStages::FRAGMENT;
            0 => (TexInfo::<_2D,TITexture>, wgpu::TextureSampleType::Float { filterable: true }),
            1 => (TexInfo::<_2D,TISampler>, wgpu::SamplerBindingType::Filtering),
        );

        let compute_bind_group_info = ComputeBindGroupInfo::new(&device);
        bind_group_info!(ComputeBindGroupInfo; wgpu::ShaderStages::COMPUTE;
            0 => (BuffInfo::<WavePoint>, wgpu::BufferBindingType::Storage { read_only: true }),
            1 => (BuffInfo::<WavePoint>, wgpu::BufferBindingType::Storage { read_only: false }),
            2 => (TexInfo::<_2D, TIStorageTexture>, wgpu::StorageTextureAccess::WriteOnly),
        );

        const WAVE_DATA: BuffInfo<WavePoint> = BuffInfo::IT;

        const SQUARE_VERTS: BuffInfo<[f32; 2]> = BuffInfo::IT;
        const SQUARE_VERTS_DATA: [[f32; 2]; 6] = [
            [-1.0, -1.0],
            [-1.0, 1.0],
            [1.0, -1.0],
            [-1.0, 1.0],
            [1.0, 1.0],
            [1.0, -1.0],
        ];
        impl VertexLayoutInfo for [f32; 2] {
            const ATTRIBUTES: &'static [wgpu::VertexAttribute] =
                &vertex_attr_array![0 => Float32x2];
        }

        let out_tex = Tex::<_2D>::create_uninit("wave-out", (SIZE as u32, SIZE as u32), &device);

        let out_tex_bind_group =
            tex2d_bind_group_info.bind(out_tex.binding_texture(), out_tex.binding_sampler());

        let (compute_pipeline, wave_data, compute_bind_group) = {
            let shader_module = device.create_shader_module(include_wgsl!("shaders/compute.wgsl"));

            let mut v = vec![WavePoint { x: 0.0, v: 0.0 }; (SIZE * SIZE) as usize];
            for x in 0..20 {
                for y in 50..100 {
                    // v[50+x+y*SIZE as usize] = WavePoint{x:2.0*(1.0-(x as f32 / 10.0)),v:0.0};
                    v[50 + x + y * SIZE as usize] = WavePoint { x: 2.0, v: 0.0 };
                }
            }

            let wave_data = std::array::from_fn(|_| Buff::new(&device, &WAVE_DATA, &v[..]));

            let compute_bind_group = std::array::from_fn(|i| {
                compute_bind_group_info.bind(
                    wave_data[i].slice(..),
                    wave_data[(i + 1) % 2].slice(..),
                    out_tex.binding_storage(),
                )
            });

            let pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
                label: None,
                layout: Some(
                    &device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                        label: None,
                        bind_group_layouts: &[&compute_bind_group_info.layout],
                        push_constant_ranges: &[],
                    }),
                ),
                module: &shader_module,
                entry_point: "main",
            });

            (pipeline, wave_data, compute_bind_group)
        };

        let (render_pipeline, square_verts) = {
            let module_vert = device.create_shader_module(include_glsl!(
                "shaders/draw.vert",
                naga::ShaderStage::Vertex
            ));
            let module_frag = device.create_shader_module(include_glsl!(
                "shaders/draw.frag",
                naga::ShaderStage::Fragment
            ));

            let square_verts = Buff::new(&device, &SQUARE_VERTS, &SQUARE_VERTS_DATA);

            let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: None,
                layout: Some(
                    &device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                        label: None,
                        bind_group_layouts: &[&tex2d_bind_group_info.layout],
                        push_constant_ranges: &[],
                    }),
                ),
                vertex: wgpu::VertexState {
                    module: &module_vert,
                    entry_point: "main",
                    buffers: &[SQUARE_VERTS.layout_vertex(wgpu::VertexStepMode::Vertex)],
                },
                fragment: Some(wgpu::FragmentState {
                    module: &module_frag,
                    entry_point: "main",
                    targets: &[Some(config.view_formats[0].into())],
                }),
                primitive: Default::default(),
                depth_stencil: None,
                multisample: Default::default(),
                multiview: None,
            });
            (render_pipeline, square_verts)
        };

        Self {
            compute_pipeline,
            render_pipeline,
            wave_data,
            compute_bind_group,
            out_tex_bind_group,
            square_verts,
            frame_num: 0,
            reset: false,
        }
    }

    fn resize(
        &mut self,
        _config: &wgpu::SurfaceConfiguration,
        _device: &wgpu::Device,
        _queue: &wgpu::Queue,
    ) {
        //
    }
    fn update(&mut self, event: winit::event::WindowEvent) {
        //
        match event {
            winit::event::WindowEvent::KeyboardInput { input, .. } => {
                if let Some(key) = input.virtual_keycode {
                    match key {
                        VirtualKeyCode::Space => match input.state {
                            winit::event::ElementState::Pressed => {
                                self.reset = true;
                            }
                            _ => {}
                        },
                        VirtualKeyCode::J => {
                            // j
                        }
                        _ => {}
                    }
                }
            }
            _ => {}
        }
    }
    fn render(
        &mut self,
        view: &wgpu::TextureView,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        _spawner: &crate::engine_base::Spawner,
    ) {
        let mut cmds = device.create_command_encoder(&Default::default());

        if self.reset {
            self.reset = false;
            let mut v = vec![WavePoint { x: 0.0, v: 0.0 }; (SIZE * SIZE) as usize];

            for x in 0..20 {
                for y in 50..100 {
                    // v[50+x+y*SIZE as usize] = WavePoint{x:2.0*(1.0-(x as f32 / 10.0)),v:0.0};
                    v[50 + x + y * SIZE as usize] = WavePoint { x: 2.0, v: 0.0 };
                }
            }

            for data in &self.wave_data {
                data.write(0, &v[..], &queue);
            }
        }

        {
            let mut pass = cmds.begin_compute_pass(&wgpu::ComputePassDescriptor { label: None });
            pass.set_pipeline(&self.compute_pipeline);
            pass.set_bind_group(
                0,
                &self.compute_bind_group[(self.frame_num % 2) as usize],
                &[],
            );
            pass.dispatch_workgroups(SIZE / WORKGROUP_SIZE, SIZE / WORKGROUP_SIZE, 1);
        }

        {
            let mut pass = cmds.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: None,
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
                        store: true,
                    },
                })],
                depth_stencil_attachment: None,
            });
            pass.set_pipeline(&self.render_pipeline);
            pass.set_vertex_buffer(0, self.square_verts.slice(..).into());
            pass.set_bind_group(0, &self.out_tex_bind_group, &[]);
            pass.draw(0..6, 0..1);
        }

        queue.submit(Some(cmds.finish().into()));

        self.frame_num = self.frame_num.wrapping_add(1);
    }
}
