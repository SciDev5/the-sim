use image::GenericImageView;

#[macro_export]
macro_rules! load_img {
    ($loc: tt) => {
        image::load_from_memory(include_bytes!($loc)).map(|it| (it, Some($loc)))
    };
}

pub struct Texture {
    pub texture: wgpu::Texture,
    pub view: wgpu::TextureView,
    pub sampler: wgpu::Sampler,
}

impl Texture {
    pub fn create_uninit(
        label: Option<&str>,
        (width, height): (u32, u32),
        device: &wgpu::Device,
    ) -> Self {
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label,
            size: wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });
        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
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
            texture,
            view,
            sampler,
        }
    }
    pub fn create(
        (img, label): (image::DynamicImage, Option<&str>),
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
    pub fn write_buffer(&self, buffer: &wgpu::Buffer, layout: wgpu::ImageDataLayout, device: &wgpu::Device, queue: &wgpu::Queue) {
            let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("texture_buffer_copy_encoder"),
            });

            self.write_buffer_commandencoder(buffer, layout, &mut encoder);

            queue.submit(Some(encoder.finish().into()));
    }
    pub fn write_buffer_commandencoder(&self, buffer: &wgpu::Buffer, layout: wgpu::ImageDataLayout, encoder: &mut wgpu::CommandEncoder) {
            encoder.copy_buffer_to_texture(
                wgpu::ImageCopyBufferBase {
                    buffer: &buffer,
                    layout,
                },
                wgpu::ImageCopyTextureBase {
                    texture: &self.texture,
                    mip_level: 0,
                    origin: wgpu::Origin3d::ZERO,
                    aspect: wgpu::TextureAspect::All,
                },
                self.texture.size(),
            );
    }
}
