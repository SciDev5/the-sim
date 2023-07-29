use std::{
    marker::PhantomData,
    mem::size_of,
    num::{NonZeroU32, NonZeroU64},
    ops::{Bound, RangeBounds},
};

use bytemuck::{Pod, Zeroable};
use image::GenericImageView;
use wgpu::util::DeviceExt;

pub struct ParitalBindGroupLayoutEntry {
    pub count: Option<NonZeroU32>,
    pub ty: wgpu::BindingType,
}

#[macro_export]
macro_rules! bind_group_info {
    ($name:ident; $vis:expr; $($binding_num:expr => ($res_type:ty $(, $data:expr)?)),* $(,)?) => {
        paste::paste! {
            #[allow(non_upper_case_globals)]
            const [<BGLD_ $name>] : wgpu::BindGroupLayoutDescriptor<'static> = wgpu::BindGroupLayoutDescriptor {
                label: None,
                entries: &[
                    $({
                        const it: crate::new_abstractions::ParitalBindGroupLayoutEntry = $res_type::bind_group_layout_entry($($data,)?);
                        wgpu::BindGroupLayoutEntry {
                            binding: $binding_num,
                            visibility: $vis,
                            count: it.count,
                            ty: it.ty
                        }
                    },)*
                ]
            };


            struct $name<'a> {
                layout: wgpu::BindGroupLayout,
                device: &'a wgpu::Device,
            }
            impl<'a> $name<'a> {
                fn new(device: &'a wgpu::Device) -> Self {
                    Self {
                        layout: device.create_bind_group_layout(&[<BGLD_ $name>]),
                        device,
                    }
                }
                #[allow(non_camel_case_types)]
                pub fn bind<
                    'data,
                    $([<TImpl $binding_num>] : $crate::new_abstractions::IsRepresentedByLayout<'data, $res_type> + Into<wgpu::BindingResource<'data>>,)*
                >(&self, $([<binding $binding_num>]: [<TImpl $binding_num>],)*) -> wgpu::BindGroup {
                    self.device.create_bind_group(
                        &wgpu::BindGroupDescriptor {
                            label: None,
                            layout: &self.layout,
                            entries: &[
                                $(
                                    wgpu::BindGroupEntry {
                                        binding: $binding_num,
                                        resource: [<binding $binding_num>].into(),
                                    },
                                )*
                            ]
                        }
                    )
                }
            }
        }
    };
}

pub trait IsRepresentedByLayout<'data, T> {}

pub struct BuffInfo<D: Pod + Zeroable>(PhantomData<D>);

impl<D: Pod + Zeroable> BuffInfo<D> {
    const fn elt_size() -> NonZeroU64 {
        if let Some(size) = NonZeroU64::new(size_of::<D>() as u64) {
            size
        } else {
            panic!("Buffer with zero-size-typed data is not allowed.")
        }
    }
    #[doc(hidden)]
    pub const fn bind_group_layout_entry(
        data: wgpu::BufferBindingType,
    ) -> ParitalBindGroupLayoutEntry {
        ParitalBindGroupLayoutEntry {
            count: None,
            ty: wgpu::BindingType::Buffer {
                ty: data,
                has_dynamic_offset: false,
                min_binding_size: Some(Self::elt_size()),
            },
        }
    }
}
impl<D: Pod + Zeroable + VertexLayoutInfo> BuffInfo<D> {
    pub fn layout_vertex<'a>(
        &self,
        step_mode: wgpu::VertexStepMode,
    ) -> wgpu::VertexBufferLayout<'a> {
        wgpu::VertexBufferLayout {
            array_stride: Self::elt_size().get(),
            step_mode,
            attributes: &D::ATTRIBUTES,
        }
    }
}

pub struct Buff<T: Pod + Zeroable> {
    buffer: wgpu::Buffer,
    _phantom_: PhantomData<T>,
}

impl<T: Pod + Zeroable> Buff<T> {
    pub fn new(device: &wgpu::Device, _spec: &BuffInfo<T>, data: &[T]) -> Self {
        Self {
            buffer: device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: None,
                contents: bytemuck::cast_slice(data),
                usage: wgpu::BufferUsages::COPY_SRC
                    | wgpu::BufferUsages::COPY_DST
                    | wgpu::BufferUsages::VERTEX
                    | wgpu::BufferUsages::UNIFORM
                    | wgpu::BufferUsages::STORAGE, // TODO make more efficient.
            }),
            _phantom_: PhantomData,
        }
    }
    pub fn write(&self, offset: u64, data: &[T], queue: &wgpu::Queue) {
        queue.write_buffer(&self.buffer, offset, bytemuck::cast_slice(data))
    }
    pub fn slice<S: RangeBounds<u64>>(&self, range: S) -> BuffSlice<T> {
        let elt_size: u64 = size_of::<T>() as u64;
        let len = self.buffer.size() / elt_size;
        let start = match range.start_bound() {
            Bound::Included(v) => *v,
            Bound::Excluded(v) => *v + 1,
            Bound::Unbounded => 0,
        };
        let end_exclusive = match range.end_bound() {
            Bound::Included(v) => *v + 1,
            Bound::Excluded(v) => *v,
            Bound::Unbounded => len,
        }
        .min(len);
        let len = end_exclusive.saturating_sub(start);

        BuffSlice {
            buff: &self,
            offset: elt_size * start as u64,
            len: NonZeroU64::new(elt_size * len).unwrap_or(NonZeroU64::new(1).unwrap()),
        }
    }
}

pub struct BuffSlice<'a, T: Pod + Zeroable> {
    buff: &'a Buff<T>,
    offset: u64,
    len: NonZeroU64,
}
impl<'a, T: Pod + Zeroable> From<BuffSlice<'a, T>> for wgpu::BufferSlice<'a> {
    fn from(value: BuffSlice<'a, T>) -> Self {
        value
            .buff
            .buffer
            .slice(value.offset..value.offset + u64::from(value.len))
    }
}

impl<'a, T: Pod + Zeroable> From<BuffSlice<'a, T>> for wgpu::BindingResource<'a> {
    fn from(value: BuffSlice<'a, T>) -> Self {
        Self::Buffer(wgpu::BufferBinding {
            buffer: &value.buff.buffer,
            offset: value.offset,
            size: Some(value.len),
        })
    }
}

impl<'a, T: Pod + Zeroable> IsRepresentedByLayout<'a, BuffInfo<T>> for BuffSlice<'a, T> {}

pub trait TextureDimension {
    type ExtentND;
    fn convert_extent(extent_nd: Self::ExtentND) -> wgpu::Extent3d;
    const DIMENSION: wgpu::TextureDimension;
    const VIEW_DIMENSION: wgpu::TextureViewDimension;
}

pub struct _2D;
impl TextureDimension for _2D {
    type ExtentND = (u32, u32);
    const DIMENSION: wgpu::TextureDimension = wgpu::TextureDimension::D2;
    const VIEW_DIMENSION: wgpu::TextureViewDimension = wgpu::TextureViewDimension::D2;
    fn convert_extent((width, height): Self::ExtentND) -> wgpu::Extent3d {
        wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        }
    }
}

pub trait TexInfoType {}
pub struct TISampler;
pub struct TITexture;
pub struct TIStorageTexture;

pub struct TexInfo<Dim: TextureDimension, Type: TexInfoType>(PhantomData<(Dim, Type)>);

impl TexInfoType for TISampler {}
impl<Dim: TextureDimension> TexInfo<Dim, TISampler> {
    #[doc(hidden)]
    pub const fn bind_group_layout_entry(
        data: wgpu::SamplerBindingType,
    ) -> ParitalBindGroupLayoutEntry {
        ParitalBindGroupLayoutEntry {
            count: None,
            ty: wgpu::BindingType::Sampler(data),
        }
    }
}

impl TexInfoType for TITexture {}
impl<Dim: TextureDimension> TexInfo<Dim, TITexture> {
    #[doc(hidden)]
    pub const fn bind_group_layout_entry(
        data: wgpu::TextureSampleType,
    ) -> ParitalBindGroupLayoutEntry {
        ParitalBindGroupLayoutEntry {
            count: None,
            ty: wgpu::BindingType::Texture {
                multisampled: false, // TODO generailize to allow multisampling
                view_dimension: Dim::VIEW_DIMENSION,
                sample_type: data,
            },
        }
    }
}

impl TexInfoType for TIStorageTexture {}
impl<Dim: TextureDimension> TexInfo<Dim, TIStorageTexture> {
    #[doc(hidden)]
    pub const fn bind_group_layout_entry(
        data: wgpu::StorageTextureAccess,
    ) -> ParitalBindGroupLayoutEntry {
        ParitalBindGroupLayoutEntry {
            count: None,
            ty: wgpu::BindingType::StorageTexture {
                access: data,
                format: wgpu::TextureFormat::Rgba8Unorm,
                view_dimension: Dim::VIEW_DIMENSION,
            },
        }
    }
}

pub enum TexResource<'a, Dim: TextureDimension, Type: TexInfoType> {
    StorageTexture(&'a Tex<Dim>, PhantomData<Type>),
    Sampler(&'a Tex<Dim>, PhantomData<Type>),
    Texture(&'a Tex<Dim>, PhantomData<Type>),
}
impl<'a, Dim: TextureDimension, Type: TexInfoType> IsRepresentedByLayout<'a, TexInfo<Dim, Type>>
    for TexResource<'a, Dim, Type>
{
}
impl<'a, Dim: TextureDimension, Type: TexInfoType> From<TexResource<'a, Dim, Type>>
    for wgpu::BindingResource<'a>
{
    fn from(value: TexResource<'a, Dim, Type>) -> Self {
        match value {
            TexResource::Sampler(v, _) => wgpu::BindingResource::Sampler(&v.sampler),
            TexResource::Texture(v, _) => wgpu::BindingResource::TextureView(&v.view),
            TexResource::StorageTexture(v, _) => wgpu::BindingResource::TextureView(&v.view),
        }
    }
}

pub struct Tex<Dim: TextureDimension> {
    pub label: String,
    pub texture: wgpu::Texture,
    pub view: wgpu::TextureView,
    pub sampler: wgpu::Sampler,
    _phantom_: PhantomData<Dim>,
}

impl<Dim: TextureDimension> Tex<Dim> {
    pub fn create_uninit(label: &str, extent_nd: Dim::ExtentND, device: &wgpu::Device) -> Self {
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some(label),
            size: Dim::convert_extent(extent_nd),
            mip_level_count: 1,
            sample_count: 1,
            dimension: Dim::DIMENSION,
            format: wgpu::TextureFormat::Rgba8Unorm,
            usage: wgpu::TextureUsages::TEXTURE_BINDING
                | wgpu::TextureUsages::COPY_DST
                | wgpu::TextureUsages::COPY_SRC
                | wgpu::TextureUsages::STORAGE_BINDING, // TODO make efficient
            view_formats: &[],
        });
        let view = texture.create_view(&wgpu::TextureViewDescriptor {
            ..Default::default()
        });
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });

        Self {
            label: label.to_string(),
            texture,
            view,
            sampler,
            _phantom_: PhantomData,
        }
    }

    pub fn binding_storage(&self) -> TexResource<Dim, TIStorageTexture> {
        TexResource::StorageTexture(&self, PhantomData)
    }
    pub fn binding_sampler(&self) -> TexResource<Dim, TISampler> {
        TexResource::Sampler(&self, PhantomData)
    }
    pub fn binding_texture(&self) -> TexResource<Dim, TITexture> {
        TexResource::Texture(&self, PhantomData)
    }
}

impl Tex<_2D> {
    pub fn create(
        (img, label): (image::DynamicImage, &str),
        device: &wgpu::Device,
        queue: &wgpu::Queue,
    ) -> Self {
        let this = Self::create_uninit(label, img.dimensions(), device);
        this.write_image(img, queue);
        this
    }

    pub fn write_image(&self, img: image::DynamicImage, queue: &wgpu::Queue) {
        let data_raw = img.to_rgba8();
        let (width, height) = img.dimensions();
        queue.write_texture(
            wgpu::ImageCopyTextureBase {
                texture: &self.texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            &data_raw,
            wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: Some(width * 4),
                rows_per_image: Some(height),
            },
            self.texture.size(),
        );
    }
}

pub trait ZSTValue {
    const IT: Self;
}

impl<D: Pod + Zeroable> ZSTValue for BuffInfo<D> {
    const IT: Self = Self(PhantomData);
}

impl<D: TextureDimension, T: TexInfoType> ZSTValue for TexInfo<D, T> {
    const IT: Self = Self(PhantomData);
}

pub trait VertexLayoutInfo {
    const ATTRIBUTES: &'static [wgpu::VertexAttribute];
}
