//! This file will be deleted later...

use crate::rendering::{bind_group::BindGroupable, generic_uniform::Uniform};
use zeroable::Zeroable;

#[derive(Copy, Clone, Debug, Zeroable)]
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
unsafe impl bytemuck::Pod for TestUniformContent {}

pub(super) struct TestUniform {
    pub(super) content: TestUniformContent,
    pub(super) buffer: wgpu::Buffer,
}

impl TestUniform {
    pub(super) fn new(device: &mut wgpu::Device) -> Self {
        let content = TestUniformContent { num: 0.5 };
        let buffer = Self::create_new_buffer(
            content,
            device,
            wgpu::BufferUsage::UNIFORM | wgpu::BufferUsage::COPY_DST,
        );
        Self { content, buffer }
    }
}

impl BindGroupable for TestUniform {
    fn get_wgpu_bind_group_layout_descriptor() -> wgpu::BindGroupLayoutDescriptor<'static> {
        wgpu::BindGroupLayoutDescriptor {
            bindings: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStage::VERTEX,
                ty: wgpu::BindingType::UniformBuffer {
                    dynamic: false, // This is NOT a dynamically sized array, it is statically sized.
                },
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
            bindings: &[wgpu::Binding {
                binding: 0,
                resource: wgpu::BindingResource::Buffer {
                    buffer: &self.buffer,
                    range: 0..std::mem::size_of_val(&self.content) as wgpu::BufferAddress,
                },
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
        usage: wgpu::BufferUsage,
    ) -> wgpu::Buffer {
        device.create_buffer_with_data(bytemuck::cast_slice(&[content]), usage)
    }

    fn update_buffer(&self, encoder: &mut wgpu::CommandEncoder, device: &mut wgpu::Device) {
        let staging_buffer =
            Self::create_new_buffer(self.content, device, wgpu::BufferUsage::COPY_SRC);

        encoder.copy_buffer_to_buffer(
            &staging_buffer,
            0,
            &self.buffer,
            0,
            std::mem::size_of_val(&self.content) as wgpu::BufferAddress,
        );
    }
}
