use crate::rendering::{vertex_buffer::VertexBufferable, wgpu_pipeline::VertexBufferSetting};

use nalgebra_glm as glm;
use std::ops::Range;

/// Instance object for `Renderer2d`. Allows translating and scaling sprites, but currently doesn't
/// allow for rotating them.
pub(super) struct Instance {
    pub position: glm::Vec2,
    pub scale: f32,
}

impl Instance {
    /// Computes the model matrix and returns an InstanceData object containing it
    ///
    /// TODO: Investigate ways of caching/storing the matrix across frames.
    pub(super) fn to_data(&self) -> InstanceData {
        let translate: glm::Vec3 = glm::vec3(self.position.x, self.position.y, 0.0);

        let scale: glm::Vec3 = glm::vec3(self.scale, self.scale, 0.0);

        let mut mat = glm::scale(&glm::identity(), &scale);
        mat = glm::translate(&glm::identity(), &translate) * mat;

        InstanceData { model_matrix: mat }
    }
}

#[derive(Copy, Clone, Debug)]
pub(super) struct InstanceData {
    model_matrix: glm::Mat4,
}

// Asserts that there is no padding in InstanceData
static_assertions::const_assert_eq!(
    std::mem::size_of::<InstanceData>(),
    std::mem::size_of::<f32>() * 4 * 4
);

// Safety:
// See https://docs.rs/bytemuck/1.2.0/bytemuck/trait.Zeroable.html
// We know that Mat4 is just 16 f32 values, so it can be zeroed.
unsafe impl bytemuck::Zeroable for InstanceData {}

// Safety:
// See https://docs.rs/bytemuck/1.2.0/bytemuck/trait.Pod.html
// We need to check for the absence of padding, see static assert above.
unsafe impl bytemuck::Pod for InstanceData {}

impl VertexBufferable for InstanceData {
    fn buffer_descriptor(shader_locations: Range<u32>) -> VertexBufferSetting {
        let start_shader_location = shader_locations.start;

        assert!(shader_locations.len() == 4);

        VertexBufferSetting {
            stride: std::mem::size_of::<InstanceData>() as u64,
            step_mode: wgpu::InputStepMode::Instance,
            attributes: vec![
                // Total of 16 floats ( 4 x 4 ) -> corresponds to one Mat4
                // We need to do this this because the largest floating point vertex attribute possible is
                // Float4.
                //
                // In the shader, you can just write mat4 and indicate start_shader_location for location,
                // and it just works.
                wgpu::VertexAttributeDescriptor {
                    offset: 0,
                    format: wgpu::VertexFormat::Float4,
                    shader_location: start_shader_location,
                },
                wgpu::VertexAttributeDescriptor {
                    offset: (std::mem::size_of::<f32>() * 4) as wgpu::BufferAddress,
                    format: wgpu::VertexFormat::Float4,
                    shader_location: start_shader_location + 1,
                },
                wgpu::VertexAttributeDescriptor {
                    offset: (std::mem::size_of::<f32>() * 8) as wgpu::BufferAddress,
                    format: wgpu::VertexFormat::Float4,
                    shader_location: start_shader_location + 2,
                },
                wgpu::VertexAttributeDescriptor {
                    offset: (std::mem::size_of::<f32>() * 12) as wgpu::BufferAddress,
                    format: wgpu::VertexFormat::Float4,
                    shader_location: start_shader_location + 3,
                },
            ],
        }
    }
}
