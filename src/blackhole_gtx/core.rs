use bytemuck::{Pod, Zeroable};
use wgpu::vertex_attr_array;
use winit::event::VirtualKeyCode;

use crate::{
    engine_base::EngineBase,
    include_glsl,
    new_abstractions::{Buff, BuffInfo, VertexLayoutInfo, ZSTValue},
    render_pipeline_info, bind_group_info,
};

#[derive(Debug, Clone, Copy, Pod, Zeroable)]
#[repr(C)]
struct Vertex(f32, f32);
impl VertexLayoutInfo for Vertex {
    const ATTRIBUTES: &'static [wgpu::VertexAttribute] = &vertex_attr_array![
        0 => Float32x2,
    ];
}

const VERTEX_BUFF: BuffInfo<Vertex> = BuffInfo::IT;
const VERTEX_DATA: [Vertex; 4] = [
    Vertex(-1.0, -1.0),
    Vertex(-1.0, 1.0),
    Vertex(1.0, -1.0),
    Vertex(1.0, 1.0),
];
const INDECES_DATA: [u16; 6] = [0, 1, 2, 3, 2, 1];

// const K: BuffInfo<>
#[derive(Debug, Clone, Copy, Pod, Zeroable)]
#[repr(C)]
struct CameraData { // NOTE THAT ORDER MATTERS!!!
    dims: [f32; 2],
    rotation: [f32; 2], // TODO quaternion
    position: [f32; 3],
    fov_y: f32,

    activated: u32,
    a: f32,
    _spacer0: u32,
    _spacer1: u32,
}
bind_group_info!(CameraDataGroup; wgpu::ShaderStages::FRAGMENT;
    0 => (BuffInfo::<CameraData>, wgpu::BufferBindingType::Uniform),
);

render_pipeline_info!(GTXRenderPipeline;
    0 => CameraDataGroupInfo<'device>,
    ;
    0 => (Vertex, wgpu::VertexStepMode::Vertex),
);

pub struct BlackholeGtx {
    gtx_render_pipeline: GTXRenderPipeline,

    index_buff: Buff<u16>,
    vertex_buff: Buff<Vertex>,

    cameradata: CameraData,
    cameradata_binding: CameraDataGroup,
    cameradata_buff: Buff<CameraData>,
    cameradata_modified: bool,

    last_mouse_pos: [f64; 2],
    activated: bool,
}

impl BlackholeGtx {
    fn send_cameradata(&self, queue: &wgpu::Queue) {
        self.cameradata_buff.write(0, &[self.cameradata], &queue);
    }
}

impl EngineBase for BlackholeGtx {
    fn title() -> &'static str {
        "Black hole geodesic tracer."
    }
    fn resize(
        &mut self,
        config: &wgpu::SurfaceConfiguration,
        _device: &wgpu::Device,
        queue: &wgpu::Queue,
    ) {
        self.cameradata.dims = [config.width as f32, config.height as f32];
        self.send_cameradata(&queue);
    }
    fn update(&mut self, event: winit::event::WindowEvent) {
        // unused
        match event {
            #[allow(deprecated)]
            winit::event::WindowEvent::CursorMoved { position, modifiers, .. } => {
                let delta_pos = [
                    position.x - self.last_mouse_pos[0],
                    position.y - self.last_mouse_pos[1],
                ];
                if modifiers.shift() {
                    self.cameradata.rotation[0] += (delta_pos[0] * 0.01) as f32; // yaw
                    self.cameradata.rotation[1] += (delta_pos[1] * 0.01) as f32; // pitch
                    self.cameradata.rotation[1] = self.cameradata.rotation[1].clamp(0.0, std::f32::consts::PI);
                    if delta_pos.iter().all(|v| *v != 0.0) {
                        self.cameradata_modified = true;
                    }
                }
                self.last_mouse_pos = [position.x, position.y];
            }
            winit::event::WindowEvent::KeyboardInput { input, .. } => {
                let keycode = input.virtual_keycode;
                if let Some(keycode) = keycode {
                    if let winit::event::ElementState::Pressed = input.state {
                        match keycode {
                            VirtualKeyCode::W => {
                                self.cameradata.position[1] += 0.1;
                                self.cameradata_modified = true;
                            }
                            VirtualKeyCode::S => {
                                self.cameradata.position[1] -= 0.1;
                                self.cameradata_modified = true;
                            }
                            VirtualKeyCode::D => {
                                self.cameradata.position[0] += 0.1;
                                self.cameradata_modified = true;
                            }
                            VirtualKeyCode::A => {
                                self.cameradata.position[0] -= 0.1;
                                self.cameradata_modified = true;
                            }
                            VirtualKeyCode::Q => {
                                self.cameradata.position[2] += 0.1;
                                self.cameradata_modified = true;
                            }
                            VirtualKeyCode::E => {
                                self.cameradata.position[2] -= 0.1;
                                self.cameradata_modified = true;
                            }
                            VirtualKeyCode::Space => {
                                self.activated = !self.activated;
                                self.cameradata.activated = if self.activated { 1 } else { 0 };
                                self.cameradata_modified = true;
                            }
                            
                            _ => {}
                        }
                    }
                } else {
                    // dunno what to do with this just gonna ignore
                }
            }
            _ => {}
        }
    }
    fn init(
        config: &wgpu::SurfaceConfiguration,
        _adapter: &wgpu::Adapter,
        device: &wgpu::Device,
        _queue: &wgpu::Queue,
    ) -> Self {
        let vertex_buff = Buff::new(&device, &VERTEX_BUFF, &VERTEX_DATA);
        let index_buff = Buff::new(&device, &BuffInfo::IT, &INDECES_DATA);

        let cameradata = CameraData{
            dims: [config.width as f32, config.height as f32],
            position: [0.0,-3.0,0.0],
            rotation: [0.0,3.1415/2.0],
            fov_y: std::f32::consts::FRAC_PI_2, // 45deg
            activated: 0,
            a: 0.0,
            _spacer0:0,
            _spacer1:0,
        };
        dbg!((cameradata, std::mem::size_of::<CameraData>()));
        let cameradata_buff = Buff::new(&device, &BuffInfo::<CameraData>::IT, &[cameradata]);
        let cameradata_info = CameraDataGroupInfo::new(&device);
        let cameradata_binding = cameradata_info.bind(cameradata_buff.slice(..));

        let gtx_render_pipeline = GTXRenderPipeline::new(
            &device,
            (
                device.create_shader_module(include_glsl!(
                    "shaders/draw.vert",
                    naga::ShaderStage::Vertex
                )),
                "main",
            ),
            (
                device.create_shader_module(include_glsl!(
                    "shaders/draw.frag",
                    naga::ShaderStage::Fragment
                )),
                "main",
            ),
            &[Some(config.view_formats[0].into())],
            &VERTEX_BUFF,
            &cameradata_info,
        );

        Self {
            gtx_render_pipeline,
            vertex_buff,
            index_buff,
            cameradata_binding,
            cameradata_buff,
            cameradata,
            cameradata_modified: false,
            last_mouse_pos: [0.0, 0.0],
            activated: false,
        }
    }
    fn render(
        &mut self,
        view: &wgpu::TextureView,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        _spawner: &crate::engine_base::Spawner,
    ) {
        let mut enc =
            device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });

        // {
        //     self.cameradata.a = std::time::SystemTime::now().duration_since(std::time::SystemTime::UNIX_EPOCH).unwrap().as_secs_f64().sin() as f32;
        //     self.cameradata_modified = true;
        // }

        if self.cameradata_modified {
            self.cameradata_modified = false;
            self.send_cameradata(queue);
        }

        {
            let mut pass = enc.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: None,
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
                        store: true,
                    },
                })],
                depth_stencil_attachment: None,
            });
            self.gtx_render_pipeline.draw_indexed(
                &mut pass,
                0..6, 0..1,
                self.index_buff.slice(..),
                self.vertex_buff.slice(..),
                &self.cameradata_binding,
            );
        }

        queue.submit(Some(enc.finish().into()));
    }
}
