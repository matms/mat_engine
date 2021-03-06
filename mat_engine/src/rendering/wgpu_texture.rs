use std::num::NonZeroU32;

use crate::typedefs::BoxErr;
use image::GenericImageView;
use wgpu::util::DeviceExt;

// See https://sotrh.github.io/learn-wgpu/beginner/tutorial5-textures/

pub(crate) struct WgpuTexture {
    #[allow(dead_code)]
    pub(crate) texture: wgpu::Texture,
    pub(crate) texture_view: wgpu::TextureView,
    pub(crate) sampler: wgpu::Sampler,
}

impl WgpuTexture {
    pub(crate) fn new_from_bytes(
        wgpu_device: &mut wgpu::Device,
        bytes: &[u8],
        label: Option<&'static str>,
    ) -> Result<(Self, wgpu::CommandBuffer), BoxErr> {
        let img = image::load_from_memory(bytes)?;
        Self::new_from_image(wgpu_device, img, label)
    }

    pub(crate) fn new_from_image(
        wgpu_device: &mut wgpu::Device,
        img: image::DynamicImage,
        label: Option<&'static str>,
    ) -> Result<(Self, wgpu::CommandBuffer), BoxErr> {
        let rgba_data = img.as_rgba8().ok_or_else(|| {
            format!(
                "Failed to get rgba8 data from image. Currently, we only support loading\
                 ImageRgba8 textures. The image passed in is {}.",
                image_type_descriptor_str(&img)
            )
        })?;

        let (width, height) = img.dimensions();

        let size = wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1, // Depth of 1 represents 2D texture
        };

        // Make empty texture
        let texture = wgpu_device.create_texture(&wgpu::TextureDescriptor {
            label,
            size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8Unorm, // TODO: Rgba8Unorm or Rgba8UnormSrgb
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
        });

        // Note: If the image is very big you will get a warning, but it shouldn't be an issue
        // bc it will allocate anyways, I think.
        let buffer = wgpu_device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: None,
            usage: wgpu::BufferUsages::COPY_SRC,
            contents: &rgba_data,
        });

        let mut encoder = wgpu_device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some(
                format!(
                    "texture_buffer_copy_encoder_for_texture__{}.",
                    if let Some(t) = label { t } else { "" }
                )
                .as_ref(),
            ),
        });

        // Copy rgba data from buffer to texture
        encoder.copy_buffer_to_texture(
            wgpu::ImageCopyBuffer {
                buffer: &buffer,
                layout: wgpu::ImageDataLayout {
                    offset: 0,
                    bytes_per_row: NonZeroU32::new(4 * width),
                    rows_per_image: NonZeroU32::new(height),
                },
            },
            wgpu::ImageCopyTexture {
                texture: &texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            size,
        );

        let cmd_buffer = encoder.finish();

        let texture_view = texture.create_view(&wgpu::TextureViewDescriptor::default());

        let sampler = wgpu_device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear, // Linear is smoother (blends pixels)
            min_filter: wgpu::FilterMode::Nearest, // Nearest is crisper (returns the nearest pixel)
            mipmap_filter: wgpu::FilterMode::Nearest,
            lod_min_clamp: -100.0,
            lod_max_clamp: 100.0,
            ..Default::default()
        });

        Ok((
            Self {
                texture,
                texture_view,
                sampler,
            },
            cmd_buffer,
        ))
    }
}

impl crate::rendering::bind_group::BindGroupable for WgpuTexture {
    fn get_wgpu_bind_group_layout_descriptor() -> wgpu::BindGroupLayoutDescriptor<'static> {
        wgpu::BindGroupLayoutDescriptor {
            // See `BindGroupDescriptor` instantiation for important info on the bindings.
            // If you change sth. here, you'll probably have to change it there also, so
            // be careful.
            entries: &[
                // Copied from https://sotrh.github.io/learn-wgpu/beginner/tutorial5-textures/#the-bindgroup
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
                    ty: wgpu::BindingType::Sampler {
                        comparison: false,
                        filtering: true,
                    },
                    count: None,
                },
                // End copied from
            ],
            label: Some("wgpu_texture_bind_group_layout"),
        }
    }

    fn make_wgpu_bind_group(
        &self,
        bind_group_layout: &wgpu::BindGroupLayout,
        device: &mut wgpu::Device,
    ) -> wgpu::BindGroup {
        let desc = &wgpu::BindGroupDescriptor {
            layout: bind_group_layout,
            // See `BindGroupLayoutDescriptor` instantiation for important info info on the bindings.
            // If you change sth. here, you'll probably have to change it there also, so
            // be careful.
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&self.texture_view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&self.sampler),
                },
            ],
            label: Some("wgpu_texture_bind_group_for_some_specific_texture"),
        };

        device.create_bind_group(desc)
    }
}

fn image_type_descriptor_str(img: &image::DynamicImage) -> &'static str {
    match img {
        image::DynamicImage::ImageLuma8(_) => "ImageLuma8",
        image::DynamicImage::ImageLumaA8(_) => "ImageLumaA8",
        image::DynamicImage::ImageRgb8(_) => "ImageRgb8",
        image::DynamicImage::ImageRgba8(_) => "ImageRgba8",
        image::DynamicImage::ImageBgr8(_) => "ImageBgr8",
        image::DynamicImage::ImageBgra8(_) => "ImageBgra8",
        image::DynamicImage::ImageLuma16(_) => "ImageLuma16",
        image::DynamicImage::ImageLumaA16(_) => "ImageLumaA16",
        image::DynamicImage::ImageRgb16(_) => "ImageRgb16",
        image::DynamicImage::ImageRgba16(_) => "ImageRgba16",
    }
}
