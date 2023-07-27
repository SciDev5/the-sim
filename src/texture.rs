use image::GenericImageView;

use crate::bind_group::{BindGroupData, TypedBindGroupGenInner, TypedBindGroupInit};

#[macro_export]
macro_rules! load_img {
    ($loc: tt) => {
        image::load_from_memory(include_bytes!($loc)).map(|it| (it, $loc))
    };
}

#[derive(Debug, Clone, Copy)]
pub struct Tex2dFragBindGroupInit;
impl TypedBindGroupInit for Tex2dFragBindGroupInit {
    fn label(&self) -> &'static str {
        "texture2d_fragment:BindGroup"
    }
    fn entries(&self) -> Vec<wgpu::BindGroupLayoutEntry> {
        vec![
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
        ]
    }
}

pub struct Tex2dFragBindGroup<'a>(BindGroupData<'a>);

impl<'a> TypedBindGroupGenInner<'a, Tex2dFragBindGroupInit, Texture2D> for Tex2dFragBindGroup<'a> {
    fn from_bind_group_data<'b: 'a>(
        _init: Tex2dFragBindGroupInit,
        data: BindGroupData<'b>,
    ) -> Self {
        Self(data)
    }
    fn bind_group_data(&self) -> &BindGroupData {
        &self.0
    }
    fn entries_from_data<'b>(&self, data: &'b Texture2D) -> Vec<wgpu::BindGroupEntry<'b>> {
        vec![
            wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::TextureView(&data.view),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: wgpu::BindingResource::Sampler(&data.sampler),
            },
        ]
    }
    fn label(data: &Texture2D) -> &str {
        data.label.as_str()
    }
}

pub struct Texture2D {
    pub label: String,
    pub texture: wgpu::Texture,
    pub view: wgpu::TextureView,
    pub sampler: wgpu::Sampler,
}

impl Texture2D {
    pub fn create_uninit(
        label: &str,
        for_compute: bool,
        (width, height): (u32, u32),
        device: &wgpu::Device,
    ) -> Self {
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some(label),
            size: wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8Unorm,
            usage: wgpu::TextureUsages::TEXTURE_BINDING
                | wgpu::TextureUsages::COPY_DST
                | if for_compute {
                    wgpu::TextureUsages::STORAGE_BINDING
                } else {
                    wgpu::TextureUsages::empty()
                },
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
        }
    }
    pub fn create(
        (img, label): (image::DynamicImage, &str),
        for_compute: bool,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
    ) -> Self {
        let this = Self::create_uninit(label, for_compute, img.dimensions(), device);
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

    pub fn layout_entry_compute(
        &self,
        binding: u32,
        access: wgpu::StorageTextureAccess,
    ) -> wgpu::BindGroupLayoutEntry {
        wgpu::BindGroupLayoutEntry {
            binding,
            visibility: wgpu::ShaderStages::COMPUTE,
            ty: wgpu::BindingType::StorageTexture {
                access,
                format: wgpu::TextureFormat::Rgba8Unorm,
                view_dimension: wgpu::TextureViewDimension::D2,
            },
            count: None,
        }
    }
    pub fn bind_group_entry(&self, binding: u32) -> wgpu::BindGroupEntry {
        wgpu::BindGroupEntry {
            binding,
            resource: wgpu::BindingResource::TextureView(&self.view),
        }
    }
}
