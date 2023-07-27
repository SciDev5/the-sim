use std::{marker::PhantomData, mem::size_of, ops::{RangeBounds, Bound}};

use bytemuck::{Pod, Zeroable};
use wgpu::util::DeviceExt;

#[derive(Debug, Clone, Copy)]
pub struct BufferInfo1d<T: Sized + Pod + Zeroable> {
    dims_min: usize,
    visibility: wgpu::ShaderStages,
    _phantom_: PhantomData<T>,
}
impl<T: Sized + Pod + Zeroable> BufferInfo1d<T> {
    pub fn new(
        dims_min: usize,
        visibility: wgpu::ShaderStages,
    ) -> Self {
        Self {
            dims_min,
            visibility,
            _phantom_: PhantomData,
        }
    }
    pub fn layout_entry(
        &self,
        binding: u32,
        ty: wgpu::BufferBindingType,
    ) -> wgpu::BindGroupLayoutEntry {
        wgpu::BindGroupLayoutEntry {
            binding,
            visibility: self.visibility,
            ty: wgpu::BindingType::Buffer {
                ty,
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
                | wgpu::BufferUsages::UNIFORM
                | wgpu::BufferUsages::STORAGE,
        })
    }
    pub fn create<'a>(&self, device: &'a wgpu::Device, data: &[T]) -> Buffer1d<T> {
        Buffer1d {
            _info: *self,
            buffer: self.create_buffer(&device, data),
            len: data.len(),
        }
    }
}

pub struct Buffer1d<T: Sized + Pod + Zeroable> {
    _info: BufferInfo1d<T>,
    buffer: wgpu::Buffer,
    len: usize,
}

impl<T: Pod + Zeroable> Buffer1d<T> {
    pub fn write(&self, offset: u64, data: &[T], queue: &wgpu::Queue) {
        queue.write_buffer(&self.buffer, offset, bytemuck::cast_slice(data))
    }
    pub fn bind_group_entry<S: RangeBounds<usize>>(&self, binding: u32, range: S) -> wgpu::BindGroupEntry {
        let start = match range.start_bound() {
            Bound::Included(v) => *v,
            Bound::Excluded(v) => *v + 1,
            Bound::Unbounded => 0,
        };
        let end_exclusive = match range.end_bound() {
            Bound::Included(v) => *v + 1,
            Bound::Excluded(v) => *v,
            Bound::Unbounded => self.len,
        }.min(self.len);
        let len = end_exclusive.saturating_sub(start);
        wgpu::BindGroupEntry {
            binding,
            resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                buffer: &self.buffer,
                offset: start as u64,
                size: wgpu::BufferSize::new((size_of::<T>() * len) as u64),
            }),
        }
    }
    pub fn slice<S: RangeBounds<wgpu::BufferAddress>>(&self, range: S) -> wgpu::BufferSlice {
        self.buffer.slice(range)
    }
}
