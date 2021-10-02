//! This file will be deleted later...

use std::num::NonZeroU64;

use crate::rendering::{bind_group::BindGroupable, generic_uniform::Uniform};
use bytemuck::{Pod, Zeroable};
use wgpu::util::DeviceExt;

#[repr(C)]
#[derive(Copy, Clone, Debug, Zeroable, Pod)]
pub(super) struct TestUniformContent {
    pub(super) num: f32,
}

// Asserts that there is no padding in TestUniformContent
static_assertions::const_assert_eq!(
    std::mem::size_of::<TestUniformContent>(),
    std::mem::size_of::<f32>()
);

// Safety:
// See https://docs.rs/bytemuck/1.2.0/bytemuck/trait.Pod.html
// We need to check for the absence of padding, see static assert above.
// unsafe impl bytemuck::Pod for TestUniformContent {}

pub(super) struct TestUniform {
    pub(super) content: TestUniformContent,
    pub(super) buffer: wgpu::Buffer,
}

#[allow(dead_code)]
impl TestUniform {
    pub(super) fn new(device: &mut wgpu::Device) -> Self {
        let content = TestUniformContent { num: 0.5 };
        let buffer = Self::create_new_buffer(
            content,
            device,
            wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        );
        Self { content, buffer }
    }
}

impl BindGroupable for TestUniform {
    fn get_wgpu_bind_group_layout_descriptor() -> wgpu::BindGroupLayoutDescriptor<'static> {
        wgpu::BindGroupLayoutDescriptor {
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false, // This is NOT a dynamically sized array, it is statically sized.
                    min_binding_size: None,
                },
                count: None,
            }],
            label: Some("test_uniform_bind_group_layout"),
        }
    }
    fn make_wgpu_bind_group<'a>(
        &self,
        bind_group_layout: &wgpu::BindGroupLayout,
        device: &mut wgpu::Device,
    ) -> wgpu::BindGroup {
        device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                    buffer: &self.buffer,
                    offset: 0,
                    size: NonZeroU64::new(std::mem::size_of_val(&self.content) as u64),
                }),
            }],
            label: Some("test_uniform_bind_group"),
        })
    }
}

impl Uniform for TestUniform {
    type Content = TestUniformContent;

    fn create_new_buffer(
        content: Self::Content,
        device: &mut wgpu::Device,
        usage: wgpu::BufferUsages,
    ) -> wgpu::Buffer {
        device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            contents: bytemuck::cast_slice(&[content]),
            usage: usage,
            label: None,
        })
    }

    fn update_buffer(&self, encoder: &mut wgpu::CommandEncoder, device: &mut wgpu::Device) {
        let staging_buffer =
            Self::create_new_buffer(self.content, device, wgpu::BufferUsages::COPY_SRC);

        encoder.copy_buffer_to_buffer(
            &staging_buffer,
            0,
            &self.buffer,
            0,
            std::mem::size_of_val(&self.content) as wgpu::BufferAddress,
        );
    }
}
