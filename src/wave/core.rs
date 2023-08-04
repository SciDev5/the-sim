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
    }, compute_pipeline_info, render_pipeline_info,
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


bind_group_info!(Tex2DBindGroup; wgpu::ShaderStages::FRAGMENT;
    0 => (TexInfo::<_2D,TITexture>, wgpu::TextureSampleType::Float { filterable: true }),
    1 => (TexInfo::<_2D,TISampler>, wgpu::SamplerBindingType::Filtering),
);
bind_group_info!(ComputeBindGroup; wgpu::ShaderStages::COMPUTE;
    0 => (BuffInfo::<WavePoint>, wgpu::BufferBindingType::Storage { read_only: true }),
    1 => (BuffInfo::<WavePoint>, wgpu::BufferBindingType::Storage { read_only: false }),
    2 => (TexInfo::<_2D, TIStorageTexture>, wgpu::StorageTextureAccess::WriteOnly),
);
compute_pipeline_info!(TheComputePipeline;
    0 => ComputeBindGroupInfo<'device>,
);
render_pipeline_info!(TheRenderPipeline;
    0 => Tex2DBindGroupInfo<'device>,
    ;
    0 => ([f32; 2], wgpu::VertexStepMode::Vertex),
);

pub struct Wave {
    compute_pipeline: TheComputePipeline,
    render_pipeline: TheRenderPipeline,

    wave_data: [Buff<WavePoint>; 2],
    compute_bind_group: [ComputeBindGroup; 2],

    out_tex_bind_group: Tex2DBindGroup,

    square_verts: Buff<[f32; 2]>,
    square_indices: Buff<u16>,

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
        let tex2d_bind_group_info = Tex2DBindGroupInfo::new(&device);
        let compute_bind_group_info = ComputeBindGroupInfo::new(&device);

        const WAVE_DATA: BuffInfo<WavePoint> = BuffInfo::IT;

        const SQUARE_VERTS: BuffInfo<[f32; 2]> = BuffInfo::IT;
        const SQUARE_INDICES: BuffInfo<u16> = BuffInfo::IT;
        const SQUARE_VERTS_DATA: [[f32; 2]; 4] = [
            [-1.0, -1.0],
            [-1.0, 1.0],
            [1.0, -1.0],
            [1.0, 1.0],
        ];
        const SQUARE_INDICES_DATA: [u16; 6] = [
            0, 1, 2,
            3, 2, 1
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

            let pipeline = TheComputePipeline::new(&device, (shader_module, "main"), &compute_bind_group_info);

            (pipeline, wave_data, compute_bind_group)
        };

        let (render_pipeline, square_verts, square_indices) = {
            let module_vert = device.create_shader_module(include_glsl!(
                "shaders/draw.vert",
                naga::ShaderStage::Vertex
            ));
            let module_frag = device.create_shader_module(include_glsl!(
                "shaders/draw.frag",
                naga::ShaderStage::Fragment
            ));

            let square_verts = Buff::new(&device, &SQUARE_VERTS, &SQUARE_VERTS_DATA);
            let square_indices = Buff::new(&device, &SQUARE_INDICES, &SQUARE_INDICES_DATA);

            let render_pipeline = TheRenderPipeline::new(
                &device,
                (module_vert, "main"),
                (module_frag, "main"),
                &[Some(config.view_formats[0].into())],
                &SQUARE_VERTS,
                &tex2d_bind_group_info,
            );
            (render_pipeline, square_verts, square_indices)
        };

        Self {
            compute_pipeline,
            render_pipeline,
            wave_data,
            compute_bind_group,
            out_tex_bind_group,
            square_verts,
            square_indices,
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
            self.compute_pipeline.dispatch(
                &mut pass,
                (SIZE / WORKGROUP_SIZE, SIZE / WORKGROUP_SIZE, 1),
                &self.compute_bind_group[(self.frame_num % 2) as usize],
            );
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
            self.render_pipeline.draw_indexed(
                &mut pass,
                0..6, 0..1,
                self.square_indices.slice(..),
                self.square_verts.slice(..),
                &self.out_tex_bind_group,
            );
        }

        queue.submit(Some(cmds.finish().into()));

        self.frame_num = self.frame_num.wrapping_add(1);
    }
}
