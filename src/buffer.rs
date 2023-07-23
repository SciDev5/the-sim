use std::{marker::PhantomData, mem::size_of, ops::RangeBounds};

use bytemuck::{Pod, Zeroable};
use wgpu::util::DeviceExt;

#[derive(Debug, Clone, Copy)]
pub struct BufferInfo1d<T: Sized + Pod + Zeroable> {
    dims_min: usize,
    visibility: wgpu::ShaderStages,
    typ: wgpu::BufferBindingType,
    _phantom_: PhantomData<T>,
}
impl<T: Sized + Pod + Zeroable> BufferInfo1d<T> {
    pub fn new(dims_min: usize, visibility: wgpu::ShaderStages, typ: wgpu::BufferBindingType) -> Self {
        Self { dims_min, visibility, typ, _phantom_: PhantomData }
    }
    pub fn layout_entry(&self, binding: u32) -> wgpu::BindGroupLayoutEntry {
        wgpu::BindGroupLayoutEntry {
            binding,
            visibility: self.visibility,
            ty: wgpu::BindingType::Buffer {
                ty: self.typ,
                has_dynamic_offset: false,
                min_binding_size: wgpu::BufferSize::new((size_of::<T>() * self.dims_min) as u64),
            },
            count: None,
        }
    }
    pub fn layout_vertex<'a>(
        &self,
        attributes: &'a [wgpu::VertexAttribute],
        step_mode: wgpu::VertexStepMode,
    ) -> wgpu::VertexBufferLayout<'a> {
        wgpu::VertexBufferLayout {
            array_stride: size_of::<T>() as u64,
            attributes,
            step_mode,
        }
    }
    fn create_buffer<'a>(&self, device: &'a wgpu::Device, data: &[T]) -> wgpu::Buffer {
        device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: None,
            contents: bytemuck::cast_slice(&data),
            usage: wgpu::BufferUsages::COPY_SRC
                | wgpu::BufferUsages::COPY_DST
                | wgpu::BufferUsages::VERTEX
                | if let wgpu::BufferBindingType::Uniform = self.typ {
                    wgpu::BufferUsages::UNIFORM
                } else {
                    wgpu::BufferUsages::STORAGE
                },
        })
    }
    pub fn create<'a>(&self, device: &'a wgpu::Device, data: &[T]) -> Buffer1d<T> {
        Buffer1d {
            _info: *self,
            buffer: self.create_buffer(&device, data),
        }
    }
}

pub struct Buffer1d<T: Sized + Pod + Zeroable> {
    _info: BufferInfo1d<T>,
    buffer: wgpu::Buffer,
}

impl<T: Pod + Zeroable> Buffer1d<T> {
    pub fn write(&self, offset: u64, data: &[T], queue: &wgpu::Queue) {
        queue.write_buffer(&self.buffer, offset, bytemuck::cast_slice(data))
    }
    pub fn bind_group_entry(&self, binding: u32) -> wgpu::BindGroupEntry {
        wgpu::BindGroupEntry {
            binding,
            resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                buffer: &self.buffer,
                offset: 0,
                size: wgpu::BufferSize::new(4 * 512 * 512),
            }),
        }
    }
    pub fn slice<S: RangeBounds<wgpu::BufferAddress>>(&self, range: S) -> wgpu::BufferSlice {
        self.buffer.slice(range)
    }
}
